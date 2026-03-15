#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/helpers.sh"

AI_TEAMLEAD_BIN="${AI_TEAMLEAD_BIN:-/test/bin/ai-teamlead}"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-run-status-denied-real-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-run-status-denied-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-run-status-denied-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-run-status-denied-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-run-status-denied-gh-snapshot-XXXXXX)"
GH_REPO_ISSUE="$(mktemp /tmp/ai-teamlead-run-status-denied-gh-issue-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"},{"id":"OPT_IMPL_PROGRESS","name":"Implementation In Progress"},{"id":"OPT_IMPL_CI","name":"Waiting for CI"},{"id":"OPT_IMPL_REVIEW","name":"Waiting for Code Review"},{"id":"OPT_IMPL_BLOCKED","name":"Implementation Blocked"},{"id":"OPT_UNEXPECTED","name":"Unexpected Status"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Unexpected Status","optionId":"OPT_UNEXPECTED"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

cat > "$GH_REPO_ISSUE" <<'EOF'
{"data":{"repository":{"issue":{"id":"ISSUE-42","number":42,"state":"OPEN","url":"https://github.com/dapi/example/issues/42"}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"
export AI_TEAMLEAD_TEST_GH_REPO_ISSUE_FILE="$GH_REPO_ISSUE"

if RUN_OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" run 42 2>&1
)"; then
    echo "  FAIL: run with unfixable status must fail"
    ((FAIL++)) || true
else
    echo "  PASS: run with unfixable status must fail"
    ((PASS++)) || true
fi

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/42.json"
assert_text_contains "$RUN_OUTPUT" "Текущий статус: \"Unexpected Status\"" "run reports current unexpected status"
assert_text_contains "$RUN_OUTPUT" "Допустимые статусы для run: Backlog" "run reports allowed statuses"
assert_text_contains "$RUN_OUTPUT" "Ready for Implementation" "run allowed statuses include implementation entry"
assert_text_contains "$RUN_OUTPUT" "Автоисправление не выполнено" "run explains why status was not auto-fixed"

if [[ ! -f "$ISSUE_INDEX" ]]; then
    echo "  PASS: denied run did not create issue index"
    ((PASS++)) || true
else
    echo "  FAIL: denied run did not create issue index"
    echo "    unexpected file: $ISSUE_INDEX"
    ((FAIL++)) || true
fi

if [[ ! -f "$STUB_OUT/codex.invoked" ]]; then
    echo "  PASS: denied run did not launch agent"
    ((PASS++)) || true
else
    echo "  FAIL: denied run did not launch agent"
    ((FAIL++)) || true
fi
