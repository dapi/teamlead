use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPaths {
    pub repo_root: PathBuf,
    pub customization_root: PathBuf,
    pub zellij_dir: PathBuf,
    pub analysis_tab_template_path: PathBuf,
    pub settings_path: PathBuf,
    pub project_init_path: PathBuf,
    pub launch_agent_path: PathBuf,
    pub flows_dir: PathBuf,
    pub issue_analysis_flow_path: PathBuf,
    pub issue_analysis_dir: PathBuf,
    pub issue_analysis_readme_path: PathBuf,
    pub issue_analysis_what_path: PathBuf,
    pub issue_analysis_how_path: PathBuf,
    pub issue_analysis_verify_path: PathBuf,
    pub readme_path: PathBuf,
    pub claude_root: PathBuf,
    pub claude_readme_path: PathBuf,
    pub codex_root: PathBuf,
    pub codex_readme_path: PathBuf,
    pub root_init_path: PathBuf,
}

impl ProjectPaths {
    pub fn from_repo_root(repo_root: &Path) -> Self {
        let repo_root = repo_root.to_path_buf();
        let customization_root = repo_root.join(".ai-teamlead");
        let zellij_dir = customization_root.join("zellij");
        let analysis_tab_template_path = zellij_dir.join("analysis-tab.kdl");
        let settings_path = customization_root.join("settings.yml");
        let project_init_path = customization_root.join("init.sh");
        let launch_agent_path = customization_root.join("launch-agent.sh");
        let flows_dir = customization_root.join("flows");
        let issue_analysis_flow_path = flows_dir.join("issue-analysis-flow.md");
        let issue_analysis_dir = flows_dir.join("issue-analysis");
        let issue_analysis_readme_path = issue_analysis_dir.join("README.md");
        let issue_analysis_what_path = issue_analysis_dir.join("01-what-we-build.md");
        let issue_analysis_how_path = issue_analysis_dir.join("02-how-we-build.md");
        let issue_analysis_verify_path = issue_analysis_dir.join("03-how-we-verify.md");
        let readme_path = customization_root.join("README.md");
        let claude_root = repo_root.join(".claude");
        let claude_readme_path = claude_root.join("README.md");
        let codex_root = repo_root.join(".codex");
        let codex_readme_path = codex_root.join("README.md");
        let root_init_path = repo_root.join("init.sh");

        Self {
            repo_root,
            customization_root,
            zellij_dir,
            analysis_tab_template_path,
            settings_path,
            project_init_path,
            launch_agent_path,
            flows_dir,
            issue_analysis_flow_path,
            issue_analysis_dir,
            issue_analysis_readme_path,
            issue_analysis_what_path,
            issue_analysis_how_path,
            issue_analysis_verify_path,
            readme_path,
            claude_root,
            claude_readme_path,
            codex_root,
            codex_readme_path,
            root_init_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ProjectPaths;

    #[test]
    fn builds_expected_project_paths() {
        let paths = ProjectPaths::from_repo_root(Path::new("/repo"));
        assert_eq!(paths.repo_root, Path::new("/repo"));
        assert_eq!(paths.customization_root, Path::new("/repo/.ai-teamlead"));
        assert_eq!(paths.zellij_dir, Path::new("/repo/.ai-teamlead/zellij"));
        assert_eq!(
            paths.analysis_tab_template_path,
            Path::new("/repo/.ai-teamlead/zellij/analysis-tab.kdl")
        );
        assert_eq!(
            paths.settings_path,
            Path::new("/repo/.ai-teamlead/settings.yml")
        );
        assert_eq!(
            paths.project_init_path,
            Path::new("/repo/.ai-teamlead/init.sh")
        );
        assert_eq!(
            paths.launch_agent_path,
            Path::new("/repo/.ai-teamlead/launch-agent.sh")
        );
        assert_eq!(paths.flows_dir, Path::new("/repo/.ai-teamlead/flows"));
        assert_eq!(
            paths.issue_analysis_flow_path,
            Path::new("/repo/.ai-teamlead/flows/issue-analysis-flow.md")
        );
        assert_eq!(
            paths.issue_analysis_dir,
            Path::new("/repo/.ai-teamlead/flows/issue-analysis")
        );
        assert_eq!(
            paths.issue_analysis_readme_path,
            Path::new("/repo/.ai-teamlead/flows/issue-analysis/README.md")
        );
        assert_eq!(
            paths.issue_analysis_what_path,
            Path::new("/repo/.ai-teamlead/flows/issue-analysis/01-what-we-build.md")
        );
        assert_eq!(
            paths.issue_analysis_how_path,
            Path::new("/repo/.ai-teamlead/flows/issue-analysis/02-how-we-build.md")
        );
        assert_eq!(
            paths.issue_analysis_verify_path,
            Path::new("/repo/.ai-teamlead/flows/issue-analysis/03-how-we-verify.md")
        );
        assert_eq!(paths.readme_path, Path::new("/repo/.ai-teamlead/README.md"));
        assert_eq!(paths.claude_root, Path::new("/repo/.claude"));
        assert_eq!(
            paths.claude_readme_path,
            Path::new("/repo/.claude/README.md")
        );
        assert_eq!(paths.codex_root, Path::new("/repo/.codex"));
        assert_eq!(paths.codex_readme_path, Path::new("/repo/.codex/README.md"));
        assert_eq!(paths.root_init_path, Path::new("/repo/init.sh"));
    }
}
