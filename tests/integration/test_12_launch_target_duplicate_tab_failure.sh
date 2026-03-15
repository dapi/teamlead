#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-launch-target-duplicate-XXXXXX)"
create_test_repo "$REPO_ROOT"

sed -i 's/^  launch_target: "tab"$/  launch_target: "pane"/' \
    "$REPO_ROOT/.ai-teamlead/settings.yml"

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"

(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" internal launch-zellij-fixture 42
)

ISSUE_INDEX_42="$REPO_ROOT/.git/.ai-teamlead/issues/42.json"
wait_for_file "$ISSUE_INDEX_42" 30 || true
SESSION_UUID_42="$(issue_session_uuid "$ISSUE_INDEX_42")"
SESSION_MANIFEST_42="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID_42/session.json"
wait_for_json_field_not_value "$SESSION_MANIFEST_42" '.zellij.tab_id' 'pending' 30 >/dev/null || true

(
    cd "$REPO_ROOT"
    env -u ZELLIJ -u ZELLIJ_SESSION_NAME -u ZELLIJ_PANE_ID \
        ZELLIJ=0 \
        ZELLIJ_SESSION_NAME=ai-teamlead-test \
        zellij action new-tab --name issue-analysis
)

sleep 1

DUPLICATE_OUTPUT="$(
    cd "$REPO_ROOT"
    if "$AI_TEAMLEAD_BIN" internal launch-zellij-fixture 43 2>&1; then
        echo "unexpected-success"
    fi
)"

assert_text_contains "$DUPLICATE_OUTPUT" "requires a unique shared tab" "pane launch target rejects duplicate shared tabs"
