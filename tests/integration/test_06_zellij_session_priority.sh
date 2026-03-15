#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-run-priority-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-run-priority-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-run-priority-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-run-priority-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-run-priority-gh-snapshot-XXXXXX)"
ENV_SESSION="outer-session-$$"
ARG_SESSION="cli-session-$$"
TARGET_LAYOUT="$(mktemp /tmp/ai-teamlead-priority-layout-XXXXXX.kdl)"
WORKTREE_BASE="$(mktemp -d /tmp/ai-teamlead-run-priority-worktrees-XXXXXX)"
OCCUPIED_WORKTREE_ROOT="${WORKTREE_BASE}/analysis/issue-42"
FOREIGN_REPO="$(mktemp -d /tmp/ai-teamlead-run-priority-foreign-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"
perl -0pi -e "s|\\$\\{HOME\\}/worktrees/\\$\\{REPO\\}/\\$\\{BRANCH\\}|${WORKTREE_BASE}/\\\${BRANCH}|g" \
    "$REPO_ROOT/.ai-teamlead/settings.yml"
git init -q -b main "$FOREIGN_REPO"
git -C "$FOREIGN_REPO" config user.name "AI Teamlead Foreign Fixture"
git -C "$FOREIGN_REPO" config user.email "ai-teamlead-foreign@example.com"
printf '# foreign fixture\n' > "$FOREIGN_REPO/README.md"
git -C "$FOREIGN_REPO" add README.md
git -C "$FOREIGN_REPO" commit -q -m "initial"
mkdir -p "$(dirname "$OCCUPIED_WORKTREE_ROOT")"
git -C "$FOREIGN_REPO" worktree add -b analysis/issue-42 "$OCCUPIED_WORKTREE_ROOT" main >/dev/null

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"
export AI_TEAMLEAD_STUB_AGENT_SLEEP=8
export ZELLIJ=0
export ZELLIJ_SESSION_NAME="$ENV_SESSION"
export ZELLIJ_PANE_ID="terminal_0"

cat > "$TARGET_LAYOUT" <<EOF
layout {
  tab name="preexisting-target" {
    pane command="sleep" {
      args "120"
    }
  }
}
EOF

script -qfc "zellij --session '$ARG_SESSION' -n '$TARGET_LAYOUT'" /dev/null &
TARGET_PID=$!
sleep 2

assert_session_alive "$ARG_SESSION" "preexisting target zellij session created before run"

RUN_OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" run --zellij-session "$ARG_SESSION" 42 2>&1
)"

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/42.json"
if ! wait_for_file "$ISSUE_INDEX"; then
    echo "  FAIL: run created issue index"
    ((FAIL++)) || true
    return 0
fi

SESSION_UUID="$(issue_session_uuid "$ISSUE_INDEX")"
SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"
if ! wait_for_file "$SESSION_MANIFEST"; then
    echo "  FAIL: run created session manifest"
    ((FAIL++)) || true
    return 0
fi

SESSION_NAME="$(jq -r '.zellij.session_name' "$SESSION_MANIFEST")"
PANE_ID="$(wait_for_json_field_not_value "$SESSION_MANIFEST" '.zellij.pane_id' 'pending' 30 || true)"
if ! wait_for_file "$STUB_OUT/worktree_root" 30; then
    echo "  FAIL: run recorded worktree root for launched agent"
    ((FAIL++)) || true
    ACTUAL_WORKTREE_ROOT=""
else
    ACTUAL_WORKTREE_ROOT="$(cat "$STUB_OUT/worktree_root")"
fi

assert_eq "$SESSION_NAME" "$ARG_SESSION" "cli zellij session override beats env session"
assert_ne "$PANE_ID" "" "cli override launch captured pane id"
assert_session_alive "$ARG_SESSION" "cli override created requested zellij session"
assert_text_contains "$RUN_OUTPUT" "zellij_session=$ARG_SESSION" "run printed cli-selected zellij session"
assert_ne "$ACTUAL_WORKTREE_ROOT" "$OCCUPIED_WORKTREE_ROOT" "run avoids occupied foreign worktree path"
if [[ -n "$ACTUAL_WORKTREE_ROOT" && "$ACTUAL_WORKTREE_ROOT" == "$OCCUPIED_WORKTREE_ROOT"-* ]]; then
    echo "  PASS: run uses deterministic fallback worktree root"
    ((PASS++)) || true
else
    echo "  FAIL: run uses deterministic fallback worktree root"
    echo "    expected prefix: ${OCCUPIED_WORKTREE_ROOT}-"
    echo "    actual:          $ACTUAL_WORKTREE_ROOT"
    ((FAIL++)) || true
fi

kill "$TARGET_PID" 2>/dev/null || true
wait "$TARGET_PID" 2>/dev/null || true
zellij kill-session "$ARG_SESSION" 2>/dev/null || true
