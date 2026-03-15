use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use crate::shell::Shell;

pub struct GhProjectClient<'a> {
    shell: &'a dyn Shell,
}

impl<'a> GhProjectClient<'a> {
    pub fn new(shell: &'a dyn Shell) -> Self {
        Self { shell }
    }

    pub fn load_project_snapshot(&self, cwd: &Path, project_id: &str) -> Result<ProjectSnapshot> {
        let query = r#"query($projectId: ID!) {
  node(id: $projectId) {
    ... on ProjectV2 {
      id
      title
      field(name: "Status") {
        ... on ProjectV2SingleSelectField {
          id
          options {
            id
            name
          }
        }
      }
      items(first: 100) {
        nodes {
          id
          fieldValueByName(name: "Status") {
            ... on ProjectV2ItemFieldSingleSelectValue {
              name
              optionId
            }
          }
          content {
            ... on Issue {
              number
              state
              repository {
                name
                owner {
                  login
                }
              }
            }
          }
        }
      }
    }
  }
}"#;

        let stdout = self.shell.run(
            cwd,
            "gh",
            &[
                "api",
                "graphql",
                "-f",
                &format!("query={query}"),
                "-F",
                &format!("projectId={project_id}"),
            ],
        )?;

        let response: GraphQlResponse<ProjectNodeData> =
            serde_json::from_str(&stdout).context("failed to parse project snapshot response")?;

        let project = response
            .data
            .node
            .ok_or_else(|| anyhow!("project node was not returned"))?;
        let field = project
            .field
            .ok_or_else(|| anyhow!("project status field was not returned"))?;

        let status_options = field
            .options
            .into_iter()
            .map(|option| (option.name, option.id))
            .collect::<HashMap<_, _>>();

        let mut items = Vec::new();
        for node in project.items.nodes {
            let Some(content) = node.content else {
                continue;
            };
            let status_name = node
                .field_value_by_name
                .as_ref()
                .map(|value| value.name.clone());
            let status_option_id = node.field_value_by_name.and_then(|value| value.option_id);

            items.push(ProjectIssueItem {
                item_id: node.id,
                issue_number: content.number,
                issue_state: content.state,
                repo_owner: content.repository.owner.login,
                repo_name: content.repository.name,
                status_name,
                status_option_id,
            });
        }

        Ok(ProjectSnapshot {
            project_id: project.id,
            title: project.title,
            status_field_id: field.id,
            status_options,
            items,
        })
    }

    pub fn update_status(
        &self,
        cwd: &Path,
        project_id: &str,
        item_id: &str,
        field_id: &str,
        option_id: &str,
    ) -> Result<()> {
        let query = r#"mutation($projectId: ID!, $itemId: ID!, $fieldId: ID!, $optionId: String!) {
  updateProjectV2ItemFieldValue(
    input: {
      projectId: $projectId
      itemId: $itemId
      fieldId: $fieldId
      value: { singleSelectOptionId: $optionId }
    }
  ) {
    projectV2Item {
      id
    }
  }
}"#;

        self.shell.run(
            cwd,
            "gh",
            &[
                "api",
                "graphql",
                "-f",
                &format!("query={query}"),
                "-F",
                &format!("projectId={project_id}"),
                "-F",
                &format!("itemId={item_id}"),
                "-F",
                &format!("fieldId={field_id}"),
                "-F",
                &format!("optionId={option_id}"),
            ],
        )?;
        Ok(())
    }

    pub fn load_repo_issue(
        &self,
        cwd: &Path,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Option<RepoIssue>> {
        let query = r#"query($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    issue(number: $number) {
      id
      number
      state
      url
    }
  }
}"#;

        let stdout = self.shell.run(
            cwd,
            "gh",
            &[
                "api",
                "graphql",
                "-f",
                &format!("query={query}"),
                "-F",
                &format!("owner={owner}"),
                "-F",
                &format!("repo={repo}"),
                "-F",
                &format!("number={issue_number}"),
            ],
        )?;

        let response: GraphQlResponse<RepositoryIssueData> =
            serde_json::from_str(&stdout).context("failed to parse repo issue response")?;

        Ok(response
            .data
            .repository
            .and_then(|repository| repository.issue)
            .map(|issue| RepoIssue {
                id: issue.id,
                number: issue.number,
                state: issue.state,
                url: issue.url,
            }))
    }

    pub fn add_issue_to_project(
        &self,
        cwd: &Path,
        project_id: &str,
        content_id: &str,
    ) -> Result<String> {
        let query = r#"mutation($projectId: ID!, $contentId: ID!) {
  addProjectV2ItemById(input: { projectId: $projectId, contentId: $contentId }) {
    item {
      id
    }
  }
}"#;

        let stdout = self.shell.run(
            cwd,
            "gh",
            &[
                "api",
                "graphql",
                "-f",
                &format!("query={query}"),
                "-F",
                &format!("projectId={project_id}"),
                "-F",
                &format!("contentId={content_id}"),
            ],
        )?;

        let response: GraphQlResponse<AddProjectItemData> =
            serde_json::from_str(&stdout).context("failed to parse add project item response")?;

        response
            .data
            .add_project_item
            .map(|payload| payload.item.id)
            .ok_or_else(|| anyhow!("project item was not returned after addProjectV2ItemById"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSnapshot {
    pub project_id: String,
    pub title: String,
    pub status_field_id: String,
    pub status_options: HashMap<String, String>,
    pub items: Vec<ProjectIssueItem>,
}

impl ProjectSnapshot {
    pub fn option_id_by_name(&self, status_name: &str) -> Result<&str> {
        self.status_options
            .get(status_name)
            .map(String::as_str)
            .ok_or_else(|| anyhow!("project does not contain status option: {status_name}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectIssueItem {
    pub item_id: String,
    pub issue_number: u64,
    pub issue_state: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub status_name: Option<String>,
    pub status_option_id: Option<String>,
}

impl ProjectIssueItem {
    pub fn matches_repo(&self, owner: &str, repo: &str) -> bool {
        self.repo_owner == owner && self.repo_name == repo
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoIssue {
    pub id: String,
    pub number: u64,
    pub state: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct ProjectNodeData {
    node: Option<ProjectNode>,
}

#[derive(Debug, Deserialize)]
struct ProjectNode {
    id: String,
    title: String,
    field: Option<ProjectField>,
    items: ProjectItemsConnection,
}

#[derive(Debug, Deserialize)]
struct ProjectField {
    id: String,
    options: Vec<ProjectFieldOption>,
}

#[derive(Debug, Deserialize)]
struct ProjectFieldOption {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ProjectItemsConnection {
    nodes: Vec<ProjectItemNode>,
}

#[derive(Debug, Deserialize)]
struct ProjectItemNode {
    id: String,
    #[serde(rename = "fieldValueByName")]
    field_value_by_name: Option<ProjectItemStatusValue>,
    content: Option<ProjectIssueContent>,
}

#[derive(Debug, Deserialize, Clone)]
struct ProjectItemStatusValue {
    name: String,
    #[serde(rename = "optionId")]
    option_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProjectIssueContent {
    number: u64,
    state: String,
    repository: ProjectIssueRepository,
}

#[derive(Debug, Deserialize)]
struct ProjectIssueRepository {
    name: String,
    owner: ProjectIssueOwner,
}

#[derive(Debug, Deserialize)]
struct ProjectIssueOwner {
    login: String,
}

#[derive(Debug, Deserialize)]
struct RepositoryIssueData {
    repository: Option<RepositoryIssueNode>,
}

#[derive(Debug, Deserialize)]
struct RepositoryIssueNode {
    issue: Option<RepositoryIssue>,
}

#[derive(Debug, Deserialize)]
struct RepositoryIssue {
    id: String,
    number: u64,
    state: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct AddProjectItemData {
    #[serde(rename = "addProjectV2ItemById")]
    add_project_item: Option<AddProjectItemPayload>,
}

#[derive(Debug, Deserialize)]
struct AddProjectItemPayload {
    item: AddProjectItem,
}

#[derive(Debug, Deserialize)]
struct AddProjectItem {
    id: String,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use anyhow::Result;

    use super::*;
    use crate::shell::Shell;

    #[derive(Default)]
    struct FakeShell {
        responses: BTreeMap<String, String>,
    }

    impl FakeShell {
        fn with_response(mut self, key: &str, value: &str) -> Self {
            self.responses.insert(key.to_string(), value.to_string());
            self
        }
    }

    impl Shell for FakeShell {
        fn run(&self, _cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
            let key = format!("{program} {}", args.join(" "));
            self.responses
                .get(&key)
                .cloned()
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
    fn parses_project_snapshot() {
        let query = r#"query($projectId: ID!) {
  node(id: $projectId) {
    ... on ProjectV2 {
      id
      title
      field(name: "Status") {
        ... on ProjectV2SingleSelectField {
          id
          options {
            id
            name
          }
        }
      }
      items(first: 100) {
        nodes {
          id
          fieldValueByName(name: "Status") {
            ... on ProjectV2ItemFieldSingleSelectValue {
              name
              optionId
            }
          }
          content {
            ... on Issue {
              number
              state
              repository {
                name
                owner {
                  login
                }
              }
            }
          }
        }
      }
    }
  }
}"#;
        let shell = FakeShell::default().with_response(
            &format!(
                "gh api graphql -f query={query} -F projectId={}",
                "PVT_project"
            ),
            r#"{"data":{"node":{"id":"PVT_project","title":"teamlead","field":{"id":"field1","name":"Status","options":[{"id":"opt-backlog","name":"Backlog"},{"id":"opt-progress","name":"Analysis In Progress"}]},"items":{"nodes":[{"id":"item-1","fieldValueByName":{"name":"Backlog","optionId":"opt-backlog"},"content":{"number":42,"state":"OPEN","repository":{"name":"teamlead","owner":{"login":"dapi"}}}}]}}}}"#,
        );

        let client = GhProjectClient::new(&shell);
        let snapshot = client
            .load_project_snapshot(&PathBuf::from("/repo"), "PVT_project")
            .expect("snapshot should parse");

        assert_eq!(snapshot.title, "teamlead");
        assert_eq!(
            snapshot.option_id_by_name("Backlog").expect("option"),
            "opt-backlog"
        );
        assert_eq!(snapshot.items.len(), 1);
        assert_eq!(snapshot.items[0].issue_number, 42);
    }

    #[test]
    fn returns_error_for_missing_status_option() {
        let snapshot = ProjectSnapshot {
            project_id: "PVT".into(),
            title: "teamlead".into(),
            status_field_id: "field".into(),
            status_options: HashMap::new(),
            items: Vec::new(),
        };

        let error = snapshot
            .option_id_by_name("Backlog")
            .expect_err("missing option should fail");
        assert!(error.to_string().contains("Backlog"));
    }

    #[test]
    fn parses_repo_issue_lookup() {
        let query = r#"query($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    issue(number: $number) {
      id
      number
      state
      url
    }
  }
}"#;
        let shell = FakeShell::default().with_response(
            &format!(
                "gh api graphql -f query={query} -F owner=dapi -F repo=teamlead -F number={}",
                42
            ),
            r#"{"data":{"repository":{"issue":{"id":"ISSUE_42","number":42,"state":"OPEN","url":"https://github.com/dapi/teamlead/issues/42"}}}}"#,
        );

        let client = GhProjectClient::new(&shell);
        let issue = client
            .load_repo_issue(&PathBuf::from("/repo"), "dapi", "teamlead", 42)
            .expect("repo issue should parse")
            .expect("issue should exist");

        assert_eq!(issue.id, "ISSUE_42");
        assert_eq!(issue.state, "OPEN");
        assert_eq!(issue.url, "https://github.com/dapi/teamlead/issues/42");
    }

    #[test]
    fn returns_none_for_missing_repo_issue() {
        let query = r#"query($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    issue(number: $number) {
      id
      number
      state
      url
    }
  }
}"#;
        let shell = FakeShell::default().with_response(
            &format!(
                "gh api graphql -f query={query} -F owner=dapi -F repo=teamlead -F number={}",
                404
            ),
            r#"{"data":{"repository":{"issue":null}}}"#,
        );

        let client = GhProjectClient::new(&shell);
        let issue = client
            .load_repo_issue(&PathBuf::from("/repo"), "dapi", "teamlead", 404)
            .expect("repo issue lookup should parse");

        assert!(issue.is_none());
    }

    #[test]
    fn parses_project_item_id_from_add_issue_response() {
        let query = r#"mutation($projectId: ID!, $contentId: ID!) {
  addProjectV2ItemById(input: { projectId: $projectId, contentId: $contentId }) {
    item {
      id
    }
  }
}"#;
        let shell = FakeShell::default().with_response(
            &format!(
                "gh api graphql -f query={query} -F projectId=PVT_project -F contentId={}",
                "ISSUE_42"
            ),
            r#"{"data":{"addProjectV2ItemById":{"item":{"id":"ITEM_42"}}}}"#,
        );

        let client = GhProjectClient::new(&shell);
        let item_id = client
            .add_issue_to_project(&PathBuf::from("/repo"), "PVT_project", "ISSUE_42")
            .expect("project add should parse");

        assert_eq!(item_id, "ITEM_42");
    }
}
