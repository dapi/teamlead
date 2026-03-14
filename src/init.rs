use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::project_files::ProjectPaths;

const PROJECT_README_TEMPLATE: &str = include_str!("../templates/init/README.md");
const SETTINGS_TEMPLATE: &str = include_str!("../templates/init/settings.yml");
const PROJECT_INIT_TEMPLATE: &str = include_str!("../templates/init/init.sh");
const LAUNCH_AGENT_TEMPLATE: &str = include_str!("../templates/init/launch-agent.sh");
const ISSUE_ANALYSIS_FLOW_TEMPLATE: &str = include_str!("../templates/init/issue-analysis-flow.md");
const ISSUE_ANALYSIS_README_TEMPLATE: &str =
    include_str!("../templates/init/issue-analysis/README.md");
const ISSUE_ANALYSIS_WHAT_TEMPLATE: &str =
    include_str!("../templates/init/issue-analysis/01-what-we-build.md");
const ISSUE_ANALYSIS_HOW_TEMPLATE: &str =
    include_str!("../templates/init/issue-analysis/02-how-we-build.md");
const ISSUE_ANALYSIS_VERIFY_TEMPLATE: &str =
    include_str!("../templates/init/issue-analysis/03-how-we-verify.md");
const CLAUDE_README_TEMPLATE: &str = include_str!("../templates/init/claude/README.md");
const CODEX_README_TEMPLATE: &str = include_str!("../templates/init/codex/README.md");

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InitReport {
    pub created: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

pub fn init_project_files(paths: &ProjectPaths) -> Result<InitReport> {
    let settings_template = render_settings_template();
    fs::create_dir_all(&paths.customization_root)
        .with_context(|| format!("failed to create {}", paths.customization_root.display()))?;
    fs::create_dir_all(&paths.flows_dir)
        .with_context(|| format!("failed to create {}", paths.flows_dir.display()))?;
    fs::create_dir_all(&paths.issue_analysis_dir)
        .with_context(|| format!("failed to create {}", paths.issue_analysis_dir.display()))?;
    fs::create_dir_all(&paths.claude_root)
        .with_context(|| format!("failed to create {}", paths.claude_root.display()))?;
    fs::create_dir_all(&paths.codex_root)
        .with_context(|| format!("failed to create {}", paths.codex_root.display()))?;

    let mut report = InitReport::default();
    write_if_missing(
        &paths.settings_path,
        &settings_template,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.readme_path,
        PROJECT_README_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_script_if_missing(
        &paths.project_init_path,
        PROJECT_INIT_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_script_if_missing(
        &paths.launch_agent_path,
        LAUNCH_AGENT_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.issue_analysis_flow_path,
        ISSUE_ANALYSIS_FLOW_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.issue_analysis_readme_path,
        ISSUE_ANALYSIS_README_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.issue_analysis_what_path,
        ISSUE_ANALYSIS_WHAT_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.issue_analysis_how_path,
        ISSUE_ANALYSIS_HOW_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.issue_analysis_verify_path,
        ISSUE_ANALYSIS_VERIFY_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.claude_readme_path,
        CLAUDE_README_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    write_if_missing(
        &paths.codex_readme_path,
        CODEX_README_TEMPLATE,
        &mut report.created,
        &mut report.skipped,
    )?;
    symlink_init_if_missing(paths, &mut report.created, &mut report.skipped)?;

    Ok(report)
}

fn render_settings_template() -> String {
    SETTINGS_TEMPLATE.to_string()
}

fn symlink_init_if_missing(
    paths: &ProjectPaths,
    created: &mut Vec<PathBuf>,
    skipped: &mut Vec<PathBuf>,
) -> Result<()> {
    if paths.root_init_path.exists() {
        skipped.push(paths.root_init_path.clone());
        return Ok(());
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(".ai-teamlead/init.sh", &paths.root_init_path).with_context(
            || {
                format!(
                    "failed to create symlink {}",
                    paths.root_init_path.display()
                )
            },
        )?;
        created.push(paths.root_init_path.clone());
        Ok(())
    }

    #[cfg(not(unix))]
    {
        let _ = paths;
        let _ = created;
        let _ = skipped;
        anyhow::bail!("bootstrap of init.sh symlink is supported only on unix")
    }
}

fn write_if_missing(
    path: &PathBuf,
    content: &str,
    created: &mut Vec<PathBuf>,
    skipped: &mut Vec<PathBuf>,
) -> Result<()> {
    if path.exists() {
        skipped.push(path.clone());
        return Ok(());
    }

    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    created.push(path.clone());
    Ok(())
}

fn write_script_if_missing(
    path: &PathBuf,
    content: &str,
    created: &mut Vec<PathBuf>,
    skipped: &mut Vec<PathBuf>,
) -> Result<()> {
    write_if_missing(path, content, created, skipped)?;

    #[cfg(unix)]
    if path.exists() {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .with_context(|| format!("failed to stat {}", path.display()))?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("failed to set executable bit on {}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::init_project_files;
    use crate::project_files::ProjectPaths;

    #[test]
    fn initializes_project_files_without_overwriting_existing_files() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("teamlead");
        std::fs::create_dir_all(&repo_root).expect("repo root");
        let paths = ProjectPaths::from_repo_root(&repo_root);

        let first = init_project_files(&paths).expect("first init");
        assert_eq!(first.created.len(), 12);
        assert!(paths.settings_path.exists());
        assert!(paths.readme_path.exists());
        assert!(paths.project_init_path.exists());
        assert!(paths.launch_agent_path.exists());
        assert!(paths.issue_analysis_flow_path.exists());
        assert!(paths.issue_analysis_readme_path.exists());
        assert!(paths.issue_analysis_what_path.exists());
        assert!(paths.issue_analysis_how_path.exists());
        assert!(paths.issue_analysis_verify_path.exists());
        assert!(paths.claude_readme_path.exists());
        assert!(paths.codex_readme_path.exists());
        assert!(paths.root_init_path.exists());
        assert_eq!(
            std::fs::read_link(&paths.root_init_path).expect("init symlink"),
            PathBuf::from(".ai-teamlead/init.sh")
        );

        let second = init_project_files(&paths).expect("second init");
        assert_eq!(second.created.len(), 0);
        assert_eq!(second.skipped.len(), 12);

        let settings = std::fs::read_to_string(&paths.settings_path).expect("settings");
        assert!(settings.contains("session_name: \"${REPO}\""));
    }
}
