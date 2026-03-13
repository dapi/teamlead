use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::config::Config;
use crate::domain::{
    RunSessionFacts, can_run_analysis, parse_issue_ref, select_next_backlog_project_item,
};
use crate::github::GhProjectClient;
use crate::repo::RepoContext;
use crate::runtime::{RuntimeLayout, derive_run_session_facts};
use crate::shell::{Shell, SystemShell};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let shell = SystemShell;

    match cli.command {
        Some(Command::Daemon) | None => run_daemon(&shell),
        Some(Command::Poll) => run_poll(&shell),
        Some(Command::Run { issue }) => run_manual_run(&shell, &issue),
    }
}

fn run_daemon(shell: &dyn Shell) -> Result<()> {
    let context = load_execution_context(shell)?;
    println!(
        "daemon ready: repo={}/{} root={} project_id={}",
        context.repo.github_owner,
        context.repo.github_repo,
        context.repo.repo_root.display(),
        context.config.github.project_id
    );
    println!(
        "runtime: poll_interval_seconds={} max_parallel={}",
        context.config.runtime.poll_interval_seconds, context.config.runtime.max_parallel
    );
    println!("runtime root: {}", context.runtime.root.display());
    Ok(())
}

fn run_poll(shell: &dyn Shell) -> Result<()> {
    let context = load_execution_context(shell)?;
    let github = GhProjectClient::new(shell);
    let snapshot =
        github.load_project_snapshot(&context.repo.repo_root, &context.config.github.project_id)?;
    let Some(issue) = select_next_backlog_project_item(
        &snapshot.items,
        &context.config.issue_analysis_flow.statuses,
        &context.repo.github_owner,
        &context.repo.github_repo,
    ) else {
        println!(
            "poll: no eligible backlog issues for repo={}/{} in project={}",
            context.repo.github_owner, context.repo.github_repo, snapshot.title
        );
        return Ok(());
    };

    let in_progress_option_id = snapshot.option_id_by_name(
        &context
            .config
            .issue_analysis_flow
            .statuses
            .analysis_in_progress,
    )?;
    github.update_status(
        &context.repo.repo_root,
        &context.config.github.project_id,
        &issue.item_id,
        &snapshot.status_field_id,
        in_progress_option_id,
    )?;
    let manifest = context.runtime.create_claim_binding(
        &context.repo,
        &context.config.github.project_id,
        &context.config.zellij,
        issue.issue_number,
    )?;

    println!(
        "poll: claimed issue #{} -> {} session_uuid={}",
        issue.issue_number,
        context
            .config
            .issue_analysis_flow
            .statuses
            .analysis_in_progress,
        manifest.session_uuid
    );
    Ok(())
}

fn run_manual_run(shell: &dyn Shell, issue_ref: &str) -> Result<()> {
    let context = load_execution_context(shell)?;
    let github = GhProjectClient::new(shell);
    let snapshot =
        github.load_project_snapshot(&context.repo.repo_root, &context.config.github.project_id)?;
    let issue_number = parse_issue_ref(issue_ref)
        .with_context(|| format!("failed to parse issue reference: {issue_ref}"))?;

    let issue = snapshot
        .items
        .iter()
        .find(|item| {
            item.issue_number == issue_number
                && item.matches_repo(&context.repo.github_owner, &context.repo.github_repo)
        })
        .ok_or_else(|| anyhow::anyhow!("issue #{issue_number} is not linked to the project"))?;

    let current_status = issue
        .status_name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("issue #{issue_number} does not have a project status"))?;

    let session_facts = if current_status == context.config.issue_analysis_flow.statuses.backlog {
        RunSessionFacts::default()
    } else {
        let issue_index = context
            .runtime
            .load_issue_index(issue_number)?
            .ok_or_else(|| {
                anyhow::anyhow!("missing issue session index for issue #{issue_number}")
            })?;
        let manifest = context
            .runtime
            .load_session_manifest(&issue_index.session_uuid)?
            .ok_or_else(|| anyhow::anyhow!("missing session manifest for issue #{issue_number}"))?;
        let questions = context.runtime.load_question_set(&manifest.session_uuid)?;
        let plan = context.runtime.load_analysis_plan(&manifest.session_uuid)?;
        let events = context
            .runtime
            .load_operator_events(&manifest.session_uuid)?;

        derive_run_session_facts(current_status, questions.as_ref(), plan.as_ref(), &events)?
    };

    let allowed = can_run_analysis(
        current_status,
        session_facts,
        &context.config.issue_analysis_flow.statuses,
    );
    if !allowed.allowed {
        bail!("run denied for issue #{issue_number}: {}", allowed.reason);
    }

    println!("run: issue=#{issue_number} is eligible from status={current_status}");
    Ok(())
}

struct ExecutionContext {
    repo: RepoContext,
    config: Config,
    runtime: RuntimeLayout,
}

fn load_execution_context(shell: &dyn Shell) -> Result<ExecutionContext> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    load_execution_context_at(shell, cwd)
}

fn load_execution_context_at(shell: &dyn Shell, cwd: PathBuf) -> Result<ExecutionContext> {
    let repo = RepoContext::discover(shell, &cwd)?;
    let config = Config::load_from_repo_root(&repo.repo_root)?;
    let runtime = RuntimeLayout::from_repo_root(&repo.repo_root);
    runtime.ensure_exists()?;

    Ok(ExecutionContext {
        repo,
        config,
        runtime,
    })
}
