use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub github: GithubConfig,
    pub issue_analysis_flow: IssueAnalysisFlowConfig,
    pub runtime: RuntimeConfig,
    pub zellij: ZellijConfig,
    pub launch_agent: LaunchAgentConfig,
}

impl Config {
    pub fn load_from_repo_root(repo_root: &Path) -> Result<Self> {
        let path = Self::path_from_repo_root(repo_root);
        Self::load_from_path(&path)
    }

    pub fn path_from_repo_root(repo_root: &Path) -> std::path::PathBuf {
        repo_root.join(".ai-teamlead").join("settings.yml")
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;
        let config: Self = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse yaml config: {}", path.display()))?;
        config.validate(path)?;
        Ok(config)
    }

    fn validate(&self, path: &Path) -> Result<()> {
        anyhow::ensure!(
            !self.github.project_id.trim().is_empty(),
            "github.project_id must not be empty in {}",
            path.display()
        );
        anyhow::ensure!(
            self.runtime.max_parallel >= 1,
            "runtime.max_parallel must be >= 1 in {}",
            path.display()
        );
        anyhow::ensure!(
            self.runtime.poll_interval_seconds >= 1,
            "runtime.poll_interval_seconds must be >= 1 in {}",
            path.display()
        );
        anyhow::ensure!(
            !self.zellij.session_name.trim().is_empty(),
            "zellij.session_name must not be empty in {}",
            path.display()
        );
        anyhow::ensure!(
            !self.zellij.tab_name.trim().is_empty(),
            "zellij.tab_name must not be empty in {}",
            path.display()
        );
        if let Some(layout) = &self.zellij.layout {
            anyhow::ensure!(
                !layout.trim().is_empty(),
                "zellij.layout must not be empty in {}",
                path.display()
            );
        }
        anyhow::ensure!(
            !self.launch_agent.analysis_branch_template.trim().is_empty(),
            "launch_agent.analysis_branch_template must not be empty in {}",
            path.display()
        );
        anyhow::ensure!(
            !self.launch_agent.worktree_root_template.trim().is_empty(),
            "launch_agent.worktree_root_template must not be empty in {}",
            path.display()
        );
        anyhow::ensure!(
            !self
                .launch_agent
                .analysis_artifacts_dir_template
                .trim()
                .is_empty(),
            "launch_agent.analysis_artifacts_dir_template must not be empty in {}",
            path.display()
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GithubConfig {
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueAnalysisFlowConfig {
    pub statuses: FlowStatuses,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlowStatuses {
    pub backlog: String,
    pub analysis_in_progress: String,
    pub waiting_for_clarification: String,
    pub waiting_for_plan_review: String,
    pub ready_for_implementation: String,
    pub analysis_blocked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub max_parallel: usize,
    pub poll_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZellijConfig {
    pub session_name: String,
    pub tab_name: String,
    pub layout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchAgentConfig {
    pub analysis_branch_template: String,
    pub worktree_root_template: String,
    pub analysis_artifacts_dir_template: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_config() -> &'static str {
        r#"
github:
  project_id: "PVT_kwHNeaPOAUaljg"

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
  session_name: "ai-teamlead"
  tab_name: "issue-analysis"
  layout: "custom-layout"

launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
"#
    }

    #[test]
    fn parses_valid_config() {
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(sample_config()).expect("yaml should parse");
        config.validate(&path).expect("config should validate");
        assert_eq!(config.github.project_id, "PVT_kwHNeaPOAUaljg");
        assert_eq!(config.runtime.max_parallel, 1);
        assert_eq!(
            config.launch_agent.worktree_root_template,
            "${HOME}/worktrees/${REPO}/${BRANCH}"
        );
        assert_eq!(config.zellij.layout.as_deref(), Some("custom-layout"));
    }

    #[test]
    fn rejects_zero_poll_interval() {
        let yaml =
            sample_config().replace("poll_interval_seconds: 3600", "poll_interval_seconds: 0");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        let error = config.validate(&path).expect_err("validation should fail");
        assert!(error.to_string().contains("poll_interval_seconds"));
    }

    #[test]
    fn parses_config_without_optional_zellij_layout() {
        let yaml = sample_config().replace("  layout: \"custom-layout\"\n", "");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        config.validate(&path).expect("config should validate");
        assert_eq!(config.zellij.layout, None);
    }

    #[test]
    fn rejects_blank_zellij_layout() {
        let yaml = sample_config().replace("custom-layout", "   ");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        let error = config.validate(&path).expect_err("validation should fail");
        assert!(error.to_string().contains("zellij.layout"));
    }
}
