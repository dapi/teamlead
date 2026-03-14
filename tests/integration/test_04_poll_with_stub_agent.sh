#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-poll-real-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-poll-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-poll-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-poll-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-poll-gh-snapshot-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-FOREIGN","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":1,"state":"OPEN","repository":{"name":"other","owner":{"login":"someone"}}}},{"id":"ITEM-7","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":7,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}},{"id":"ITEM-42","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"
export AI_TEAMLEAD_STUB_AGENT_SLEEP=8

(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" poll
)

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/7.json"
if ! wait_for_file "$ISSUE_INDEX"; then
    echo "  FAIL: poll created issue index for top backlog issue"
    ((FAIL++)) || true
    return 0
fi

SESSION_UUID="$(issue_session_uuid "$ISSUE_INDEX")"
SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"
if ! wait_for_file "$SESSION_MANIFEST"; then
    echo "  FAIL: poll created session manifest"
    ((FAIL++)) || true
    return 0
fi

wait_for_file "$STUB_OUT/issue_url" 30 || true

assert_file_exists "$ISSUE_INDEX" "poll created issue index for first matching backlog issue"
assert_file_exists "$SESSION_MANIFEST" "poll created session manifest"
assert_eq "$(cat "$STUB_OUT/issue_url")" "https://github.com/dapi/example/issues/7" "poll launched stub agent for top backlog issue"
assert_file_contains "$GH_LOG" "itemId=ITEM-7" "poll updated status for first matching backlog item"

if [[ -e "$REPO_ROOT/.git/.ai-teamlead/issues/42.json" ]]; then
    echo "  FAIL: poll must not claim second backlog issue when max_parallel=1"
    ((FAIL++)) || true
else
    echo "  PASS: poll does not claim second backlog issue when max_parallel=1"
    ((PASS++)) || true
fi
