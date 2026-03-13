use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, FixedOffset, Utc};
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
        let root = repo_root.join(".git").join("ai-teamlead");
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

    pub fn load_question_set(&self, session_uuid: &str) -> Result<Option<QuestionSet>> {
        self.read_optional_json(self.sessions_dir.join(session_uuid).join("questions.json"))
    }

    pub fn load_analysis_plan(&self, session_uuid: &str) -> Result<Option<AnalysisPlan>> {
        self.read_optional_json(
            self.sessions_dir
                .join(session_uuid)
                .join("analysis-plan.json"),
        )
    }

    pub fn load_operator_events(&self, session_uuid: &str) -> Result<Vec<OperatorEvent>> {
        let path = self
            .sessions_dir
            .join(session_uuid)
            .join("operator-events.jsonl");
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<OperatorEvent>(line).with_context(|| {
                    format!("failed to parse operator event in {}", path.display())
                })
            })
            .collect()
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuestionSet {
    pub session_uuid: String,
    pub issue_number: u64,
    pub revision: u64,
    pub generated_at: String,
    pub questions: Vec<QuestionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuestionItem {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisPlan {
    pub session_uuid: String,
    pub issue_number: u64,
    pub revision: u64,
    pub generated_at: String,
    pub summary: String,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub assumptions: Vec<String>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
    pub implementation_plan: Vec<String>,
    #[serde(default)]
    pub feature_story: Option<String>,
    #[serde(default)]
    pub use_cases: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorEvent {
    pub timestamp: String,
    pub session_uuid: String,
    pub issue_number: u64,
    pub event_type: String,
    pub payload: serde_json::Value,
}

impl OperatorEvent {
    pub fn parsed_timestamp(&self) -> Result<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.timestamp)
            .with_context(|| format!("invalid operator event timestamp: {}", self.timestamp))
    }
}

impl QuestionSet {
    pub fn parsed_generated_at(&self) -> Result<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.generated_at)
            .with_context(|| format!("invalid question set timestamp: {}", self.generated_at))
    }
}

impl AnalysisPlan {
    pub fn parsed_generated_at(&self) -> Result<DateTime<FixedOffset>> {
        DateTime::parse_from_rfc3339(&self.generated_at)
            .with_context(|| format!("invalid analysis plan timestamp: {}", self.generated_at))
    }
}

pub fn derive_run_session_facts(
    flow_status: &str,
    questions: Option<&QuestionSet>,
    plan: Option<&AnalysisPlan>,
    events: &[OperatorEvent],
) -> Result<crate::domain::RunSessionFacts> {
    let mut facts = crate::domain::RunSessionFacts::default();

    match flow_status {
        "Waiting for Clarification" => {
            let questions = questions.ok_or_else(|| {
                anyhow!("waiting_for_clarification requires questions.json in session artifacts")
            })?;
            let generated_at = questions.parsed_generated_at()?;
            facts.has_new_answers = events.iter().any(|event| {
                event.event_type == "answers_submitted"
                    && event
                        .parsed_timestamp()
                        .map(|timestamp| timestamp > generated_at)
                        .unwrap_or(false)
            });
        }
        "Waiting for Plan Review" => {
            let plan = plan.ok_or_else(|| {
                anyhow!("waiting_for_plan_review requires analysis-plan.json in session artifacts")
            })?;
            let generated_at = plan.parsed_generated_at()?;
            facts.has_plan_revision_request = events.iter().any(|event| {
                event.event_type == "plan_revision_requested"
                    && event
                        .parsed_timestamp()
                        .map(|timestamp| timestamp > generated_at)
                        .unwrap_or(false)
            });
        }
        "Analysis Blocked" => {
            facts.manual_retry_requested = true;
        }
        _ => {}
    }

    Ok(facts)
}

fn write_json_pretty<T: Serialize>(path: PathBuf, value: &T) -> Result<()> {
    let json = serde_json::to_vec_pretty(value).context("failed to serialize runtime json")?;
    fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AnalysisPlan, OperatorEvent, QuestionSet, RuntimeLayout, SessionManifest,
        derive_run_session_facts,
    };
    use crate::config::ZellijConfig;
    use crate::domain::RunSessionFacts;
    use crate::repo::RepoContext;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn builds_expected_layout() {
        let layout = RuntimeLayout::from_repo_root(Path::new("/repo"));
        assert_eq!(layout.root, Path::new("/repo/.git/ai-teamlead"));
        assert_eq!(layout.lock_dir, Path::new("/repo/.git/ai-teamlead/lock"));
        assert_eq!(
            layout.sessions_dir,
            Path::new("/repo/.git/ai-teamlead/sessions")
        );
        assert_eq!(
            layout.issues_dir,
            Path::new("/repo/.git/ai-teamlead/issues")
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
    fn derives_answers_submitted_after_questions() {
        let questions = QuestionSet {
            session_uuid: "s1".into(),
            issue_number: 42,
            revision: 1,
            generated_at: "2026-03-13T12:00:00+00:00".into(),
            questions: vec![],
        };
        let events = vec![OperatorEvent {
            timestamp: "2026-03-13T12:01:00+00:00".into(),
            session_uuid: "s1".into(),
            issue_number: 42,
            event_type: "answers_submitted".into(),
            payload: serde_json::json!({}),
        }];

        let facts =
            derive_run_session_facts("Waiting for Clarification", Some(&questions), None, &events)
                .expect("facts");
        assert_eq!(
            facts,
            RunSessionFacts {
                has_new_answers: true,
                ..RunSessionFacts::default()
            }
        );
    }

    #[test]
    fn derives_plan_revision_request_after_plan() {
        let plan = AnalysisPlan {
            session_uuid: "s1".into(),
            issue_number: 42,
            revision: 1,
            generated_at: "2026-03-13T12:00:00+00:00".into(),
            summary: "summary".into(),
            scope: vec![],
            non_goals: vec![],
            assumptions: vec![],
            risks: vec![],
            open_questions: vec![],
            implementation_plan: vec![],
            feature_story: None,
            use_cases: None,
        };
        let events = vec![OperatorEvent {
            timestamp: "2026-03-13T12:01:00+00:00".into(),
            session_uuid: "s1".into(),
            issue_number: 42,
            event_type: "plan_revision_requested".into(),
            payload: serde_json::json!({}),
        }];

        let facts = derive_run_session_facts("Waiting for Plan Review", None, Some(&plan), &events)
            .expect("facts");
        assert_eq!(
            facts,
            RunSessionFacts {
                has_plan_revision_request: true,
                ..RunSessionFacts::default()
            }
        );
    }
}
