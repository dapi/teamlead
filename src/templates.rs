use anyhow::{Result, bail};

pub fn render_template(template: &str, variables: &[(&str, &str)]) -> String {
    let mut rendered = template.to_string();
    for (key, value) in variables {
        rendered = rendered.replace(&format!("${{{key}}}"), value);
    }
    rendered
}

pub fn render_zellij_session_name(template: &str, repo: &str) -> Result<String> {
    let rendered = render_template(template, &[("REPO", repo)]);
    let unresolved = find_placeholders(&rendered);
    if !unresolved.is_empty() {
        bail!(
            "invalid zellij.session_name: only ${{REPO}} is supported; unresolved placeholders: {}",
            unresolved.join(", ")
        );
    }
    Ok(rendered)
}

pub fn render_zellij_tab_name(
    fallback: &str,
    template: Option<&str>,
    issue_number: u64,
) -> Result<String> {
    let Some(template) = template else {
        return Ok(fallback.to_string());
    };

    let issue_number = issue_number.to_string();
    let rendered = render_template(template, &[("ISSUE_NUMBER", issue_number.as_str())]);
    let unresolved = find_placeholders(&rendered);
    if !unresolved.is_empty() {
        bail!(
            "invalid zellij.tab_name_template: only ${{ISSUE_NUMBER}} is supported; unresolved placeholders: {}",
            unresolved.join(", ")
        );
    }
    Ok(rendered)
}

fn find_placeholders(value: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut offset = 0;

    while let Some(start) = value[offset..].find("${") {
        let start = offset + start;
        if let Some(end) = value[start + 2..].find('}') {
            let end = start + 2 + end + 1;
            placeholders.push(value[start..end].to_string());
            offset = end;
        } else {
            placeholders.push(value[start..].to_string());
            break;
        }
    }

    placeholders
}

#[cfg(test)]
mod tests {
    use super::{render_template, render_zellij_session_name, render_zellij_tab_name};

    #[test]
    fn renders_template_variables() {
        let rendered = render_template(
            "${HOME}/worktrees/${REPO}/${BRANCH}",
            &[
                ("HOME", "/home/danil"),
                ("REPO", "teamlead"),
                ("BRANCH", "analysis/issue-42"),
            ],
        );

        assert_eq!(rendered, "/home/danil/worktrees/teamlead/analysis/issue-42");
    }

    #[test]
    fn renders_zellij_session_name_from_repo_placeholder() {
        let rendered = render_zellij_session_name("${REPO}", "teamlead").expect("rendered");
        assert_eq!(rendered, "teamlead");
    }

    #[test]
    fn keeps_literal_zellij_session_name() {
        let rendered = render_zellij_session_name("shared-session", "teamlead").expect("rendered");
        assert_eq!(rendered, "shared-session");
    }

    #[test]
    fn rejects_unsupported_zellij_placeholders() {
        let error =
            render_zellij_session_name("${REPO}-${BRANCH}", "teamlead").expect_err("must fail");
        assert!(error.to_string().contains("${BRANCH}"));
    }

    #[test]
    fn keeps_fallback_tab_name_without_template() {
        let rendered =
            render_zellij_tab_name("issue-analysis", None, 42).expect("tab name rendered");
        assert_eq!(rendered, "issue-analysis");
    }

    #[test]
    fn renders_zellij_tab_name_from_issue_number_template() {
        let rendered = render_zellij_tab_name("issue-analysis", Some("#${ISSUE_NUMBER}"), 42)
            .expect("tab name rendered");
        assert_eq!(rendered, "#42");
    }

    #[test]
    fn keeps_literal_zellij_tab_name_template() {
        let rendered = render_zellij_tab_name("issue-analysis", Some("analysis-issue"), 42)
            .expect("tab name rendered");
        assert_eq!(rendered, "analysis-issue");
    }

    #[test]
    fn rejects_unsupported_zellij_tab_name_placeholders() {
        let error = render_zellij_tab_name("issue-analysis", Some("${ISSUE_NUMBER}-${BRANCH}"), 42)
            .expect_err("must fail");
        assert!(error.to_string().contains("${BRANCH}"));
    }
}
