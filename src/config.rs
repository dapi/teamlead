use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub github: GithubConfig,
    pub issue_analysis_flow: IssueAnalysisFlowConfig,
    #[serde(default)]
    pub issue_implementation_flow: IssueImplementationFlowConfig,
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
        if let Some(tab_name_template) = &self.zellij.tab_name_template {
            anyhow::ensure!(
                !tab_name_template.trim().is_empty(),
                "zellij.tab_name_template must not be empty in {}",
                path.display()
            );
        }
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
        validate_global_args(path, "claude", &self.launch_agent.global_args.claude)?;
        validate_global_args(path, "codex", &self.launch_agent.global_args.codex)?;
        anyhow::ensure!(
            !self
                .launch_agent
                .implementation_branch_template
                .trim()
                .is_empty(),
            "launch_agent.implementation_branch_template must not be empty in {}",
            path.display()
        );
        anyhow::ensure!(
            !self
                .launch_agent
                .implementation_worktree_root_template
                .trim()
                .is_empty(),
            "launch_agent.implementation_worktree_root_template must not be empty in {}",
            path.display()
        );
        anyhow::ensure!(
            !self
                .launch_agent
                .implementation_artifacts_dir_template
                .trim()
                .is_empty(),
            "launch_agent.implementation_artifacts_dir_template must not be empty in {}",
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
pub struct IssueImplementationFlowConfig {
    pub statuses: ImplementationFlowStatuses,
}

impl Default for IssueImplementationFlowConfig {
    fn default() -> Self {
        Self {
            statuses: ImplementationFlowStatuses::default(),
        }
    }
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
pub struct ImplementationFlowStatuses {
    pub ready_for_implementation: String,
    pub implementation_in_progress: String,
    pub waiting_for_ci: String,
    pub waiting_for_code_review: String,
    pub implementation_blocked: String,
}

impl Default for ImplementationFlowStatuses {
    fn default() -> Self {
        Self {
            ready_for_implementation: "Ready for Implementation".into(),
            implementation_in_progress: "Implementation In Progress".into(),
            waiting_for_ci: "Waiting for CI".into(),
            waiting_for_code_review: "Waiting for Code Review".into(),
            implementation_blocked: "Implementation Blocked".into(),
        }
    }
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
    pub tab_name_template: Option<String>,
    pub layout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchAgentConfig {
    pub analysis_branch_template: String,
    pub worktree_root_template: String,
    pub analysis_artifacts_dir_template: String,
    #[serde(default)]
    pub global_args: LaunchAgentGlobalArgsConfig,
    #[serde(default = "default_implementation_branch_template")]
    pub implementation_branch_template: String,
    #[serde(default = "default_implementation_worktree_root_template")]
    pub implementation_worktree_root_template: String,
    #[serde(default = "default_implementation_artifacts_dir_template")]
    pub implementation_artifacts_dir_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchAgentGlobalArgsConfig {
    #[serde(default = "default_claude_global_args")]
    pub claude: Vec<String>,
    #[serde(default = "default_codex_global_args")]
    pub codex: Vec<String>,
}

impl Default for LaunchAgentGlobalArgsConfig {
    fn default() -> Self {
        Self {
            claude: default_claude_global_args(),
            codex: default_codex_global_args(),
        }
    }
}

fn default_claude_global_args() -> Vec<String> {
    vec!["--permission-mode".into(), "auto".into()]
}

fn default_codex_global_args() -> Vec<String> {
    vec!["--full-auto".into()]
}

fn default_implementation_branch_template() -> String {
    "implementation/issue-${ISSUE_NUMBER}".into()
}

fn default_implementation_worktree_root_template() -> String {
    "${HOME}/worktrees/${REPO}/${BRANCH}".into()
}

fn default_implementation_artifacts_dir_template() -> String {
    "specs/issues/${ISSUE_NUMBER}".into()
}

fn validate_global_args(path: &Path, agent_name: &str, args: &[String]) -> Result<()> {
    for (index, arg) in args.iter().enumerate() {
        anyhow::ensure!(
            !arg.trim().is_empty(),
            "launch_agent.global_args.{agent_name}[{index}] must not be blank in {}",
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_config() -> &'static str {
        r##"
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
  session_name: "ai-teamlead"
  tab_name: "issue-analysis"
  tab_name_template: "#${ISSUE_NUMBER}"
  layout: "custom-layout"

launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
  global_args:
    claude:
      - "--permission-mode"
      - "auto"
    codex:
      - "--full-auto"
  implementation_branch_template: "implementation/issue-${ISSUE_NUMBER}"
  implementation_worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  implementation_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
"##
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
        assert_eq!(
            config.zellij.tab_name_template.as_deref(),
            Some("#${ISSUE_NUMBER}")
        );
        assert_eq!(config.zellij.layout.as_deref(), Some("custom-layout"));
        assert_eq!(
            config.launch_agent.global_args.codex,
            vec!["--full-auto".to_string()]
        );
        assert_eq!(
            config.launch_agent.global_args.claude,
            vec!["--permission-mode".to_string(), "auto".to_string()]
        );
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
    fn parses_legacy_config_without_implementation_fields() {
        let yaml = r#"
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

launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
"#;
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(yaml).expect("yaml should parse");
        config
            .validate(&path)
            .expect("legacy config should validate");
        assert_eq!(
            config.issue_implementation_flow.statuses.waiting_for_ci,
            "Waiting for CI"
        );
        assert_eq!(
            config.launch_agent.implementation_branch_template,
            "implementation/issue-${ISSUE_NUMBER}"
        );
        assert_eq!(
            config.launch_agent.global_args.codex,
            vec!["--full-auto".to_string()]
        );
        assert_eq!(
            config.launch_agent.global_args.claude,
            vec!["--permission-mode".to_string(), "auto".to_string()]
        );
    }

    #[test]
    fn allows_overriding_agent_global_args() {
        let yaml = sample_config().replace(
            "  global_args:\n    claude:\n      - \"--permission-mode\"\n      - \"auto\"\n    codex:\n      - \"--full-auto\"\n",
            "  global_args:\n    claude:\n      - \"--dangerously-skip-permissions\"\n    codex:\n      - \"--sandbox\"\n      - \"workspace-write\"\n",
        );
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        config.validate(&path).expect("config should validate");

        assert_eq!(
            config.launch_agent.global_args.claude,
            vec!["--dangerously-skip-permissions".to_string()]
        );
        assert_eq!(
            config.launch_agent.global_args.codex,
            vec!["--sandbox".to_string(), "workspace-write".to_string()]
        );
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
    fn parses_config_without_optional_tab_name_template() {
        let yaml = sample_config().replace("  tab_name_template: \"#${ISSUE_NUMBER}\"\n", "");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        config.validate(&path).expect("config should validate");
        assert_eq!(config.zellij.tab_name_template, None);
    }

    #[test]
    fn rejects_blank_zellij_tab_name_template() {
        let yaml = sample_config().replace("#${ISSUE_NUMBER}", "   ");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        let error = config.validate(&path).expect_err("validation should fail");
        assert!(error.to_string().contains("zellij.tab_name_template"));
    }

    #[test]
    fn rejects_blank_zellij_layout() {
        let yaml = sample_config().replace("custom-layout", "   ");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        let error = config.validate(&path).expect_err("validation should fail");
        assert!(error.to_string().contains("zellij.layout"));
    }

    #[test]
    fn rejects_blank_global_args() {
        let yaml = sample_config().replace("\"--full-auto\"", "\"   \"");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        let error = config.validate(&path).expect_err("validation should fail");
        assert!(
            error
                .to_string()
                .contains("launch_agent.global_args.codex[0]")
        );
    }
}
