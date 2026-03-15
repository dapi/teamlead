use crate::config::{FlowStatuses, ImplementationFlowStatuses};
use crate::github::ProjectIssueItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueCandidate {
    pub number: u64,
    pub status: String,
}

pub fn select_next_backlog_issue<'a>(
    issues: &'a [IssueCandidate],
    statuses: &FlowStatuses,
) -> Option<&'a IssueCandidate> {
    issues.iter().find(|issue| issue.status == statuses.backlog)
}

pub fn select_next_backlog_project_item<'a>(
    items: &'a [ProjectIssueItem],
    statuses: &FlowStatuses,
    owner: &str,
    repo: &str,
    assignee_filter: Option<&str>,
) -> Option<&'a ProjectIssueItem> {
    items
        .iter()
        .filter(|item| item.matches_repo(owner, repo))
        .filter(|item| item.issue_state == "OPEN")
        .filter(|item| {
            assignee_filter.is_none_or(|assignee_filter| {
                item.assignees
                    .iter()
                    .any(|assignee| assignee == assignee_filter)
            })
        })
        .find(|item| item.status_name.as_deref() == Some(statuses.backlog.as_str()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunDecision {
    pub allowed: bool,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum FlowStage {
    Analysis,
    Implementation,
}

impl FlowStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Analysis => "analysis",
            Self::Implementation => "implementation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunStageDecision {
    pub allowed: bool,
    pub reason: &'static str,
    pub stage: Option<FlowStage>,
}

pub fn allowed_run_statuses(
    analysis_statuses: &FlowStatuses,
    implementation_statuses: &ImplementationFlowStatuses,
) -> Vec<String> {
    vec![
        analysis_statuses.backlog.clone(),
        analysis_statuses.waiting_for_clarification.clone(),
        analysis_statuses.waiting_for_plan_review.clone(),
        analysis_statuses.analysis_blocked.clone(),
        implementation_statuses.ready_for_implementation.clone(),
        implementation_statuses.implementation_in_progress.clone(),
        implementation_statuses.waiting_for_ci.clone(),
        implementation_statuses.waiting_for_code_review.clone(),
        implementation_statuses.implementation_blocked.clone(),
    ]
}

pub fn format_run_denied_message(
    issue_number: u64,
    current_status: &str,
    allowed_statuses: &[String],
) -> String {
    format!(
        "Невозможно запустить run для issue #{issue_number}\n\nТекущий статус: \"{current_status}\"\nДопустимые статусы для run: {}\n\nАвтоисправление не выполнено: система не может однозначно выбрать корректный target status.\nИзмените статус issue в GitHub Project вручную и повторите run.",
        allowed_statuses.join(", ")
    )
}

pub fn format_missing_issue_message(issue_number: u64, owner: &str, repo: &str) -> String {
    format!(
        "Issue #{issue_number} не найдена в репозитории {owner}/{repo}.\nПроверьте номер issue или URL и повторите run."
    )
}

pub fn format_closed_issue_message(issue_number: u64, state: &str, issue_url: &str) -> String {
    format!(
        "Невозможно запустить run для issue #{issue_number}\n\nТекущее состояние issue: {state}\nURL: {issue_url}\n\nrun работает только для открытых issue и не переоткрывает их автоматически."
    )
}

pub fn format_project_attachment_failure_message(
    issue_number: u64,
    project_id: &str,
    issue_url: &str,
) -> String {
    format!(
        "Issue #{issue_number} не привязана к GitHub Project \"{project_id}\", и автоисправление не удалось.\nURL: {issue_url}\nПроверьте доступ к проекту и привязку issue, затем повторите run."
    )
}

pub fn can_run_analysis(status: &str, statuses: &FlowStatuses) -> RunDecision {
    if status == statuses.backlog {
        return RunDecision {
            allowed: true,
            reason: "backlog issues may be claimed",
        };
    }
    if status == statuses.waiting_for_clarification {
        return RunDecision {
            allowed: true,
            reason: "waiting_for_clarification may be resumed explicitly by operator",
        };
    }
    if status == statuses.waiting_for_plan_review {
        return RunDecision {
            allowed: true,
            reason: "waiting_for_plan_review may be reopened explicitly by operator",
        };
    }
    if status == statuses.analysis_blocked {
        return RunDecision {
            allowed: true,
            reason: "analysis_blocked may be retried explicitly by operator",
        };
    }

    RunDecision {
        allowed: false,
        reason: "status is not a valid run entry point",
    }
}

pub fn decide_run_stage(
    status: &str,
    analysis_statuses: &FlowStatuses,
    implementation_statuses: &ImplementationFlowStatuses,
) -> RunStageDecision {
    let analysis = can_run_analysis(status, analysis_statuses);
    if analysis.allowed {
        return RunStageDecision {
            allowed: true,
            reason: analysis.reason,
            stage: Some(FlowStage::Analysis),
        };
    }

    if status == implementation_statuses.ready_for_implementation {
        return RunStageDecision {
            allowed: true,
            reason: "ready_for_implementation may enter implementation stage",
            stage: Some(FlowStage::Implementation),
        };
    }
    if status == implementation_statuses.implementation_in_progress {
        return RunStageDecision {
            allowed: true,
            reason: "implementation_in_progress may be resumed explicitly by operator",
            stage: Some(FlowStage::Implementation),
        };
    }
    if status == implementation_statuses.waiting_for_ci {
        return RunStageDecision {
            allowed: true,
            reason: "waiting_for_ci may be reopened explicitly by operator",
            stage: Some(FlowStage::Implementation),
        };
    }
    if status == implementation_statuses.waiting_for_code_review {
        return RunStageDecision {
            allowed: true,
            reason: "waiting_for_code_review may be reopened explicitly by operator",
            stage: Some(FlowStage::Implementation),
        };
    }
    if status == implementation_statuses.implementation_blocked {
        return RunStageDecision {
            allowed: true,
            reason: "implementation_blocked may be retried explicitly by operator",
            stage: Some(FlowStage::Implementation),
        };
    }

    RunStageDecision {
        allowed: false,
        reason: "status is not a valid run entry point",
        stage: None,
    }
}

pub fn parse_issue_ref(input: &str) -> anyhow::Result<u64> {
    if let Ok(number) = input.parse::<u64>() {
        return Ok(number);
    }

    let trimmed = input.trim_end_matches('/');
    if let Some(segment) = trimmed.rsplit('/').next()
        && let Ok(number) = segment.parse::<u64>()
    {
        return Ok(number);
    }

    anyhow::bail!("issue reference must be a number or issue URL")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statuses() -> FlowStatuses {
        FlowStatuses {
            backlog: "Backlog".into(),
            analysis_in_progress: "Analysis In Progress".into(),
            waiting_for_clarification: "Waiting for Clarification".into(),
            waiting_for_plan_review: "Waiting for Plan Review".into(),
            ready_for_implementation: "Ready for Implementation".into(),
            analysis_blocked: "Analysis Blocked".into(),
        }
    }

    fn implementation_statuses() -> ImplementationFlowStatuses {
        ImplementationFlowStatuses {
            ready_for_implementation: "Ready for Implementation".into(),
            implementation_in_progress: "Implementation In Progress".into(),
            waiting_for_ci: "Waiting for CI".into(),
            waiting_for_code_review: "Waiting for Code Review".into(),
            done: "Done".into(),
            implementation_blocked: "Implementation Blocked".into(),
        }
    }

    #[test]
    fn selects_first_backlog_issue_in_input_order() {
        let items = vec![
            IssueCandidate {
                number: 42,
                status: "Backlog".into(),
            },
            IssueCandidate {
                number: 7,
                status: "Backlog".into(),
            },
            IssueCandidate {
                number: 5,
                status: "Analysis In Progress".into(),
            },
        ];

        let selected = select_next_backlog_issue(&items, &statuses()).expect("issue expected");
        assert_eq!(selected.number, 42);
    }

    #[test]
    fn waiting_for_clarification_requires_new_answers() {
        let allowed = can_run_analysis("Waiting for Clarification", &statuses());
        assert!(allowed.allowed);
    }

    #[test]
    fn waiting_for_plan_review_allows_explicit_reopen() {
        let allowed = can_run_analysis("Waiting for Plan Review", &statuses());
        assert!(allowed.allowed);
    }

    #[test]
    fn analysis_blocked_allows_explicit_retry() {
        let allowed = can_run_analysis("Analysis Blocked", &statuses());
        assert!(allowed.allowed);
    }

    #[test]
    fn ready_for_implementation_dispatches_to_implementation_stage() {
        let decision = decide_run_stage(
            "Ready for Implementation",
            &statuses(),
            &implementation_statuses(),
        );
        assert!(decision.allowed);
        assert_eq!(decision.stage, Some(FlowStage::Implementation));
    }

    #[test]
    fn backlog_dispatches_to_analysis_stage() {
        let decision = decide_run_stage("Backlog", &statuses(), &implementation_statuses());
        assert!(decision.allowed);
        assert_eq!(decision.stage, Some(FlowStage::Analysis));
    }

    #[test]
    fn allowed_run_statuses_include_analysis_and_implementation_entries() {
        let allowed = allowed_run_statuses(&statuses(), &implementation_statuses());

        assert_eq!(
            allowed,
            vec![
                "Backlog",
                "Waiting for Clarification",
                "Waiting for Plan Review",
                "Analysis Blocked",
                "Ready for Implementation",
                "Implementation In Progress",
                "Waiting for CI",
                "Waiting for Code Review",
                "Implementation Blocked",
            ]
        );
    }

    #[test]
    fn format_run_denied_message_lists_current_and_allowed_statuses() {
        let message = format_run_denied_message(
            42,
            "Analysis In Progress",
            &allowed_run_statuses(&statuses(), &implementation_statuses()),
        );

        assert!(message.contains("issue #42"));
        assert!(message.contains("Analysis In Progress"));
        assert!(message.contains("Ready for Implementation"));
        assert!(message.contains("Автоисправление не выполнено"));
    }

    #[test]
    fn format_missing_issue_message_mentions_repo() {
        let message = format_missing_issue_message(42, "dapi", "teamlead");
        assert!(message.contains("dapi/teamlead"));
    }

    #[test]
    fn format_closed_issue_message_mentions_url_and_state() {
        let message =
            format_closed_issue_message(42, "CLOSED", "https://github.com/dapi/teamlead/issues/42");
        assert!(message.contains("CLOSED"));
        assert!(message.contains("https://github.com/dapi/teamlead/issues/42"));
    }

    #[test]
    fn format_project_attachment_failure_mentions_project() {
        let message = format_project_attachment_failure_message(
            42,
            "PVT_project",
            "https://github.com/dapi/teamlead/issues/42",
        );
        assert!(message.contains("PVT_project"));
        assert!(message.contains("автоисправление не удалось"));
    }

    #[test]
    fn parses_issue_number_and_url() {
        assert_eq!(parse_issue_ref("15").expect("number"), 15);
        assert_eq!(
            parse_issue_ref("https://github.com/dapi/teamlead/issues/27").expect("url"),
            27
        );
    }

    #[test]
    fn selects_project_item_for_current_repo() {
        let items = vec![
            ProjectIssueItem {
                item_id: "item-1".into(),
                issue_number: 11,
                issue_state: "OPEN".into(),
                repo_owner: "other".into(),
                repo_name: "repo".into(),
                assignees: vec![],
                status_name: Some("Backlog".into()),
                status_option_id: Some("1".into()),
            },
            ProjectIssueItem {
                item_id: "item-2".into(),
                issue_number: 7,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                assignees: vec!["dapi".into()],
                status_name: Some("Backlog".into()),
                status_option_id: Some("2".into()),
            },
            ProjectIssueItem {
                item_id: "item-3".into(),
                issue_number: 5,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                assignees: vec!["someone".into()],
                status_name: Some("Backlog".into()),
                status_option_id: Some("3".into()),
            },
        ];

        let selected =
            select_next_backlog_project_item(&items, &statuses(), "dapi", "teamlead", None)
                .unwrap();
        assert_eq!(selected.item_id, "item-2");
    }

    #[test]
    fn keeps_old_behavior_when_assignee_filter_is_not_set() {
        let items = vec![
            ProjectIssueItem {
                item_id: "item-1".into(),
                issue_number: 7,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                assignees: vec![],
                status_name: Some("Backlog".into()),
                status_option_id: Some("1".into()),
            },
            ProjectIssueItem {
                item_id: "item-2".into(),
                issue_number: 8,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                assignees: vec!["alice".into()],
                status_name: Some("Backlog".into()),
                status_option_id: Some("2".into()),
            },
        ];

        let selected =
            select_next_backlog_project_item(&items, &statuses(), "dapi", "teamlead", None)
                .unwrap();

        assert_eq!(selected.item_id, "item-1");
    }

    #[test]
    fn filters_backlog_items_by_assignee() {
        let items = vec![
            ProjectIssueItem {
                item_id: "item-1".into(),
                issue_number: 7,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                assignees: vec!["alice".into()],
                status_name: Some("Backlog".into()),
                status_option_id: Some("1".into()),
            },
            ProjectIssueItem {
                item_id: "item-2".into(),
                issue_number: 8,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                assignees: vec!["bob".into(), "alice".into()],
                status_name: Some("Backlog".into()),
                status_option_id: Some("2".into()),
            },
        ];

        let selected =
            select_next_backlog_project_item(&items, &statuses(), "dapi", "teamlead", Some("bob"))
                .unwrap();

        assert_eq!(selected.item_id, "item-2");
    }

    #[test]
    fn skips_unassigned_items_when_assignee_filter_is_set() {
        let items = vec![
            ProjectIssueItem {
                item_id: "item-1".into(),
                issue_number: 7,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                assignees: vec![],
                status_name: Some("Backlog".into()),
                status_option_id: Some("1".into()),
            },
            ProjectIssueItem {
                item_id: "item-2".into(),
                issue_number: 8,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                assignees: vec!["alice".into()],
                status_name: Some("Backlog".into()),
                status_option_id: Some("2".into()),
            },
        ];

        let selected = select_next_backlog_project_item(
            &items,
            &statuses(),
            "dapi",
            "teamlead",
            Some("alice"),
        )
        .unwrap();

        assert_eq!(selected.item_id, "item-2");
    }
}
