#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-launch-target-pane-XXXXXX)"
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
LAUNCH_LOG_42="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID_42/launch.log"
TAB_ID_42="$(wait_for_json_field_not_value "$SESSION_MANIFEST_42" '.zellij.tab_id' 'pending' 30 || true)"
PANE_ID_42="$(wait_for_json_field_not_value "$SESSION_MANIFEST_42" '.zellij.pane_id' 'pending' 30 || true)"

(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" internal launch-zellij-fixture 43
)

ISSUE_INDEX_43="$REPO_ROOT/.git/.ai-teamlead/issues/43.json"
wait_for_file "$ISSUE_INDEX_43" 30 || true
SESSION_UUID_43="$(issue_session_uuid "$ISSUE_INDEX_43")"
SESSION_MANIFEST_43="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID_43/session.json"
LAUNCH_LOG_43="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID_43/launch.log"
TAB_ID_43="$(wait_for_json_field_not_value "$SESSION_MANIFEST_43" '.zellij.tab_id' 'pending' 30 || true)"
PANE_ID_43="$(wait_for_json_field_not_value "$SESSION_MANIFEST_43" '.zellij.pane_id' 'pending' 30 || true)"

assert_eq "$(jq -r '.zellij.launch_target' "$SESSION_MANIFEST_42")" "pane" "first pane launch stores effective pane target"
assert_eq "$(jq -r '.zellij.launch_target' "$SESSION_MANIFEST_43")" "pane" "second pane launch stores effective pane target"
assert_eq "$TAB_ID_42" "$TAB_ID_43" "pane launch target reuses the shared tab"
assert_ne "$PANE_ID_42" "$PANE_ID_43" "pane launch target opens a new pane for each run"
assert_file_contains "$LAUNCH_LOG_42" "launch-target: pane" "first pane launch records launch target in launch log"
assert_file_contains "$LAUNCH_LOG_43" "launch-target: pane" "second pane launch records launch target in launch log"
