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
    use super::{render_template, render_zellij_session_name};

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
}
