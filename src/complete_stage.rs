use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

use crate::config::{Config, FlowStatuses, ImplementationFlowStatuses};
use crate::domain::FlowStage;
use crate::github::{GhProjectClient, ProjectIssueItem};
use crate::runtime::{RuntimeLayout, SessionManifest};
use crate::shell::Shell;

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
pub enum StageOutcome {
    PlanReady,
    NeedsClarification,
    ReadyForCi,
    ReadyForReview,
    NeedsRework,
    Blocked,
}

impl StageOutcome {
    pub fn target_status<'a>(
        &self,
        stage: FlowStage,
        analysis_statuses: &'a FlowStatuses,
        implementation_statuses: &'a ImplementationFlowStatuses,
    ) -> Result<&'a str> {
        let status = match (stage, self) {
            (FlowStage::Analysis, Self::PlanReady) => &analysis_statuses.waiting_for_plan_review,
            (FlowStage::Analysis, Self::NeedsClarification) => {
                &analysis_statuses.waiting_for_clarification
            }
            (FlowStage::Analysis, Self::Blocked) => &analysis_statuses.analysis_blocked,
            (FlowStage::Implementation, Self::ReadyForCi) => {
                &implementation_statuses.waiting_for_ci
            }
            (FlowStage::Implementation, Self::ReadyForReview) => {
                &implementation_statuses.waiting_for_code_review
            }
            (FlowStage::Implementation, Self::NeedsRework) => {
                &implementation_statuses.implementation_in_progress
            }
            (FlowStage::Implementation, Self::Blocked) => {
                &implementation_statuses.implementation_blocked
            }
            _ => bail!(
                "outcome '{}' is not valid for stage '{}'",
                self.as_str(),
                stage.as_str()
            ),
        };
        Ok(status)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PlanReady => "plan-ready",
            Self::NeedsClarification => "needs-clarification",
            Self::ReadyForCi => "ready-for-ci",
            Self::ReadyForReview => "ready-for-review",
            Self::NeedsRework => "needs-rework",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone)]
struct ExecutionContext {
    stage: FlowStage,
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
    stage: &FlowStage,
    outcome: &StageOutcome,
    message: &str,
) -> Result<()> {
    let repo_root = resolve_repo_root(shell)?;
    let worktree_root = resolve_worktree_root()?;
    let context = ExecutionContext {
        stage: *stage,
        repo_root,
        worktree_root,
        branch: resolve_stage_branch(*stage),
        artifacts_dir: resolve_stage_artifacts_dir(*stage),
        message: message.trim().to_string(),
        outcome: outcome.clone(),
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

    anyhow::ensure!(
        manifest.stage == context.stage,
        "session stage mismatch: session {} belongs to '{}' but complete-stage requested '{}'",
        session_uuid,
        manifest.stage.as_str(),
        context.stage.as_str()
    );

    let branch = if context.branch == default_branch_sentinel(context.stage) {
        format!(
            "{}/issue-{}",
            stage_commit_prefix(context.stage),
            manifest.issue_number
        )
    } else {
        context.branch.clone()
    };
    let artifacts_dir = if context.artifacts_dir == "specs/issues" {
        format!("specs/issues/{}", manifest.issue_number)
    } else {
        context.artifacts_dir.clone()
    };
    let artifacts_path = context.worktree_root.join(&artifacts_dir);
    let commit_title = format!(
        "{}(#{}): {}",
        stage_commit_prefix(context.stage),
        manifest.issue_number,
        context.message
    );

    let committed =
        git_add_and_commit(shell, &context.worktree_root, &artifacts_dir, &commit_title)?;

    if committed || artifacts_path.exists() {
        git_push(shell, &context.worktree_root, &branch)?;
    }

    if matches!(
        context.outcome,
        StageOutcome::PlanReady | StageOutcome::ReadyForCi | StageOutcome::ReadyForReview
    ) {
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

    if context.stage == FlowStage::Implementation
        && matches!(context.outcome, StageOutcome::ReadyForReview)
    {
        mark_pr_ready_if_possible(shell, &context.worktree_root, &branch)?;
    }

    let target_status = context.outcome.target_status(
        context.stage,
        &config.issue_analysis_flow.statuses,
        &config.issue_implementation_flow.statuses,
    )?;
    update_project_status(shell, &context.repo_root, &manifest, target_status)?;
    runtime.update_issue_flow_status(manifest.issue_number, target_status)?;
    runtime.update_session_status(session_uuid, "completed")?;

    println!(
        "complete-stage: issue=#{} stage={} outcome={} status={}",
        manifest.issue_number,
        context.stage.as_str(),
        context.outcome.as_str(),
        target_status
    );
    Ok(())
}

fn resolve_stage_branch(stage: FlowStage) -> String {
    std::env::var("AI_TEAMLEAD_BRANCH")
        .or_else(|_| match stage {
            FlowStage::Analysis => std::env::var("AI_TEAMLEAD_ANALYSIS_BRANCH"),
            FlowStage::Implementation => std::env::var("AI_TEAMLEAD_IMPLEMENTATION_BRANCH"),
        })
        .unwrap_or_else(|_| default_branch_sentinel(stage).to_string())
}

fn resolve_stage_artifacts_dir(stage: FlowStage) -> String {
    std::env::var("AI_TEAMLEAD_ARTIFACTS_DIR")
        .or_else(|_| match stage {
            FlowStage::Analysis => std::env::var("AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR"),
            FlowStage::Implementation => std::env::var("AI_TEAMLEAD_IMPLEMENTATION_ARTIFACTS_DIR"),
        })
        .unwrap_or_else(|_| "specs/issues".to_string())
}

fn default_branch_sentinel(stage: FlowStage) -> &'static str {
    match stage {
        FlowStage::Analysis => "analysis/issue-unknown",
        FlowStage::Implementation => "implementation/issue-unknown",
    }
}

fn stage_commit_prefix(stage: FlowStage) -> &'static str {
    match stage {
        FlowStage::Analysis => "analysis",
        FlowStage::Implementation => "implementation",
    }
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
    match shell.run(
        worktree_root,
        "gh",
        &[
            "pr", "list", "--head", branch, "--json", "number", "--jq", "length",
        ],
    ) {
        Ok(count) if count.trim() != "0" => {
            eprintln!("complete-stage: draft PR already exists for branch {branch}");
            return Ok(());
        }
        Ok(_) => {} // count == 0, proceed to create
        Err(e) => {
            eprintln!("complete-stage: warning: failed to check existing PRs: {e}");
        }
    }

    let result = shell.run(
        worktree_root,
        "gh",
        &["pr", "create", "--draft", "--title", title, "--body", body],
    );
    match result {
        Ok(url) => println!("complete-stage: created draft PR: {url}"),
        Err(e) => eprintln!("complete-stage: warning: failed to create draft PR: {e}"),
    }
    Ok(())
}

fn mark_pr_ready_if_possible(shell: &dyn Shell, worktree_root: &Path, branch: &str) -> Result<()> {
    match shell.run(worktree_root, "gh", &["pr", "ready", branch]) {
        Ok(output) => {
            if !output.trim().is_empty() {
                println!("complete-stage: marked PR ready: {output}");
            }
        }
        Err(error) => {
            eprintln!("complete-stage: warning: failed to mark PR ready: {error}");
        }
    }
    Ok(())
}

fn update_project_status(
    shell: &dyn Shell,
    repo_root: &Path,
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::path::Path;

    use super::*;
    use crate::config::{ImplementationFlowStatuses, ZellijConfig};
    use crate::domain::FlowStage;
    use crate::repo::RepoContext;
    use crate::runtime::RuntimeLayout;
    use clap::ValueEnum;
    use tempfile::tempdir;

    fn sample_statuses() -> FlowStatuses {
        FlowStatuses {
            backlog: "Backlog".into(),
            analysis_in_progress: "Analysis In Progress".into(),
            waiting_for_clarification: "Waiting for Clarification".into(),
            waiting_for_plan_review: "Waiting for Plan Review".into(),
            ready_for_implementation: "Ready for Implementation".into(),
            analysis_blocked: "Analysis Blocked".into(),
        }
    }

    fn sample_implementation_statuses() -> ImplementationFlowStatuses {
        ImplementationFlowStatuses {
            ready_for_implementation: "Ready for Implementation".into(),
            implementation_in_progress: "Implementation In Progress".into(),
            waiting_for_ci: "Waiting for CI".into(),
            waiting_for_code_review: "Waiting for Code Review".into(),
            implementation_blocked: "Implementation Blocked".into(),
        }
    }

    #[test]
    fn parses_valid_outcomes_via_value_enum() {
        let variants = StageOutcome::value_variants();
        assert_eq!(variants.len(), 6);

        let plan_ready = StageOutcome::from_str("plan-ready", true).unwrap();
        assert_eq!(plan_ready, StageOutcome::PlanReady);

        let needs_clar = StageOutcome::from_str("needs-clarification", true).unwrap();
        assert_eq!(needs_clar, StageOutcome::NeedsClarification);

        let ready_for_ci = StageOutcome::from_str("ready-for-ci", true).unwrap();
        assert_eq!(ready_for_ci, StageOutcome::ReadyForCi);

        let ready_for_review = StageOutcome::from_str("ready-for-review", true).unwrap();
        assert_eq!(ready_for_review, StageOutcome::ReadyForReview);

        let needs_rework = StageOutcome::from_str("needs-rework", true).unwrap();
        assert_eq!(needs_rework, StageOutcome::NeedsRework);

        let blocked = StageOutcome::from_str("blocked", true).unwrap();
        assert_eq!(blocked, StageOutcome::Blocked);
    }

    #[test]
    fn rejects_invalid_outcome_via_value_enum() {
        let result = StageOutcome::from_str("unknown", true);
        assert!(result.is_err());
    }

    #[test]
    fn maps_outcome_to_correct_status() {
        let analysis_statuses = sample_statuses();
        let implementation_statuses = sample_implementation_statuses();
        assert_eq!(
            StageOutcome::PlanReady
                .target_status(
                    FlowStage::Analysis,
                    &analysis_statuses,
                    &implementation_statuses,
                )
                .unwrap(),
            "Waiting for Plan Review"
        );
        assert_eq!(
            StageOutcome::NeedsClarification
                .target_status(
                    FlowStage::Analysis,
                    &analysis_statuses,
                    &implementation_statuses,
                )
                .unwrap(),
            "Waiting for Clarification"
        );
        assert_eq!(
            StageOutcome::Blocked
                .target_status(
                    FlowStage::Analysis,
                    &analysis_statuses,
                    &implementation_statuses,
                )
                .unwrap(),
            "Analysis Blocked"
        );
        assert_eq!(
            StageOutcome::ReadyForCi
                .target_status(
                    FlowStage::Implementation,
                    &analysis_statuses,
                    &implementation_statuses,
                )
                .unwrap(),
            "Waiting for CI"
        );
        assert_eq!(
            StageOutcome::ReadyForReview
                .target_status(
                    FlowStage::Implementation,
                    &analysis_statuses,
                    &implementation_statuses,
                )
                .unwrap(),
            "Waiting for Code Review"
        );
        assert_eq!(
            StageOutcome::NeedsRework
                .target_status(
                    FlowStage::Implementation,
                    &analysis_statuses,
                    &implementation_statuses,
                )
                .unwrap(),
            "Implementation In Progress"
        );
        assert_eq!(
            StageOutcome::Blocked
                .target_status(
                    FlowStage::Implementation,
                    &analysis_statuses,
                    &implementation_statuses,
                )
                .unwrap(),
            "Implementation Blocked"
        );
        assert!(
            StageOutcome::PlanReady
                .target_status(
                    FlowStage::Implementation,
                    &analysis_statuses,
                    &implementation_statuses,
                )
                .is_err()
        );
    }

    #[derive(Debug, Clone)]
    enum FakeResponse {
        Ok(String),
        Err(String),
    }

    #[derive(Default)]
    struct FakeShell {
        responses: BTreeMap<String, FakeResponse>,
        calls: RefCell<Vec<String>>,
    }

    impl FakeShell {
        fn with_response(mut self, key: &str, value: &str) -> Self {
            self.responses
                .insert(key.to_string(), FakeResponse::Ok(value.to_string()));
            self
        }

        fn with_error(mut self, key: &str, value: &str) -> Self {
            self.responses
                .insert(key.to_string(), FakeResponse::Err(value.to_string()));
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
                .map(|(_, response)| response)
                .map(|response| match response {
                    FakeResponse::Ok(value) => Ok(value.clone()),
                    FakeResponse::Err(error) => Err(anyhow!(error.clone())),
                })
                .transpose()?
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
                    tab_name_template: None,
                    layout: None,
                },
                15,
                FlowStage::Analysis,
                "Analysis In Progress",
            )
            .expect("claim binding");

        let shell = FakeShell::default()
            .with_response(
                "gh api graphql -f query=",
                r#"{"data":{"node":{"id":"PVT_project","title":"ai-teamlead","field":{"id":"status_field","options":[{"id":"opt_progress","name":"Analysis In Progress"},{"id":"opt_clarify","name":"Waiting for Clarification"},{"id":"opt_review","name":"Waiting for Plan Review"},{"id":"opt_blocked","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM_15","fieldValueByName":{"name":"Analysis In Progress","optionId":"opt_progress"},"content":{"number":15,"state":"OPEN","repository":{"name":"ai-teamlead","owner":{"login":"dapi"}}}}]}}}}"#,
            );

        let context = ExecutionContext {
            stage: FlowStage::Analysis,
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
    fn retries_push_when_artifacts_exist_without_new_changes() {
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
                    tab_name_template: None,
                    layout: None,
                },
                15,
                FlowStage::Analysis,
                "Analysis In Progress",
            )
            .expect("claim binding");

        let shell = FakeShell::default()
            .with_response("git add specs/issues/15", "")
            .with_response("git diff --cached --name-only -- specs/issues/15", "")
            .with_response("git push origin analysis/issue-15", "")
            .with_response(
                "gh api graphql -f query=",
                r#"{"data":{"node":{"id":"PVT_project","title":"ai-teamlead","field":{"id":"status_field","options":[{"id":"opt_progress","name":"Analysis In Progress"},{"id":"opt_clarify","name":"Waiting for Clarification"},{"id":"opt_review","name":"Waiting for Plan Review"},{"id":"opt_blocked","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM_15","fieldValueByName":{"name":"Analysis In Progress","optionId":"opt_progress"},"content":{"number":15,"state":"OPEN","repository":{"name":"ai-teamlead","owner":{"login":"dapi"}}}}]}}}}"#,
            );

        let context = ExecutionContext {
            stage: FlowStage::Analysis,
            repo_root: repo_root.clone(),
            worktree_root,
            branch: "analysis/issue-15".into(),
            artifacts_dir: "specs/issues/15".into(),
            message: "нужны ответы пользователя".into(),
            outcome: StageOutcome::NeedsClarification,
        };

        execute_complete_stage(&shell, &manifest.session_uuid, context).expect("complete stage");

        let calls = shell.calls();
        assert!(
            calls
                .iter()
                .any(|call| call == "git push origin analysis/issue-15")
        );

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
                    tab_name_template: None,
                    layout: None,
                },
                15,
                FlowStage::Analysis,
                "Analysis In Progress",
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
            .with_response(
                "gh pr list --head analysis/issue-15 --json number --jq length",
                "0",
            )
            .with_response(
                "gh pr create --draft --title analysis(#15): SDD готов --body Ref https://github.com/dapi/ai-teamlead/issues/15",
                "https://github.com/dapi/ai-teamlead/pull/15",
            )
            .with_response(
                "gh api graphql -f query=",
                r#"{"data":{"node":{"id":"PVT_project","title":"ai-teamlead","field":{"id":"status_field","options":[{"id":"opt_progress","name":"Analysis In Progress"},{"id":"opt_clarify","name":"Waiting for Clarification"},{"id":"opt_review","name":"Waiting for Plan Review"},{"id":"opt_blocked","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM_15","fieldValueByName":{"name":"Analysis In Progress","optionId":"opt_progress"},"content":{"number":15,"state":"OPEN","repository":{"name":"ai-teamlead","owner":{"login":"dapi"}}}}]}}}}"#,
            );

        let context = ExecutionContext {
            stage: FlowStage::Analysis,
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

    #[test]
    fn plan_ready_still_attempts_pr_create_when_pr_list_fails() {
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
                    tab_name_template: None,
                    layout: None,
                },
                15,
                FlowStage::Analysis,
                "Analysis In Progress",
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
            .with_error(
                "gh pr list --head analysis/issue-15 --json number --jq length",
                "transient gh failure",
            )
            .with_response(
                "gh pr create --draft --title analysis(#15): SDD готов --body Ref https://github.com/dapi/ai-teamlead/issues/15",
                "https://github.com/dapi/ai-teamlead/pull/15",
            )
            .with_response(
                "gh api graphql -f query=",
                r#"{"data":{"node":{"id":"PVT_project","title":"ai-teamlead","field":{"id":"status_field","options":[{"id":"opt_progress","name":"Analysis In Progress"},{"id":"opt_clarify","name":"Waiting for Clarification"},{"id":"opt_review","name":"Waiting for Plan Review"},{"id":"opt_blocked","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM_15","fieldValueByName":{"name":"Analysis In Progress","optionId":"opt_progress"},"content":{"number":15,"state":"OPEN","repository":{"name":"ai-teamlead","owner":{"login":"dapi"}}}}]}}}}"#,
            );

        let context = ExecutionContext {
            stage: FlowStage::Analysis,
            repo_root: repo_root.clone(),
            worktree_root,
            branch: "analysis/issue-15".into(),
            artifacts_dir: "specs/issues/15".into(),
            message: "SDD готов".into(),
            outcome: StageOutcome::PlanReady,
        };

        execute_complete_stage(&shell, &manifest.session_uuid, context).expect("complete stage");

        let calls = shell.calls();
        assert!(
            calls
                .iter()
                .any(|call| call
                    == "gh pr list --head analysis/issue-15 --json number --jq length")
        );
        assert!(
            calls
                .iter()
                .any(|call| call
                    .starts_with("gh pr create --draft --title analysis(#15): SDD готов"))
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

issue_implementation_flow:
  statuses:
    ready_for_implementation: "Ready for Implementation"
    implementation_in_progress: "Implementation In Progress"
    waiting_for_ci: "Waiting for CI"
    waiting_for_code_review: "Waiting for Code Review"
    implementation_blocked: "Implementation Blocked"

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
  implementation_branch_template: "implementation/issue-${ISSUE_NUMBER}"
  implementation_worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  implementation_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
"#,
        )
        .expect("config");
    }
}
