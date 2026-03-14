use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::cli::{Cli, Command, InternalCommand};
use crate::config::Config;
use crate::domain::{can_run_analysis, parse_issue_ref, select_next_backlog_project_item};
use crate::github::GhProjectClient;
use crate::init::init_project_files;
use crate::project_files::ProjectPaths;
use crate::repo::RepoContext;
use crate::runtime::RuntimeLayout;
use crate::shell::{Shell, SystemShell};
use crate::templates::{render_template, render_zellij_session_name};
use crate::zellij::{ZellijLauncher, capture_current_binding};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let shell = SystemShell;

    match cli.command {
        Command::Init => run_init(&shell),
        Command::Poll => run_poll(&shell),
        Command::Run { issue, debug } => run_manual_run(&shell, &issue, debug),
        Command::Internal { internal } => run_internal(&shell, internal),
    }
}

fn run_init(shell: &dyn Shell) -> Result<()> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let repo = RepoContext::discover(shell, &cwd)?;
    let paths = ProjectPaths::from_repo_root(&repo.repo_root);
    let report = init_project_files(&paths)?;

    println!(
        "init: repo={}/{} root={}",
        repo.github_owner,
        repo.github_repo,
        repo.repo_root.display()
    );
    for path in &report.created {
        println!("created: {}", path.display());
    }
    for path in &report.skipped {
        println!("skipped: {}", path.display());
    }

    Ok(())
}

fn run_poll(shell: &dyn Shell) -> Result<()> {
    let context = load_execution_context(shell)?;
    let github = GhProjectClient::new(shell);
    let zellij = ZellijLauncher::new(shell);
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
    let issue_url = format!(
        "https://github.com/{}/{}/issues/{}",
        context.repo.github_owner, context.repo.github_repo, issue.issue_number
    );
    let binary_path = std::env::current_exe().context("failed to resolve ai-teamlead binary")?;
    if let Err(error) = zellij.launch_issue_analysis(
        &context.repo.repo_root,
        &context.runtime,
        &context.config.zellij,
        &issue_url,
        &manifest.session_uuid,
        &binary_path,
        false,
    ) {
        if let Ok(blocked_option_id) = snapshot
            .option_id_by_name(&context.config.issue_analysis_flow.statuses.analysis_blocked)
        {
            let _ = github.update_status(
                &context.repo.repo_root,
                &context.config.github.project_id,
                &issue.item_id,
                &snapshot.status_field_id,
                blocked_option_id,
            );
            let _ = context.runtime.update_issue_flow_status(
                issue.issue_number,
                &context.config.issue_analysis_flow.statuses.analysis_blocked,
            );
        }
        return Err(error).context("failed to launch zellij issue-analysis session");
    }

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
    print_zellij_launch_target(
        &context.runtime,
        &manifest.session_uuid,
        &context.config.zellij,
    );
    Ok(())
}

fn run_manual_run(shell: &dyn Shell, issue_ref: &str, debug: bool) -> Result<()> {
    let context = load_execution_context(shell)?;
    let github = GhProjectClient::new(shell);
    let zellij = ZellijLauncher::new(shell);
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

    let allowed = can_run_analysis(current_status, &context.config.issue_analysis_flow.statuses);
    if !allowed.allowed {
        bail!("run denied for issue #{issue_number}: {}", allowed.reason);
    }

    let issue_index = context.runtime.load_issue_index(issue_number)?;
    let manifest = if current_status == context.config.issue_analysis_flow.statuses.backlog {
        github.update_status(
            &context.repo.repo_root,
            &context.config.github.project_id,
            &issue.item_id,
            &snapshot.status_field_id,
            snapshot.option_id_by_name(
                &context
                    .config
                    .issue_analysis_flow
                    .statuses
                    .analysis_in_progress,
            )?,
        )?;
        context.runtime.create_claim_binding(
            &context.repo,
            &context.config.github.project_id,
            &context.config.zellij,
            issue_number,
        )?
    } else {
        let issue_index = issue_index.ok_or_else(|| {
            anyhow::anyhow!("missing issue session index for issue #{issue_number}")
        })?;
        let _manifest = context
            .runtime
            .load_session_manifest(&issue_index.session_uuid)?
            .ok_or_else(|| anyhow::anyhow!("missing session manifest for issue #{issue_number}"))?;
        github.update_status(
            &context.repo.repo_root,
            &context.config.github.project_id,
            &issue.item_id,
            &snapshot.status_field_id,
            snapshot.option_id_by_name(
                &context
                    .config
                    .issue_analysis_flow
                    .statuses
                    .analysis_in_progress,
            )?,
        )?;
        context.runtime.update_issue_flow_status(
            issue_number,
            &context
                .config
                .issue_analysis_flow
                .statuses
                .analysis_in_progress,
        )?;
        context
            .runtime
            .load_session_manifest(&issue_index.session_uuid)?
            .ok_or_else(|| anyhow::anyhow!("missing session manifest for issue #{issue_number}"))?
    };

    let issue_url = format!(
        "https://github.com/{}/{}/issues/{}",
        context.repo.github_owner, context.repo.github_repo, issue_number
    );
    let binary_path = std::env::current_exe().context("failed to resolve ai-teamlead binary")?;
    zellij.launch_issue_analysis(
        &context.repo.repo_root,
        &context.runtime,
        &context.config.zellij,
        &issue_url,
        &manifest.session_uuid,
        &binary_path,
        debug,
    )?;

    println!(
        "run: issue=#{issue_number} relaunched in zellij session_uuid={}",
        manifest.session_uuid
    );
    print_zellij_launch_target(
        &context.runtime,
        &manifest.session_uuid,
        &context.config.zellij,
    );
    Ok(())
}

fn print_zellij_launch_target(
    runtime: &RuntimeLayout,
    session_uuid: &str,
    zellij: &crate::config::ZellijConfig,
) {
    let launch_log_path = runtime.session_dir(session_uuid).join("launch.log");
    let manifest = wait_for_zellij_binding(runtime, session_uuid, Duration::from_secs(5));
    let session_id = manifest
        .as_ref()
        .map(|session| session.zellij.session_id.as_str())
        .unwrap_or(zellij.session_name.as_str());
    let tab_id = manifest
        .as_ref()
        .map(|session| session.zellij.tab_id.as_str())
        .unwrap_or("pending");
    let pane_id = manifest
        .as_ref()
        .map(|session| session.zellij.pane_id.as_str())
        .unwrap_or("pending");

    println!(
        "launch target: zellij_session={} tab={} tab_id={} pane_id={} log={}",
        session_id,
        zellij.tab_name,
        tab_id,
        pane_id,
        launch_log_path.display()
    );
}

fn wait_for_zellij_binding(
    runtime: &RuntimeLayout,
    session_uuid: &str,
    timeout: Duration,
) -> Option<crate::runtime::SessionManifest> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(manifest) = runtime.load_session_manifest(session_uuid).ok().flatten() {
            if manifest.zellij.tab_id != "pending" && manifest.zellij.pane_id != "pending" {
                return Some(manifest);
            }
            if Instant::now() >= deadline {
                return Some(manifest);
            }
        } else if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn run_internal(shell: &dyn Shell, internal: InternalCommand) -> Result<()> {
    match internal {
        InternalCommand::BindZellijPane { session_uuid } => {
            run_internal_bind_zellij_pane(shell, &session_uuid)
        }
        InternalCommand::LaunchZellijFixture { issue } => {
            run_internal_launch_zellij_fixture(shell, issue)
        }
        InternalCommand::RenderLaunchAgentContext { issue } => {
            run_internal_render_launch_agent_context(shell, &issue)
        }
    }
}

fn run_internal_bind_zellij_pane(shell: &dyn Shell, session_uuid: &str) -> Result<()> {
    let context = load_execution_context(shell)?;
    let (session_id, tab_id, pane_id) = capture_current_binding(
        shell,
        &context.repo.repo_root,
        &context.runtime,
        &context.config.zellij,
        session_uuid,
    )?;
    println!(
        "bound zellij pane: session_uuid={} session_id={} tab_id={} pane_id={}",
        session_uuid, session_id, tab_id, pane_id
    );
    Ok(())
}

fn run_internal_launch_zellij_fixture(shell: &dyn Shell, issue_number: u64) -> Result<()> {
    let context = load_execution_context(shell)?;
    let manifest = context.runtime.create_claim_binding(
        &context.repo,
        &context.config.github.project_id,
        &context.config.zellij,
        issue_number,
    )?;
    let zellij = ZellijLauncher::new(shell);
    let issue_url = format!(
        "https://github.com/{}/{}/issues/{}",
        context.repo.github_owner, context.repo.github_repo, issue_number
    );
    let binary_path = std::env::current_exe().context("failed to resolve ai-teamlead binary")?;
    zellij.launch_issue_analysis(
        &context.repo.repo_root,
        &context.runtime,
        &context.config.zellij,
        &issue_url,
        &manifest.session_uuid,
        &binary_path,
        false,
    )?;
    println!(
        "fixture launch requested: issue=#{issue_number} session_uuid={}",
        manifest.session_uuid
    );
    Ok(())
}

fn run_internal_render_launch_agent_context(shell: &dyn Shell, issue_ref: &str) -> Result<()> {
    let context = load_execution_context(shell)?;
    let issue_number = parse_issue_ref(issue_ref)
        .with_context(|| format!("failed to parse issue reference: {issue_ref}"))?;
    let rendered = render_launch_agent_context(&context, issue_number)?;

    println!(
        "ISSUE_NUMBER={}",
        shell_quote(&rendered.issue_number.to_string())
    );
    println!("REPO={}", shell_quote(&rendered.repo_name));
    println!("BRANCH={}", shell_quote(&rendered.analysis_branch));
    println!("WORKTREE_ROOT={}", shell_quote(&rendered.worktree_root));
    println!(
        "ANALYSIS_ARTIFACTS_DIR={}",
        shell_quote(&rendered.analysis_artifacts_dir)
    );
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
    let mut config = Config::load_from_repo_root(&repo.repo_root)?;
    config.zellij.session_name =
        render_zellij_session_name(&config.zellij.session_name, &repo.github_repo)?;
    let runtime = RuntimeLayout::from_repo_root(&repo.repo_root);
    runtime.ensure_exists()?;

    Ok(ExecutionContext {
        repo,
        config,
        runtime,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchAgentContext {
    issue_number: u64,
    repo_name: String,
    analysis_branch: String,
    worktree_root: String,
    analysis_artifacts_dir: String,
}

fn render_launch_agent_context(
    context: &ExecutionContext,
    issue_number: u64,
) -> Result<LaunchAgentContext> {
    let repo_name = context.repo.github_repo.clone();
    let home = std::env::var("HOME").context("HOME is not set")?;
    let issue_number_str = issue_number.to_string();

    let analysis_branch = render_template(
        &context.config.launch_agent.analysis_branch_template,
        &[
            ("HOME", home.as_str()),
            ("REPO", repo_name.as_str()),
            ("ISSUE_NUMBER", issue_number_str.as_str()),
        ],
    );
    let worktree_root = render_template(
        &context.config.launch_agent.worktree_root_template,
        &[
            ("HOME", home.as_str()),
            ("REPO", repo_name.as_str()),
            ("ISSUE_NUMBER", issue_number_str.as_str()),
            ("BRANCH", analysis_branch.as_str()),
        ],
    );
    let analysis_artifacts_dir = render_template(
        &context.config.launch_agent.analysis_artifacts_dir_template,
        &[
            ("HOME", home.as_str()),
            ("REPO", repo_name.as_str()),
            ("ISSUE_NUMBER", issue_number_str.as_str()),
            ("BRANCH", analysis_branch.as_str()),
        ],
    );

    Ok(LaunchAgentContext {
        issue_number,
        repo_name,
        analysis_branch,
        worktree_root,
        analysis_artifacts_dir,
    })
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod launch_agent_tests {
    use super::LaunchAgentContext;
    use crate::templates::render_template;

    #[test]
    fn renders_launch_agent_templates() {
        let branch = render_template(
            "analysis/issue-${ISSUE_NUMBER}",
            &[("ISSUE_NUMBER", "42"), ("REPO", "teamlead")],
        );
        let worktree = render_template(
            "${HOME}/worktrees/${REPO}/${BRANCH}",
            &[
                ("HOME", "/home/danil"),
                ("REPO", "teamlead"),
                ("BRANCH", &branch),
            ],
        );
        let artifacts = render_template("specs/issues/${ISSUE_NUMBER}", &[("ISSUE_NUMBER", "42")]);

        let context = LaunchAgentContext {
            issue_number: 42,
            repo_name: "teamlead".into(),
            analysis_branch: branch,
            worktree_root: worktree,
            analysis_artifacts_dir: artifacts,
        };

        assert_eq!(context.analysis_branch, "analysis/issue-42");
        assert_eq!(
            context.worktree_root,
            "/home/danil/worktrees/teamlead/analysis/issue-42"
        );
        assert_eq!(context.analysis_artifacts_dir, "specs/issues/42");
    }
}
