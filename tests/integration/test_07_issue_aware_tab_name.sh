#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-zellij-template-XXXXXX)"
create_test_repo "$REPO_ROOT"

sed -i '/^  tab_name: "issue-analysis"$/a\  tab_name_template: "#${ISSUE_NUMBER}"' \
    "$REPO_ROOT/.ai-teamlead/settings.yml"

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

SESSION_UUID="$(issue_session_uuid "$ISSUE_INDEX")"
SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"
LAYOUT_FILE="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/launch-layout.kdl"
LAUNCH_LOG="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/launch.log"

if ! wait_for_file "$SESSION_MANIFEST"; then
    echo "  FAIL: session manifest created"
    ((FAIL++)) || true
    return 0
fi

assert_file_exists "$SESSION_MANIFEST" "template launch created session manifest"
assert_file_exists "$LAYOUT_FILE" "template launch created layout file"
assert_file_exists "$LAUNCH_LOG" "template launch created launch log"
assert_eq "$(jq -r '.zellij.tab_name' "$SESSION_MANIFEST")" "#42" "manifest stores effective issue-aware tab name"
assert_file_contains "$LAYOUT_FILE" 'name="#42"' "layout renders issue-aware tab name"
assert_file_contains "$LAUNCH_LOG" 'analysis-tab-name: #42' "launch log records effective issue-aware tab name"
