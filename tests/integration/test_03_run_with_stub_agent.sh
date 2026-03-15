#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-run-real-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-run-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-run-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-run-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-run-gh-snapshot-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"
export AI_TEAMLEAD_STUB_AGENT_SLEEP=8

RUN_OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" run -d 42 2>&1
)"

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/42.json"
if ! wait_for_file "$ISSUE_INDEX"; then
    echo "  FAIL: run created issue index"
    ((FAIL++)) || true
    return 0
fi

SESSION_UUID="$(issue_session_uuid "$ISSUE_INDEX")"
SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"
LAUNCH_LOG="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/launch.log"
if ! wait_for_file "$SESSION_MANIFEST"; then
    echo "  FAIL: run created session manifest"
    ((FAIL++)) || true
    return 0
fi

WORKTREE_ROOT="${HOME}/worktrees/example/analysis/issue-42"
ARTIFACTS_DIR="$WORKTREE_ROOT/specs/issues/42"
PANE_ID="$(wait_for_json_field_not_value "$SESSION_MANIFEST" '.zellij.pane_id' 'pending' 30 || true)"
SESSION_NAME="$(jq -r '.zellij.session_name' "$SESSION_MANIFEST")"
wait_for_dir "$WORKTREE_ROOT" 30 || true
wait_for_dir "$ARTIFACTS_DIR" 30 || true
wait_for_file "$STUB_OUT/codex.invoked" 30 || true
wait_for_file "$LAUNCH_LOG" 30 || true

assert_file_exists "$ISSUE_INDEX" "run created issue index"
assert_file_exists "$SESSION_MANIFEST" "run created session manifest"
assert_file_exists "$LAUNCH_LOG" "run created launch log"
assert_dir_exists "$WORKTREE_ROOT" "run created analysis worktree"
assert_dir_exists "$ARTIFACTS_DIR" "run created analysis artifacts directory"
assert_file_exists "$STUB_OUT/codex.invoked" "run started stub agent inside zellij pane"
assert_file_contains "$STUB_OUT/codex.args" "--full-auto" "run passes canonical codex default args"
assert_eq "$(cat "$STUB_OUT/issue_url")" "https://github.com/dapi/example/issues/42" "run passed issue URL to stub agent"
assert_eq "$(cat "$STUB_OUT/session_uuid")" "$SESSION_UUID" "run passed session UUID to stub agent"
assert_eq "$(cat "$STUB_OUT/analysis_branch")" "analysis/issue-42" "run passed analysis branch to stub agent"
assert_eq "$(cat "$STUB_OUT/worktree_root")" "$WORKTREE_ROOT" "run passed worktree root to stub agent"
assert_eq "$(cat "$STUB_OUT/analysis_artifacts_dir")" "specs/issues/42" "run passed artifacts dir to stub agent"
assert_eq "$(cat "$STUB_OUT/codex.cwd")" "$WORKTREE_ROOT" "stub agent started in analysis worktree"
assert_eq "$SESSION_NAME" "example" "run uses rendered settings zellij session outside zellij env"
assert_ne "$PANE_ID" "" "run captured zellij pane id"
assert_file_contains "$LAUNCH_LOG" "pane-entrypoint: stage=analysis session_uuid=$SESSION_UUID" "run wrote pane entrypoint bootstrap log"
assert_file_contains "$LAUNCH_LOG" "launch-agent: starting agent in $WORKTREE_ROOT" "run wrote launch-agent progress log"
assert_file_contains "$STUB_OUT/prompt.txt" "# issue-analysis-flow" "run injected issue-analysis flow into prompt"
assert_file_contains "$STUB_OUT/prompt.txt" "Issue URL: https://github.com/dapi/example/issues/42" "run injected issue URL into prompt"
assert_file_contains "$GH_LOG" "itemId=ITEM-42" "run updated project status for selected item"
assert_text_contains "$RUN_OUTPUT" "launch target: launch_target=tab zellij_session=" "run printed zellij launch target"
assert_text_contains "$RUN_OUTPUT" "pane_id=" "run printed zellij pane id"
