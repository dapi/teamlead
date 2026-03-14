#!/usr/bin/env bash
set -euo pipefail

# Regression test for issue #27: agent session exit must not kill other
# zellij sessions or crash the zellij server.
#
# Scenario:
#   1. Create an "outer" zellij session (simulates user's terminal)
#   2. From inside that session, launch ai-teamlead which must reuse the
#      current zellij session by default
#   3. Wait for the stub agent to finish and the launched pane to exit
#   4. Verify the outer session is still alive (server not crashed)

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-crash-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-crash-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-crash-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-crash-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-crash-gh-snapshot-XXXXXX)"
OUTER_SESSION="outer-session-$$"
MARKER_OUTER_ALIVE="$STUB_OUT/outer_session_alive"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-99","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":99,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_agent_stubs "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"
# Agent exits quickly so the inner pane terminates
export AI_TEAMLEAD_STUB_AGENT_SLEEP=2

# --- Step 1: Create an outer zellij session (simulates user's terminal) ---
# Use a layout with a long-running pane so the session stays alive.
OUTER_LAYOUT="$(mktemp /tmp/ai-teamlead-crash-layout-XXXXXX.kdl)"
cat > "$OUTER_LAYOUT" <<EOF
layout {
  tab name="user-terminal" {
    pane command="sleep" {
      args "120"
    }
  }
}
EOF

# Start the outer session (detached via script)
script -qfc "zellij --session '$OUTER_SESSION' -n '$OUTER_LAYOUT'" /dev/null &
OUTER_PID=$!
sleep 2

assert_session_alive "$OUTER_SESSION" "outer session created before test"

# --- Step 2: Launch ai-teamlead from inside the outer zellij session ---
# We simulate running inside zellij by setting ZELLIJ env vars, which is
# exactly what happens when a user runs ai-teamlead from their zellij pane.
LAUNCH_SCRIPT="$(mktemp /tmp/ai-teamlead-crash-launch-XXXXXX.sh)"
cat > "$LAUNCH_SCRIPT" <<EOF
#!/usr/bin/env bash
set -euo pipefail
export ZELLIJ=0
export ZELLIJ_SESSION_NAME=$OUTER_SESSION
export ZELLIJ_PANE_ID=terminal_0
export PATH="$STUB_BIN:\$PATH"
export AI_TEAMLEAD_TEST_GH_SNAPSHOT="$GH_SNAPSHOT"
export AI_TEAMLEAD_TEST_GH_LOG="$GH_LOG"
export AI_TEAMLEAD_STUB_OUT_DIR="$STUB_OUT"
export AI_TEAMLEAD_STUB_AGENT_SLEEP=2
cd "$REPO_ROOT"
"$AI_TEAMLEAD_BIN" run -d 99 >"$STUB_OUT/run_output.log" 2>&1
EOF
chmod +x "$LAUNCH_SCRIPT"
bash "$LAUNCH_SCRIPT" &
LAUNCH_PID=$!

# --- Step 3: Wait for the stub agent to finish ---
if ! wait_for_file "$STUB_OUT/codex.invoked" 30; then
    echo "  FAIL: stub agent was never invoked"
    ((FAIL++)) || true
    kill "$OUTER_PID" 2>/dev/null || true
    return 0
fi

# Wait for the stub agent sleep to complete (2s) plus buffer for pane exit
sleep 6

# --- Step 4: Verify the outer session survived ---
assert_session_alive "$OUTER_SESSION" "outer zellij session survives after agent exit (issue #27)"

# Additional check: verify the runtime binding points at the reused outer
# session rather than the fallback session from settings.
ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/99.json"
if [[ -f "$ISSUE_INDEX" ]]; then
    SESSION_UUID="$(jq -r '.session_uuid' "$ISSUE_INDEX")"
    LAUNCH_LOG="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/launch.log"
    LAYOUT_FILE="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/launch-layout.kdl"
    SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"

    if [[ -f "$LAYOUT_FILE" ]]; then
        assert_file_contains "$LAYOUT_FILE" "close_on_exit false" "layout includes close_on_exit false for pane lifecycle"
    fi
    if [[ -f "$SESSION_MANIFEST" ]]; then
        assert_eq "$(jq -r '.zellij.session_name' "$SESSION_MANIFEST")" "$OUTER_SESSION" "run reused current zellij session from env"
    fi
fi

# Cleanup
kill "$OUTER_PID" 2>/dev/null || true
wait "$OUTER_PID" 2>/dev/null || true
kill "$LAUNCH_PID" 2>/dev/null || true
wait "$LAUNCH_PID" 2>/dev/null || true
zellij kill-session "$OUTER_SESSION" 2>/dev/null || true
