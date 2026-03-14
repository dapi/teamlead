#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="${AI_TEAMLEAD_BIN:-/test/bin/ai-teamlead}"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-run-implementation-real-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-run-implementation-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-run-implementation-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-run-implementation-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-run-implementation-gh-snapshot-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_ANALYSIS_BLOCKED","name":"Analysis Blocked"},{"id":"OPT_IMPL_PROGRESS","name":"Implementation In Progress"},{"id":"OPT_IMPL_CI","name":"Waiting for CI"},{"id":"OPT_IMPL_REVIEW","name":"Waiting for Code Review"},{"id":"OPT_IMPL_BLOCKED","name":"Implementation Blocked"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Ready for Implementation","optionId":"OPT_READY"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"

if RUN_OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" run 42 2>&1
)"; then
    echo "  FAIL: implementation run without approved artifacts must fail"
    ((FAIL++)) || true
else
    echo "  PASS: implementation run without approved artifacts must fail"
    ((PASS++)) || true
fi

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/42.json"
assert_text_contains "$RUN_OUTPUT" "approved analysis artifacts are missing" "run reported missing approved artifacts"
assert_file_contains "$GH_LOG" "itemId=ITEM-42" "run updated project status for selected implementation issue"
assert_file_contains "$GH_LOG" "optionId=OPT_IMPL_BLOCKED" "run moved implementation issue to blocked on invalid artifacts"

if [[ ! -f "$ISSUE_INDEX" ]]; then
    echo "  PASS: implementation run did not create issue index on invalid artifacts"
    ((PASS++)) || true
else
    echo "  FAIL: implementation run did not create issue index on invalid artifacts"
    echo "    unexpected file: $ISSUE_INDEX"
    ((FAIL++)) || true
fi

if [[ ! -f "$STUB_OUT/codex.invoked" ]]; then
    echo "  PASS: implementation run did not launch agent on invalid artifacts"
    ((PASS++)) || true
else
    echo "  FAIL: implementation run did not launch agent on invalid artifacts"
    ((FAIL++)) || true
fi
