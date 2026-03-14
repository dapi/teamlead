use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use clap::ValueEnum;
use serde::Deserialize;
use uuid::Uuid;

use crate::shell::Shell;

const DEFAULT_SCENARIO_ROOT: &str = ".ai-teamlead/tests/agent-flow";
const DEFAULT_FIXTURES_DIR: &str = ".ai-teamlead/tests/agent-flow/fixtures";
const DEFAULT_ARTIFACTS_DIR_NAME: &str = "test-runs";
const DEFAULT_TIMEOUT_SECONDS: u64 = 900;
const DEFAULT_SANDBOX_IMAGE: &str = "ai-teamlead-agent-flow-test:local";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentFlowMode {
    Stub,
    Live,
}

impl AgentFlowMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stub => "stub",
            Self::Live => "live",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentFlowAgent {
    Stub,
    Codex,
    Claude,
}

impl AgentFlowAgent {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stub => "stub",
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    fn default_mode(self) -> AgentFlowMode {
        match self {
            Self::Stub => AgentFlowMode::Stub,
            Self::Codex | Self::Claude => AgentFlowMode::Live,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentFlowTestRequest {
    pub scenario: String,
    pub agent: Option<AgentFlowAgent>,
    pub mode: Option<AgentFlowMode>,
    pub keep_sandbox: bool,
    pub artifacts_dir: Option<PathBuf>,
    pub timeout_seconds: Option<u64>,
    pub no_build: bool,
}

#[derive(Debug, Clone)]
pub struct AgentFlowTestPlan {
    pub run_id: String,
    pub manifest_path: PathBuf,
    pub manifest: ScenarioManifest,
    pub agent: AgentFlowAgent,
    pub mode: AgentFlowMode,
    pub keep_sandbox: bool,
    pub artifacts_dir: PathBuf,
    pub timeout_seconds: u64,
    pub no_build: bool,
    pub preflight: PreflightSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxRunResult {
    pub run_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub container_name: Option<String>,
    pub image: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioManifest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mode: Option<AgentFlowMode>,
    #[serde(default)]
    pub agent: Option<AgentFlowAgent>,
    #[serde(default)]
    pub fixtures: serde_yaml::Value,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub assertions: Vec<serde_yaml::Value>,
}

impl ScenarioManifest {
    fn validate(&self, scenario_name: &str, path: &Path) -> Result<()> {
        let name = self.name.trim();
        anyhow::ensure!(
            !name.is_empty(),
            "scenario manifest name must not be empty in {}",
            path.display()
        );
        anyhow::ensure!(
            name == scenario_name,
            "scenario manifest name '{}' does not match requested scenario '{}' in {}",
            name,
            scenario_name,
            path.display()
        );
        anyhow::ensure!(
            !self.commands.is_empty(),
            "scenario manifest commands must not be empty in {}",
            path.display()
        );

        if let (Some(agent), Some(mode)) = (self.agent, self.mode) {
            validate_mode_agent_pair(mode, agent)
                .with_context(|| format!("invalid scenario manifest: {}", path.display()))?;
        }

        Ok(())
    }

    fn github_fixture_name(&self) -> Option<&str> {
        self.fixtures
            .get("github_stub")
            .and_then(serde_yaml::Value::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightSummary {
    pub binary_name: Option<String>,
    pub binary_path: Option<PathBuf>,
    pub forwarded_env_vars: Vec<String>,
    pub mounted_paths: Vec<PathBuf>,
    pub auth_path: AuthPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthPath {
    NotRequired,
    ApiKey,
    SubscriptionAccount,
}

impl AuthPath {
    fn as_str(self) -> &'static str {
        match self {
            Self::NotRequired => "not-required",
            Self::ApiKey => "api-key",
            Self::SubscriptionAccount => "subscription-account",
        }
    }
}

#[derive(Debug, Clone)]
struct AgentProfileSpec {
    binary_name: Option<&'static str>,
    auth_env_vars: &'static [&'static str],
    file_mounts: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContainerMount {
    host_path: PathBuf,
    container_path: String,
}

pub fn plan_agent_flow_test(
    repo_root: &Path,
    git_dir: &Path,
    request: &AgentFlowTestRequest,
) -> Result<AgentFlowTestPlan> {
    let manifest_path = resolve_manifest_path(repo_root, &request.scenario)?;
    let manifest = load_manifest(&manifest_path, &request.scenario)?;
    let mode = resolve_effective_mode(&manifest, request.mode)?;
    let agent = resolve_effective_agent(&manifest, request.agent, mode)?;
    validate_mode_agent_pair(mode, agent)?;
    validate_scenario_fixtures(repo_root, &manifest, mode, agent)?;

    let preflight = run_preflight(agent, mode)?;
    let run_id = format!(
        "agent-flow-{}-{}",
        Utc::now().format("%Y%m%dT%H%M%SZ"),
        Uuid::new_v4()
    );
    let artifacts_dir = request
        .artifacts_dir
        .clone()
        .map(|path| normalize_repo_relative_path(repo_root, path))
        .unwrap_or_else(|| {
            git_dir
                .join(".ai-teamlead")
                .join(DEFAULT_ARTIFACTS_DIR_NAME)
        });

    Ok(AgentFlowTestPlan {
        run_id,
        manifest_path,
        manifest,
        agent,
        mode,
        keep_sandbox: request.keep_sandbox,
        artifacts_dir,
        timeout_seconds: request.timeout_seconds.unwrap_or(DEFAULT_TIMEOUT_SECONDS),
        no_build: request.no_build,
        preflight,
    })
}

pub fn run_agent_flow_test(
    shell: &dyn Shell,
    repo_root: &Path,
    git_dir: &Path,
    plan: &AgentFlowTestPlan,
) -> Result<SandboxRunResult> {
    let run_dir = plan.artifacts_dir.join(&plan.run_id);
    let exported_artifacts_dir = run_dir.join("bundle");
    fs::create_dir_all(&exported_artifacts_dir).with_context(|| {
        format!(
            "failed to create sandbox artifact directory {}",
            exported_artifacts_dir.display()
        )
    })?;

    let image = DEFAULT_SANDBOX_IMAGE.to_string();
    ensure_sandbox_image(shell, repo_root, &image, plan.no_build)?;
    let common_git_dir = resolve_common_git_dir(git_dir)?;
    let current_exe = env::current_exe().context("failed to resolve ai-teamlead binary path")?;

    let container_name = if plan.keep_sandbox {
        Some(format!("ai-teamlead-agent-flow-{}", plan.run_id))
    } else {
        None
    };

    run_sandbox_container(
        shell,
        &image,
        repo_root,
        git_dir,
        &common_git_dir,
        &exported_artifacts_dir,
        &current_exe,
        plan,
        container_name.as_deref(),
        plan.keep_sandbox,
    )?;

    let result = SandboxRunResult {
        run_dir,
        artifacts_dir: exported_artifacts_dir,
        container_name,
        image,
    };
    verify_sandbox_result(plan, &result)?;
    Ok(result)
}

fn load_manifest(path: &Path, scenario_name: &str) -> Result<ScenarioManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read scenario manifest: {}", path.display()))?;
    let manifest: ScenarioManifest = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse scenario manifest: {}", path.display()))?;
    manifest.validate(scenario_name, path)?;
    Ok(manifest)
}

fn resolve_manifest_path(repo_root: &Path, scenario_name: &str) -> Result<PathBuf> {
    anyhow::ensure!(
        !scenario_name.trim().is_empty(),
        "scenario name must not be empty"
    );

    let scenario_root = repo_root.join(DEFAULT_SCENARIO_ROOT);
    let yml = scenario_root.join(format!("{scenario_name}.yml"));
    let yaml = scenario_root.join(format!("{scenario_name}.yaml"));

    match (yml.exists(), yaml.exists()) {
        (true, false) => Ok(yml),
        (false, true) => Ok(yaml),
        (true, true) => bail!(
            "scenario manifest is ambiguous for '{}': both {} and {} exist",
            scenario_name,
            yml.display(),
            yaml.display()
        ),
        (false, false) => bail!(
            "scenario manifest '{}' was not found under {}",
            scenario_name,
            scenario_root.display()
        ),
    }
}

fn ensure_sandbox_image(
    shell: &dyn Shell,
    repo_root: &Path,
    image: &str,
    no_build: bool,
) -> Result<()> {
    if no_build {
        shell.run(repo_root, "docker", &["image", "inspect", image])?;
        return Ok(());
    }

    let (tag, sha) = read_pinned_zellij_release(repo_root)?;
    shell.run(
        repo_root,
        "docker",
        &[
            "build",
            "-f",
            "Dockerfile.test",
            "--build-arg",
            &format!("ZELLIJ_TAG={tag}"),
            "--build-arg",
            &format!("ZELLIJ_SHA256={sha}"),
            "-t",
            image,
            ".",
        ],
    )?;
    Ok(())
}

fn run_sandbox_container(
    shell: &dyn Shell,
    image: &str,
    repo_root: &Path,
    git_dir: &Path,
    common_git_dir: &Path,
    artifacts_dir: &Path,
    ai_teamlead_binary: &Path,
    plan: &AgentFlowTestPlan,
    container_name: Option<&str>,
    keep_sandbox: bool,
) -> Result<()> {
    let repo_mount = format!("{}:/input/repo:ro", repo_root.display());
    let git_dir_mount = format!("{}:/input/worktree-git:ro", git_dir.display());
    let common_git_dir_mount = format!("{}:/input/common-git:ro", common_git_dir.display());
    let artifacts_mount = format!("{}:/artifacts", artifacts_dir.display());
    let binary_dir = ai_teamlead_binary.parent().ok_or_else(|| {
        anyhow!(
            "failed to resolve parent directory for ai-teamlead binary {}",
            ai_teamlead_binary.display()
        )
    })?;
    let binary_dir_mount = format!("{}:/input/ai-teamlead-bin:ro", binary_dir.display());
    let binary_name = ai_teamlead_binary
        .file_name()
        .ok_or_else(|| anyhow!("failed to resolve ai-teamlead binary file name"))?
        .to_string_lossy()
        .to_string();
    let container_binary_path = format!("/input/ai-teamlead-bin/{binary_name}");
    let container_home = "/tmp/ai-teamlead-agent-flow-workspace/home";
    let live_agent_mounts = resolve_container_mounts(plan.agent, &plan.preflight.mounted_paths);
    let agent_binary_mount = resolve_agent_binary_mount(plan)?;
    let container_agent_binary_path = agent_binary_mount
        .as_ref()
        .map(|(_, container_binary_path)| container_binary_path.clone());
    let script = sandbox_entrypoint_script(
        plan,
        &container_binary_path,
        container_agent_binary_path.as_deref(),
        container_home,
    )?;

    let mut args = vec!["run".to_string()];
    if !keep_sandbox {
        args.push("--rm".to_string());
    }
    if let Some(container_name) = container_name {
        args.push("--name".to_string());
        args.push(container_name.to_string());
    }
    args.push("-v".to_string());
    args.push(repo_mount);
    args.push("-v".to_string());
    args.push(git_dir_mount);
    args.push("-v".to_string());
    args.push(common_git_dir_mount);
    args.push("-v".to_string());
    args.push(artifacts_mount);
    args.push("-v".to_string());
    args.push(binary_dir_mount);
    if let Some((mount, _)) = agent_binary_mount.as_ref() {
        args.push("-v".to_string());
        args.push(format!(
            "{}:{}:ro",
            mount.host_path.display(),
            mount.container_path
        ));
    }
    for mount in &live_agent_mounts {
        args.push("-v".to_string());
        args.push(format!(
            "{}:{}:ro",
            mount.host_path.display(),
            mount.container_path
        ));
    }
    for env_name in &plan.preflight.forwarded_env_vars {
        args.push("-e".to_string());
        args.push(env_name.clone());
    }
    args.push(image.to_string());
    args.push("bash".to_string());
    args.push("-lc".to_string());
    args.push(script);

    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    shell.run(repo_root, "docker", &arg_refs)?;
    Ok(())
}

fn read_pinned_zellij_release(repo_root: &Path) -> Result<(String, String)> {
    let content = fs::read_to_string(repo_root.join("ZELLIJ_VERSION"))
        .context("failed to read ZELLIJ_VERSION")?;
    let mut parts = content.split_whitespace();
    let tag = parts
        .next()
        .ok_or_else(|| anyhow!("ZELLIJ_VERSION is missing pinned tag"))?;
    let sha = parts
        .next()
        .ok_or_else(|| anyhow!("ZELLIJ_VERSION is missing sha256"))?;
    anyhow::ensure!(
        parts.next().is_none(),
        "ZELLIJ_VERSION contains unexpected extra fields"
    );
    Ok((tag.to_string(), sha.to_string()))
}

fn resolve_common_git_dir(git_dir: &Path) -> Result<PathBuf> {
    let commondir_path = git_dir.join("commondir");
    if !commondir_path.exists() {
        return Ok(git_dir.to_path_buf());
    }

    let commondir = fs::read_to_string(&commondir_path)
        .with_context(|| format!("failed to read {}", commondir_path.display()))?;
    let commondir = commondir.trim();
    anyhow::ensure!(
        !commondir.is_empty(),
        "git commondir must not be empty in {}",
        commondir_path.display()
    );

    let path = PathBuf::from(commondir);
    if path.is_absolute() {
        Ok(path)
    } else {
        let resolved = git_dir.join(path);
        if resolved.exists() {
            fs::canonicalize(&resolved)
                .with_context(|| format!("failed to canonicalize {}", resolved.display()))
        } else {
            Ok(resolved)
        }
    }
}

fn validate_scenario_fixtures(
    repo_root: &Path,
    manifest: &ScenarioManifest,
    _mode: AgentFlowMode,
    _agent: AgentFlowAgent,
) -> Result<()> {
    let github_fixture = manifest
        .github_fixture_name()
        .ok_or_else(|| anyhow!("scenario requires fixtures.github_stub"))?;
    let fixture_path = repo_root.join(DEFAULT_FIXTURES_DIR).join(github_fixture);
    anyhow::ensure!(
        fixture_path.is_file(),
        "github fixture '{}' was not found at {}",
        github_fixture,
        fixture_path.display()
    );
    Ok(())
}

fn resolve_agent_binary_mount(
    plan: &AgentFlowTestPlan,
) -> Result<Option<(ContainerMount, String)>> {
    let Some(binary_path) = plan.preflight.binary_path.as_deref() else {
        return Ok(None);
    };
    let binary_parent = binary_path.parent().ok_or_else(|| {
        anyhow!(
            "failed to resolve parent directory for live agent binary {}",
            binary_path.display()
        )
    })?;
    let mount_root = match fs::read_link(binary_path) {
        Ok(link_target) => {
            let resolved_target = if link_target.is_absolute() {
                link_target
            } else {
                binary_parent.join(link_target)
            };
            let resolved_target = fs::canonicalize(&resolved_target).with_context(|| {
                format!(
                    "failed to canonicalize live agent binary symlink target for {}",
                    binary_path.display()
                )
            })?;
            let mount_root = binary_parent
                .parent()
                .filter(|candidate| resolved_target.starts_with(candidate))
                .unwrap_or(binary_parent);
            mount_root.to_path_buf()
        }
        Err(_) => binary_parent.to_path_buf(),
    };
    let relative_binary_path = binary_path
        .strip_prefix(&mount_root)
        .map_err(|_| {
            anyhow!(
                "live agent binary {} is not inside mount root {}",
                binary_path.display(),
                mount_root.display()
            )
        })?
        .to_string_lossy()
        .to_string();
    Ok(Some((
        ContainerMount {
            host_path: mount_root,
            container_path: "/input/agent-root".to_string(),
        },
        format!("/input/agent-root/{relative_binary_path}"),
    )))
}

fn resolve_container_mounts(
    agent: AgentFlowAgent,
    mounted_paths: &[PathBuf],
) -> Vec<ContainerMount> {
    let mut mounts = Vec::new();
    for path in mounted_paths {
        if let Some(container_path) = map_container_mount_path(agent, path) {
            mounts.push(ContainerMount {
                host_path: path.clone(),
                container_path,
            });
        }
    }
    mounts
}

fn map_container_mount_path(agent: AgentFlowAgent, host_path: &Path) -> Option<String> {
    let value = host_path.to_string_lossy();
    match agent {
        AgentFlowAgent::Stub => None,
        AgentFlowAgent::Codex => {
            if value.ends_with("/.codex")
                || host_path.file_name().and_then(|v| v.to_str()) == Some(".codex")
            {
                Some("/input/live-home/.codex".to_string())
            } else {
                None
            }
        }
        AgentFlowAgent::Claude => {
            if value.ends_with("/.claude")
                || host_path.file_name().and_then(|v| v.to_str()) == Some(".claude")
            {
                Some("/input/live-home/.claude".to_string())
            } else if value.ends_with("/.config/claude") {
                Some("/input/live-home/.config/claude".to_string())
            } else {
                None
            }
        }
    }
}

fn gh_stub_script() -> String {
    r#"#!/usr/bin/env bash
set -euo pipefail

LOG_FILE="${AI_TEAMLEAD_TEST_GH_LOG:?}"
SNAPSHOT_FILE="${AI_TEAMLEAD_TEST_GH_SNAPSHOT:?}"
ARGS="$*"
printf 'gh %s\n' "$ARGS" >> "$LOG_FILE"

if [[ "${1:-}" == "repo" && "${2:-}" == "view" ]]; then
    printf 'main\n'
    exit 0
fi

if [[ "${1:-}" == "pr" && "${2:-}" == "list" ]]; then
    printf '[]\n'
    exit 0
fi

if [[ "${1:-}" == "pr" && "${2:-}" == "create" ]]; then
    printf 'https://github.com/dapi/ai-teamlead/pull/999\n'
    exit 0
fi

if [[ "$ARGS" == *"updateProjectV2ItemFieldValue"* ]]; then
    printf '{"data":{"updateProjectV2ItemFieldValue":{"projectV2Item":{"id":"updated-item"}}}}\n'
    exit 0
fi

cat "$SNAPSHOT_FILE"
"#
    .to_string()
}

fn agent_stub_script() -> String {
    r#"#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${AI_TEAMLEAD_STUB_OUT_DIR:?}"
AI_TEAMLEAD_BIN="${AI_TEAMLEAD_STUB_AI_TEAMLEAD_BIN:?}"
TARGET_CD=""
PROMPT=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --cd)
            TARGET_CD="$2"
            shift 2
            ;;
        --no-alt-screen)
            shift
            ;;
        *)
            PROMPT="$1"
            shift
            ;;
    esac
done

if [[ -n "$TARGET_CD" ]]; then
    cd "$TARGET_CD"
fi

mkdir -p "$OUT_DIR"
mkdir -p "${AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR:?}"
cat > "${AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR}/README.md" <<'__STUB_README__'
# План анализа

- Issue переведен в deterministic sandbox flow.
- Stub-агент завершил stage через internal complete-stage.
__STUB_README__

printf 'invoked\n' > "$OUT_DIR/codex.invoked"
printf '%s\n' "$PWD" > "$OUT_DIR/codex.cwd"
printf '%s\n' "${AI_TEAMLEAD_ISSUE_URL:-}" > "$OUT_DIR/issue_url"
printf '%s\n' "${AI_TEAMLEAD_SESSION_UUID:-}" > "$OUT_DIR/session_uuid"
printf '%s\n' "${AI_TEAMLEAD_ANALYSIS_BRANCH:-}" > "$OUT_DIR/analysis_branch"
printf '%s\n' "${AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR:-}" > "$OUT_DIR/analysis_artifacts_dir"
printf '%s\n' "${AI_TEAMLEAD_WORKTREE_ROOT:-}" > "$OUT_DIR/worktree_root"
printf '%s\n' "$PROMPT" > "$OUT_DIR/prompt.txt"

sleep "${AI_TEAMLEAD_STUB_AGENT_SLEEP:-1}"

"$AI_TEAMLEAD_BIN" internal complete-stage \
    "${AI_TEAMLEAD_SESSION_UUID:?}" \
    --outcome plan-ready \
    --message "stub analysis ready"
"#
    .to_string()
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn sandbox_entrypoint_script(
    plan: &AgentFlowTestPlan,
    container_binary_path: &str,
    container_agent_binary_path: Option<&str>,
    container_home_dir: &str,
) -> Result<String> {
    let mut script = String::from(
        r#"set -euo pipefail
WORKSPACE_BASE="/tmp/ai-teamlead-agent-flow-workspace"
WORKSPACE_ROOT="$WORKSPACE_BASE/repo"
WORKSPACE_GITDIR="$WORKSPACE_ROOT/.git"
WORKSPACE_BIN="$WORKSPACE_BASE/bin"
LOCAL_PUSH_REMOTE="$WORKSPACE_BASE/origin-push.git"
STUB_OUT_DIR="/artifacts/stub-out"
COMMANDS_DIR="/artifacts/commands"
mkdir -p /artifacts "$COMMANDS_DIR"
mkdir -p "$(dirname "$WORKSPACE_ROOT")"
export HOME="#,
    );
    script.push_str(&shell_single_quote(container_home_dir));
    script.push_str(
        r#"
mkdir -p "$HOME"
"#,
    );
    match plan.agent {
        AgentFlowAgent::Codex => {
            script.push_str(
                "if [[ -d /input/live-home/.codex && ! -e \"$HOME/.codex\" ]]; then cp -a /input/live-home/.codex \"$HOME/.codex\"; fi\n",
            );
        }
        AgentFlowAgent::Claude => {
            script.push_str(
                "if [[ -d /input/live-home/.claude && ! -e \"$HOME/.claude\" ]]; then cp -a /input/live-home/.claude \"$HOME/.claude\"; fi\n",
            );
            script.push_str(
                "if [[ -d /input/live-home/.config/claude && ! -e \"$HOME/.config/claude\" ]]; then mkdir -p \"$HOME/.config\" && cp -a /input/live-home/.config/claude \"$HOME/.config/claude\"; fi\n",
            );
        }
        AgentFlowAgent::Stub => {}
    }
    script.push_str(
        r#"
if touch /input/repo/.agent-flow-write-probe 2>/dev/null; then
  echo "repo mount must be read-only" > /artifacts/read-only-violation.txt
  exit 41
fi
mkdir -p "$WORKSPACE_ROOT"
(cd /input/repo && tar --exclude='./target' --exclude='./.git' -cf - .) | tar -xf - -C "$WORKSPACE_ROOT"
mkdir -p "$WORKSPACE_GITDIR"
cp -a /input/common-git/. "$WORKSPACE_GITDIR"
cp -a /input/worktree-git/. "$WORKSPACE_GITDIR"
rm -f "$WORKSPACE_GITDIR/commondir" "$WORKSPACE_GITDIR/gitdir"
rm -rf "$WORKSPACE_GITDIR/.ai-teamlead"
git config --global --add safe.directory "$WORKSPACE_ROOT"
cd "$WORKSPACE_ROOT"
git worktree prune --verbose >/artifacts/git-worktree-prune.log 2>&1 || true
git config user.name "AI Teamlead Sandbox"
git config user.email "ai-teamlead-sandbox@example.invalid"
if ! git show-ref --verify --quiet refs/heads/main; then
  if git show-ref --verify --quiet refs/remotes/origin/main; then
    git branch main refs/remotes/origin/main >/artifacts/git-branch-main.log 2>&1 || true
  else
    git branch main HEAD >/artifacts/git-branch-main.log 2>&1 || true
  fi
fi
rm -rf "$LOCAL_PUSH_REMOTE"
git init --bare "$LOCAL_PUSH_REMOTE" >/artifacts/git-init-bare.log 2>&1
git remote set-url --push origin "$LOCAL_PUSH_REMOTE"
git push "$LOCAL_PUSH_REMOTE" refs/heads/main:refs/heads/main >/artifacts/git-seed-origin.log 2>&1 || true
git symbolic-ref refs/remotes/origin/HEAD refs/remotes/origin/main >/artifacts/git-origin-head.log 2>&1 || true
mkdir -p "$WORKSPACE_BIN"
ln -sf "#,
    );
    script.push_str(&shell_single_quote(container_binary_path));
    script.push_str(
        r#" "$WORKSPACE_BIN/ai-teamlead"
export PATH="$WORKSPACE_BIN:$PATH"
printf '%s\n' "$WORKSPACE_ROOT" > /artifacts/workspace-root.txt
git rev-parse --show-toplevel > /artifacts/git-top-level.txt
git rev-parse --git-dir > /artifacts/git-dir.txt
git status --short --untracked-files=all > /artifacts/workspace-status.txt
printf 'snapshot_prepared\nsandbox_ready\n' > /artifacts/state-transitions.log
printf 'read-only\n' > /artifacts/repo-mount-access.txt
printf 'docker\n' > /artifacts/sandbox-runtime.txt
"#,
    );

    let github_fixture = plan
        .manifest
        .github_fixture_name()
        .ok_or_else(|| anyhow!("scenario requires fixtures.github_stub"))?;
    let fixture_relative_path = format!("{DEFAULT_FIXTURES_DIR}/{github_fixture}");
    let forwarded_env_vars = if plan.preflight.forwarded_env_vars.is_empty() {
        "-".to_string()
    } else {
        plan.preflight.forwarded_env_vars.join(",")
    };
    let forwarded_mounts = {
        let mounts = resolve_container_mounts(plan.agent, &plan.preflight.mounted_paths)
            .into_iter()
            .map(|mount| mount.container_path)
            .collect::<Vec<_>>();
        if mounts.is_empty() {
            "-".to_string()
        } else {
            mounts.join(",")
        }
    };

    script.push_str("mkdir -p \"$STUB_OUT_DIR\"\n");
    script.push_str("cat > \"$WORKSPACE_BIN/gh\" <<'EOF'\n");
    script.push_str(&gh_stub_script());
    script.push_str("EOF\nchmod +x \"$WORKSPACE_BIN/gh\"\n");
    script.push_str("ln -sf \"$WORKSPACE_BIN/gh\" /usr/local/bin/gh\n");
    script.push_str("export AI_TEAMLEAD_TEST_GH_LOG=\"/artifacts/gh.log\"\n");
    let _ = writeln!(
        script,
        "export AI_TEAMLEAD_TEST_GH_SNAPSHOT=\"$WORKSPACE_ROOT/{}\"",
        fixture_relative_path
    );
    let _ = writeln!(
        script,
        "export HOME={}",
        shell_single_quote(container_home_dir)
    );
    let _ = writeln!(
        script,
        "printf '%s\\n' {} > /artifacts/agent-mode.txt",
        shell_single_quote(plan.mode.as_str())
    );
    let _ = writeln!(
        script,
        "printf '%s\\n' {} > /artifacts/agent-profile.txt",
        shell_single_quote(plan.agent.as_str())
    );
    let _ = writeln!(
        script,
        "printf '%s\\n' {} > /artifacts/auth-path.txt",
        shell_single_quote(plan.preflight.auth_path.as_str())
    );
    let _ = writeln!(
        script,
        "printf '%s\\n' {} > /artifacts/forwarded-env-vars.txt",
        shell_single_quote(&forwarded_env_vars)
    );
    let _ = writeln!(
        script,
        "printf '%s\\n' {} > /artifacts/forwarded-mounts.txt",
        shell_single_quote(&forwarded_mounts)
    );

    match (plan.mode, plan.agent) {
        (AgentFlowMode::Stub, AgentFlowAgent::Stub) => {
            script.push_str("cat > \"$WORKSPACE_BIN/codex\" <<'EOF'\n");
            script.push_str(&agent_stub_script());
            script.push_str(
                "EOF\nchmod +x \"$WORKSPACE_BIN/codex\"\nln -sf codex \"$WORKSPACE_BIN/claude\"\n",
            );
            script.push_str("ln -sf \"$WORKSPACE_BIN/codex\" /usr/local/bin/codex\n");
            script.push_str("ln -sf \"$WORKSPACE_BIN/claude\" /usr/local/bin/claude\n");
            script.push_str("export AI_TEAMLEAD_AGENT_BIN=\"$WORKSPACE_BIN/codex\"\n");
            script.push_str("export AI_TEAMLEAD_STUB_OUT_DIR=\"$STUB_OUT_DIR\"\n");
            script.push_str(
                "export AI_TEAMLEAD_STUB_AI_TEAMLEAD_BIN=\"$WORKSPACE_BIN/ai-teamlead\"\n",
            );
            script.push_str("export AI_TEAMLEAD_STUB_AGENT_SLEEP=1\n");
        }
        (AgentFlowMode::Live, AgentFlowAgent::Codex | AgentFlowAgent::Claude) => {
            let live_binary_name = plan
                .preflight
                .binary_name
                .as_deref()
                .ok_or_else(|| anyhow!("live scenario requires resolved agent binary name"))?;
            let container_agent_binary_path = container_agent_binary_path
                .ok_or_else(|| anyhow!("live scenario requires mounted live agent binary"))?;
            let _ = writeln!(
                script,
                "ln -sf {} \"$WORKSPACE_BIN/{}\"",
                shell_single_quote(container_agent_binary_path),
                live_binary_name
            );
            let _ = writeln!(
                script,
                "ln -sf \"$WORKSPACE_BIN/{0}\" \"/usr/local/bin/{0}\"",
                live_binary_name
            );
            let _ = writeln!(
                script,
                "export AI_TEAMLEAD_AGENT_BIN=\"$WORKSPACE_BIN/{}\"",
                live_binary_name
            );
            let _ = writeln!(
                script,
                "export PATH=\"$(dirname {})${{PATH:+:$PATH}}\"",
                shell_single_quote(container_agent_binary_path)
            );
        }
        _ => {
            bail!(
                "unsupported sandbox launch pair mode='{}' agent='{}'",
                plan.mode.as_str(),
                plan.agent.as_str()
            );
        }
    }

    script.push_str(
        r#"if [[ ! -f "$AI_TEAMLEAD_TEST_GH_SNAPSHOT" ]]; then
  echo "missing github fixture: $AI_TEAMLEAD_TEST_GH_SNAPSHOT" > /artifacts/missing-github-fixture.txt
  exit 42
fi

find_primary_issue_index() {
  find "$WORKSPACE_ROOT/.git/.ai-teamlead/issues" -maxdepth 1 -name '*.json' 2>/dev/null | sort | head -n 1
}

wait_for_session_completion() {
  local deadline=$((SECONDS + "#,
        );
    script.push_str(&plan.timeout_seconds.to_string());
    script.push_str(
            r#"))
  while (( SECONDS <= deadline )); do
    local issue_index
    issue_index="$(find_primary_issue_index || true)"
    if [[ -n "$issue_index" && -f "$issue_index" ]]; then
      local session_uuid
      session_uuid="$(jq -r '.session_uuid' "$issue_index")"
      local session_manifest="$WORKSPACE_ROOT/.git/.ai-teamlead/sessions/$session_uuid/session.json"
      if [[ -f "$session_manifest" ]]; then
        local session_status
        session_status="$(jq -r '.status' "$session_manifest")"
        printf '[%s] session_status=%s session_uuid=%s\n' "$(date -Iseconds)" "$session_status" "$session_uuid" >> /artifacts/state-transitions.log
        if [[ "$session_status" == "completed" ]]; then
          printf '%s\n' "$session_uuid" > /artifacts/session_uuid.txt
          return 0
        fi
      fi
    fi
    sleep 1
  done
  echo "session did not reach completed state before timeout" > /artifacts/session-timeout.txt
  return 124
}

run_manifest_command() {
  local index="$1"
  local command="$2"
  local prefix
  prefix="$(printf '%02d' "$index")"
  printf '%s\n' "$command" > "$COMMANDS_DIR/${prefix}.command.txt"
  if bash -lc "$command" >"$COMMANDS_DIR/${prefix}.stdout.log" 2>"$COMMANDS_DIR/${prefix}.stderr.log"; then
    printf '0\n' > "$COMMANDS_DIR/${prefix}.exit-code.txt"
    return 0
  fi
  local exit_code="$?"
  printf '%s\n' "$exit_code" > "$COMMANDS_DIR/${prefix}.exit-code.txt"
  return "$exit_code"
}

capture_runtime_artifacts() {
  mkdir -p /artifacts/runtime
  if [[ -d "$WORKSPACE_ROOT/.git/.ai-teamlead/issues" ]]; then
    cp -a "$WORKSPACE_ROOT/.git/.ai-teamlead/issues" /artifacts/runtime/issues
  fi
  if [[ -d "$WORKSPACE_ROOT/.git/.ai-teamlead/sessions" ]]; then
    cp -a "$WORKSPACE_ROOT/.git/.ai-teamlead/sessions" /artifacts/runtime/sessions
  fi
  if [[ -f "$STUB_OUT_DIR/worktree_root" ]]; then
    local analysis_root
    analysis_root="$(cat "$STUB_OUT_DIR/worktree_root")"
    printf '%s\n' "$analysis_root" > /artifacts/analysis-worktree-root.txt
    if [[ -d "$analysis_root/specs" ]]; then
      cp -a "$analysis_root/specs" /artifacts/analysis-specs
    fi
    git -C "$analysis_root" status --short --untracked-files=all >/artifacts/analysis-worktree-status.txt 2>&1 || true
  fi
}

overall_exit=0
"#,
    );

    for (index, command) in plan.manifest.commands.iter().enumerate() {
        let quoted_command = shell_single_quote(command);
        let command_index = index + 1;
        let _ = writeln!(
            script,
            "run_manifest_command {command_index} {quoted_command} || {{ overall_exit=\"$?\"; capture_runtime_artifacts; printf '%s\\n' \"$overall_exit\" > /artifacts/exit-code.txt; exit \"$overall_exit\"; }}"
        );
        script.push_str(
            "wait_for_session_completion || { overall_exit=\"$?\"; capture_runtime_artifacts; printf '%s\\n' \"$overall_exit\" > /artifacts/exit-code.txt; exit \"$overall_exit\"; }\n",
        );
    }
    script.push_str(
        r#"capture_runtime_artifacts
printf '%s\n' "$overall_exit" > /artifacts/exit-code.txt
"#,
    );

    script.push_str(
        "git status --short --untracked-files=all > /artifacts/workspace-status-post.txt\n",
    );
    Ok(script)
}

fn resolve_effective_mode(
    manifest: &ScenarioManifest,
    requested_mode: Option<AgentFlowMode>,
) -> Result<AgentFlowMode> {
    if let (Some(cli_mode), Some(manifest_mode)) = (requested_mode, manifest.mode) {
        anyhow::ensure!(
            cli_mode == manifest_mode,
            "scenario manifest requires mode='{}', but CLI requested mode='{}'",
            manifest_mode.as_str(),
            cli_mode.as_str()
        );
    }

    Ok(requested_mode
        .or(manifest.mode)
        .or_else(|| manifest.agent.map(AgentFlowAgent::default_mode))
        .unwrap_or(AgentFlowMode::Live))
}

fn resolve_effective_agent(
    manifest: &ScenarioManifest,
    requested_agent: Option<AgentFlowAgent>,
    mode: AgentFlowMode,
) -> Result<AgentFlowAgent> {
    if let (Some(cli_agent), Some(manifest_agent)) = (requested_agent, manifest.agent) {
        anyhow::ensure!(
            cli_agent == manifest_agent,
            "scenario manifest requires agent='{}', but CLI requested agent='{}'",
            manifest_agent.as_str(),
            cli_agent.as_str()
        );
    }

    Ok(requested_agent
        .or(manifest.agent)
        .unwrap_or_else(|| default_agent_for_mode(mode)))
}

fn default_agent_for_mode(mode: AgentFlowMode) -> AgentFlowAgent {
    match mode {
        AgentFlowMode::Stub => AgentFlowAgent::Stub,
        AgentFlowMode::Live => AgentFlowAgent::Codex,
    }
}

fn validate_mode_agent_pair(mode: AgentFlowMode, agent: AgentFlowAgent) -> Result<()> {
    match (mode, agent) {
        (AgentFlowMode::Stub, AgentFlowAgent::Stub) => Ok(()),
        (AgentFlowMode::Live, AgentFlowAgent::Codex | AgentFlowAgent::Claude) => Ok(()),
        (AgentFlowMode::Stub, _) => bail!(
            "mode='stub' разрешает только agent='stub'; получено agent='{}'",
            agent.as_str()
        ),
        (AgentFlowMode::Live, AgentFlowAgent::Stub) => {
            bail!("mode='live' не разрешает agent='stub'")
        }
    }
}

fn run_preflight(agent: AgentFlowAgent, mode: AgentFlowMode) -> Result<PreflightSummary> {
    let home_dir = home_dir()?;
    let path_var = env::var_os("PATH");
    let env_lookup = |name: &str| env::var_os(name);
    run_preflight_with_host(agent, mode, &home_dir, path_var.as_deref(), &env_lookup)
}

fn run_preflight_with_host(
    agent: AgentFlowAgent,
    mode: AgentFlowMode,
    home_dir: &Path,
    path_var: Option<&OsStr>,
    env_lookup: &dyn Fn(&str) -> Option<OsString>,
) -> Result<PreflightSummary> {
    validate_mode_agent_pair(mode, agent)?;
    let spec = profile_spec(agent);

    let binary_path = match spec.binary_name {
        Some(binary_name) => Some(resolve_binary_path(binary_name, path_var).ok_or_else(|| {
            anyhow!(
                "preflight failed: required agent binary '{}' is not available in PATH",
                binary_name
            )
        })?),
        None => None,
    };

    let mut forwarded_env_vars = Vec::new();
    let mut mounted_paths = Vec::new();
    let auth_path = match agent {
        AgentFlowAgent::Stub => AuthPath::NotRequired,
        AgentFlowAgent::Codex | AgentFlowAgent::Claude => {
            for env_name in spec.auth_env_vars {
                if env_lookup(env_name).is_some() {
                    forwarded_env_vars.push((*env_name).to_string());
                }
            }

            for mount in spec.file_mounts {
                let expanded = expand_mount_path(mount, home_dir);
                if expanded.exists() {
                    mounted_paths.push(expanded);
                }
            }

            if !forwarded_env_vars.is_empty() {
                AuthPath::ApiKey
            } else if !mounted_paths.is_empty() {
                AuthPath::SubscriptionAccount
            } else {
                bail!(
                    "preflight failed: no supported auth path found for agent '{}'; checked env vars [{}] and mounts [{}]",
                    agent.as_str(),
                    spec.auth_env_vars.join(", "),
                    spec.file_mounts.join(", ")
                );
            }
        }
    };

    Ok(PreflightSummary {
        binary_name: spec.binary_name.map(str::to_string),
        binary_path,
        forwarded_env_vars,
        mounted_paths,
        auth_path,
    })
}

fn profile_spec(agent: AgentFlowAgent) -> AgentProfileSpec {
    match agent {
        AgentFlowAgent::Stub => AgentProfileSpec {
            binary_name: None,
            auth_env_vars: &[],
            file_mounts: &[],
        },
        AgentFlowAgent::Codex => AgentProfileSpec {
            binary_name: Some("codex"),
            auth_env_vars: &["OPENAI_API_KEY"],
            file_mounts: &["~/.codex"],
        },
        AgentFlowAgent::Claude => AgentProfileSpec {
            binary_name: Some("claude"),
            auth_env_vars: &["ANTHROPIC_API_KEY"],
            file_mounts: &["~/.claude", "~/.config/claude"],
        },
    }
}

fn resolve_binary_path(binary_name: &str, path_var: Option<&OsStr>) -> Option<PathBuf> {
    let path_var = path_var?;
    for entry in env::split_paths(path_var) {
        let candidate = entry.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn expand_mount_path(path: &str, home_dir: &Path) -> PathBuf {
    if path == "~" {
        return home_dir.to_path_buf();
    }
    if let Some(suffix) = path.strip_prefix("~/") {
        return home_dir.join(suffix);
    }
    PathBuf::from(path)
}

fn home_dir() -> Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("HOME is not set"))
}

pub fn print_plan(plan: &AgentFlowTestPlan) {
    println!(
        "agent-flow test: run_id={} scenario={} agent={} mode={}",
        plan.run_id,
        plan.manifest.name,
        plan.agent.as_str(),
        plan.mode.as_str()
    );
    println!(
        "agent-flow test: manifest={} timeout_seconds={} keep_sandbox={} no_build={}",
        plan.manifest_path.display(),
        plan.timeout_seconds,
        plan.keep_sandbox,
        plan.no_build
    );
    println!(
        "agent-flow test: artifacts_dir={}",
        plan.artifacts_dir.display()
    );
    if let Some(description) = plan.manifest.description.as_deref() {
        println!("agent-flow test: description={description}");
    }
    println!(
        "agent-flow test: commands={} assertions={}",
        plan.manifest.commands.len(),
        plan.manifest.assertions.len()
    );
    if let Some(binary_name) = plan.preflight.binary_name.as_deref() {
        println!(
            "agent-flow test: binary={} path={}",
            binary_name,
            plan.preflight
                .binary_path
                .as_deref()
                .map(Path::display)
                .map(|display| display.to_string())
                .unwrap_or_else(|| "pending".to_string())
        );
    }
    println!(
        "agent-flow test: forwarded_env_vars={}",
        join_or_dash(&plan.preflight.forwarded_env_vars)
    );
    let mounted_paths = plan
        .preflight
        .mounted_paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    println!(
        "agent-flow test: mounted_paths={}",
        join_or_dash(&mounted_paths)
    );
    println!(
        "agent-flow test: auth_path={}",
        plan.preflight.auth_path.as_str()
    );
    println!("agent-flow test: preflight passed");
}

pub fn print_sandbox_result(result: &SandboxRunResult) {
    println!("agent-flow test: sandbox image={}", result.image);
    println!("agent-flow test: run_dir={}", result.run_dir.display());
    println!(
        "agent-flow test: artifact_bundle={}",
        result.artifacts_dir.display()
    );
    println!("agent-flow test: sandbox execution passed");
    if let Some(container_name) = result.container_name.as_deref() {
        println!("agent-flow test: kept_container={container_name}");
    }
}

fn join_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}

fn normalize_repo_relative_path(repo_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_relative() {
        repo_root.join(path)
    } else {
        path
    }
}

fn verify_sandbox_result(plan: &AgentFlowTestPlan, result: &SandboxRunResult) -> Result<()> {
    for assertion in &plan.manifest.assertions {
        let assertion_type = assertion
            .get("type")
            .and_then(serde_yaml::Value::as_str)
            .ok_or_else(|| anyhow!("scenario assertion is missing string field 'type'"))?;
        match assertion_type {
            "exit_code" => {
                let expected = assertion
                    .get("equals")
                    .and_then(serde_yaml::Value::as_i64)
                    .ok_or_else(|| {
                        anyhow!("exit_code assertion requires integer field 'equals'")
                    })?;
                let actual = read_i64_file(&result.artifacts_dir.join("exit-code.txt"))?;
                anyhow::ensure!(
                    actual == expected,
                    "assertion failed: exit_code expected {}, got {}",
                    expected,
                    actual
                );
            }
            "issue_status" => {
                let expected = assertion
                    .get("equals")
                    .and_then(serde_yaml::Value::as_str)
                    .ok_or_else(|| {
                        anyhow!("issue_status assertion requires string field 'equals'")
                    })?;
                let actual = read_primary_issue_status(&result.artifacts_dir)?;
                anyhow::ensure!(
                    actual == expected,
                    "assertion failed: issue_status expected '{}', got '{}'",
                    expected,
                    actual
                );
            }
            other => bail!(
                "unsupported scenario assertion type '{}' in {}",
                other,
                plan.manifest_path.display()
            ),
        }
    }

    Ok(())
}

fn read_i64_file(path: &Path) -> Result<i64> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read assertion artifact {}", path.display()))?;
    content.trim().parse::<i64>().with_context(|| {
        format!(
            "failed to parse integer assertion artifact {}",
            path.display()
        )
    })
}

fn read_primary_issue_status(artifacts_dir: &Path) -> Result<String> {
    let issues_dir = artifacts_dir.join("runtime").join("issues");
    let mut issue_files = fs::read_dir(&issues_dir)
        .with_context(|| {
            format!(
                "failed to read runtime issue artifacts {}",
                issues_dir.display()
            )
        })?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    issue_files.sort();

    anyhow::ensure!(
        issue_files.len() == 1,
        "expected exactly one runtime issue artifact in {}, found {}",
        issues_dir.display(),
        issue_files.len()
    );

    let content = fs::read_to_string(&issue_files[0]).with_context(|| {
        format!(
            "failed to read runtime issue artifact {}",
            issue_files[0].display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse runtime issue artifact {}",
            issue_files[0].display()
        )
    })?;
    value
        .get("last_known_flow_status")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            anyhow!(
                "runtime issue artifact {} is missing 'last_known_flow_status'",
                issue_files[0].display()
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn write_manifest(repo_root: &Path, file_name: &str, content: &str) -> PathBuf {
        let scenario_root = repo_root.join(DEFAULT_SCENARIO_ROOT);
        fs::create_dir_all(&scenario_root).expect("scenario root");
        let path = scenario_root.join(file_name);
        fs::write(&path, content).expect("manifest");
        path
    }

    fn write_fixture(repo_root: &Path, file_name: &str, content: &str) -> PathBuf {
        let fixture_root = repo_root.join(DEFAULT_FIXTURES_DIR);
        fs::create_dir_all(&fixture_root).expect("fixture root");
        let path = fixture_root.join(file_name);
        fs::write(&path, content).expect("fixture");
        path
    }

    #[test]
    fn resolves_manifest_and_defaults_live_agent_to_codex() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        write_manifest(
            repo_root,
            "live-defaults.yml",
            r#"
name: live-defaults
description: test
mode: live
commands:
  - ai-teamlead run 42
assertions:
  - type: exit_code
    equals: 0
"#,
        );

        let fake_bin_dir = temp.path().join("bin");
        fs::create_dir_all(&fake_bin_dir).expect("bin dir");
        let fake_codex = fake_bin_dir.join("codex");
        fs::write(&fake_codex, "#!/usr/bin/env bash\n").expect("binary");
        let mut perms = fs::metadata(&fake_codex).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_codex, perms).expect("chmod");

        let home_dir = temp.path().join("home");
        fs::create_dir_all(home_dir.join(".codex")).expect("codex dir");
        let path_var = env::join_paths([fake_bin_dir]).expect("PATH");
        let env_lookup = |_name: &str| None;

        let manifest_path = resolve_manifest_path(repo_root, "live-defaults").expect("path");
        let manifest = load_manifest(&manifest_path, "live-defaults").expect("manifest");
        let mode = resolve_effective_mode(&manifest, None).expect("mode");
        let agent = resolve_effective_agent(&manifest, None, mode).expect("agent");
        let preflight = run_preflight_with_host(
            agent,
            mode,
            &home_dir,
            Some(path_var.as_os_str()),
            &env_lookup,
        )
        .expect("preflight");

        assert_eq!(mode, AgentFlowMode::Live);
        assert_eq!(agent, AgentFlowAgent::Codex);
        assert_eq!(preflight.auth_path, AuthPath::SubscriptionAccount);
    }

    #[test]
    fn rejects_cli_agent_that_contradicts_manifest() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let manifest = ScenarioManifest {
            name: "run-happy-path".into(),
            description: None,
            mode: Some(AgentFlowMode::Stub),
            agent: Some(AgentFlowAgent::Stub),
            fixtures: serde_yaml::Value::Null,
            commands: vec!["ai-teamlead run 42".into()],
            assertions: vec![],
        };

        let error =
            resolve_effective_agent(&manifest, Some(AgentFlowAgent::Codex), AgentFlowMode::Stub)
                .expect_err("mismatch should fail");
        assert!(
            error
                .to_string()
                .contains("scenario manifest requires agent='stub'")
        );

        write_manifest(
            repo_root,
            "run-happy-path.yml",
            r#"
name: run-happy-path
mode: stub
agent: stub
commands:
  - ai-teamlead run 42
"#,
        );
        let _ = resolve_manifest_path(repo_root, "run-happy-path").expect("path");
    }

    #[test]
    fn rejects_invalid_mode_agent_pair() {
        let error = validate_mode_agent_pair(AgentFlowMode::Live, AgentFlowAgent::Stub)
            .expect_err("invalid pair");
        assert!(error.to_string().contains("mode='live'"));
    }

    #[test]
    fn preflight_fails_when_live_agent_has_no_auth_path() {
        let temp = tempdir().expect("tempdir");
        let fake_bin_dir = temp.path().join("bin");
        fs::create_dir_all(&fake_bin_dir).expect("bin dir");
        let fake_codex = fake_bin_dir.join("codex");
        fs::write(&fake_codex, "#!/usr/bin/env bash\n").expect("binary");
        let mut perms = fs::metadata(&fake_codex).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_codex, perms).expect("chmod");
        let path_var = env::join_paths([fake_bin_dir]).expect("PATH");
        let env_lookup = |_name: &str| None;

        let error = run_preflight_with_host(
            AgentFlowAgent::Codex,
            AgentFlowMode::Live,
            temp.path(),
            Some(path_var.as_os_str()),
            &env_lookup,
        )
        .expect_err("missing auth should fail");

        assert!(error.to_string().contains("preflight failed"));
        assert!(error.to_string().contains("OPENAI_API_KEY"));
        assert!(error.to_string().contains("~/.codex"));
    }

    #[test]
    fn resolves_binary_from_path() {
        let temp = tempdir().expect("tempdir");
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("bin dir");
        let binary = bin_dir.join("claude");
        fs::write(&binary, "#!/usr/bin/env bash\n").expect("binary");
        let mut perms = fs::metadata(&binary).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary, perms).expect("chmod");
        let path_var = env::join_paths([bin_dir]).expect("PATH");

        let resolved = resolve_binary_path("claude", Some(path_var.as_os_str())).expect("binary");
        assert_eq!(resolved, binary);
    }

    #[test]
    fn plan_uses_system_env_in_smoke_form() {
        let _guard = env_lock().lock().expect("env lock");
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        write_fixture(repo_root, "basic.json", "{}\n");
        write_manifest(
            repo_root,
            "stub-smoke.yml",
            r#"
name: stub-smoke
mode: stub
agent: stub
fixtures:
  github_stub: basic.json
commands:
  - ai-teamlead run 42
assertions:
  - type: exit_code
    equals: 0
"#,
        );

        let previous_home = env::var_os("HOME");
        // SAFETY: the test serializes environment mutations via `env_lock`.
        unsafe { env::set_var("HOME", repo_root) };

        let plan = plan_agent_flow_test(
            repo_root,
            repo_root,
            &AgentFlowTestRequest {
                scenario: "stub-smoke".into(),
                agent: None,
                mode: None,
                keep_sandbox: false,
                artifacts_dir: None,
                timeout_seconds: None,
                no_build: false,
            },
        )
        .expect("stub plan");

        assert_eq!(plan.agent, AgentFlowAgent::Stub);
        assert_eq!(plan.mode, AgentFlowMode::Stub);
        assert_eq!(plan.preflight.auth_path, AuthPath::NotRequired);

        match previous_home {
            // SAFETY: the test serializes environment mutations via `env_lock`.
            Some(home) => unsafe { env::set_var("HOME", home) },
            // SAFETY: the test serializes environment mutations via `env_lock`.
            None => unsafe { env::remove_var("HOME") },
        }
    }

    #[test]
    fn rejects_stub_plan_without_github_fixture() {
        let temp = tempdir().expect("tempdir");
        write_manifest(
            temp.path(),
            "missing-fixture.yml",
            r#"
name: missing-fixture
mode: stub
agent: stub
commands:
  - ai-teamlead run 42
"#,
        );

        let error = plan_agent_flow_test(
            temp.path(),
            temp.path(),
            &AgentFlowTestRequest {
                scenario: "missing-fixture".into(),
                agent: None,
                mode: None,
                keep_sandbox: false,
                artifacts_dir: None,
                timeout_seconds: None,
                no_build: false,
            },
        )
        .expect_err("fixture validation should fail");

        assert!(error.to_string().contains("fixtures.github_stub"));
    }

    #[derive(Default)]
    struct RecordingShell {
        runs: Mutex<Vec<String>>,
    }

    impl RecordingShell {
        fn commands(&self) -> Vec<String> {
            self.runs.lock().expect("lock").clone()
        }
    }

    impl Shell for RecordingShell {
        fn run(&self, _cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
            self.runs
                .lock()
                .expect("lock")
                .push(format!("{program} {}", args.join(" ")));
            Ok(String::new())
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
            _cwd: &Path,
            _envs: &[(&str, &str)],
            _program: &str,
            _args: &[&str],
            _stdout_stderr_log_path: Option<&Path>,
        ) -> Result<()> {
            unreachable!("not used in agent_flow tests")
        }
    }

    #[test]
    fn reads_pinned_zellij_release_from_file() {
        let temp = tempdir().expect("tempdir");
        fs::write(temp.path().join("ZELLIJ_VERSION"), "v0.44.0 abc123\n").expect("version");

        let pinned = read_pinned_zellij_release(temp.path()).expect("pinned");
        assert_eq!(pinned, ("v0.44.0".into(), "abc123".into()));
    }

    #[test]
    fn sandbox_run_uses_read_only_repo_mount_and_artifact_mount() {
        let temp = tempdir().expect("tempdir");
        let artifacts = temp.path().join("artifacts");
        let git_dir = temp.path().join("git");
        let common_git_dir = temp.path().join("common-git");
        let fake_binary = temp.path().join("ai-teamlead");
        fs::write(&fake_binary, "#!/usr/bin/env bash\n").expect("binary");
        let shell = RecordingShell::default();
        let plan = AgentFlowTestPlan {
            run_id: "run-id".into(),
            manifest_path: temp.path().join("scenario.yml"),
            manifest: ScenarioManifest {
                name: "scenario".into(),
                description: None,
                mode: Some(AgentFlowMode::Stub),
                agent: Some(AgentFlowAgent::Stub),
                fixtures: serde_yaml::from_str("github_stub: basic.json").expect("fixtures"),
                commands: vec!["ai-teamlead run 42".into()],
                assertions: vec![],
            },
            agent: AgentFlowAgent::Stub,
            mode: AgentFlowMode::Stub,
            keep_sandbox: true,
            artifacts_dir: artifacts.clone(),
            timeout_seconds: 30,
            no_build: true,
            preflight: PreflightSummary {
                binary_name: None,
                binary_path: None,
                forwarded_env_vars: vec![],
                mounted_paths: vec![],
                auth_path: AuthPath::NotRequired,
            },
        };

        run_sandbox_container(
            &shell,
            DEFAULT_SANDBOX_IMAGE,
            temp.path(),
            &git_dir,
            &common_git_dir,
            &artifacts,
            &fake_binary,
            &plan,
            Some("sandbox-1"),
            true,
        )
        .expect("sandbox");

        let commands = shell.commands();
        let command = commands.last().expect("command");
        assert!(command.contains("docker run --name sandbox-1"));
        assert!(command.contains(&format!("-v {}:/input/repo:ro", temp.path().display())));
        assert!(command.contains(&format!("-v {}:/input/worktree-git:ro", git_dir.display())));
        assert!(command.contains(&format!(
            "-v {}:/input/common-git:ro",
            common_git_dir.display()
        )));
        assert!(command.contains(&format!("-v {}:/artifacts", artifacts.display())));
        assert!(command.contains(&format!(
            "-v {}:/input/ai-teamlead-bin:ro",
            temp.path().display()
        )));
        assert!(command.contains("bash -lc"));
    }

    #[test]
    fn sandbox_run_for_live_codex_forwards_env_and_mounts() {
        let temp = tempdir().expect("tempdir");
        let artifacts = temp.path().join("artifacts");
        let git_dir = temp.path().join("git");
        let common_git_dir = temp.path().join("common-git");
        let fake_binary = temp.path().join("ai-teamlead");
        let install_root = temp.path().join("node-install");
        let agent_bin_dir = install_root.join("bin");
        let agent_lib_dir = install_root.join("lib/node_modules/@openai/codex/bin");
        let agent_binary = agent_bin_dir.join("codex");
        let codex_home = temp.path().join(".codex");
        fs::create_dir_all(&agent_bin_dir).expect("agent bin dir");
        fs::create_dir_all(&agent_lib_dir).expect("agent lib dir");
        fs::create_dir_all(&codex_home).expect("codex home");
        fs::write(&fake_binary, "#!/usr/bin/env bash\n").expect("binary");
        fs::write(
            agent_lib_dir.join("codex.js"),
            "#!/usr/bin/env node\nconsole.log('codex');\n",
        )
        .expect("agent binary target");
        #[cfg(unix)]
        std::os::unix::fs::symlink("../lib/node_modules/@openai/codex/bin/codex.js", &agent_binary)
            .expect("agent binary symlink");
        #[cfg(not(unix))]
        fs::write(&agent_binary, "#!/usr/bin/env bash\n").expect("agent binary");
        let shell = RecordingShell::default();
        let plan = AgentFlowTestPlan {
            run_id: "run-id".into(),
            manifest_path: temp.path().join("scenario.yml"),
            manifest: ScenarioManifest {
                name: "live-codex".into(),
                description: None,
                mode: Some(AgentFlowMode::Live),
                agent: Some(AgentFlowAgent::Codex),
                fixtures: serde_yaml::from_str("github_stub: basic.json").expect("fixtures"),
                commands: vec!["ai-teamlead run 42".into()],
                assertions: vec![],
            },
            agent: AgentFlowAgent::Codex,
            mode: AgentFlowMode::Live,
            keep_sandbox: false,
            artifacts_dir: artifacts.clone(),
            timeout_seconds: 30,
            no_build: true,
            preflight: PreflightSummary {
                binary_name: Some("codex".into()),
                binary_path: Some(agent_binary.clone()),
                forwarded_env_vars: vec!["OPENAI_API_KEY".into()],
                mounted_paths: vec![codex_home.clone()],
                auth_path: AuthPath::ApiKey,
            },
        };

        run_sandbox_container(
            &shell,
            DEFAULT_SANDBOX_IMAGE,
            temp.path(),
            &git_dir,
            &common_git_dir,
            &artifacts,
            &fake_binary,
            &plan,
            None,
            false,
        )
        .expect("sandbox");

        let command = shell.commands().last().cloned().expect("command");
        assert!(command.contains(&format!(
            "-v {}:/input/agent-root:ro",
            install_root.display()
        )));
        assert!(command.contains(&format!(
            "-v {}:/input/live-home/.codex:ro",
            codex_home.display()
        )));
        assert!(command.contains("-e OPENAI_API_KEY"));
    }

    #[test]
    fn maps_codex_subscription_mount_to_container_home() {
        let mapped =
            map_container_mount_path(AgentFlowAgent::Codex, Path::new("/home/test/.codex"))
                .expect("mount");
        assert_eq!(mapped, "/input/live-home/.codex");
    }

    #[test]
    fn sandbox_build_uses_pinned_zellij_release_args() {
        let temp = tempdir().expect("tempdir");
        fs::write(temp.path().join("ZELLIJ_VERSION"), "v0.44.0 abc123\n").expect("version");
        let shell = RecordingShell::default();

        ensure_sandbox_image(&shell, temp.path(), DEFAULT_SANDBOX_IMAGE, false).expect("build");

        let commands = shell.commands();
        let command = commands.last().expect("command");
        assert!(command.contains("docker build -f Dockerfile.test"));
        assert!(command.contains("--build-arg ZELLIJ_TAG=v0.44.0"));
        assert!(command.contains("--build-arg ZELLIJ_SHA256=abc123"));
        assert!(command.contains(&format!("-t {}", DEFAULT_SANDBOX_IMAGE)));
    }

    #[test]
    fn resolves_relative_common_git_dir() {
        let temp = tempdir().expect("tempdir");
        let git_dir = temp.path().join("worktrees").join("feature");
        fs::create_dir_all(&git_dir).expect("git dir");
        fs::write(git_dir.join("commondir"), "../..\n").expect("commondir");

        let resolved = resolve_common_git_dir(&git_dir).expect("common dir");
        assert_eq!(resolved, temp.path());
    }
}
