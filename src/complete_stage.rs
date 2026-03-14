use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;

use crate::config::{Config, FlowStatuses};
use crate::github::{GhProjectClient, ProjectIssueItem};
use crate::runtime::{RuntimeLayout, SessionManifest};
use crate::shell::Shell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageOutcome {
    PlanReady,
    NeedsClarification,
    Blocked,
}

impl StageOutcome {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "plan-ready" => Ok(Self::PlanReady),
            "needs-clarification" => Ok(Self::NeedsClarification),
            "blocked" => Ok(Self::Blocked),
            other => bail!(
                "invalid outcome: {other}. Expected: plan-ready, needs-clarification, blocked"
            ),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlanReady => "plan-ready",
            Self::NeedsClarification => "needs-clarification",
            Self::Blocked => "blocked",
        }
    }

    pub fn target_status<'a>(&self, statuses: &'a FlowStatuses) -> &'a str {
        match self {
            Self::PlanReady => &statuses.waiting_for_plan_review,
            Self::NeedsClarification => &statuses.waiting_for_clarification,
            Self::Blocked => &statuses.analysis_blocked,
        }
    }
}

#[derive(Debug, Clone)]
struct ExecutionContext {
    repo_root: PathBuf,
    worktree_root: PathBuf,
    branch: String,
    artifacts_dir: String,
    message: String,
    outcome: StageOutcome,
}

pub fn run_complete_stage(
    shell: &dyn Shell,
    session_uuid: &str,
    outcome: &str,
    message: &str,
) -> Result<()> {
    let repo_root = resolve_repo_root(shell)?;
    let worktree_root = resolve_worktree_root()?;
    let context = ExecutionContext {
        repo_root,
        worktree_root,
        branch: std::env::var("AI_TEAMLEAD_ANALYSIS_BRANCH")
            .unwrap_or_else(|_| "analysis/issue-unknown".to_string()),
        artifacts_dir: std::env::var("AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR")
            .unwrap_or_else(|_| "specs/issues".to_string()),
        message: message.trim().to_string(),
        outcome: StageOutcome::parse(outcome)?,
    };
    execute_complete_stage(shell, session_uuid, context)
}

fn execute_complete_stage(
    shell: &dyn Shell,
    session_uuid: &str,
    context: ExecutionContext,
) -> Result<()> {
    if context.message.is_empty() {
        bail!("complete-stage requires a non-empty --message");
    }

    let config = Config::load_from_repo_root(&context.repo_root)?;
    let runtime = RuntimeLayout::from_repo_root(&context.repo_root);
    let manifest = runtime
        .load_session_manifest(session_uuid)?
        .ok_or_else(|| anyhow!("session not found: {session_uuid}"))?;

    if manifest.status == "completed" {
        eprintln!("complete-stage: session {session_uuid} is already completed");
        return Ok(());
    }

    let branch = if context.branch == "analysis/issue-unknown" {
        format!("analysis/issue-{}", manifest.issue_number)
    } else {
        context.branch.clone()
    };
    let artifacts_dir = if context.artifacts_dir == "specs/issues" {
        format!("specs/issues/{}", manifest.issue_number)
    } else {
        context.artifacts_dir.clone()
    };
    let commit_title = format!("analysis(#{}): {}", manifest.issue_number, context.message);

    let committed =
        git_add_and_commit(shell, &context.worktree_root, &artifacts_dir, &commit_title)?;

    if committed {
        git_push(shell, &context.worktree_root, &branch)?;
    }

    if matches!(context.outcome, StageOutcome::PlanReady) {
        let issue_url = format!(
            "https://github.com/{}/{}/issues/{}",
            manifest.github_owner, manifest.github_repo, manifest.issue_number
        );
        let artifact_files = collect_artifact_files(&context.worktree_root, &artifacts_dir)?;
        let pr_body = build_pr_body(
            &issue_url,
            context.outcome.as_str(),
            &artifacts_dir,
            &artifact_files,
        );
        create_draft_pr_if_needed(
            shell,
            &context.worktree_root,
            &branch,
            &commit_title,
            &pr_body,
        )?;
    }

    let target_status = context
        .outcome
        .target_status(&config.issue_analysis_flow.statuses);
    update_project_status(shell, &context.repo_root, &config, &manifest, target_status)?;
    runtime.update_session_status(session_uuid, "completed")?;
    runtime.update_issue_flow_status(manifest.issue_number, target_status)?;

    println!(
        "complete-stage: issue=#{} outcome={} status={}",
        manifest.issue_number,
        context.outcome.as_str(),
        target_status
    );
    Ok(())
}

fn resolve_repo_root(shell: &dyn Shell) -> Result<PathBuf> {
    if let Ok(root) = std::env::var("AI_TEAMLEAD_REPO_ROOT") {
        return Ok(PathBuf::from(root));
    }

    let cwd = std::env::current_dir().context("failed to get cwd")?;
    let output = shell.run(&cwd, "git", &["worktree", "list", "--porcelain"])?;
    let first_worktree = output
        .lines()
        .find_map(|line| line.strip_prefix("worktree "))
        .ok_or_else(|| anyhow!("cannot determine primary repo root from git worktree list"))?;
    Ok(PathBuf::from(first_worktree))
}

fn resolve_worktree_root() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("AI_TEAMLEAD_WORKTREE_ROOT") {
        return Ok(PathBuf::from(root));
    }
    std::env::current_dir().context("failed to get cwd")
}

fn git_add_and_commit(
    shell: &dyn Shell,
    worktree_root: &Path,
    artifacts_dir: &str,
    commit_message: &str,
) -> Result<bool> {
    let artifacts_path = worktree_root.join(artifacts_dir);
    if !artifacts_path.exists() {
        eprintln!("complete-stage: no artifacts directory at {artifacts_dir}, skipping commit");
        return Ok(false);
    }

    shell.run(worktree_root, "git", &["add", artifacts_dir])?;
    let staged = shell.run(
        worktree_root,
        "git",
        &["diff", "--cached", "--name-only", "--", artifacts_dir],
    )?;
    if staged.trim().is_empty() {
        eprintln!("complete-stage: no staged changes, skipping commit");
        return Ok(false);
    }

    shell.run(worktree_root, "git", &["commit", "-m", commit_message])?;
    Ok(true)
}

fn git_push(shell: &dyn Shell, worktree_root: &Path, branch: &str) -> Result<()> {
    shell
        .run(worktree_root, "git", &["push", "origin", branch])
        .context("failed to push analysis branch")?;
    Ok(())
}

fn create_draft_pr_if_needed(
    shell: &dyn Shell,
    worktree_root: &Path,
    branch: &str,
    title: &str,
    body: &str,
) -> Result<()> {
    if let Ok(existing_json) = shell.run(
        worktree_root,
        "gh",
        &["pr", "list", "--head", branch, "--json", "number,url"],
    ) {
        let existing: Vec<ExistingPr> =
            serde_json::from_str(&existing_json).context("failed to parse existing PR list")?;
        if let Some(pr) = existing.first() {
            eprintln!(
                "complete-stage: draft PR already exists for branch {}: {}",
                branch, pr.url
            );
            return Ok(());
        }
    }

    match shell.run(
        worktree_root,
        "gh",
        &["pr", "create", "--draft", "--title", title, "--body", body],
    ) {
        Ok(url) => println!("complete-stage: created draft PR: {url}"),
        Err(error) => eprintln!("complete-stage: warning: failed to create draft PR: {error}"),
    }
    Ok(())
}

fn update_project_status(
    shell: &dyn Shell,
    repo_root: &Path,
    config: &Config,
    manifest: &SessionManifest,
    target_status: &str,
) -> Result<()> {
    let github = GhProjectClient::new(shell);
    let snapshot = github.load_project_snapshot(repo_root, &manifest.project_id)?;
    let issue_item = find_project_item(&snapshot.items, manifest)?;
    let option_id = snapshot.option_id_by_name(target_status)?;

    github.update_status(
        repo_root,
        &manifest.project_id,
        &issue_item.item_id,
        &snapshot.status_field_id,
        option_id,
    )?;

    let _ = config;
    Ok(())
}

fn find_project_item<'a>(
    items: &'a [ProjectIssueItem],
    manifest: &SessionManifest,
) -> Result<&'a ProjectIssueItem> {
    items
        .iter()
        .find(|item| {
            item.issue_number == manifest.issue_number
                && item.matches_repo(&manifest.github_owner, &manifest.github_repo)
        })
        .ok_or_else(|| {
            anyhow!(
                "issue #{} not found in project snapshot",
                manifest.issue_number
            )
        })
}

fn collect_artifact_files(worktree_root: &Path, artifacts_dir: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();
    let root = worktree_root.join(artifacts_dir);
    if !root.exists() {
        return Ok(files);
    }
    collect_artifact_files_recursive(worktree_root, &root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_artifact_files_recursive(
    worktree_root: &Path,
    current: &Path,
    files: &mut Vec<String>,
) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read directory {}", current.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", current.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_artifact_files_recursive(worktree_root, &path, files)?;
            continue;
        }
        let relative = path
            .strip_prefix(worktree_root)
            .with_context(|| format!("failed to relativize {}", path.display()))?;
        files.push(relative.display().to_string());
    }
    Ok(())
}

fn build_pr_body(
    issue_url: &str,
    outcome: &str,
    artifacts_dir: &str,
    artifacts: &[String],
) -> String {
    let mut body = format!("Ref {issue_url}\n\nOutcome: {outcome}\n\nArtifacts:\n");
    if artifacts.is_empty() {
        body.push_str(&format!("- `{artifacts_dir}/`\n"));
        return body;
    }

    for artifact in artifacts {
        body.push_str(&format!("- `{artifact}`\n"));
    }
    body
}

#[derive(Debug, Deserialize)]
struct ExistingPr {
    url: String,
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::*;
    use crate::config::ZellijConfig;
    use crate::repo::RepoContext;
    use crate::runtime::RuntimeLayout;
    use tempfile::tempdir;

    #[derive(Default)]
    struct FakeShell {
        responses: BTreeMap<String, String>,
        calls: RefCell<Vec<String>>,
    }

    impl FakeShell {
        fn with_response(mut self, key: &str, value: &str) -> Self {
            self.responses.insert(key.to_string(), value.to_string());
            self
        }

        fn calls(&self) -> Vec<String> {
            self.calls.borrow().clone()
        }
    }

    impl Shell for FakeShell {
        fn run(&self, _cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
            let key = format!("{program} {}", args.join(" "));
            self.calls.borrow_mut().push(key.clone());
            self.responses
                .iter()
                .find(|(pattern, _)| key.starts_with(pattern.as_str()))
                .map(|(_, value)| value.clone())
                .ok_or_else(|| anyhow!("missing fake response for: {key}"))
        }

        fn run_with_env(
            &self,
            cwd: &Path,
            _envs: &[(&str, &str)],
            program: &str,
            args: &[&str],
        ) -> Result<String> {
            self.run(cwd, program, args)
        }

        fn spawn_with_env(
            &self,
            cwd: &Path,
            _envs: &[(&str, &str)],
            program: &str,
            args: &[&str],
            _stdout_stderr_log_path: Option<&Path>,
        ) -> Result<()> {
            self.run(cwd, program, args).map(|_| ())
        }
    }

    #[test]
    fn parses_stage_outcome_values() {
        assert!(matches!(
            StageOutcome::parse("plan-ready").expect("plan-ready"),
            StageOutcome::PlanReady
        ));
        assert!(matches!(
            StageOutcome::parse("needs-clarification").expect("needs-clarification"),
            StageOutcome::NeedsClarification
        ));
        assert!(matches!(
            StageOutcome::parse("blocked").expect("blocked"),
            StageOutcome::Blocked
        ));
    }

    #[test]
    fn maps_stage_outcome_to_target_status() {
        let statuses = FlowStatuses {
            backlog: "Backlog".into(),
            analysis_in_progress: "Analysis In Progress".into(),
            waiting_for_clarification: "Waiting for Clarification".into(),
            waiting_for_plan_review: "Waiting for Plan Review".into(),
            ready_for_implementation: "Ready for Implementation".into(),
            analysis_blocked: "Analysis Blocked".into(),
        };

        assert_eq!(
            StageOutcome::PlanReady.target_status(&statuses),
            "Waiting for Plan Review"
        );
        assert_eq!(
            StageOutcome::NeedsClarification.target_status(&statuses),
            "Waiting for Clarification"
        );
        assert_eq!(
            StageOutcome::Blocked.target_status(&statuses),
            "Analysis Blocked"
        );
    }

    #[test]
    fn completes_stage_without_git_changes_and_updates_runtime_state() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let worktree_root = temp.path().join("worktree");
        std::fs::create_dir_all(repo_root.join(".git")).expect("git dir");
        std::fs::create_dir_all(&worktree_root).expect("worktree");
        write_config(&repo_root);

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime layout");
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir: repo_root.join(".git"),
            github_owner: "dapi".into(),
            github_repo: "ai-teamlead".into(),
        };
        let manifest = runtime
            .create_claim_binding(
                &repo,
                "PVT_project",
                &ZellijConfig {
                    session_name: "teamlead".into(),
                    tab_name: "issue-analysis".into(),
                    layout: None,
                },
                15,
            )
            .expect("claim binding");

        let shell = FakeShell::default()
            .with_response(
                "gh api graphql -f query=",
                r#"{"data":{"node":{"id":"PVT_project","title":"ai-teamlead","field":{"id":"status_field","options":[{"id":"opt_progress","name":"Analysis In Progress"},{"id":"opt_clarify","name":"Waiting for Clarification"},{"id":"opt_review","name":"Waiting for Plan Review"},{"id":"opt_blocked","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM_15","fieldValueByName":{"name":"Analysis In Progress","optionId":"opt_progress"},"content":{"number":15,"state":"OPEN","repository":{"name":"ai-teamlead","owner":{"login":"dapi"}}}}]}}}}"#,
            );

        let context = ExecutionContext {
            repo_root: repo_root.clone(),
            worktree_root,
            branch: "analysis/issue-15".into(),
            artifacts_dir: "specs/issues/15".into(),
            message: "нужны ответы пользователя".into(),
            outcome: StageOutcome::NeedsClarification,
        };

        execute_complete_stage(&shell, &manifest.session_uuid, context).expect("complete stage");

        let updated_manifest = runtime
            .load_session_manifest(&manifest.session_uuid)
            .expect("manifest read")
            .expect("manifest exists");
        let updated_issue = runtime
            .load_issue_index(15)
            .expect("index read")
            .expect("index exists");

        assert_eq!(updated_manifest.status, "completed");
        assert_eq!(
            updated_issue.last_known_flow_status,
            "Waiting for Clarification"
        );
        assert!(
            shell.calls().iter().all(|call| !call.starts_with("git ")),
            "unexpected git command for no-artifacts flow"
        );
    }

    #[test]
    fn plan_ready_commits_pushes_and_creates_pr() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let worktree_root = temp.path().join("worktree");
        std::fs::create_dir_all(repo_root.join(".git")).expect("git dir");
        std::fs::create_dir_all(worktree_root.join("specs/issues/15")).expect("artifacts");
        std::fs::write(
            worktree_root.join("specs/issues/15/README.md"),
            "# result\n",
        )
        .expect("artifact file");
        write_config(&repo_root);

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime layout");
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir: repo_root.join(".git"),
            github_owner: "dapi".into(),
            github_repo: "ai-teamlead".into(),
        };
        let manifest = runtime
            .create_claim_binding(
                &repo,
                "PVT_project",
                &ZellijConfig {
                    session_name: "teamlead".into(),
                    tab_name: "issue-analysis".into(),
                    layout: None,
                },
                15,
            )
            .expect("claim binding");

        let shell = FakeShell::default()
            .with_response("git add specs/issues/15", "")
            .with_response(
                "git diff --cached --name-only -- specs/issues/15",
                "specs/issues/15/README.md",
            )
            .with_response("git commit -m analysis(#15): SDD готов", "")
            .with_response("git push origin analysis/issue-15", "")
            .with_response("gh pr list --head analysis/issue-15 --json number,url", "[]")
            .with_response(
                "gh pr create --draft --title analysis(#15): SDD готов --body Ref https://github.com/dapi/ai-teamlead/issues/15",
                "https://github.com/dapi/ai-teamlead/pull/15",
            )
            .with_response(
                "gh api graphql -f query=",
                r#"{"data":{"node":{"id":"PVT_project","title":"ai-teamlead","field":{"id":"status_field","options":[{"id":"opt_progress","name":"Analysis In Progress"},{"id":"opt_clarify","name":"Waiting for Clarification"},{"id":"opt_review","name":"Waiting for Plan Review"},{"id":"opt_blocked","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM_15","fieldValueByName":{"name":"Analysis In Progress","optionId":"opt_progress"},"content":{"number":15,"state":"OPEN","repository":{"name":"ai-teamlead","owner":{"login":"dapi"}}}}]}}}}"#,
            );

        let context = ExecutionContext {
            repo_root: repo_root.clone(),
            worktree_root,
            branch: "analysis/issue-15".into(),
            artifacts_dir: "specs/issues/15".into(),
            message: "SDD готов".into(),
            outcome: StageOutcome::PlanReady,
        };

        execute_complete_stage(&shell, &manifest.session_uuid, context).expect("complete stage");

        let calls = shell.calls();
        assert!(calls.iter().any(|call| call == "git add specs/issues/15"));
        assert!(
            calls
                .iter()
                .any(|call| call == "git push origin analysis/issue-15")
        );
        assert!(
            calls
                .iter()
                .any(|call| call
                    .starts_with("gh pr create --draft --title analysis(#15): SDD готов"))
        );

        let updated_issue = runtime
            .load_issue_index(15)
            .expect("index read")
            .expect("index exists");
        assert_eq!(
            updated_issue.last_known_flow_status,
            "Waiting for Plan Review"
        );
    }

    fn write_config(repo_root: &Path) {
        let settings_dir = repo_root.join(".ai-teamlead");
        std::fs::create_dir_all(&settings_dir).expect("settings dir");
        std::fs::write(
            settings_dir.join("settings.yml"),
            r#"github:
  project_id: "PVT_project"

issue_analysis_flow:
  statuses:
    backlog: "Backlog"
    analysis_in_progress: "Analysis In Progress"
    waiting_for_clarification: "Waiting for Clarification"
    waiting_for_plan_review: "Waiting for Plan Review"
    ready_for_implementation: "Ready for Implementation"
    analysis_blocked: "Analysis Blocked"

runtime:
  max_parallel: 1
  poll_interval_seconds: 3600

zellij:
  session_name: "teamlead"
  tab_name: "issue-analysis"

launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
"#,
        )
        .expect("config");
    }
}
