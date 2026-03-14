#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-zellij-XXXXXX)"
create_test_repo "$REPO_ROOT"

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"

(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" internal launch-zellij-fixture 42
)

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/42.json"
if ! wait_for_file "$ISSUE_INDEX"; then
    echo "  FAIL: issue index file created"
    ((FAIL++)) || true
    return 0
fi
assert_file_exists "$ISSUE_INDEX" "issue index file created"

SESSION_UUID="$(issue_session_uuid "$ISSUE_INDEX")"
SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"
LAYOUT_FILE="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/launch-layout.kdl"
HELPER_LOG="$REPO_ROOT/.git/.ai-teamlead/launch-helper.log"
HELPER_MARKER="$REPO_ROOT/.git/.ai-teamlead/launch-helper.started"

if ! wait_for_file "$SESSION_MANIFEST"; then
    echo "  FAIL: session manifest created"
    ((FAIL++)) || true
    return 0
fi

assert_file_exists "$SESSION_MANIFEST" "session manifest created"
assert_file_exists "$LAYOUT_FILE" "launcher layout created"

TAB_ID="$(wait_for_json_field_not_value "$SESSION_MANIFEST" '.zellij.tab_id' 'pending' 30 || true)"
PANE_ID="$(wait_for_json_field_not_value "$SESSION_MANIFEST" '.zellij.pane_id' 'pending' 30 || true)"
SESSION_ID="$(jq -r '.zellij.session_id' "$SESSION_MANIFEST")"

if [[ -z "$TAB_ID" || -z "$PANE_ID" ]] && [[ -f "$HELPER_LOG" ]]; then
    echo "  INFO: launch-helper.log"
    sed -n '1,200p' "$HELPER_LOG"
fi

if [[ -z "$TAB_ID" || -z "$PANE_ID" ]] && [[ -f "$HELPER_MARKER" ]]; then
    echo "  INFO: launch-helper.started present"
fi

assert_eq "$SESSION_ID" "ai-teamlead-test" "session_id captured from configured session name"
assert_ne "$TAB_ID" "" "tab_id captured from zellij"
assert_ne "$PANE_ID" "" "pane_id captured from zellij"
assert_session_alive "ai-teamlead-test" "zellij session is alive after launcher run"
