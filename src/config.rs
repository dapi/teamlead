use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub github: GithubConfig,
    pub issue_analysis_flow: IssueAnalysisFlowConfig,
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
        Self::load_from_str(&content, path)
    }

    fn load_from_str(content: &str, path: &Path) -> Result<Self> {
        let raw: Option<RawConfig> = serde_yaml::from_str(content)
            .with_context(|| format!("failed to parse yaml config: {}", path.display()))?;
        let config = raw.unwrap_or_default().into_config();
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

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
struct RawConfig {
    github: RawGithubConfig,
    issue_analysis_flow: IssueAnalysisFlowConfig,
    issue_implementation_flow: IssueImplementationFlowConfig,
    runtime: RuntimeConfig,
    zellij: ZellijConfig,
    launch_agent: LaunchAgentConfig,
}

impl RawConfig {
    fn into_config(self) -> Config {
        Config {
            github: GithubConfig {
                project_id: self.github.project_id,
            },
            issue_analysis_flow: self.issue_analysis_flow,
            issue_implementation_flow: self.issue_implementation_flow,
            runtime: self.runtime,
            zellij: self.zellij,
            launch_agent: self.launch_agent,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
struct RawGithubConfig {
    project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GithubConfig {
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueAnalysisFlowConfig {
    #[serde(default)]
    pub statuses: FlowStatuses,
}

impl Default for IssueAnalysisFlowConfig {
    fn default() -> Self {
        Self {
            statuses: FlowStatuses::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueImplementationFlowConfig {
    #[serde(default)]
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
    #[serde(default = "default_analysis_status_backlog")]
    pub backlog: String,
    #[serde(default = "default_analysis_status_in_progress")]
    pub analysis_in_progress: String,
    #[serde(default = "default_analysis_status_waiting_for_clarification")]
    pub waiting_for_clarification: String,
    #[serde(default = "default_analysis_status_waiting_for_plan_review")]
    pub waiting_for_plan_review: String,
    #[serde(default = "default_analysis_status_ready_for_implementation")]
    pub ready_for_implementation: String,
    #[serde(default = "default_analysis_status_blocked")]
    pub analysis_blocked: String,
}

impl Default for FlowStatuses {
    fn default() -> Self {
        Self {
            backlog: default_analysis_status_backlog(),
            analysis_in_progress: default_analysis_status_in_progress(),
            waiting_for_clarification: default_analysis_status_waiting_for_clarification(),
            waiting_for_plan_review: default_analysis_status_waiting_for_plan_review(),
            ready_for_implementation: default_analysis_status_ready_for_implementation(),
            analysis_blocked: default_analysis_status_blocked(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImplementationFlowStatuses {
    #[serde(default = "default_implementation_status_ready_for_implementation")]
    pub ready_for_implementation: String,
    #[serde(default = "default_implementation_status_in_progress")]
    pub implementation_in_progress: String,
    #[serde(default = "default_implementation_status_waiting_for_ci")]
    pub waiting_for_ci: String,
    #[serde(default = "default_implementation_status_waiting_for_code_review")]
    pub waiting_for_code_review: String,
    #[serde(default = "default_done_status")]
    pub done: String,
    #[serde(default = "default_implementation_status_blocked")]
    pub implementation_blocked: String,
}

impl Default for ImplementationFlowStatuses {
    fn default() -> Self {
        Self {
            ready_for_implementation: default_implementation_status_ready_for_implementation(),
            implementation_in_progress: default_implementation_status_in_progress(),
            waiting_for_ci: default_implementation_status_waiting_for_ci(),
            waiting_for_code_review: default_implementation_status_waiting_for_code_review(),
            done: default_done_status(),
            implementation_blocked: default_implementation_status_blocked(),
        }
    }
}

fn default_done_status() -> String {
    "Done".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeConfig {
    #[serde(default = "default_runtime_max_parallel")]
    pub max_parallel: usize,
    #[serde(default = "default_runtime_poll_interval_seconds")]
    pub poll_interval_seconds: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_parallel: default_runtime_max_parallel(),
            poll_interval_seconds: default_runtime_poll_interval_seconds(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ValueEnum, Default)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lowercase")]
pub enum LaunchTarget {
    Pane,
    #[default]
    Tab,
}

impl LaunchTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pane => "pane",
            Self::Tab => "tab",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZellijConfig {
    #[serde(default = "default_zellij_session_name")]
    pub session_name: String,
    #[serde(default = "default_zellij_tab_name")]
    pub tab_name: String,
    #[serde(default = "default_zellij_launch_target")]
    pub launch_target: LaunchTarget,
    #[serde(default)]
    pub tab_name_template: Option<String>,
    #[serde(default = "default_zellij_layout")]
    pub layout: Option<String>,
}

impl Default for ZellijConfig {
    fn default() -> Self {
        Self {
            session_name: default_zellij_session_name(),
            tab_name: default_zellij_tab_name(),
            launch_target: default_zellij_launch_target(),
            tab_name_template: None,
            layout: default_zellij_layout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchAgentConfig {
    #[serde(default = "default_analysis_branch_template")]
    pub analysis_branch_template: String,
    #[serde(default = "default_worktree_root_template")]
    pub worktree_root_template: String,
    #[serde(default = "default_analysis_artifacts_dir_template")]
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

impl Default for LaunchAgentConfig {
    fn default() -> Self {
        Self {
            analysis_branch_template: default_analysis_branch_template(),
            worktree_root_template: default_worktree_root_template(),
            analysis_artifacts_dir_template: default_analysis_artifacts_dir_template(),
            global_args: LaunchAgentGlobalArgsConfig::default(),
            implementation_branch_template: default_implementation_branch_template(),
            implementation_worktree_root_template: default_implementation_worktree_root_template(),
            implementation_artifacts_dir_template: default_implementation_artifacts_dir_template(),
        }
    }
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

fn default_analysis_status_backlog() -> String {
    "Backlog".into()
}

fn default_analysis_status_in_progress() -> String {
    "Analysis In Progress".into()
}

fn default_analysis_status_waiting_for_clarification() -> String {
    "Waiting for Clarification".into()
}

fn default_analysis_status_waiting_for_plan_review() -> String {
    "Waiting for Plan Review".into()
}

fn default_analysis_status_ready_for_implementation() -> String {
    "Ready for Implementation".into()
}

fn default_analysis_status_blocked() -> String {
    "Analysis Blocked".into()
}

fn default_implementation_status_ready_for_implementation() -> String {
    "Ready for Implementation".into()
}

fn default_implementation_status_in_progress() -> String {
    "Implementation In Progress".into()
}

fn default_implementation_status_waiting_for_ci() -> String {
    "Waiting for CI".into()
}

fn default_implementation_status_waiting_for_code_review() -> String {
    "Waiting for Code Review".into()
}

fn default_implementation_status_blocked() -> String {
    "Implementation Blocked".into()
}

fn default_runtime_max_parallel() -> usize {
    1
}

fn default_runtime_poll_interval_seconds() -> u64 {
    3600
}

fn default_zellij_session_name() -> String {
    "${REPO}".into()
}

fn default_zellij_tab_name() -> String {
    "issue-analysis".into()
}

fn default_zellij_launch_target() -> LaunchTarget {
    LaunchTarget::Tab
}

fn default_zellij_layout() -> Option<String> {
    None
}

fn default_analysis_branch_template() -> String {
    "analysis/issue-${ISSUE_NUMBER}".into()
}

fn default_worktree_root_template() -> String {
    "${HOME}/worktrees/${REPO}/${BRANCH}".into()
}

fn default_analysis_artifacts_dir_template() -> String {
    "specs/issues/${ISSUE_NUMBER}".into()
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
    use serde_yaml::Value;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum FieldKind {
        RequiredWithoutDefault,
        DefaultedByApplication,
        ExampleOnlyExtension,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct FieldContract {
        key: &'static str,
        kind: FieldKind,
        runtime_default: Option<&'static str>,
        template_line: &'static str,
    }

    const SETTINGS_TEMPLATE: &str = include_str!("../templates/init/settings.yml");
    const FIELD_CONTRACTS: [FieldContract; 29] = [
        FieldContract {
            key: "github.project_id",
            kind: FieldKind::RequiredWithoutDefault,
            runtime_default: None,
            template_line: "#   project_id: \"PVT_replace_me\"",
        },
        FieldContract {
            key: "issue_analysis_flow.statuses.backlog",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Backlog"),
            template_line: "#     backlog: \"Backlog\"",
        },
        FieldContract {
            key: "issue_analysis_flow.statuses.analysis_in_progress",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Analysis In Progress"),
            template_line: "#     analysis_in_progress: \"Analysis In Progress\"",
        },
        FieldContract {
            key: "issue_analysis_flow.statuses.waiting_for_clarification",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Waiting for Clarification"),
            template_line: "#     waiting_for_clarification: \"Waiting for Clarification\"",
        },
        FieldContract {
            key: "issue_analysis_flow.statuses.waiting_for_plan_review",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Waiting for Plan Review"),
            template_line: "#     waiting_for_plan_review: \"Waiting for Plan Review\"",
        },
        FieldContract {
            key: "issue_analysis_flow.statuses.ready_for_implementation",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Ready for Implementation"),
            template_line: "#     ready_for_implementation: \"Ready for Implementation\"",
        },
        FieldContract {
            key: "issue_analysis_flow.statuses.analysis_blocked",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Analysis Blocked"),
            template_line: "#     analysis_blocked: \"Analysis Blocked\"",
        },
        FieldContract {
            key: "issue_implementation_flow.statuses.ready_for_implementation",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Ready for Implementation"),
            template_line: "#     ready_for_implementation: \"Ready for Implementation\"",
        },
        FieldContract {
            key: "issue_implementation_flow.statuses.implementation_in_progress",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Implementation In Progress"),
            template_line: "#     implementation_in_progress: \"Implementation In Progress\"",
        },
        FieldContract {
            key: "issue_implementation_flow.statuses.waiting_for_ci",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Waiting for CI"),
            template_line: "#     waiting_for_ci: \"Waiting for CI\"",
        },
        FieldContract {
            key: "issue_implementation_flow.statuses.waiting_for_code_review",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Waiting for Code Review"),
            template_line: "#     waiting_for_code_review: \"Waiting for Code Review\"",
        },
        FieldContract {
            key: "issue_implementation_flow.statuses.done",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Done"),
            template_line: "#     done: \"Done\"",
        },
        FieldContract {
            key: "issue_implementation_flow.statuses.implementation_blocked",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("Implementation Blocked"),
            template_line: "#     implementation_blocked: \"Implementation Blocked\"",
        },
        FieldContract {
            key: "runtime.max_parallel",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("1"),
            template_line: "#   max_parallel: 1",
        },
        FieldContract {
            key: "runtime.poll_interval_seconds",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("3600"),
            template_line: "#   poll_interval_seconds: 3600",
        },
        FieldContract {
            key: "zellij.session_name",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("${REPO}"),
            template_line: "#   session_name: \"${REPO}\"",
        },
        FieldContract {
            key: "zellij.tab_name",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("issue-analysis"),
            template_line: "#   tab_name: \"issue-analysis\"",
        },
        FieldContract {
            key: "zellij.launch_target",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("tab"),
            template_line: "#   launch_target: \"tab\"",
        },
        FieldContract {
            key: "zellij.tab_name_template",
            kind: FieldKind::ExampleOnlyExtension,
            runtime_default: None,
            template_line: "#   tab_name_template: \"#${ISSUE_NUMBER}\"",
        },
        FieldContract {
            key: "zellij.layout",
            kind: FieldKind::ExampleOnlyExtension,
            runtime_default: None,
            template_line: "#   layout: \"compact\"",
        },
        FieldContract {
            key: "launch_agent.analysis_branch_template",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("analysis/issue-${ISSUE_NUMBER}"),
            template_line: "#   analysis_branch_template: \"analysis/issue-${ISSUE_NUMBER}\"",
        },
        FieldContract {
            key: "launch_agent.worktree_root_template",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("${HOME}/worktrees/${REPO}/${BRANCH}"),
            template_line: "#   worktree_root_template: \"${HOME}/worktrees/${REPO}/${BRANCH}\"",
        },
        FieldContract {
            key: "launch_agent.analysis_artifacts_dir_template",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("specs/issues/${ISSUE_NUMBER}"),
            template_line: "#   analysis_artifacts_dir_template: \"specs/issues/${ISSUE_NUMBER}\"",
        },
        FieldContract {
            key: "launch_agent.global_args.claude[0]",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("--permission-mode"),
            template_line: "#       - \"--permission-mode\"",
        },
        FieldContract {
            key: "launch_agent.global_args.claude[1]",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("auto"),
            template_line: "#       - \"auto\"",
        },
        FieldContract {
            key: "launch_agent.global_args.codex[0]",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("--full-auto"),
            template_line: "#       - \"--full-auto\"",
        },
        FieldContract {
            key: "launch_agent.implementation_branch_template",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("implementation/issue-${ISSUE_NUMBER}"),
            template_line: "#   implementation_branch_template: \"implementation/issue-${ISSUE_NUMBER}\"",
        },
        FieldContract {
            key: "launch_agent.implementation_worktree_root_template",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("${HOME}/worktrees/${REPO}/${BRANCH}"),
            template_line: "#   implementation_worktree_root_template: \"${HOME}/worktrees/${REPO}/${BRANCH}\"",
        },
        FieldContract {
            key: "launch_agent.implementation_artifacts_dir_template",
            kind: FieldKind::DefaultedByApplication,
            runtime_default: Some("specs/issues/${ISSUE_NUMBER}"),
            template_line: "#   implementation_artifacts_dir_template: \"specs/issues/${ISSUE_NUMBER}\"",
        },
    ];

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
    done: "Done"
    implementation_blocked: "Implementation Blocked"

runtime:
  max_parallel: 1
  poll_interval_seconds: 3600

zellij:
  session_name: "ai-teamlead"
  tab_name: "issue-analysis"
  launch_target: "tab"
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

    fn runtime_defaults_with_project(project_id: &str) -> Config {
        Config {
            github: GithubConfig {
                project_id: project_id.into(),
            },
            issue_analysis_flow: IssueAnalysisFlowConfig::default(),
            issue_implementation_flow: IssueImplementationFlowConfig::default(),
            runtime: RuntimeConfig::default(),
            zellij: ZellijConfig::default(),
            launch_agent: LaunchAgentConfig::default(),
        }
    }

    fn flatten_yaml_scalars(value: &Value) -> BTreeMap<String, String> {
        fn walk(value: &Value, prefix: Option<String>, out: &mut BTreeMap<String, String>) {
            match value {
                Value::Mapping(mapping) => {
                    for (key, value) in mapping {
                        let Some(key) = key.as_str() else {
                            continue;
                        };
                        let next = match &prefix {
                            Some(prefix) => format!("{prefix}.{key}"),
                            None => key.to_string(),
                        };
                        walk(value, Some(next), out);
                    }
                }
                Value::String(value) => {
                    if let Some(prefix) = prefix {
                        out.insert(prefix, value.clone());
                    }
                }
                Value::Number(value) => {
                    if let Some(prefix) = prefix {
                        out.insert(prefix, value.to_string());
                    }
                }
                Value::Bool(value) => {
                    if let Some(prefix) = prefix {
                        out.insert(prefix, value.to_string());
                    }
                }
                Value::Sequence(sequence) => {
                    for (index, value) in sequence.iter().enumerate() {
                        let Some(prefix) = &prefix else {
                            continue;
                        };
                        walk(value, Some(format!("{prefix}[{index}]")), out);
                    }
                }
                Value::Null | Value::Tagged(_) => {}
            }
        }

        let mut out = BTreeMap::new();
        walk(value, None, &mut out);
        out
    }

    #[test]
    fn parses_valid_config() {
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config = Config::load_from_str(sample_config(), &path).expect("yaml should parse");
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
        assert_eq!(config.zellij.launch_target, LaunchTarget::Tab);
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
        let error = Config::load_from_str(&yaml, &path).expect_err("validation should fail");
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
        let config = Config::load_from_str(yaml, &path).expect("legacy config should validate");
        assert_eq!(
            config.issue_implementation_flow.statuses.waiting_for_ci,
            "Waiting for CI"
        );
        assert_eq!(
            config.launch_agent.implementation_branch_template,
            "implementation/issue-${ISSUE_NUMBER}"
        );
        assert_eq!(config.zellij.layout, None);
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
    fn loads_minimal_yaml_with_runtime_defaults() {
        let yaml = r#"
github:
  project_id: "PVT_kwHNeaPOAUaljg"
"#;
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config = Config::load_from_str(yaml, &path).expect("minimal yaml should validate");
        assert_eq!(config.runtime.max_parallel, 1);
        assert_eq!(config.runtime.poll_interval_seconds, 3600);
        assert_eq!(config.zellij.session_name, "${REPO}");
        assert_eq!(config.zellij.tab_name, "issue-analysis");
        assert_eq!(config.zellij.launch_target, LaunchTarget::Tab);
        assert_eq!(config.zellij.tab_name_template, None);
        assert_eq!(config.zellij.layout, None);
        assert_eq!(
            config.launch_agent.analysis_branch_template,
            "analysis/issue-${ISSUE_NUMBER}"
        );
        assert_eq!(
            config.launch_agent.global_args.claude,
            vec!["--permission-mode".to_string(), "auto".to_string()]
        );
        assert_eq!(
            config.launch_agent.global_args.codex,
            vec!["--full-auto".to_string()]
        );
    }

    #[test]
    fn rejects_comment_only_yaml_without_required_project_id() {
        let yaml = r#"
# github:
#   project_id: "PVT_replace_me"
"#;
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let error = Config::load_from_str(yaml, &path).expect_err("validation should fail");
        assert!(error.to_string().contains("github.project_id"));
    }

    #[test]
    fn parses_partial_override_on_top_of_defaults() {
        let yaml = r#"
github:
  project_id: "PVT_kwHNeaPOAUaljg"

runtime:
  poll_interval_seconds: 60

zellij:
  session_name: "custom-session"
"#;
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config = Config::load_from_str(yaml, &path).expect("yaml should validate");
        assert_eq!(config.runtime.max_parallel, 1);
        assert_eq!(config.runtime.poll_interval_seconds, 60);
        assert_eq!(config.zellij.session_name, "custom-session");
        assert_eq!(config.zellij.tab_name, "issue-analysis");
        assert_eq!(config.zellij.launch_target, LaunchTarget::Tab);
        assert_eq!(config.zellij.tab_name_template, None);
        assert_eq!(config.zellij.layout, None);
        assert_eq!(
            config.launch_agent.global_args.codex,
            vec!["--full-auto".to_string()]
        );
    }

    #[test]
    fn allows_overriding_agent_global_args() {
        let yaml = sample_config().replace(
            "  global_args:\n    claude:\n      - \"--permission-mode\"\n      - \"auto\"\n    codex:\n      - \"--full-auto\"\n",
            "  global_args:\n    claude:\n      - \"--dangerously-skip-permissions\"\n    codex:\n      - \"--sandbox\"\n      - \"workspace-write\"\n",
        );
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config = Config::load_from_str(&yaml, &path).expect("yaml should validate");

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
    fn parses_config_without_optional_tab_name_template() {
        let yaml = sample_config().replace("  tab_name_template: \"#${ISSUE_NUMBER}\"\n", "");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config: Config = serde_yaml::from_str(&yaml).expect("yaml should parse");
        config.validate(&path).expect("config should validate");
        assert_eq!(config.zellij.tab_name_template, None);
    }

    #[test]
    fn parses_config_without_launch_target_as_runtime_default_tab() {
        let yaml = sample_config().replace("  launch_target: \"tab\"\n", "");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let config = Config::load_from_str(&yaml, &path).expect("yaml should parse");
        assert_eq!(config.zellij.launch_target, LaunchTarget::Tab);
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
    fn rejects_invalid_zellij_launch_target() {
        let yaml = sample_config().replace("launch_target: \"tab\"", "launch_target: \"split\"");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        Config::load_from_str(&yaml, &path).expect_err("validation should fail");
    }

    #[test]
    fn rejects_blank_zellij_layout() {
        let yaml = sample_config().replace("custom-layout", "   ");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let error = Config::load_from_str(&yaml, &path).expect_err("validation should fail");
        assert!(error.to_string().contains("zellij.layout"));
    }

    #[test]
    fn contract_metadata_covers_every_config_key() {
        let config = runtime_defaults_with_project("PVT_placeholder");
        let value = serde_yaml::to_value(config).expect("config should serialize");
        let actual_keys: BTreeSet<_> = flatten_yaml_scalars(&value).into_keys().collect();
        let runtime_backed_keys: BTreeSet<_> = FIELD_CONTRACTS
            .iter()
            .filter(|field| field.kind != FieldKind::ExampleOnlyExtension)
            .map(|field| field.key.to_string())
            .collect();
        assert_eq!(runtime_backed_keys, actual_keys);

        let example_only_keys: BTreeSet<_> = FIELD_CONTRACTS
            .iter()
            .filter(|field| field.kind == FieldKind::ExampleOnlyExtension)
            .map(|field| field.key.to_string())
            .collect();
        assert!(
            !example_only_keys.is_empty(),
            "contract must explicitly model allowed example-only exceptions",
        );
        assert!(
            actual_keys.is_disjoint(&example_only_keys),
            "example-only extensions must stay outside runtime defaults until explicitly enabled"
        );
    }

    #[test]
    fn template_defaults_match_runtime_defaults() {
        let config = runtime_defaults_with_project("PVT_placeholder");
        let value = serde_yaml::to_value(config).expect("config should serialize");
        let flattened = flatten_yaml_scalars(&value);

        for field in FIELD_CONTRACTS {
            assert!(
                SETTINGS_TEMPLATE.contains(field.template_line),
                "settings template must document {}",
                field.key
            );

            if field.kind == FieldKind::DefaultedByApplication {
                assert_eq!(
                    flattened.get(field.key).map(String::as_str),
                    field.runtime_default,
                    "runtime default mismatch for {}",
                    field.key
                );
            }
        }
    }

    #[test]
    fn rejects_blank_global_args() {
        let yaml = sample_config().replace("\"--full-auto\"", "\"   \"");
        let path = PathBuf::from("/tmp/.ai-teamlead/settings.yml");
        let error = Config::load_from_str(&yaml, &path).expect_err("validation should fail");
        assert!(
            error
                .to_string()
                .contains("launch_agent.global_args.codex[0]")
        );
    }
}
