#!/usr/bin/env bash
set -euo pipefail

SESSION_UUID="${1:?usage: launch-agent.sh <session_uuid> <issue_url>}"
ISSUE_URL="${2:?usage: launch-agent.sh <session_uuid> <issue_url>}"
FLOW_STAGE="${AI_TEAMLEAD_FLOW_STAGE:-analysis}"
case "$FLOW_STAGE" in
  analysis)
    FLOW_PATH="./.ai-teamlead/flows/issue-analysis-flow.md"
    ;;
  implementation)
    FLOW_PATH="./.ai-teamlead/flows/issue-implementation-flow.md"
    ;;
  *)
    printf 'launch-agent.sh: unsupported flow stage: %s\n' "$FLOW_STAGE" >&2
    exit 1
    ;;
esac
REPO_ROOT="$(pwd -P)"

AI_TEAMLEAD_BIN="${AI_TEAMLEAD_BIN:-ai-teamlead}"
AI_TEAMLEAD_DEBUG="${AI_TEAMLEAD_DEBUG:-0}"
AI_TEAMLEAD_LAUNCH_LOG="${AI_TEAMLEAD_LAUNCH_LOG:-}"

enable_debug_trace() {
    if [[ "$AI_TEAMLEAD_DEBUG" != "1" || -z "$AI_TEAMLEAD_LAUNCH_LOG" ]]; then
        return 0
    fi

    mkdir -p "$(dirname "$AI_TEAMLEAD_LAUNCH_LOG")"
    exec 9>>"$AI_TEAMLEAD_LAUNCH_LOG"
    export BASH_XTRACEFD=9
    export PS4='+ launch-agent:${LINENO}: '
    set -x
}

append_launch_log() {
    if [[ -z "$AI_TEAMLEAD_LAUNCH_LOG" ]]; then
        return 0
    fi

    printf '[%s] launch-agent: %s\n' "$(date -Iseconds)" "$*" >>"$AI_TEAMLEAD_LAUNCH_LOG"
}

enable_debug_trace
append_launch_log "bootstrap stage=$FLOW_STAGE session_uuid=$SESSION_UUID issue_url=$ISSUE_URL"

if ! command -v "$AI_TEAMLEAD_BIN" >/dev/null 2>&1; then
    append_launch_log "ai-teamlead binary is not available: $AI_TEAMLEAD_BIN"
    printf 'launch-agent.sh: ai-teamlead binary is not available: %s\n' "$AI_TEAMLEAD_BIN" >&2
    exit 1
fi

append_launch_log "binding zellij pane"
"$AI_TEAMLEAD_BIN" internal bind-zellij-pane "$SESSION_UUID"

if [[ ! -f "$FLOW_PATH" ]]; then
    append_launch_log "missing flow file $FLOW_PATH"
    printf 'launch-agent.sh: missing flow file %s\n' "$FLOW_PATH" >&2
    exit 1
fi

append_launch_log "rendering launch-agent context for stage=$FLOW_STAGE issue=$ISSUE_URL"
eval "$("$AI_TEAMLEAD_BIN" internal render-launch-agent-context "$ISSUE_URL")"

detect_default_branch() {
    local remote_head
    remote_head="$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's#refs/remotes/origin/##' || true)"
    if [[ -n "$remote_head" ]]; then
        printf '%s\n' "$remote_head"
        return 0
    fi

    if command -v gh >/dev/null 2>&1; then
        remote_head="$(gh repo view --json defaultBranchRef --jq '.defaultBranchRef.name' 2>/dev/null || true)"
        if [[ -n "$remote_head" ]]; then
            printf '%s\n' "$remote_head"
            return 0
        fi
    fi

    remote_head="$(git remote show -n origin 2>/dev/null | sed -n 's/.*HEAD branch: //p' | head -n 1 || true)"
    if [[ -n "$remote_head" && "$remote_head" != "(unknown)" && "$remote_head" != "(not queried)" ]]; then
        printf '%s\n' "$remote_head"
        return 0
    fi

    if git show-ref --verify --quiet refs/heads/main; then
        printf 'main\n'
        return 0
    fi

    if git show-ref --verify --quiet refs/heads/master; then
        printf 'master\n'
        return 0
    fi

    git rev-parse --abbrev-ref HEAD
}

find_worktree_for_branch() {
    local target_branch="$1"
    local current_worktree=""

    while IFS= read -r line; do
        if [[ "$line" == worktree\ * ]]; then
            current_worktree="${line#worktree }"
            continue
        fi

        if [[ "$line" == branch\ refs/heads/* ]]; then
            local current_branch="${line#branch refs/heads/}"
            if [[ "$current_branch" == "$target_branch" ]]; then
                printf '%s\n' "$current_worktree"
                return 0
            fi
        fi
    done < <(git worktree list --porcelain)

    return 1
}

worktree_path_is_available() {
    local target_path="$1"

    if [[ ! -e "$target_path" ]]; then
        return 0
    fi

    if [[ -d "$target_path" ]] && [[ -z "$(find "$target_path" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null)" ]]; then
        return 0
    fi

    return 1
}

resolve_available_worktree_root() {
    local requested_root="$1"
    if worktree_path_is_available "$requested_root"; then
        printf '%s\n' "$requested_root"
        return 0
    fi

    local repo_key
    repo_key="$(
        git rev-parse --git-common-dir \
            | sha256sum \
            | cut -c1-8
    )"
    local attempt=1
    while true; do
        local candidate_root="${requested_root}-${repo_key}"
        if (( attempt > 1 )); then
            candidate_root="${candidate_root}-${attempt}"
        fi

        if worktree_path_is_available "$candidate_root"; then
            printf '%s\n' "$candidate_root"
            return 0
        fi

        attempt=$((attempt + 1))
    done
}

ensure_stage_worktree() {
    local existing_worktree
    existing_worktree="$(find_worktree_for_branch "$BRANCH" || true)"
    if [[ -n "$existing_worktree" ]]; then
        WORKTREE_ROOT="$(cd "$existing_worktree" && pwd -P)"
        return 0
    fi

    local requested_worktree_root="$WORKTREE_ROOT"
    WORKTREE_ROOT="$(resolve_available_worktree_root "$requested_worktree_root")"
    if [[ "$WORKTREE_ROOT" != "$requested_worktree_root" ]]; then
        append_launch_log "requested worktree root is busy, using fallback root $WORKTREE_ROOT"
    fi

    mkdir -p "$(dirname "$WORKTREE_ROOT")"

    local default_branch
    default_branch="$(detect_default_branch)"

    if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
        git worktree add "$WORKTREE_ROOT" "$BRANCH"
    else
        git worktree add -b "$BRANCH" "$WORKTREE_ROOT" "$default_branch"
    fi
}

run_project_init() {
    if [[ -x "./init.sh" ]]; then
        ./init.sh
        return 0
    fi

    if [[ -f "./init.sh" ]]; then
        bash ./init.sh
    fi
}

start_agent() {
    local prompt
    local issue_body_section=""
    local issue_title="${ISSUE_TITLE:-}"
    local issue_body="${ISSUE_BODY:-}"
    if [[ "$AI_TEAMLEAD_DEBUG" == "1" || ${#issue_body} -le 300 ]]; then
        issue_body_section="

Issue Body:
$issue_body"
    fi
    prompt="$(cat "$FLOW_PATH")

Issue URL: $ISSUE_URL
Issue Title: $issue_title${issue_body_section}
Session UUID: $SESSION_UUID
Flow stage: $FLOW_STAGE
Stage branch: $BRANCH
Stage artifacts dir: $ARTIFACTS_DIR"

    local agent_bin="${AI_TEAMLEAD_AGENT_BIN:-}"
    local agent_kind="${AI_TEAMLEAD_AGENT_KIND:-}"
    if [[ -n "$agent_bin" && -x "$agent_bin" ]]; then
        case "$agent_kind" in
            claude)
                append_launch_log "starting claude override binary args_count=${#CLAUDE_GLOBAL_ARGS[@]} worktree=$WORKTREE_ROOT"
                exec "$agent_bin" "${CLAUDE_GLOBAL_ARGS[@]}" "$prompt"
                ;;
            *)
                append_launch_log "starting codex override binary args_count=${#CODEX_GLOBAL_ARGS[@]} worktree=$WORKTREE_ROOT"
                exec "$agent_bin" --cd "$WORKTREE_ROOT" --no-alt-screen "${CODEX_GLOBAL_ARGS[@]}" "$prompt"
                ;;
        esac
    fi

    if command -v codex >/dev/null 2>&1; then
        append_launch_log "starting codex agent args_count=${#CODEX_GLOBAL_ARGS[@]} worktree=$WORKTREE_ROOT"
        exec codex --cd "$WORKTREE_ROOT" --no-alt-screen "${CODEX_GLOBAL_ARGS[@]}" "$prompt"
    fi

    if command -v claude >/dev/null 2>&1; then
        append_launch_log "starting claude agent args_count=${#CLAUDE_GLOBAL_ARGS[@]} worktree=$WORKTREE_ROOT"
        exec claude "${CLAUDE_GLOBAL_ARGS[@]}" "$prompt"
    fi

    append_launch_log "starting degraded shell fallback worktree=$WORKTREE_ROOT"
    printf 'launch-agent.sh: no supported agent found, staying in interactive shell inside %s\n' "$WORKTREE_ROOT" >&2
    exec "${SHELL:-/bin/bash}" -l
}

ensure_stage_worktree
cd "$WORKTREE_ROOT"
append_launch_log "worktree ready at $WORKTREE_ROOT"
run_project_init
mkdir -p "$ARTIFACTS_DIR"
append_launch_log "artifacts dir ready at $ARTIFACTS_DIR"

export AI_TEAMLEAD_SESSION_UUID="$SESSION_UUID"
export AI_TEAMLEAD_ISSUE_URL="$ISSUE_URL"
export AI_TEAMLEAD_FLOW_STAGE="$FLOW_STAGE"
export AI_TEAMLEAD_BRANCH="$BRANCH"
export AI_TEAMLEAD_WORKTREE_ROOT="$WORKTREE_ROOT"
export AI_TEAMLEAD_ARTIFACTS_DIR="$ARTIFACTS_DIR"
export AI_TEAMLEAD_REPO_ROOT="$REPO_ROOT"
if [[ "$FLOW_STAGE" == "analysis" ]]; then
    export AI_TEAMLEAD_ANALYSIS_BRANCH="$BRANCH"
    export AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR="$ARTIFACTS_DIR"
fi
if [[ "$FLOW_STAGE" == "implementation" ]]; then
    export AI_TEAMLEAD_IMPLEMENTATION_BRANCH="$BRANCH"
    export AI_TEAMLEAD_IMPLEMENTATION_ARTIFACTS_DIR="$ARTIFACTS_DIR"
fi

append_launch_log "starting agent in $WORKTREE_ROOT"
start_agent
