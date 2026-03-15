#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-loop-real-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-loop-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-loop-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-loop-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-loop-gh-snapshot-XXXXXX)"
LOOP_LOG="$(mktemp /tmp/ai-teamlead-loop-output-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"
perl -0pi -e 's/poll_interval_seconds: 3600/poll_interval_seconds: 1/' \
    "$REPO_ROOT/.ai-teamlead/settings.yml"
cat >> "$REPO_ROOT/.ai-teamlead/settings.yml" <<'EOF'

poll:
  assignee_filter: "$me"
EOF

cat > "$GH_SNAPSHOT" <<'EOF'
{"broken":
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"
export AI_TEAMLEAD_STUB_AGENT_SLEEP=1
export AI_TEAMLEAD_TEST_GH_USER_LOGIN=dapi

(
    sleep 0.5
    cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[]}}}}
EOF

    sleep 1
    cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-7","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":7,"state":"OPEN","assignees":{"nodes":[{"login":"dapi"}]},"repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

    sleep 1.5
    cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-7","fieldValueByName":{"name":"Analysis In Progress","optionId":"OPT_ANALYSIS"},"content":{"number":7,"state":"OPEN","assignees":{"nodes":[{"login":"dapi"}]},"repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF
) &
SNAPSHOT_UPDATER_PID=$!

LOOP_EXIT=0
(
    cd "$REPO_ROOT"
    timeout 7 "$AI_TEAMLEAD_BIN" loop >"$LOOP_LOG" 2>&1
) || LOOP_EXIT=$?

wait "$SNAPSHOT_UPDATER_PID"

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/7.json"
if ! wait_for_file "$ISSUE_INDEX"; then
    echo "  FAIL: loop created issue index after recovery"
    ((FAIL++)) || true
    return 0
fi

SESSION_UUID="$(issue_session_uuid "$ISSUE_INDEX")"
SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"
wait_for_file "$SESSION_MANIFEST" 30 || true

assert_eq "$LOOP_EXIT" "124" "loop keeps running until stopped externally"
assert_file_exists "$ISSUE_INDEX" "loop created issue index after failed and empty cycles"
assert_file_exists "$SESSION_MANIFEST" "loop created session manifest"
assert_file_contains "$GH_LOG" "itemId=ITEM-7" "loop updated status for recovered backlog item"
assert_file_contains "$LOOP_LOG" "loop: cycle=1 failed:" "loop logged first cycle failure"
assert_file_contains "$LOOP_LOG" "loop: cycle=2 no eligible backlog issues in project=Test Project" "loop survived empty cycle"
assert_file_contains "$LOOP_LOG" "loop: cycle=3 launched issue #7 session_uuid=" "loop launched issue on later cycle"
assert_file_contains "$LOOP_LOG" "loop: cycle=3 sleeping 1s" "loop kept scheduling after successful cycle"
assert_eq "$(grep -Fc 'gh api user --jq .login' "$GH_LOG")" "1" 'loop resolves "$me" exactly once per process'
