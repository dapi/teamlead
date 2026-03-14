use crate::config::FlowStatuses;
use crate::github::ProjectIssueItem;

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
) -> Option<&'a ProjectIssueItem> {
    items
        .iter()
        .filter(|item| item.matches_repo(owner, repo))
        .filter(|item| item.issue_state == "OPEN")
        .find(|item| item.status_name.as_deref() == Some(statuses.backlog.as_str()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunDecision {
    pub allowed: bool,
    pub reason: &'static str,
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
                status_name: Some("Backlog".into()),
                status_option_id: Some("1".into()),
            },
            ProjectIssueItem {
                item_id: "item-2".into(),
                issue_number: 7,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                status_name: Some("Backlog".into()),
                status_option_id: Some("2".into()),
            },
            ProjectIssueItem {
                item_id: "item-3".into(),
                issue_number: 5,
                issue_state: "OPEN".into(),
                repo_owner: "dapi".into(),
                repo_name: "teamlead".into(),
                status_name: Some("Backlog".into()),
                status_option_id: Some("3".into()),
            },
        ];

        let selected =
            select_next_backlog_project_item(&items, &statuses(), "dapi", "teamlead").unwrap();
        assert_eq!(selected.item_id, "item-2");
    }
}
