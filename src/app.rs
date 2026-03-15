use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;

use crate::cli::{Cli, Command, InternalCommand};
use crate::complete_stage::{
    canonical_pr_is_merged, finalize_merged_implementation, run_complete_stage,
};
use crate::config::Config;
use crate::domain::{
    FlowStage, decide_run_stage, parse_issue_ref, select_next_backlog_project_item,
};
use crate::github::{GhProjectClient, ProjectIssueItem, ProjectSnapshot};
use crate::init::init_project_files;
use crate::project_files::ProjectPaths;
use crate::repo::RepoContext;
use crate::runtime::{RuntimeLayout, SessionManifest};
use crate::shell::{Shell, SystemShell};
use crate::templates::{render_template, render_zellij_session_name, render_zellij_tab_name};
use crate::zellij::{ZellijLauncher, capture_current_binding};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let shell = SystemShell;

    match cli.command {
        Command::Init => run_init(&shell),
        Command::Poll { zellij_session } => run_poll(&shell, zellij_session.as_deref()),
        Command::Loop { zellij_session } => run_loop(&shell, zellij_session.as_deref()),
        Command::Run {
            issue,
            debug,
            zellij_session,
        } => run_manual_run(&shell, &issue, debug, zellij_session.as_deref()),
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

fn run_poll(shell: &dyn Shell, zellij_session_override: Option<&str>) -> Result<()> {
    let context = load_execution_context(shell, zellij_session_override)?;
    let github = GhProjectClient::new(shell);
    let zellij = ZellijLauncher::new(shell);
    match run_poll_cycle(shell, &context, &github, &zellij)? {
        PollCycleOutcome::NoEligibleIssue { project_title } => {
            println!(
                "poll: no eligible backlog issues for repo={}/{} in project={}",
                context.repo.github_owner, context.repo.github_repo, project_title
            );
        }
        PollCycleOutcome::Launched(launch) => {
            let launch_zellij =
                resolve_launch_zellij_config(&context.config.zellij, launch.issue_number)?;
            println!(
                "poll: claimed issue #{} -> {} session_uuid={}",
                launch.issue_number,
                context
                    .config
                    .issue_analysis_flow
                    .statuses
                    .analysis_in_progress,
                launch.session_uuid
            );
            print_zellij_launch_target(&context.runtime, &launch.session_uuid, &launch_zellij);
        }
    }
    Ok(())
}

fn run_loop(shell: &dyn Shell, zellij_session_override: Option<&str>) -> Result<()> {
    let context = load_execution_context(shell, zellij_session_override)?;
    let github = GhProjectClient::new(shell);
    let zellij = ZellijLauncher::new(shell);
    let interval = Duration::from_secs(context.config.runtime.poll_interval_seconds);
    let mut cycle_number = 1_u64;

    loop {
        println!("loop: cycle={cycle_number} started");
        match run_poll_cycle(shell, &context, &github, &zellij) {
            Ok(PollCycleOutcome::NoEligibleIssue { project_title }) => {
                println!(
                    "loop: cycle={cycle_number} no eligible backlog issues in project={}",
                    project_title
                );
            }
            Ok(PollCycleOutcome::Launched(launch)) => {
                let launch_zellij =
                    resolve_launch_zellij_config(&context.config.zellij, launch.issue_number)?;
                println!(
                    "loop: cycle={cycle_number} launched issue #{} session_uuid={}",
                    launch.issue_number, launch.session_uuid
                );
                print_zellij_launch_target(&context.runtime, &launch.session_uuid, &launch_zellij);
            }
            Err(error) => {
                eprintln!("loop: cycle={cycle_number} failed: {error:#}");
            }
        }

        println!(
            "loop: cycle={cycle_number} sleeping {}s",
            interval.as_secs()
        );
        thread::sleep(interval);
        cycle_number += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PollCycleOutcome {
    NoEligibleIssue { project_title: String },
    Launched(LaunchOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchOutcome {
    issue_number: u64,
    session_uuid: String,
    launched: bool,
}

fn run_poll_cycle(
    shell: &dyn Shell,
    context: &ExecutionContext,
    github: &GhProjectClient<'_>,
    zellij: &ZellijLauncher<'_>,
) -> Result<PollCycleOutcome> {
    let snapshot =
        github.load_project_snapshot(&context.repo.repo_root, &context.config.github.project_id)?;
    let Some(issue) = select_next_backlog_project_item(
        &snapshot.items,
        &context.config.issue_analysis_flow.statuses,
        &context.repo.github_owner,
        &context.repo.github_repo,
    ) else {
        return Ok(PollCycleOutcome::NoEligibleIssue {
            project_title: snapshot.title,
        });
    };

    let launch = run_issue_entrypoint(shell, context, github, zellij, &snapshot, issue, false)?;
    Ok(PollCycleOutcome::Launched(launch))
}

fn run_manual_run(
    shell: &dyn Shell,
    issue_ref: &str,
    debug: bool,
    zellij_session_override: Option<&str>,
) -> Result<()> {
    let context = load_execution_context(shell, zellij_session_override)?;
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

    let launch = run_issue_entrypoint(shell, &context, &github, &zellij, &snapshot, issue, debug)?;
    if launch.launched {
        let launch_zellij =
            resolve_launch_zellij_config(&context.config.zellij, launch.issue_number)?;
        println!(
            "run: issue=#{} launched in zellij session_uuid={}",
            launch.issue_number, launch.session_uuid
        );
        print_zellij_launch_target(&context.runtime, &launch.session_uuid, &launch_zellij);
    } else {
        println!(
            "run: issue=#{} finalized without launch session_uuid={}",
            launch.issue_number, launch.session_uuid
        );
    }
    Ok(())
}

fn run_issue_entrypoint(
    shell: &dyn Shell,
    context: &ExecutionContext,
    github: &GhProjectClient<'_>,
    zellij: &ZellijLauncher<'_>,
    snapshot: &ProjectSnapshot,
    issue: &ProjectIssueItem,
    debug: bool,
) -> Result<LaunchOutcome> {
    let current_status = issue.status_name.as_deref().ok_or_else(|| {
        anyhow::anyhow!(
            "issue #{} does not have a project status",
            issue.issue_number
        )
    })?;

    let decision = decide_run_stage(
        current_status,
        &context.config.issue_analysis_flow.statuses,
        &context.config.issue_implementation_flow.statuses,
    );
    if !decision.allowed {
        bail!(
            "run denied for issue #{}: {}",
            issue.issue_number,
            decision.reason
        );
    }
    let stage = decision
        .stage
        .expect("allowed run decision must include stage");

    if let Some(outcome) =
        maybe_finalize_merged_implementation(shell, context, issue, current_status)?
    {
        return Ok(outcome);
    }

    if let Err(error) = validate_stage_preconditions(context, issue.issue_number, stage) {
        if stage == FlowStage::Implementation {
            mark_issue_as_blocked(context, github, snapshot, issue, stage);
        }
        return Err(error)
            .with_context(|| format!("failed to validate {} preconditions", stage.as_str()));
    }

    let launch_zellij = resolve_launch_zellij_config(&context.config.zellij, issue.issue_number)?;
    let manifest = prepare_session_manifest(
        context,
        github,
        snapshot,
        issue,
        current_status,
        stage,
        &launch_zellij,
    )?;
    let issue_url = format!(
        "https://github.com/{}/{}/issues/{}",
        context.repo.github_owner, context.repo.github_repo, issue.issue_number
    );
    let binary_path = std::env::current_exe().context("failed to resolve ai-teamlead binary")?;
    if let Err(error) = zellij.launch_issue_stage(
        &context.repo,
        &context.repo.repo_root,
        &context.runtime,
        &launch_zellij,
        stage,
        &issue_url,
        &manifest.session_uuid,
        &binary_path,
        debug,
    ) {
        mark_issue_as_blocked(context, github, snapshot, issue, stage);
        return Err(error)
            .with_context(|| format!("failed to launch zellij {} session", stage.as_str()));
    }

    Ok(LaunchOutcome {
        issue_number: issue.issue_number,
        session_uuid: manifest.session_uuid,
        launched: true,
    })
}

fn validate_stage_preconditions(
    context: &ExecutionContext,
    issue_number: u64,
    stage: FlowStage,
) -> Result<()> {
    match stage {
        FlowStage::Analysis => Ok(()),
        FlowStage::Implementation => validate_approved_analysis_artifacts(context, issue_number),
    }
}

fn validate_approved_analysis_artifacts(
    context: &ExecutionContext,
    issue_number: u64,
) -> Result<()> {
    let artifacts_dir =
        render_analysis_artifacts_dir(&context.config, &context.repo.github_repo, issue_number);
    let readme_path = context.repo.repo_root.join(artifacts_dir).join("README.md");
    let content = fs::read_to_string(&readme_path).with_context(|| {
        format!(
            "approved analysis artifacts are missing: {}",
            readme_path.display()
        )
    })?;

    ensure_markdown_metadata_value(&content, "Статус согласования", "approved")?;
    ensure_markdown_metadata_present(&content, "Approved By")?;
    ensure_markdown_metadata_present(&content, "Approved At")?;
    Ok(())
}

fn render_analysis_artifacts_dir(config: &Config, repo_name: &str, issue_number: u64) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let issue_number_str = issue_number.to_string();
    let branch = render_template(
        &config.launch_agent.analysis_branch_template,
        &[
            ("HOME", home.as_str()),
            ("REPO", repo_name),
            ("ISSUE_NUMBER", issue_number_str.as_str()),
        ],
    );

    render_template(
        &config.launch_agent.analysis_artifacts_dir_template,
        &[
            ("HOME", home.as_str()),
            ("REPO", repo_name),
            ("ISSUE_NUMBER", issue_number_str.as_str()),
            ("BRANCH", branch.as_str()),
        ],
    )
}

fn ensure_markdown_metadata_value(content: &str, key: &str, expected: &str) -> Result<()> {
    let value = markdown_metadata_value(content, key)
        .ok_or_else(|| anyhow!("analysis artifact metadata '{}' is missing", key))?;
    anyhow::ensure!(
        value == expected,
        "analysis artifact metadata '{}' must be '{}', got '{}'",
        key,
        expected,
        value
    );
    Ok(())
}

fn ensure_markdown_metadata_present(content: &str, key: &str) -> Result<()> {
    markdown_metadata_value(content, key)
        .ok_or_else(|| anyhow!("analysis artifact metadata '{}' is missing", key))?;
    Ok(())
}

fn markdown_metadata_value<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}:");
    content
        .lines()
        .find_map(|line| line.strip_prefix(&prefix).map(str::trim))
        .filter(|value| !value.is_empty())
}

fn prepare_session_manifest(
    context: &ExecutionContext,
    github: &GhProjectClient<'_>,
    snapshot: &ProjectSnapshot,
    issue: &ProjectIssueItem,
    current_status: &str,
    stage: FlowStage,
    launch_zellij: &crate::config::ZellijConfig,
) -> Result<SessionManifest> {
    let target_status = match stage {
        FlowStage::Analysis => context
            .config
            .issue_analysis_flow
            .statuses
            .analysis_in_progress
            .as_str(),
        FlowStage::Implementation => context
            .config
            .issue_implementation_flow
            .statuses
            .implementation_in_progress
            .as_str(),
    };

    let claim_status = match stage {
        FlowStage::Analysis => context.config.issue_analysis_flow.statuses.backlog.as_str(),
        FlowStage::Implementation => context
            .config
            .issue_implementation_flow
            .statuses
            .ready_for_implementation
            .as_str(),
    };

    if current_status == claim_status {
        github.update_status(
            &context.repo.repo_root,
            &context.config.github.project_id,
            &issue.item_id,
            &snapshot.status_field_id,
            snapshot.option_id_by_name(target_status)?,
        )?;
        let manifest = context.runtime.create_claim_binding(
            &context.repo,
            &context.config.github.project_id,
            launch_zellij,
            issue.issue_number,
            stage,
            target_status,
        )?;
        return persist_stage_workspace(context, &manifest.session_uuid, issue.issue_number, stage);
    }

    let issue_index = context
        .runtime
        .load_issue_index(issue.issue_number)?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing issue session index for issue #{}",
                issue.issue_number
            )
        })?;
    let session_uuid = issue_index.session_uuid_for_stage(stage).ok_or_else(|| {
        anyhow::anyhow!(
            "missing {} session index for issue #{}",
            stage.as_str(),
            issue.issue_number
        )
    })?;
    let manifest = context
        .runtime
        .load_session_manifest(session_uuid)?
        .ok_or_else(|| {
            anyhow::anyhow!("missing session manifest for issue #{}", issue.issue_number)
        })?;
    anyhow::ensure!(
        manifest.stage == stage,
        "session manifest stage mismatch for issue #{}: expected {}, got {}",
        issue.issue_number,
        stage.as_str(),
        manifest.stage.as_str(),
    );

    github.update_status(
        &context.repo.repo_root,
        &context.config.github.project_id,
        &issue.item_id,
        &snapshot.status_field_id,
        snapshot.option_id_by_name(target_status)?,
    )?;
    context
        .runtime
        .update_issue_flow_status(issue.issue_number, target_status)?;

    persist_stage_workspace(context, &manifest.session_uuid, issue.issue_number, stage)
}

fn persist_stage_workspace(
    context: &ExecutionContext,
    session_uuid: &str,
    issue_number: u64,
    stage: FlowStage,
) -> Result<SessionManifest> {
    let launch_context = render_launch_agent_context(context, issue_number, stage)?;
    context.runtime.update_stage_workspace(
        session_uuid,
        &launch_context.branch,
        Path::new(&launch_context.worktree_root),
        &launch_context.artifacts_dir,
    )
}

fn maybe_finalize_merged_implementation(
    shell: &dyn Shell,
    context: &ExecutionContext,
    issue: &ProjectIssueItem,
    current_status: &str,
) -> Result<Option<LaunchOutcome>> {
    if current_status
        != context
            .config
            .issue_implementation_flow
            .statuses
            .waiting_for_code_review
    {
        return Ok(None);
    }

    let launch_context =
        render_launch_agent_context(context, issue.issue_number, FlowStage::Implementation)?;
    if !canonical_pr_is_merged(shell, &context.repo.repo_root, &launch_context.branch)? {
        return Ok(None);
    }

    let manifest = context
        .runtime
        .load_issue_index(issue.issue_number)?
        .and_then(|issue_index| {
            issue_index
                .session_uuid_for_stage(FlowStage::Implementation)
                .map(str::to_string)
        })
        .map(|session_uuid| context.runtime.load_session_manifest(&session_uuid))
        .transpose()?
        .flatten();
    let target_status = finalize_merged_implementation(
        shell,
        &context.repo.repo_root,
        &context.runtime,
        &context.config,
        manifest.as_ref(),
        issue.issue_number,
        &context.config.github.project_id,
        &context.repo.github_owner,
        &context.repo.github_repo,
        &launch_context.branch,
    )?;
    println!(
        "run: issue=#{} reconciled merged implementation PR -> {}",
        issue.issue_number, target_status
    );

    Ok(Some(LaunchOutcome {
        issue_number: issue.issue_number,
        session_uuid: manifest
            .map(|manifest| manifest.session_uuid)
            .unwrap_or_else(|| "none".to_string()),
        launched: false,
    }))
}

fn mark_issue_as_blocked(
    context: &ExecutionContext,
    github: &GhProjectClient<'_>,
    snapshot: &ProjectSnapshot,
    issue: &ProjectIssueItem,
    stage: FlowStage,
) {
    let blocked_status = match stage {
        FlowStage::Analysis => context
            .config
            .issue_analysis_flow
            .statuses
            .analysis_blocked
            .as_str(),
        FlowStage::Implementation => context
            .config
            .issue_implementation_flow
            .statuses
            .implementation_blocked
            .as_str(),
    };

    if let Ok(blocked_option_id) = snapshot.option_id_by_name(blocked_status) {
        let _ = github.update_status(
            &context.repo.repo_root,
            &context.config.github.project_id,
            &issue.item_id,
            &snapshot.status_field_id,
            blocked_option_id,
        );
    }

    let _ = context
        .runtime
        .update_issue_flow_status(issue.issue_number, blocked_status);
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
    let tab_name = manifest
        .as_ref()
        .map(|session| session.zellij.tab_name.as_str())
        .unwrap_or(zellij.tab_name.as_str());
    let pane_id = manifest
        .as_ref()
        .map(|session| session.zellij.pane_id.as_str())
        .unwrap_or("pending");

    println!(
        "launch target: zellij_session={} tab={} tab_id={} pane_id={} log={}",
        session_id,
        tab_name,
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
        InternalCommand::CompleteStage {
            session_uuid,
            stage,
            outcome,
            message,
        } => run_complete_stage(shell, &session_uuid, &stage, &outcome, &message),
    }
}

fn run_internal_bind_zellij_pane(shell: &dyn Shell, session_uuid: &str) -> Result<()> {
    let context = load_execution_context(shell, None)?;
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
    let context = load_execution_context(shell, None)?;
    let launch_zellij = resolve_launch_zellij_config(&context.config.zellij, issue_number)?;
    let manifest = context.runtime.create_claim_binding(
        &context.repo,
        &context.config.github.project_id,
        &launch_zellij,
        issue_number,
        FlowStage::Analysis,
        &context
            .config
            .issue_analysis_flow
            .statuses
            .analysis_in_progress,
    )?;
    let zellij = ZellijLauncher::new(shell);
    let issue_url = format!(
        "https://github.com/{}/{}/issues/{}",
        context.repo.github_owner, context.repo.github_repo, issue_number
    );
    let binary_path = std::env::current_exe().context("failed to resolve ai-teamlead binary")?;
    zellij.launch_issue_stage(
        &context.repo,
        &context.repo.repo_root,
        &context.runtime,
        &launch_zellij,
        FlowStage::Analysis,
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
    let context = load_execution_context(shell, None)?;
    let issue_number = parse_issue_ref(issue_ref)
        .with_context(|| format!("failed to parse issue reference: {issue_ref}"))?;
    let flow_stage = std::env::var("AI_TEAMLEAD_FLOW_STAGE")
        .ok()
        .as_deref()
        .and_then(|value| match value {
            "implementation" => Some(FlowStage::Implementation),
            "analysis" => Some(FlowStage::Analysis),
            _ => None,
        })
        .unwrap_or(FlowStage::Analysis);
    let rendered = render_launch_agent_context(&context, issue_number, flow_stage)?;

    println!(
        "ISSUE_NUMBER={}",
        shell_quote(&rendered.issue_number.to_string())
    );
    println!("REPO={}", shell_quote(&rendered.repo_name));
    println!("FLOW_STAGE={}", shell_quote(rendered.stage.as_str()));
    println!("BRANCH={}", shell_quote(&rendered.branch));
    println!("WORKTREE_ROOT={}", shell_quote(&rendered.worktree_root));
    println!("ARTIFACTS_DIR={}", shell_quote(&rendered.artifacts_dir));
    println!(
        "CLAUDE_GLOBAL_ARGS={}",
        shell_quote_array(&rendered.claude_global_args)
    );
    println!(
        "CODEX_GLOBAL_ARGS={}",
        shell_quote_array(&rendered.codex_global_args)
    );
    Ok(())
}

struct ExecutionContext {
    repo: RepoContext,
    config: Config,
    runtime: RuntimeLayout,
}

fn load_execution_context(
    shell: &dyn Shell,
    zellij_session_override: Option<&str>,
) -> Result<ExecutionContext> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    load_execution_context_at(shell, cwd, zellij_session_override)
}

fn load_execution_context_at(
    shell: &dyn Shell,
    cwd: PathBuf,
    zellij_session_override: Option<&str>,
) -> Result<ExecutionContext> {
    let repo = RepoContext::discover(shell, &cwd)?;
    let mut config = Config::load_from_repo_root(&repo.repo_root)?;
    let configured_session_name =
        render_zellij_session_name(&config.zellij.session_name, &repo.github_repo)?;
    config.zellij.session_name = resolve_zellij_session_name(
        &configured_session_name,
        zellij_session_override,
        std::env::var("ZELLIJ_SESSION_NAME").ok().as_deref(),
    );
    let runtime = RuntimeLayout::from_repo_root(&repo.repo_root);
    runtime.ensure_exists()?;

    Ok(ExecutionContext {
        repo,
        config,
        runtime,
    })
}

fn resolve_zellij_session_name(
    configured_session_name: &str,
    zellij_session_override: Option<&str>,
    zellij_session_from_env: Option<&str>,
) -> String {
    zellij_session_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            zellij_session_from_env
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or(configured_session_name)
        .to_string()
}

fn resolve_launch_zellij_config(
    zellij: &crate::config::ZellijConfig,
    issue_number: u64,
) -> Result<crate::config::ZellijConfig> {
    let mut resolved = zellij.clone();
    resolved.tab_name = render_zellij_tab_name(
        &zellij.tab_name,
        zellij.tab_name_template.as_deref(),
        issue_number,
    )?;
    Ok(resolved)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LaunchAgentContext {
    issue_number: u64,
    repo_name: String,
    stage: FlowStage,
    branch: String,
    worktree_root: String,
    artifacts_dir: String,
    claude_global_args: Vec<String>,
    codex_global_args: Vec<String>,
}

fn render_launch_agent_context(
    context: &ExecutionContext,
    issue_number: u64,
    stage: FlowStage,
) -> Result<LaunchAgentContext> {
    let repo_name = context.repo.github_repo.clone();
    let home = std::env::var("HOME").context("HOME is not set")?;
    let issue_number_str = issue_number.to_string();

    let branch_template = match stage {
        FlowStage::Analysis => &context.config.launch_agent.analysis_branch_template,
        FlowStage::Implementation => &context.config.launch_agent.implementation_branch_template,
    };
    let worktree_template = match stage {
        FlowStage::Analysis => &context.config.launch_agent.worktree_root_template,
        FlowStage::Implementation => {
            &context
                .config
                .launch_agent
                .implementation_worktree_root_template
        }
    };
    let artifacts_template = match stage {
        FlowStage::Analysis => &context.config.launch_agent.analysis_artifacts_dir_template,
        FlowStage::Implementation => {
            &context
                .config
                .launch_agent
                .implementation_artifacts_dir_template
        }
    };

    let branch = render_template(
        branch_template,
        &[
            ("HOME", home.as_str()),
            ("REPO", repo_name.as_str()),
            ("ISSUE_NUMBER", issue_number_str.as_str()),
        ],
    );
    let worktree_root = render_template(
        worktree_template,
        &[
            ("HOME", home.as_str()),
            ("REPO", repo_name.as_str()),
            ("ISSUE_NUMBER", issue_number_str.as_str()),
            ("BRANCH", branch.as_str()),
        ],
    );
    let artifacts_dir = render_template(
        artifacts_template,
        &[
            ("HOME", home.as_str()),
            ("REPO", repo_name.as_str()),
            ("ISSUE_NUMBER", issue_number_str.as_str()),
            ("BRANCH", branch.as_str()),
        ],
    );

    Ok(LaunchAgentContext {
        issue_number,
        repo_name,
        stage,
        branch,
        worktree_root,
        artifacts_dir,
        claude_global_args: context.config.launch_agent.global_args.claude.clone(),
        codex_global_args: context.config.launch_agent.global_args.codex.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::{resolve_launch_zellij_config, resolve_zellij_session_name};
    use crate::config::ZellijConfig;

    #[test]
    fn zellij_session_override_has_highest_priority() {
        let resolved = resolve_zellij_session_name(
            "settings-session",
            Some("cli-session"),
            Some("env-session"),
        );
        assert_eq!(resolved, "cli-session");
    }

    #[test]
    fn zellij_session_from_env_beats_settings() {
        let resolved = resolve_zellij_session_name("settings-session", None, Some("env-session"));
        assert_eq!(resolved, "env-session");
    }

    #[test]
    fn zellij_session_falls_back_to_settings() {
        let resolved = resolve_zellij_session_name("settings-session", None, None);
        assert_eq!(resolved, "settings-session");
    }

    #[test]
    fn zellij_session_ignores_blank_override_and_env() {
        let resolved = resolve_zellij_session_name("settings-session", Some("   "), Some(""));
        assert_eq!(resolved, "settings-session");
    }

    #[test]
    fn resolves_issue_aware_tab_name_from_template() {
        let zellij = ZellijConfig {
            session_name: "example".into(),
            tab_name: "issue-analysis".into(),
            tab_name_template: Some("#${ISSUE_NUMBER}".into()),
            layout: None,
        };

        let resolved = resolve_launch_zellij_config(&zellij, 42).expect("resolved config");
        assert_eq!(resolved.tab_name, "#42");
    }

    #[test]
    fn keeps_stable_tab_name_without_template() {
        let zellij = ZellijConfig {
            session_name: "example".into(),
            tab_name: "issue-analysis".into(),
            tab_name_template: None,
            layout: None,
        };

        let resolved = resolve_launch_zellij_config(&zellij, 42).expect("resolved config");
        assert_eq!(resolved.tab_name, "issue-analysis");
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn shell_quote_array(values: &[String]) -> String {
    let quoted = values
        .iter()
        .map(|value| shell_quote(value))
        .collect::<Vec<_>>()
        .join(" ");
    format!("({quoted})")
}

#[cfg(test)]
mod launch_agent_tests {
    use super::{
        ExecutionContext, LaunchAgentContext, render_launch_agent_context, shell_quote_array,
        validate_stage_preconditions,
    };
    use crate::config::{
        Config, FlowStatuses, GithubConfig, ImplementationFlowStatuses, IssueAnalysisFlowConfig,
        IssueImplementationFlowConfig, LaunchAgentConfig, LaunchAgentGlobalArgsConfig,
        RuntimeConfig, ZellijConfig,
    };
    use crate::domain::FlowStage;
    use crate::repo::RepoContext;
    use crate::runtime::RuntimeLayout;
    use crate::templates::render_template;
    use std::path::PathBuf;
    use tempfile::tempdir;

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
            stage: FlowStage::Analysis,
            branch,
            worktree_root: worktree,
            artifacts_dir: artifacts,
            claude_global_args: vec!["--permission-mode".into(), "auto".into()],
            codex_global_args: vec!["--full-auto".into()],
        };

        assert_eq!(context.branch, "analysis/issue-42");
        assert_eq!(
            context.worktree_root,
            "/home/danil/worktrees/teamlead/analysis/issue-42"
        );
        assert_eq!(context.artifacts_dir, "specs/issues/42");
        assert_eq!(
            shell_quote_array(&context.codex_global_args),
            "('--full-auto')"
        );
    }

    #[test]
    fn render_launch_agent_context_includes_default_global_args() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        std::fs::create_dir_all(repo_root.join(".git")).expect("git dir");

        let context = test_execution_context(repo_root);
        let rendered = render_launch_agent_context(&context, 42, FlowStage::Analysis)
            .expect("render launch context");

        assert_eq!(rendered.codex_global_args, vec!["--full-auto".to_string()]);
        assert_eq!(
            rendered.claude_global_args,
            vec!["--permission-mode".to_string(), "auto".to_string()]
        );
    }

    #[test]
    fn implementation_preconditions_accept_approved_analysis_artifacts() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        std::fs::create_dir_all(repo_root.join(".git")).expect("git dir");
        std::fs::create_dir_all(repo_root.join("specs/issues/42")).expect("artifacts dir");
        std::fs::write(
            repo_root.join("specs/issues/42/README.md"),
            "# Issue 42\n\nСтатус согласования: approved\nApproved By: dapi\nApproved At: 2026-03-14T19:14:28+03:00\n",
        )
        .expect("write README");

        let context = test_execution_context(repo_root);
        validate_stage_preconditions(&context, 42, FlowStage::Implementation)
            .expect("implementation preconditions");
    }

    #[test]
    fn implementation_preconditions_reject_missing_approval_metadata() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        std::fs::create_dir_all(repo_root.join(".git")).expect("git dir");
        std::fs::create_dir_all(repo_root.join("specs/issues/42")).expect("artifacts dir");
        std::fs::write(
            repo_root.join("specs/issues/42/README.md"),
            "# Issue 42\n\nСтатус согласования: pending human review\n",
        )
        .expect("write README");

        let context = test_execution_context(repo_root);
        let error = validate_stage_preconditions(&context, 42, FlowStage::Implementation)
            .expect_err("preconditions must fail");
        assert!(
            error
                .to_string()
                .contains("metadata 'Статус согласования' must be 'approved'"),
            "unexpected error: {error:#}"
        );
    }

    fn test_execution_context(repo_root: PathBuf) -> ExecutionContext {
        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime layout");

        ExecutionContext {
            repo: RepoContext {
                repo_root: repo_root.clone(),
                git_dir: repo_root.join(".git"),
                github_owner: "dapi".into(),
                github_repo: "example".into(),
            },
            config: Config {
                github: GithubConfig {
                    project_id: "PVT_test_project".into(),
                },
                issue_analysis_flow: IssueAnalysisFlowConfig {
                    statuses: FlowStatuses {
                        backlog: "Backlog".into(),
                        analysis_in_progress: "Analysis In Progress".into(),
                        waiting_for_clarification: "Waiting for Clarification".into(),
                        waiting_for_plan_review: "Waiting for Plan Review".into(),
                        ready_for_implementation: "Ready for Implementation".into(),
                        analysis_blocked: "Analysis Blocked".into(),
                    },
                },
                issue_implementation_flow: IssueImplementationFlowConfig {
                    statuses: ImplementationFlowStatuses {
                        ready_for_implementation: "Ready for Implementation".into(),
                        implementation_in_progress: "Implementation In Progress".into(),
                        waiting_for_ci: "Waiting for CI".into(),
                        waiting_for_code_review: "Waiting for Code Review".into(),
                        done: "Done".into(),
                        implementation_blocked: "Implementation Blocked".into(),
                    },
                },
                runtime: RuntimeConfig {
                    max_parallel: 1,
                    poll_interval_seconds: 60,
                },
                zellij: ZellijConfig {
                    session_name: "example".into(),
                    tab_name: "issue-analysis".into(),
                    tab_name_template: None,
                    layout: None,
                },
                launch_agent: LaunchAgentConfig {
                    analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}".into(),
                    worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}".into(),
                    analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}".into(),
                    global_args: LaunchAgentGlobalArgsConfig::default(),
                    implementation_branch_template: "implementation/issue-${ISSUE_NUMBER}".into(),
                    implementation_worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
                        .into(),
                    implementation_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}".into(),
                },
            },
            runtime,
        }
    }
}
