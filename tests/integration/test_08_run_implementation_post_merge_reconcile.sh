#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="${AI_TEAMLEAD_BIN:-/test/bin/ai-teamlead}"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-run-implementation-merged-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-run-implementation-merged-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-run-implementation-merged-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-run-implementation-merged-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-run-implementation-merged-gh-snapshot-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_ANALYSIS_BLOCKED","name":"Analysis Blocked"},{"id":"OPT_IMPL_PROGRESS","name":"Implementation In Progress"},{"id":"OPT_IMPL_CI","name":"Waiting for CI"},{"id":"OPT_IMPL_REVIEW","name":"Waiting for Code Review"},{"id":"OPT_DONE","name":"Done"},{"id":"OPT_IMPL_BLOCKED","name":"Implementation Blocked"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Waiting for Code Review","optionId":"OPT_IMPL_REVIEW"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"
export AI_TEAMLEAD_TEST_GH_PR_LIST_RESULT='[{"number":99,"url":"https://github.com/dapi/example/pull/99"}]'
export AI_TEAMLEAD_TEST_GH_PR_VIEW_RESULT='{"number":99,"url":"https://github.com/dapi/example/pull/99","state":"MERGED","mergedAt":"2026-03-14T20:00:00Z","isDraft":false,"headRefName":"implementation/issue-42","baseRefName":"main"}'

SESSION_UUID="implementation-session-42"
mkdir -p "$REPO_ROOT/.git/.ai-teamlead/issues"
mkdir -p "$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID"

cat > "$REPO_ROOT/.git/.ai-teamlead/issues/42.json" <<EOF
{
  "issue_number": 42,
  "bindings": {
    "implementation": "$SESSION_UUID"
  },
  "last_known_flow_status": "Waiting for Code Review",
  "updated_at": "2026-03-14T20:00:00Z"
}
EOF

cat > "$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json" <<EOF
{
  "session_uuid": "$SESSION_UUID",
  "issue_number": 42,
  "repo_root": "$REPO_ROOT",
  "github_owner": "dapi",
  "github_repo": "example",
  "project_id": "PVT_test_project",
  "stage": "implementation",
  "status": "completed",
  "created_at": "2026-03-14T20:00:00Z",
  "updated_at": "2026-03-14T20:00:00Z",
  "stage_branch": "implementation/issue-42",
  "stage_artifacts_dir": "specs/issues/42",
  "zellij": {
    "session_name": "example",
    "tab_name": "issue-analysis",
    "session_id": "pending",
    "tab_id": "pending",
    "pane_id": "pending"
  }
}
EOF

RUN_OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" run 42 2>&1
)"

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/42.json"
SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"

assert_text_contains "$RUN_OUTPUT" "reconciled merged implementation PR -> Done" "run reported merged reconciliation"
assert_text_contains "$RUN_OUTPUT" "finalized without launch" "run skipped zellij launch after merge"
assert_eq "$(jq -r '.last_known_flow_status' "$ISSUE_INDEX")" "Done" "run moved issue flow status to Done"
assert_eq "$(jq -r '.status' "$SESSION_MANIFEST")" "completed" "run kept implementation session completed"
assert_file_contains "$GH_LOG" "gh pr list --head implementation/issue-42 --json number,url" "run listed canonical implementation PR by branch"
assert_file_contains "$GH_LOG" "gh pr view 99 --json number,url,state,mergedAt,isDraft,headRefName,baseRefName" "run inspected canonical PR merge state"
assert_file_contains "$GH_LOG" "optionId=OPT_DONE" "run updated GitHub Project status to Done"
assert_file_contains "$GH_LOG" "gh issue close 42" "run closed GitHub issue after merged PR"

if [[ ! -f "$STUB_OUT/codex.invoked" ]]; then
    echo "  PASS: merged reconciliation did not launch agent"
    ((PASS++)) || true
else
    echo "  FAIL: merged reconciliation did not launch agent"
    ((FAIL++)) || true
fi

print_summary
