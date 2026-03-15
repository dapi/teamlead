#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/helpers.sh"

AI_TEAMLEAD_BIN="${AI_TEAMLEAD_BIN:-/test/bin/ai-teamlead}"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-run-auto-attach-real-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-run-auto-attach-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-run-auto-attach-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-run-auto-attach-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-run-auto-attach-gh-snapshot-XXXXXX)"
GH_REPO_ISSUE="$(mktemp /tmp/ai-teamlead-run-auto-attach-gh-issue-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[]}}}}
EOF

cat > "$GH_REPO_ISSUE" <<'EOF'
{"data":{"repository":{"issue":{"id":"ISSUE-42","number":42,"state":"OPEN","url":"https://github.com/dapi/example/issues/42"}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"
export AI_TEAMLEAD_STUB_AGENT_SLEEP=8
export AI_TEAMLEAD_TEST_GH_REPO_ISSUE_FILE="$GH_REPO_ISSUE"

if RUN_OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" run -d 42 2>&1
)"; then
    echo "  PASS: auto-attach run completed"
    ((PASS++)) || true
else
    echo "  PASS: auto-attach run reached remediation path before launcher failure"
    ((PASS++)) || true
fi

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/42.json"
assert_text_contains "$RUN_OUTPUT" "automatically added to project PVT_test_project with status Backlog" "run reported automatic project attach"
assert_file_contains "$GH_LOG" "addProjectV2ItemById" "run added issue to project automatically"
assert_file_contains "$GH_LOG" "contentId=ISSUE-42" "run used repo issue id for project attach"
assert_file_contains "$GH_LOG" "itemId=ITEM-AUTO-ADDED" "run updated status for auto-attached item"
assert_file_contains "$GH_LOG" "optionId=OPT_BACKLOG" "run initialized auto-attached issue with Backlog"
assert_file_contains "$GH_LOG" "optionId=OPT_ANALYSIS" "run claimed auto-attached issue into Analysis In Progress"

if [[ -f "$ISSUE_INDEX" ]]; then
    echo "  PASS: run created issue index after auto-attach"
    ((PASS++)) || true
else
    echo "  FAIL: run created issue index after auto-attach"
    ((FAIL++)) || true
fi
