use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::config::ZellijConfig;
use crate::repo::RepoContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLayout {
    pub root: PathBuf,
    pub lock_dir: PathBuf,
    pub sessions_dir: PathBuf,
    pub issues_dir: PathBuf,
}

impl RuntimeLayout {
    pub fn from_repo_root(repo_root: &Path) -> Self {
        let root = repo_root.join(".git").join(".ai-teamlead");
        let lock_dir = root.join("lock");
        let sessions_dir = root.join("sessions");
        let issues_dir = root.join("issues");
        Self {
            root,
            lock_dir,
            sessions_dir,
            issues_dir,
        }
    }

    pub fn ensure_exists(&self) -> Result<()> {
        fs::create_dir_all(&self.lock_dir)
            .with_context(|| format!("failed to create {}", self.lock_dir.display()))?;
        fs::create_dir_all(&self.sessions_dir)
            .with_context(|| format!("failed to create {}", self.sessions_dir.display()))?;
        fs::create_dir_all(&self.issues_dir)
            .with_context(|| format!("failed to create {}", self.issues_dir.display()))?;
        Ok(())
    }

    pub fn create_claim_binding(
        &self,
        repo: &RepoContext,
        project_id: &str,
        zellij: &ZellijConfig,
        issue_number: u64,
    ) -> Result<SessionManifest> {
        let session_uuid = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();
        let manifest = SessionManifest {
            session_uuid: session_uuid.clone(),
            issue_number,
            repo_root: repo.repo_root.clone(),
            github_owner: repo.github_owner.clone(),
            github_repo: repo.github_repo.clone(),
            project_id: project_id.to_string(),
            status: "active".to_string(),
            created_at: timestamp.clone(),
            updated_at: timestamp.clone(),
            zellij: ZellijBinding {
                session_name: zellij.session_name.clone(),
                tab_name: zellij.tab_name.clone(),
                session_id: "pending".to_string(),
                tab_id: "pending".to_string(),
                pane_id: "pending".to_string(),
            },
        };
        let index = IssueSessionIndex {
            issue_number,
            session_uuid: session_uuid.clone(),
            last_known_flow_status: "Analysis In Progress".to_string(),
            updated_at: timestamp,
        };

        let session_dir = self.sessions_dir.join(&session_uuid);
        fs::create_dir_all(&session_dir)
            .with_context(|| format!("failed to create {}", session_dir.display()))?;

        write_json_pretty(session_dir.join("session.json"), &manifest)?;
        write_json_pretty(self.issues_dir.join(format!("{issue_number}.json")), &index)?;

        Ok(manifest)
    }

    pub fn load_issue_index(&self, issue_number: u64) -> Result<Option<IssueSessionIndex>> {
        self.read_optional_json(self.issues_dir.join(format!("{issue_number}.json")))
    }

    pub fn load_session_manifest(&self, session_uuid: &str) -> Result<Option<SessionManifest>> {
        self.read_optional_json(self.sessions_dir.join(session_uuid).join("session.json"))
    }

    pub fn update_zellij_binding(
        &self,
        session_uuid: &str,
        session_id: &str,
        tab_id: &str,
        pane_id: &str,
    ) -> Result<SessionManifest> {
        let mut manifest = self
            .load_session_manifest(session_uuid)?
            .ok_or_else(|| anyhow!("missing session manifest for session_uuid={session_uuid}"))?;
        manifest.updated_at = Utc::now().to_rfc3339();
        manifest.zellij.session_id = session_id.to_string();
        manifest.zellij.tab_id = tab_id.to_string();
        manifest.zellij.pane_id = pane_id.to_string();

        let session_path = self.sessions_dir.join(session_uuid).join("session.json");
        write_json_pretty(session_path, &manifest)?;
        Ok(manifest)
    }

    pub fn update_session_status(
        &self,
        session_uuid: &str,
        status: &str,
    ) -> Result<SessionManifest> {
        let mut manifest = self
            .load_session_manifest(session_uuid)?
            .ok_or_else(|| anyhow!("missing session manifest for session_uuid={session_uuid}"))?;
        manifest.status = status.to_string();
        manifest.updated_at = Utc::now().to_rfc3339();

        let session_path = self.sessions_dir.join(session_uuid).join("session.json");
        write_json_pretty(session_path, &manifest)?;
        Ok(manifest)
    }

    pub fn update_issue_flow_status(&self, issue_number: u64, flow_status: &str) -> Result<()> {
        let mut index = self
            .load_issue_index(issue_number)?
            .ok_or_else(|| anyhow!("missing issue session index for issue #{issue_number}"))?;
        index.last_known_flow_status = flow_status.to_string();
        index.updated_at = Utc::now().to_rfc3339();

        write_json_pretty(self.issues_dir.join(format!("{issue_number}.json")), &index)?;
        Ok(())
    }

    fn read_optional_json<T>(&self, path: PathBuf) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !path.exists() {
            return Ok(None);
        }
        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let value = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(Some(value))
    }

    pub fn session_dir(&self, session_uuid: &str) -> PathBuf {
        self.sessions_dir.join(session_uuid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionManifest {
    pub session_uuid: String,
    pub issue_number: u64,
    pub repo_root: PathBuf,
    pub github_owner: String,
    pub github_repo: String,
    pub project_id: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub zellij: ZellijBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ZellijBinding {
    pub session_name: String,
    pub tab_name: String,
    pub session_id: String,
    pub tab_id: String,
    pub pane_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueSessionIndex {
    pub issue_number: u64,
    pub session_uuid: String,
    pub last_known_flow_status: String,
    pub updated_at: String,
}

fn write_json_pretty<T: Serialize>(path: PathBuf, value: &T) -> Result<()> {
    let json = serde_json::to_vec_pretty(value).context("failed to serialize runtime json")?;
    fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{RuntimeLayout, SessionManifest};
    use crate::config::ZellijConfig;
    use crate::repo::RepoContext;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn builds_expected_layout() {
        let layout = RuntimeLayout::from_repo_root(Path::new("/repo"));
        assert_eq!(layout.root, Path::new("/repo/.git/.ai-teamlead"));
        assert_eq!(layout.lock_dir, Path::new("/repo/.git/.ai-teamlead/lock"));
        assert_eq!(
            layout.sessions_dir,
            Path::new("/repo/.git/.ai-teamlead/sessions")
        );
        assert_eq!(
            layout.issues_dir,
            Path::new("/repo/.git/.ai-teamlead/issues")
        );
    }

    #[test]
    fn creates_claim_binding_files() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let layout = RuntimeLayout::from_repo_root(&repo_root);
        layout.ensure_exists().expect("runtime layout");

        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir,
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
        };

        let manifest = layout
            .create_claim_binding(&repo, "PVT_project", &zellij, 42)
            .expect("claim binding");

        let session_path = layout
            .sessions_dir
            .join(&manifest.session_uuid)
            .join("session.json");
        let issue_index_path = layout.issues_dir.join("42.json");

        assert!(session_path.exists());
        assert!(issue_index_path.exists());

        let stored: SessionManifest = serde_json::from_slice(
            &std::fs::read(&session_path).expect("session manifest should exist"),
        )
        .expect("session manifest should parse");
        assert_eq!(stored.issue_number, 42);
        assert_eq!(stored.zellij.pane_id, "pending");
    }

    #[test]
    fn updates_zellij_binding_in_session_manifest() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let layout = RuntimeLayout::from_repo_root(&repo_root);
        layout.ensure_exists().expect("runtime layout");

        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir,
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
        };

        let manifest = layout
            .create_claim_binding(&repo, "PVT_project", &zellij, 42)
            .expect("claim binding");

        let updated = layout
            .update_zellij_binding(&manifest.session_uuid, "ai-teamlead", "7", "terminal_9")
            .expect("binding updated");

        assert_eq!(updated.zellij.session_id, "ai-teamlead");
        assert_eq!(updated.zellij.tab_id, "7");
        assert_eq!(updated.zellij.pane_id, "terminal_9");
    }

    #[test]
    fn updates_session_status_to_completed() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let layout = RuntimeLayout::from_repo_root(&repo_root);
        layout.ensure_exists().expect("runtime layout");

        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir,
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
        };

        let manifest = layout
            .create_claim_binding(&repo, "PVT_project", &zellij, 42)
            .expect("claim binding");

        assert_eq!(manifest.status, "active");
        let updated = layout
            .update_session_status(&manifest.session_uuid, "completed")
            .expect("status updated");

        assert_eq!(updated.status, "completed");

        let reloaded = layout
            .load_session_manifest(&manifest.session_uuid)
            .expect("reload")
            .expect("manifest exists");
        assert_eq!(reloaded.status, "completed");
    }
}
