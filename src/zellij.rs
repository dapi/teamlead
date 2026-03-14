use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;

use crate::config::ZellijConfig;
use crate::repo::RepoContext;
use crate::runtime::RuntimeLayout;
use crate::shell::Shell;

const ANALYSIS_TAB_TEMPLATE_PATH: &str = ".ai-teamlead/zellij/analysis-tab.kdl";
const ANALYSIS_TAB_TEMPLATE_NAME_PLACEHOLDER: &str = "${TAB_NAME}";
const ANALYSIS_TAB_TEMPLATE_ENTRYPOINT_PLACEHOLDER: &str = "${PANE_ENTRYPOINT}";

pub struct ZellijLauncher<'a> {
    shell: &'a dyn Shell,
}

impl<'a> ZellijLauncher<'a> {
    pub fn new(shell: &'a dyn Shell) -> Self {
        Self { shell }
    }

    pub fn launch_issue_analysis(
        &self,
        repo: &RepoContext,
        repo_root: &Path,
        runtime: &RuntimeLayout,
        zellij: &ZellijConfig,
        issue_url: &str,
        session_uuid: &str,
        binary_path: &Path,
        debug: bool,
    ) -> Result<()> {
        let session_dir = runtime.session_dir(session_uuid);
        fs::create_dir_all(&session_dir)
            .with_context(|| format!("failed to create {}", session_dir.display()))?;

        let entrypoint_path = session_dir.join("pane-entrypoint.sh");
        let layout_path = session_dir.join("launch-layout.kdl");
        let launch_log_path = session_dir.join("launch.log");
        let analysis_tab_template_path = repo_root.join(ANALYSIS_TAB_TEMPLATE_PATH);
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&launch_log_path)
            .with_context(|| format!("failed to create {}", launch_log_path.display()))?;
        append_launch_log(
            &launch_log_path,
            &format!(
                "analysis-tab-contract: {}",
                analysis_tab_template_path.display()
            ),
        )?;
        let quoted_repo_root = shell_single_quote(repo_root.to_string_lossy().as_ref());
        let quoted_launch_agent = shell_single_quote("./.ai-teamlead/launch-agent.sh");
        let quoted_session_uuid = shell_single_quote(session_uuid);
        let quoted_issue_url = shell_single_quote(issue_url);
        let quoted_binary = shell_single_quote(binary_path.to_string_lossy().as_ref());
        let quoted_launch_log = shell_single_quote(launch_log_path.to_string_lossy().as_ref());
        let debug_flag = if debug { "1" } else { "0" };

        let entrypoint = format!(
            "#!/usr/bin/env bash\n\
set -euo pipefail\n\
cd {quoted_repo_root}\n\
export AI_TEAMLEAD_BIN={quoted_binary}\n\
export AI_TEAMLEAD_DEBUG={debug_flag}\n\
export AI_TEAMLEAD_LAUNCH_LOG={quoted_launch_log}\n\
mkdir -p \"$(dirname \"$AI_TEAMLEAD_LAUNCH_LOG\")\"\n\
printf '[%s] pane-entrypoint: session_uuid=%s issue_url=%s debug=%s\\n' \"$(date -Iseconds)\" {quoted_session_uuid} {quoted_issue_url} \"$AI_TEAMLEAD_DEBUG\" >>\"$AI_TEAMLEAD_LAUNCH_LOG\"\n\
exec {quoted_launch_agent} {quoted_session_uuid} {quoted_issue_url}\n"
        );
        fs::write(&entrypoint_path, entrypoint)
            .with_context(|| format!("failed to write {}", entrypoint_path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&entrypoint_path)
                .with_context(|| format!("failed to stat {}", entrypoint_path.display()))?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&entrypoint_path, permissions).with_context(|| {
                format!(
                    "failed to set executable bit on {}",
                    entrypoint_path.display()
                )
            })?;
        }

        let analysis_tab_template = fs::read_to_string(&analysis_tab_template_path)
            .with_context(|| format!("failed to read {}", analysis_tab_template_path.display()))?;
        let layout = render_analysis_tab_layout(
            &analysis_tab_template,
            &zellij.tab_name,
            entrypoint_path.to_string_lossy().as_ref(),
        )
        .with_context(|| {
            format!(
                "failed to render analysis tab contract {}",
                analysis_tab_template_path.display()
            )
        })?;
        fs::write(&layout_path, layout)
            .with_context(|| format!("failed to write {}", layout_path.display()))?;

        let sessions = self
            .shell
            .run(repo_root, "zellij", &["list-sessions", "--short"])
            .unwrap_or_default();
        let session_exists = sessions
            .lines()
            .any(|line| line.trim() == zellij.session_name);

        let layout_str = layout_path.to_string_lossy();

        if session_exists {
            append_launch_log(
                &launch_log_path,
                "launcher-path: existing session, add analysis tab",
            )?;
            ensure_session_repo_scope(self.shell, repo_root, repo, &zellij.session_name)?;
            add_analysis_tab_to_existing_session(
                self.shell,
                repo_root,
                &zellij.session_name,
                &layout_str,
            )?;
            Ok(())
        } else {
            let launch = if let Some(layout_name) = zellij.layout.as_deref() {
                append_launch_log(
                    &launch_log_path,
                    &format!("launcher-path: custom layout '{layout_name}'"),
                )?;
                NewSessionLaunch::CustomLayout(layout_name)
            } else {
                append_launch_log(
                    &launch_log_path,
                    "launcher-path: default fallback session create",
                )?;
                NewSessionLaunch::DefaultFallback
            };
            spawn_new_session(
                self.shell,
                repo_root,
                &zellij.session_name,
                launch,
                &launch_log_path,
            )?;
            wait_for_session_creation(
                self.shell,
                repo_root,
                &zellij.session_name,
                &launch_log_path,
            )?;
            add_analysis_tab_with_retry(
                self.shell,
                repo_root,
                &zellij.session_name,
                &layout_str,
                &launch_log_path,
            )
        }
    }
}

enum NewSessionLaunch<'a> {
    CustomLayout(&'a str),
    DefaultFallback,
}

fn spawn_new_session(
    shell: &dyn Shell,
    repo_root: &Path,
    session_name: &str,
    launch: NewSessionLaunch<'_>,
    launch_log_path: &Path,
) -> Result<()> {
    let zellij_command = match launch {
        NewSessionLaunch::CustomLayout(layout_name) => format!(
            "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID zellij --session {} --layout {}",
            shell_single_quote(session_name),
            shell_single_quote(layout_name)
        ),
        NewSessionLaunch::DefaultFallback => format!(
            "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID zellij --session {}",
            shell_single_quote(session_name)
        ),
    };

    shell.spawn_with_env(
        repo_root,
        &[],
        "script",
        &["-qfc", &zellij_command, "/dev/null"],
        Some(launch_log_path),
    )
}

fn wait_for_session_creation(
    shell: &dyn Shell,
    repo_root: &Path,
    session_name: &str,
    launch_log_path: &Path,
) -> Result<()> {
    for attempt in 1..=20 {
        if let Ok(sessions) = shell.run(repo_root, "zellij", &["list-sessions", "--short"]) {
            if sessions.lines().any(|line| line.trim() == session_name) {
                append_launch_log(
                    launch_log_path,
                    &format!("launcher-step: session created on attempt {attempt}"),
                )?;
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    let launch_log_excerpt = read_launch_log_excerpt(launch_log_path);
    Err(anyhow!(
        "failed to create zellij session '{}'; session did not appear after spawn; launch_log_excerpt={}",
        session_name,
        launch_log_excerpt
    ))
    .context("failed at launcher step: create session")
}

fn add_analysis_tab_to_existing_session(
    shell: &dyn Shell,
    repo_root: &Path,
    session_name: &str,
    layout_path: &str,
) -> Result<()> {
    // For existing sessions: use `zellij action new-tab` IPC command.
    // Clear inherited ZELLIJ_* vars before session-scoped IPC. When
    // ai-teamlead runs inside another zellij pane, leaking the outer
    // ZELLIJ_PANE_ID breaks list-panes/new-tab against the target
    // existing session on zellij 0.44.
    run_session_scoped_zellij_action(
        shell,
        repo_root,
        session_name,
        &["action", "new-tab", "--layout", layout_path],
    )?;
    Ok(())
}

fn add_analysis_tab_with_retry(
    shell: &dyn Shell,
    repo_root: &Path,
    session_name: &str,
    layout_path: &str,
    launch_log_path: &Path,
) -> Result<()> {
    let mut last_error = None;
    for attempt in 1..=10 {
        match add_analysis_tab_to_existing_session(shell, repo_root, session_name, layout_path) {
            Ok(()) => {
                append_launch_log(
                    launch_log_path,
                    &format!("launcher-step: analysis tab attached on attempt {attempt}"),
                )?;
                return Ok(());
            }
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }

    let error = last_error.expect("retry loop must capture at least one error");
    append_launch_log(
        launch_log_path,
        &format!("launcher-step: failed to attach analysis tab: {error:#}"),
    )?;
    Err(error).context("failed at launcher step: add analysis tab")
}

fn run_session_scoped_zellij_action(
    shell: &dyn Shell,
    cwd: &Path,
    session_name: &str,
    args: &[&str],
) -> Result<String> {
    let mut command = vec![
        "-u".to_string(),
        "ZELLIJ".to_string(),
        "-u".to_string(),
        "ZELLIJ_SESSION_NAME".to_string(),
        "-u".to_string(),
        "ZELLIJ_PANE_ID".to_string(),
        "ZELLIJ=0".to_string(),
        format!("ZELLIJ_SESSION_NAME={session_name}"),
        "zellij".to_string(),
    ];
    command.extend(args.iter().map(|arg| arg.to_string()));
    let command_refs = command.iter().map(String::as_str).collect::<Vec<_>>();
    shell.run(cwd, "env", &command_refs)
}

fn ensure_session_repo_scope(
    shell: &dyn Shell,
    repo_root: &Path,
    repo: &RepoContext,
    session_name: &str,
) -> Result<()> {
    let panes_output = run_session_scoped_zellij_action(
        shell,
        repo_root,
        session_name,
        &["action", "list-panes", "--json", "-a", "-c", "-t", "-s"],
    )?;
    let foreign_repos = find_foreign_repos_in_session(shell, &panes_output, repo)?;
    if foreign_repos.is_empty() {
        return Ok(());
    }

    bail!(
        "zellij session '{}' already contains panes from other repos: {}; shared multi-repo sessions are not allowed",
        session_name,
        foreign_repos.into_iter().collect::<Vec<_>>().join(", ")
    );
}

fn find_foreign_repos_in_session(
    shell: &dyn Shell,
    panes_output: &str,
    repo: &RepoContext,
) -> Result<BTreeSet<String>> {
    let value: Value = serde_json::from_str(panes_output)
        .context("failed to parse zellij pane metadata while validating session repo scope")?;
    let mut pane_cwds = BTreeSet::new();
    collect_pane_cwds(&value, &mut pane_cwds);

    let expected_repo = format!("{}/{}", repo.github_owner, repo.github_repo);
    let mut foreign_repos = BTreeSet::new();
    for pane_cwd in pane_cwds {
        let pane_path = Path::new(&pane_cwd);
        if !pane_path.is_dir() {
            continue;
        }
        let Ok(pane_repo) = RepoContext::discover(shell, pane_path) else {
            continue;
        };
        let pane_repo_slug = format!("{}/{}", pane_repo.github_owner, pane_repo.github_repo);
        if pane_repo_slug != expected_repo {
            foreign_repos.insert(pane_repo_slug);
        }
    }

    Ok(foreign_repos)
}

pub fn capture_current_binding(
    shell: &dyn Shell,
    repo_root: &Path,
    runtime: &RuntimeLayout,
    zellij: &ZellijConfig,
    session_uuid: &str,
) -> Result<(String, String, String)> {
    let pane_id = std::env::var("ZELLIJ_PANE_ID")
        .context("ZELLIJ_PANE_ID is not set in the launched zellij pane")?;
    let session_id =
        std::env::var("ZELLIJ_SESSION_NAME").unwrap_or_else(|_| zellij.session_name.clone());
    let current_tab_output = shell
        .run(
            repo_root,
            "zellij",
            &["action", "current-tab-info", "--json"],
        )
        .ok();
    let list_panes_output = shell
        .run(
            repo_root,
            "zellij",
            &["action", "list-panes", "--json", "-t", "-s"],
        )
        .ok();
    let tab_id = current_tab_output
        .as_deref()
        .and_then(resolve_tab_id)
        .or_else(|| {
            list_panes_output
                .as_deref()
                .and_then(|json| resolve_tab_id_from_panes(json, &pane_id))
        });
    let tab_id = tab_id.ok_or_else(|| {
        anyhow!(
            "failed to resolve zellij tab_id for current pane; current_tab_output={:?}; list_panes_output={:?}",
            current_tab_output,
            list_panes_output
        )
    })?;

    runtime.update_zellij_binding(session_uuid, &session_id, &tab_id, &pane_id)?;
    Ok((session_id, tab_id, pane_id))
}

fn resolve_tab_id(json: &str) -> Option<String> {
    let value: Value = serde_json::from_str(json).ok()?;
    find_first_scalar_by_keys(&value, &["tab_id", "tabId", "id"])
}

fn resolve_tab_id_from_panes(json: &str, pane_id: &str) -> Option<String> {
    let value: Value = serde_json::from_str(json).ok()?;
    find_object_for_pane(&value, pane_id)
        .and_then(|entry| find_first_scalar_by_keys(entry, &["tab_id", "tabId", "tab"]))
}

fn find_object_for_pane<'a>(value: &'a Value, pane_id: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if map
                .get("id")
                .and_then(value_to_string)
                .map(|value| value == pane_id)
                .unwrap_or(false)
                || map
                    .get("pane_id")
                    .and_then(value_to_string)
                    .map(|value| value == pane_id)
                    .unwrap_or(false)
            {
                return Some(value);
            }
            map.values()
                .find_map(|child| find_object_for_pane(child, pane_id))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|child| find_object_for_pane(child, pane_id)),
        _ => None,
    }
}

fn collect_pane_cwds(value: &Value, pane_cwds: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(cwd) = map.get("pane_cwd").and_then(value_to_string) {
                pane_cwds.insert(cwd);
            }
            for child in map.values() {
                collect_pane_cwds(child, pane_cwds);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_pane_cwds(child, pane_cwds);
            }
        }
        _ => {}
    }
}

fn find_first_scalar_by_keys(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(found) = map.get(*key).and_then(value_to_string) {
                    return Some(found);
                }
            }
            map.values()
                .find_map(|child| find_first_scalar_by_keys(child, keys))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|child| find_first_scalar_by_keys(child, keys)),
        _ => None,
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn escape_kdl_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn render_analysis_tab_layout(
    template: &str,
    tab_name: &str,
    pane_entrypoint: &str,
) -> Result<String> {
    if !template.contains(ANALYSIS_TAB_TEMPLATE_NAME_PLACEHOLDER) {
        bail!(
            "analysis tab template must contain {} placeholder",
            ANALYSIS_TAB_TEMPLATE_NAME_PLACEHOLDER
        );
    }
    if !template.contains(ANALYSIS_TAB_TEMPLATE_ENTRYPOINT_PLACEHOLDER) {
        bail!(
            "analysis tab template must contain {} placeholder",
            ANALYSIS_TAB_TEMPLATE_ENTRYPOINT_PLACEHOLDER
        );
    }

    Ok(template
        .replace(
            ANALYSIS_TAB_TEMPLATE_NAME_PLACEHOLDER,
            &escape_kdl_string(tab_name),
        )
        .replace(
            ANALYSIS_TAB_TEMPLATE_ENTRYPOINT_PLACEHOLDER,
            &escape_kdl_string(pane_entrypoint),
        ))
}

fn append_launch_log(path: &Path, line: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    writeln!(file, "{line}").with_context(|| format!("failed to append {}", path.display()))
}

fn read_launch_log_excerpt(path: &Path) -> String {
    match fs::read_to_string(path) {
        Ok(contents) => contents
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" | "),
        Err(error) => format!("<failed to read {}: {}>", path.display(), error),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::{BTreeMap, VecDeque};
    use std::path::Path;

    use anyhow::{Result, anyhow};
    use tempfile::tempdir;

    use super::{
        ZellijLauncher, render_analysis_tab_layout, resolve_tab_id, resolve_tab_id_from_panes,
    };
    use crate::config::ZellijConfig;
    use crate::repo::RepoContext;
    use crate::runtime::RuntimeLayout;
    use crate::shell::Shell;

    #[derive(Default)]
    struct FakeShell {
        responses: BTreeMap<String, String>,
        response_sequences: RefCell<BTreeMap<String, VecDeque<String>>>,
        runs: RefCell<Vec<String>>,
        spawns: RefCell<Vec<String>>,
        run_with_env_calls: RefCell<Vec<(String, Vec<(String, String)>)>>,
    }

    impl FakeShell {
        fn with_response(mut self, key: &str, value: &str) -> Self {
            self.responses.insert(key.to_string(), value.to_string());
            self
        }

        fn with_response_sequence(self, key: &str, values: &[&str]) -> Self {
            self.response_sequences.borrow_mut().insert(
                key.to_string(),
                values.iter().map(|value| value.to_string()).collect(),
            );
            self
        }

        fn with_cwd_response(
            mut self,
            cwd: &Path,
            program: &str,
            args: &[&str],
            value: &str,
        ) -> Self {
            self.responses.insert(
                format!("cwd={}::{program} {}", cwd.display(), args.join(" ")),
                value.to_string(),
            );
            self
        }
    }

    impl Shell for FakeShell {
        fn run(&self, _cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
            let cwd_key = format!("cwd={}::{program} {}", _cwd.display(), args.join(" "));
            self.runs
                .borrow_mut()
                .push(format!("{program} {}", args.join(" ")));
            if let Some(value) = self
                .response_sequences
                .borrow_mut()
                .get_mut(&cwd_key)
                .and_then(VecDeque::pop_front)
            {
                return Ok(value);
            }
            if let Some(value) = self.responses.get(&cwd_key) {
                return Ok(value.clone());
            }
            let key = format!("{program} {}", args.join(" "));
            if let Some(value) = self
                .response_sequences
                .borrow_mut()
                .get_mut(&key)
                .and_then(VecDeque::pop_front)
            {
                return Ok(value);
            }
            if let Some(value) = self.responses.get(&key) {
                return Ok(value.clone());
            }
            for (prefix, value) in &self.responses {
                if key.starts_with(prefix) {
                    return Ok(value.clone());
                }
            }
            Err(anyhow!("missing fake response for: {key}"))
        }

        fn run_with_env(
            &self,
            _cwd: &Path,
            envs: &[(&str, &str)],
            program: &str,
            args: &[&str],
        ) -> Result<String> {
            let cwd_key = format!("cwd={}::{program} {}", _cwd.display(), args.join(" "));
            if let Some(value) = self.responses.get(&cwd_key) {
                return Ok(value.clone());
            }
            let key = format!("{program} {}", args.join(" "));
            let env_pairs: Vec<(String, String)> = envs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            self.run_with_env_calls
                .borrow_mut()
                .push((key.clone(), env_pairs));
            // Try exact match first, then prefix match for dynamic paths
            if let Some(value) = self.responses.get(&key) {
                return Ok(value.clone());
            }
            for (prefix, value) in &self.responses {
                if key.starts_with(prefix) {
                    return Ok(value.clone());
                }
            }
            Err(anyhow!("missing fake response for: {key}"))
        }

        fn spawn_with_env(
            &self,
            _cwd: &Path,
            _envs: &[(&str, &str)],
            program: &str,
            args: &[&str],
            _stdout_stderr_log_path: Option<&Path>,
        ) -> Result<()> {
            self.spawns
                .borrow_mut()
                .push(format!("{program} {}", args.join(" ")));
            Ok(())
        }
    }

    fn write_analysis_tab_template(repo_root: &Path, template: &str) {
        let zellij_dir = repo_root.join(".ai-teamlead/zellij");
        std::fs::create_dir_all(&zellij_dir).expect("zellij dir");
        std::fs::write(zellij_dir.join("analysis-tab.kdl"), template).expect("analysis tab");
    }

    fn default_analysis_tab_template() -> &'static str {
        "layout {\n  tab name=\"${TAB_NAME}\" {\n    pane command=\"bash\" {\n      args \"${PANE_ENTRYPOINT}\"\n    }\n  }\n}\n"
    }

    #[test]
    fn resolves_tab_id_from_current_tab_info() {
        let json = r#"{"name":"issue-analysis","tab_id":7}"#;
        assert_eq!(resolve_tab_id(json).as_deref(), Some("7"));
    }

    #[test]
    fn resolves_tab_id_from_panes_output() {
        let json = r#"[
  {"id":"terminal_4","tab_id":9,"focused":true},
  {"id":"terminal_5","tab_id":11,"focused":false}
]"#;
        assert_eq!(
            resolve_tab_id_from_panes(json, "terminal_4").as_deref(),
            Some("9")
        );
    }

    #[test]
    fn launcher_uses_new_session_layout_when_session_is_missing() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime");

        let shell = FakeShell::default()
            .with_response_sequence("zellij list-sessions --short", &["", "ai-teamlead"])
            .with_response(
                "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID ZELLIJ=0 ZELLIJ_SESSION_NAME=ai-teamlead zellij action new-tab --layout",
                "",
            );
        let launcher = ZellijLauncher::new(&shell);
        write_analysis_tab_template(&repo_root, default_analysis_tab_template());
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir: git_dir.clone(),
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
            layout: None,
        };

        launcher
            .launch_issue_analysis(
                &repo,
                &repo_root,
                &runtime,
                &zellij,
                "https://github.com/dapi/teamlead/issues/42",
                "session-uuid",
                Path::new("/tmp/ai-teamlead"),
                false,
            )
            .expect("launch should succeed");

        let spawns = shell.spawns.borrow();
        assert_eq!(spawns.len(), 1);
        assert!(spawns[0].contains("script -qfc"));
        assert!(spawns[0].contains("--session 'ai-teamlead'"));
        assert!(
            !spawns[0].contains(" -n "),
            "default fallback must not use generated layout for session create: {}",
            spawns[0]
        );
        // Verify ZELLIJ env vars are cleared to prevent server crash
        assert!(
            spawns[0].contains("env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID"),
            "new session command must clear ZELLIJ env vars, got: {}",
            spawns[0]
        );
        let runs = shell.runs.borrow();
        assert!(
            runs.iter()
                .any(|cmd| cmd.contains("zellij action new-tab --layout")),
            "new session path must attach analysis tab after session create"
        );
        assert!(
            runtime
                .session_dir("session-uuid")
                .join("launch-layout.kdl")
                .exists()
        );
        assert!(
            runtime
                .session_dir("session-uuid")
                .join("pane-entrypoint.sh")
                .exists()
        );
        assert!(
            runtime
                .session_dir("session-uuid")
                .join("launch.log")
                .exists()
        );
    }

    #[test]
    fn launcher_uses_named_layout_when_configured_for_new_session() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime");

        let shell = FakeShell::default()
            .with_response_sequence("zellij list-sessions --short", &["", "ai-teamlead"])
            .with_response(
                "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID ZELLIJ=0 ZELLIJ_SESSION_NAME=ai-teamlead zellij action new-tab --layout",
                "",
            );
        let launcher = ZellijLauncher::new(&shell);
        write_analysis_tab_template(&repo_root, default_analysis_tab_template());
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir: git_dir.clone(),
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
            layout: Some("custom-layout".into()),
        };

        launcher
            .launch_issue_analysis(
                &repo,
                &repo_root,
                &runtime,
                &zellij,
                "https://github.com/dapi/teamlead/issues/42",
                "session-uuid",
                Path::new("/tmp/ai-teamlead"),
                false,
            )
            .expect("launch should succeed");

        let spawns = shell.spawns.borrow();
        assert_eq!(spawns.len(), 1);
        assert!(
            spawns[0].contains("--session 'ai-teamlead' --layout 'custom-layout'"),
            "custom layout path must create session via named layout: {}",
            spawns[0]
        );
    }

    #[test]
    fn launcher_uses_action_new_tab_for_existing_session() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime");

        let shell = FakeShell::default()
            .with_response("zellij list-sessions --short", "ai-teamlead")
            .with_response(
                "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID ZELLIJ=0 ZELLIJ_SESSION_NAME=ai-teamlead zellij action list-panes --json -a -c -t -s",
                &format!(
                    r#"[{{"id":"terminal_1","pane_cwd":"{}"}}]"#,
                    repo_root.display()
                ),
            )
            .with_cwd_response(
                &repo_root,
                "git",
                &["rev-parse", "--show-toplevel"],
                repo_root.to_string_lossy().as_ref(),
            )
            .with_cwd_response(&repo_root, "git", &["rev-parse", "--git-dir"], ".git")
            .with_cwd_response(
                &repo_root,
                "git",
                &["remote", "get-url", "origin"],
                "git@github.com:dapi/teamlead.git",
            )
            .with_response(
                "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID ZELLIJ=0 ZELLIJ_SESSION_NAME=ai-teamlead zellij action new-tab --layout",
                "",
            );
        let launcher = ZellijLauncher::new(&shell);
        write_analysis_tab_template(&repo_root, default_analysis_tab_template());
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir: git_dir.clone(),
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
            layout: None,
        };

        launcher
            .launch_issue_analysis(
                &repo,
                &repo_root,
                &runtime,
                &zellij,
                "https://github.com/dapi/teamlead/issues/42",
                "session-uuid",
                Path::new("/tmp/ai-teamlead"),
                false,
            )
            .expect("launch should succeed");

        // No script -qfc spawn for existing sessions
        let spawns = shell.spawns.borrow();
        assert_eq!(
            spawns.len(),
            0,
            "existing session must not use spawn/script"
        );

        // Verify existing-session IPC clears inherited ZELLIJ_* vars.
        let runs = shell.runs.borrow();
        let zellij_call = runs
            .iter()
            .find(|cmd| cmd.contains("zellij action new-tab --layout"))
            .expect("should call zellij action new-tab");
        assert!(
            zellij_call.contains("env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID"),
            "existing session IPC must clear inherited ZELLIJ vars, got: {zellij_call}"
        );
        assert!(
            zellij_call.contains("ZELLIJ_SESSION_NAME=ai-teamlead"),
            "existing session IPC must target requested session, got: {zellij_call}"
        );
        assert!(
            shell.run_with_env_calls.borrow().is_empty(),
            "existing session path should not rely on inherited env via run_with_env"
        );
    }

    #[test]
    fn launcher_rejects_existing_session_with_foreign_repo_panes() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let foreign_root = temp.path().join("foreign");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");
        std::fs::create_dir_all(&foreign_root).expect("foreign dir");

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime");

        let shell = FakeShell::default()
            .with_response("zellij list-sessions --short", "ai-teamlead")
            .with_response(
                "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID ZELLIJ=0 ZELLIJ_SESSION_NAME=ai-teamlead zellij action list-panes --json -a -c -t -s",
                &format!(
                    r#"[{{"id":"terminal_9","pane_cwd":"{}"}}]"#,
                    foreign_root.display()
                ),
            )
            .with_cwd_response(
                &foreign_root,
                "git",
                &["rev-parse", "--show-toplevel"],
                foreign_root.to_string_lossy().as_ref(),
            )
            .with_cwd_response(&foreign_root, "git", &["rev-parse", "--git-dir"], ".git")
            .with_cwd_response(
                &foreign_root,
                "git",
                &["remote", "get-url", "origin"],
                "git@github.com:dapi/foreign.git",
            );
        let launcher = ZellijLauncher::new(&shell);
        write_analysis_tab_template(&repo_root, default_analysis_tab_template());
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir,
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
            layout: None,
        };

        let error = launcher
            .launch_issue_analysis(
                &repo,
                &repo_root,
                &runtime,
                &zellij,
                "https://github.com/dapi/teamlead/issues/42",
                "session-uuid",
                Path::new("/tmp/ai-teamlead"),
                false,
            )
            .expect_err("launch should fail for multi-repo session");

        assert!(
            error
                .to_string()
                .contains("shared multi-repo sessions are not allowed"),
            "unexpected error: {error:#}"
        );
        assert!(
            error.to_string().contains("dapi/foreign"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn layout_does_not_force_close_on_exit_false() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime");

        let shell = FakeShell::default()
            .with_response_sequence("zellij list-sessions --short", &["", "ai-teamlead"])
            .with_response(
                "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID ZELLIJ=0 ZELLIJ_SESSION_NAME=ai-teamlead zellij action new-tab --layout",
                "",
            );
        let launcher = ZellijLauncher::new(&shell);
        write_analysis_tab_template(&repo_root, default_analysis_tab_template());
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir: git_dir.clone(),
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
            layout: None,
        };

        launcher
            .launch_issue_analysis(
                &repo,
                &repo_root,
                &runtime,
                &zellij,
                "https://github.com/dapi/teamlead/issues/42",
                "session-uuid",
                Path::new("/tmp/ai-teamlead"),
                false,
            )
            .expect("launch should succeed");

        let layout_content = std::fs::read_to_string(
            runtime
                .session_dir("session-uuid")
                .join("launch-layout.kdl"),
        )
        .expect("layout file");
        assert!(
            !layout_content.contains("close_on_exit false"),
            "layout must not force close_on_exit false; pane should use default exit behavior, got: {}",
            layout_content
        );
    }

    #[test]
    fn launcher_renders_analysis_tab_from_project_contract() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime");

        let shell = FakeShell::default()
            .with_response_sequence("zellij list-sessions --short", &["", "ai-teamlead"])
            .with_response(
                "env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID ZELLIJ=0 ZELLIJ_SESSION_NAME=ai-teamlead zellij action new-tab --layout",
                "",
            );
        let launcher = ZellijLauncher::new(&shell);
        write_analysis_tab_template(
            &repo_root,
            "layout {\n  tab name=\"${TAB_NAME}\" split_direction=\"vertical\" {\n    pane command=\"bash\" {\n      args \"${PANE_ENTRYPOINT}\"\n    }\n  }\n}\n",
        );
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir: git_dir.clone(),
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
            layout: None,
        };

        launcher
            .launch_issue_analysis(
                &repo,
                &repo_root,
                &runtime,
                &zellij,
                "https://github.com/dapi/teamlead/issues/42",
                "session-uuid",
                Path::new("/tmp/ai-teamlead"),
                false,
            )
            .expect("launch should succeed");

        let layout_content = std::fs::read_to_string(
            runtime
                .session_dir("session-uuid")
                .join("launch-layout.kdl"),
        )
        .expect("layout file");
        assert!(layout_content.contains("split_direction=\"vertical\""));
        assert!(layout_content.contains("name=\"issue-analysis\""));
        assert!(
            !layout_content.contains("${TAB_NAME}")
                && !layout_content.contains("${PANE_ENTRYPOINT}")
        );
    }

    #[test]
    fn launcher_reports_create_session_failure_before_attaching_tab() {
        let temp = tempdir().expect("temp dir");
        let repo_root = temp.path().join("repo");
        let git_dir = repo_root.join(".git");
        std::fs::create_dir_all(&git_dir).expect("git dir");

        let runtime = RuntimeLayout::from_repo_root(&repo_root);
        runtime.ensure_exists().expect("runtime");

        let shell = FakeShell::default().with_response("zellij list-sessions --short", "");
        let launcher = ZellijLauncher::new(&shell);
        write_analysis_tab_template(&repo_root, default_analysis_tab_template());
        let repo = RepoContext {
            repo_root: repo_root.clone(),
            git_dir: git_dir.clone(),
            github_owner: "dapi".into(),
            github_repo: "teamlead".into(),
        };
        let zellij = ZellijConfig {
            session_name: "ai-teamlead".into(),
            tab_name: "issue-analysis".into(),
            layout: Some("missing-layout".into()),
        };

        let error = launcher
            .launch_issue_analysis(
                &repo,
                &repo_root,
                &runtime,
                &zellij,
                "https://github.com/dapi/teamlead/issues/42",
                "session-uuid",
                Path::new("/tmp/ai-teamlead"),
                false,
            )
            .expect_err("launch should fail");

        let message = error.to_string();
        assert!(message.contains("failed at launcher step: create session"));
        assert!(!message.contains("failed at launcher step: add analysis tab"));
    }

    #[test]
    fn rejects_analysis_tab_template_without_required_placeholders() {
        let error = render_analysis_tab_layout(
            "layout { tab { pane command=\"bash\" { args \"literal\" } } }\n",
            "issue-analysis",
            "/tmp/pane-entrypoint.sh",
        )
        .expect_err("template must fail");
        assert!(error.to_string().contains("${TAB_NAME}"));
    }
}
