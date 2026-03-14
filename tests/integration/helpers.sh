#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0
ZELLIJ_TEST_TIMEOUT="${ZELLIJ_TEST_TIMEOUT:-20}"

assert_eq() {
    local actual="$1" expected="$2" msg="$3"
    if [[ "$actual" == "$expected" ]]; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    expected: '$expected'"
        echo "    actual:   '$actual'"
        ((FAIL++)) || true
    fi
}

assert_ne() {
    local actual="$1" expected="$2" msg="$3"
    if [[ "$actual" != "$expected" ]]; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    unexpected: '$actual'"
        ((FAIL++)) || true
    fi
}

assert_file_exists() {
    local path="$1" msg="$2"
    if [[ -f "$path" ]]; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    missing file: $path"
        ((FAIL++)) || true
    fi
}

assert_dir_exists() {
    local path="$1" msg="$2"
    if [[ -d "$path" ]]; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    missing directory: $path"
        ((FAIL++)) || true
    fi
}

assert_file_contains() {
    local path="$1" pattern="$2" msg="$3"
    if [[ -f "$path" ]] && grep -Fq "$pattern" "$path"; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    missing pattern: $pattern"
        echo "    file: $path"
        ((FAIL++)) || true
    fi
}

assert_text_contains() {
    local text="$1" pattern="$2" msg="$3"
    if grep -Fq "$pattern" <<<"$text"; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        echo "    missing pattern: $pattern"
        ((FAIL++)) || true
    fi
}

assert_session_alive() {
    local session_name="$1" msg="$2"
    if zellij list-sessions --short 2>/dev/null | grep -Fxq "$session_name"; then
        echo "  PASS: $msg"
        ((PASS++)) || true
    else
        echo "  FAIL: $msg"
        ((FAIL++)) || true
    fi
}

wait_for_file() {
    local path="$1"
    local timeout_seconds="${2:-$ZELLIJ_TEST_TIMEOUT}"
    local deadline=$((SECONDS + timeout_seconds))
    while (( SECONDS < deadline )); do
        if [[ -f "$path" ]]; then
            return 0
        fi
        sleep 0.2
    done
    return 1
}

wait_for_dir() {
    local path="$1"
    local timeout_seconds="${2:-$ZELLIJ_TEST_TIMEOUT}"
    local deadline=$((SECONDS + timeout_seconds))
    while (( SECONDS < deadline )); do
        if [[ -d "$path" ]]; then
            return 0
        fi
        sleep 0.2
    done
    return 1
}

wait_for_json_field_not_value() {
    local path="$1" field="$2" bad_value="$3"
    local timeout_seconds="${4:-$ZELLIJ_TEST_TIMEOUT}"
    local deadline=$((SECONDS + timeout_seconds))
    while (( SECONDS < deadline )); do
        if [[ -f "$path" ]]; then
            local value
            value=$(jq -r "$field" "$path" 2>/dev/null || true)
            if [[ -n "$value" && "$value" != "null" && "$value" != "$bad_value" ]]; then
                echo "$value"
                return 0
            fi
        fi
        sleep 0.2
    done
    return 1
}

create_test_repo() {
    local repo_root="$1"
    mkdir -p "$repo_root/.ai-teamlead"
    git init -q "$repo_root"
    git -C "$repo_root" remote add origin git@github.com:dapi/teamlead.git
    cat > "$repo_root/.ai-teamlead/settings.yml" <<'EOF'
github:
  project_id: "PVT_test_project"

issue_analysis_flow:
  statuses:
    backlog: "Backlog"
    analysis_in_progress: "Analysis In Progress"
    waiting_for_clarification: "Waiting for Clarification"
    waiting_for_plan_review: "Waiting for Plan Review"
    ready_for_implementation: "Ready for Implementation"
    analysis_blocked: "Analysis Blocked"

runtime:
  max_parallel: 1
  poll_interval_seconds: 3600

zellij:
  session_name: "ai-teamlead-test"
  tab_name: "issue-analysis"

launch_agent:
  analysis_branch_template: "analysis/issue-${ISSUE_NUMBER}"
  worktree_root_template: "${HOME}/worktrees/${REPO}/${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/${ISSUE_NUMBER}"
EOF
    cat > "$repo_root/.ai-teamlead/launch-agent.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

SESSION_UUID="${1:?usage: launch-agent.sh <session_uuid> <issue_url>}"
ISSUE_URL="${2:?usage: launch-agent.sh <session_uuid> <issue_url>}"
export ISSUE_URL
LOG_FILE="./.git/.ai-teamlead/launch-helper.log"
MARKER_FILE="./.git/.ai-teamlead/launch-helper.started"

printf 'started\n' >"$MARKER_FILE"

{
    printf 'launch-helper: session_uuid=%s\n' "$SESSION_UUID"
    printf 'launch-helper: issue_url=%s\n' "$ISSUE_URL"
    printf 'launch-helper: ai_teamlead_bin=%s\n' "${AI_TEAMLEAD_BIN:-ai-teamlead}"
    printf 'launch-helper: pane_id=%s\n' "${ZELLIJ_PANE_ID:-unset}"
    "${AI_TEAMLEAD_BIN:-ai-teamlead}" internal bind-zellij-pane "$SESSION_UUID"
} >>"$LOG_FILE" 2>&1
exec "${SHELL:-/bin/bash}" -l
EOF
    chmod +x "$repo_root/.ai-teamlead/launch-agent.sh"
    mkdir -p "$repo_root/.ai-teamlead/flows"
    cat > "$repo_root/.ai-teamlead/flows/issue-analysis-flow.md" <<'EOF'
# issue-analysis-flow fixture
EOF
}

create_initialized_repo() {
    local repo_root="$1"
    local ai_teamlead_bin="${2:-/test/bin/ai-teamlead}"

    git init -q -b main "$repo_root"
    git -C "$repo_root" remote add origin git@github.com:dapi/example.git
    git -C "$repo_root" config user.name "AI Teamlead Test"
    git -C "$repo_root" config user.email "ai-teamlead@example.com"
    printf '# integration fixture\n' > "$repo_root/README.md"
    git -C "$repo_root" add README.md
    git -C "$repo_root" commit -q -m "initial"

    (
        cd "$repo_root"
        "$ai_teamlead_bin" init >/dev/null
        git add .ai-teamlead .claude .codex init.sh
        git commit -q -m "bootstrap ai-teamlead"
    )
}

install_gh_stub() {
    local bin_dir="$1" snapshot_file="$2" log_file="$3"
    mkdir -p "$bin_dir"
    cat > "$bin_dir/gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

LOG_FILE="__LOG_FILE__"
SNAPSHOT_FILE="__SNAPSHOT_FILE__"
ARGS="$*"
printf 'gh %s\n' "$ARGS" >> "$LOG_FILE"

if [[ "${1:-}" == "repo" && "${2:-}" == "view" ]]; then
    printf 'main\n'
    exit 0
fi

if [[ "${1:-}" == "pr" && "${2:-}" == "list" ]]; then
    printf '%s\n' "${AI_TEAMLEAD_TEST_GH_PR_LIST_RESULT:-0}"
    exit 0
fi

if [[ "${1:-}" == "pr" && "${2:-}" == "create" ]]; then
    printf '%s\n' "${AI_TEAMLEAD_TEST_GH_PR_CREATE_RESULT:-https://github.com/dapi/example/pull/99}"
    exit 0
fi

if [[ "$ARGS" == *"updateProjectV2ItemFieldValue"* ]]; then
    printf '{"data":{"updateProjectV2ItemFieldValue":{"projectV2Item":{"id":"updated-item"}}}}\n'
    exit 0
fi

cat "$SNAPSHOT_FILE"
EOF
    sed -i \
        -e "s|__LOG_FILE__|$log_file|g" \
        -e "s|__SNAPSHOT_FILE__|$snapshot_file|g" \
        "$bin_dir/gh"
    chmod +x "$bin_dir/gh"
}

install_agent_stubs() {
    local bin_dir="$1" out_dir="$2"
    mkdir -p "$bin_dir" "$out_dir"
    cat > "$bin_dir/codex" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${AI_TEAMLEAD_STUB_OUT_DIR:?}"
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

printf 'invoked\n' > "$OUT_DIR/codex.invoked"
printf '%s\n' "$PWD" > "$OUT_DIR/codex.cwd"
printf '%s\n' "${AI_TEAMLEAD_ISSUE_URL:-}" > "$OUT_DIR/issue_url"
printf '%s\n' "${AI_TEAMLEAD_SESSION_UUID:-}" > "$OUT_DIR/session_uuid"
printf '%s\n' "${AI_TEAMLEAD_ANALYSIS_BRANCH:-}" > "$OUT_DIR/analysis_branch"
printf '%s\n' "${AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR:-}" > "$OUT_DIR/analysis_artifacts_dir"
printf '%s\n' "${AI_TEAMLEAD_WORKTREE_ROOT:-}" > "$OUT_DIR/worktree_root"
printf '%s\n' "$PROMPT" > "$OUT_DIR/prompt.txt"

sleep "${AI_TEAMLEAD_STUB_AGENT_SLEEP:-5}"
EOF
    chmod +x "$bin_dir/codex"
    ln -sf codex "$bin_dir/claude"
    export AI_TEAMLEAD_STUB_OUT_DIR="$out_dir"
}

install_complete_stage_agent_stub() {
    local bin_dir="$1" out_dir="$2"
    mkdir -p "$bin_dir" "$out_dir"
    cat > "$bin_dir/codex" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${AI_TEAMLEAD_STUB_OUT_DIR:?}"
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

mkdir -p "${AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR:?}"
cat > "${AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR}/README.md" <<'DOC'
# Test artifact

- generated from stub codex agent
DOC

printf 'invoked\n' > "$OUT_DIR/codex.invoked"
printf '%s\n' "$PWD" > "$OUT_DIR/codex.cwd"
printf '%s\n' "${AI_TEAMLEAD_ISSUE_URL:-}" > "$OUT_DIR/issue_url"
printf '%s\n' "${AI_TEAMLEAD_SESSION_UUID:-}" > "$OUT_DIR/session_uuid"
printf '%s\n' "${AI_TEAMLEAD_ANALYSIS_BRANCH:-}" > "$OUT_DIR/analysis_branch"
printf '%s\n' "${AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR:-}" > "$OUT_DIR/analysis_artifacts_dir"
printf '%s\n' "${AI_TEAMLEAD_WORKTREE_ROOT:-}" > "$OUT_DIR/worktree_root"
printf '%s\n' "$PROMPT" > "$OUT_DIR/prompt.txt"

if "${AI_TEAMLEAD_BIN:-ai-teamlead}" internal complete-stage \
    "${AI_TEAMLEAD_SESSION_UUID:?}" \
    --outcome plan-ready \
    --message "stub analysis ready" \
    >"$OUT_DIR/complete-stage.stdout" \
    2>"$OUT_DIR/complete-stage.stderr"; then
    printf '0\n' > "$OUT_DIR/complete-stage.exit_code"
else
    status=$?
    printf '%s\n' "$status" > "$OUT_DIR/complete-stage.exit_code"
    exit "$status"
fi
EOF
    chmod +x "$bin_dir/codex"
    ln -sf codex "$bin_dir/claude"
    export AI_TEAMLEAD_STUB_OUT_DIR="$out_dir"
}

cleanup_zellij() {
    while IFS= read -r session_name; do
        [[ -n "$session_name" ]] || continue
        zellij kill-session "$session_name" >/dev/null 2>&1 || true
    done < <(zellij list-sessions --short 2>/dev/null || true)
}

print_summary() {
    echo ""
    echo "=== Summary ==="
    echo "PASS: $PASS"
    echo "FAIL: $FAIL"
    if [[ "$FAIL" -ne 0 ]]; then
        exit 1
    fi
}
