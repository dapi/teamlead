use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::agent_flow::{
    AgentFlowTestRequest, plan_agent_flow_test, print_plan, print_sandbox_result,
    run_agent_flow_test,
};
use crate::cli::{Cli, Command, InternalCommand, TestAgentFlowArgs, TestCommand};
use crate::complete_stage::run_complete_stage;
use crate::config::Config;
use crate::domain::{can_run_analysis, parse_issue_ref, select_next_backlog_project_item};
use crate::github::{GhProjectClient, ProjectIssueItem, ProjectSnapshot};
use crate::init::init_project_files;
use crate::project_files::ProjectPaths;
use crate::repo::RepoContext;
use crate::runtime::{RuntimeLayout, SessionManifest};
use crate::shell::{Shell, SystemShell};
use crate::templates::{render_template, render_zellij_session_name};
use crate::zellij::{ZellijLauncher, capture_current_binding};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let shell = SystemShell;

    match cli.command {
        Command::Init => run_init(&shell),
        Command::Test { test } => run_test(&shell, test),
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

fn run_test(shell: &dyn Shell, test: TestCommand) -> Result<()> {
    match test {
        TestCommand::AgentFlow(args) => run_test_agent_flow(shell, args),
    }
}

fn run_test_agent_flow(shell: &dyn Shell, args: TestAgentFlowArgs) -> Result<()> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let repo = RepoContext::discover(shell, &cwd)?;
    let plan = plan_agent_flow_test(
        &repo.repo_root,
        &repo.git_dir,
        &AgentFlowTestRequest {
            scenario: args.scenario,
            agent: args.agent,
            mode: args.mode,
            keep_sandbox: args.keep_sandbox,
            artifacts_dir: args.artifacts_dir,
            timeout_seconds: args.timeout_seconds,
            no_build: args.no_build,
        },
    )?;
    print_plan(&plan);
    let result = run_agent_flow_test(shell, &repo.repo_root, &repo.git_dir, &plan)?;
    print_sandbox_result(&result);
    Ok(())
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
    match run_poll_cycle(&context, &github, &zellij)? {
        PollCycleOutcome::NoEligibleIssue { project_title } => {
            println!(
                "poll: no eligible backlog issues for repo={}/{} in project={}",
                context.repo.github_owner, context.repo.github_repo, project_title
            );
        }
        PollCycleOutcome::Launched(launch) => {
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
            print_zellij_launch_target(
                &context.runtime,
                &launch.session_uuid,
                &context.config.zellij,
            );
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
        match run_poll_cycle(&context, &github, &zellij) {
            Ok(PollCycleOutcome::NoEligibleIssue { project_title }) => {
                println!(
                    "loop: cycle={cycle_number} no eligible backlog issues in project={}",
                    project_title
                );
            }
            Ok(PollCycleOutcome::Launched(launch)) => {
                println!(
                    "loop: cycle={cycle_number} launched issue #{} session_uuid={}",
                    launch.issue_number, launch.session_uuid
                );
                print_zellij_launch_target(
                    &context.runtime,
                    &launch.session_uuid,
                    &context.config.zellij,
                );
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
}

fn run_poll_cycle(
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

    let launch = run_issue_entrypoint(context, github, zellij, &snapshot, issue, false)?;
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

    let launch = run_issue_entrypoint(&context, &github, &zellij, &snapshot, issue, debug)?;
    println!(
        "run: issue=#{} launched in zellij session_uuid={}",
        launch.issue_number, launch.session_uuid
    );
    print_zellij_launch_target(
        &context.runtime,
        &launch.session_uuid,
        &context.config.zellij,
    );
    Ok(())
}

fn run_issue_entrypoint(
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

    let allowed = can_run_analysis(current_status, &context.config.issue_analysis_flow.statuses);
    if !allowed.allowed {
        bail!(
            "run denied for issue #{}: {}",
            issue.issue_number,
            allowed.reason
        );
    }

    let manifest = prepare_session_manifest(context, github, snapshot, issue, current_status)?;
    let issue_url = format!(
        "https://github.com/{}/{}/issues/{}",
        context.repo.github_owner, context.repo.github_repo, issue.issue_number
    );
    let binary_path = std::env::current_exe().context("failed to resolve ai-teamlead binary")?;
    if let Err(error) = zellij.launch_issue_analysis(
        &context.repo,
        &context.repo.repo_root,
        &context.runtime,
        &context.config.zellij,
        &issue_url,
        &manifest.session_uuid,
        &binary_path,
        debug,
    ) {
        mark_issue_as_blocked(context, github, snapshot, issue);
        return Err(error).context("failed to launch zellij issue-analysis session");
    }

    Ok(LaunchOutcome {
        issue_number: issue.issue_number,
        session_uuid: manifest.session_uuid,
    })
}

fn prepare_session_manifest(
    context: &ExecutionContext,
    github: &GhProjectClient<'_>,
    snapshot: &ProjectSnapshot,
    issue: &ProjectIssueItem,
    current_status: &str,
) -> Result<SessionManifest> {
    let analysis_in_progress = &context
        .config
        .issue_analysis_flow
        .statuses
        .analysis_in_progress;

    if current_status == context.config.issue_analysis_flow.statuses.backlog {
        github.update_status(
            &context.repo.repo_root,
            &context.config.github.project_id,
            &issue.item_id,
            &snapshot.status_field_id,
            snapshot.option_id_by_name(analysis_in_progress)?,
        )?;
        return context.runtime.create_claim_binding(
            &context.repo,
            &context.config.github.project_id,
            &context.config.zellij,
            issue.issue_number,
        );
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
    let manifest = context
        .runtime
        .load_session_manifest(&issue_index.session_uuid)?
        .ok_or_else(|| {
            anyhow::anyhow!("missing session manifest for issue #{}", issue.issue_number)
        })?;

    github.update_status(
        &context.repo.repo_root,
        &context.config.github.project_id,
        &issue.item_id,
        &snapshot.status_field_id,
        snapshot.option_id_by_name(analysis_in_progress)?,
    )?;
    context
        .runtime
        .update_issue_flow_status(issue.issue_number, analysis_in_progress)?;

    Ok(manifest)
}

fn mark_issue_as_blocked(
    context: &ExecutionContext,
    github: &GhProjectClient<'_>,
    snapshot: &ProjectSnapshot,
    issue: &ProjectIssueItem,
) {
    if let Ok(blocked_option_id) =
        snapshot.option_id_by_name(&context.config.issue_analysis_flow.statuses.analysis_blocked)
    {
        let _ = github.update_status(
            &context.repo.repo_root,
            &context.config.github.project_id,
            &issue.item_id,
            &snapshot.status_field_id,
            blocked_option_id,
        );
    }

    let _ = context.runtime.update_issue_flow_status(
        issue.issue_number,
        &context.config.issue_analysis_flow.statuses.analysis_blocked,
    );
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
        InternalCommand::CompleteStage {
            session_uuid,
            outcome,
            message,
        } => run_complete_stage(shell, &session_uuid, &outcome, &message),
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
        &context.repo,
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
    let context = load_execution_context(shell, None)?;
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

#[cfg(test)]
mod tests {
    use super::resolve_zellij_session_name;

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
