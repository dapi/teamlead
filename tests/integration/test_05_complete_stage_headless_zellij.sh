#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"
REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-complete-stage-real-XXXXXX)"
REMOTE_REPO="$(mktemp -d /tmp/ai-teamlead-complete-stage-remote-XXXXXX)"
STUB_BIN="$(mktemp -d /tmp/ai-teamlead-complete-stage-stub-bin-XXXXXX)"
STUB_OUT="$(mktemp -d /tmp/ai-teamlead-complete-stage-stub-out-XXXXXX)"
GH_LOG="$(mktemp /tmp/ai-teamlead-complete-stage-gh-log-XXXXXX)"
GH_SNAPSHOT="$(mktemp /tmp/ai-teamlead-complete-stage-gh-snapshot-XXXXXX)"

create_initialized_repo "$REPO_ROOT" "$AI_TEAMLEAD_BIN"
git init --bare -q "$REMOTE_REPO"
git -C "$REPO_ROOT" config remote.origin.pushurl "$REMOTE_REPO"
cat > "$REPO_ROOT/.ai-teamlead/launch-agent.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

LOG_FILE="./.git/.ai-teamlead/complete-stage-launch.log"
exec >"$LOG_FILE" 2>&1
set -x

SESSION_UUID="${1:?usage: launch-agent.sh <session_uuid> <issue_url>}"
ISSUE_URL="${2:?usage: launch-agent.sh <session_uuid> <issue_url>}"
AI_TEAMLEAD_BIN="${AI_TEAMLEAD_BIN:-ai-teamlead}"
PRIMARY_REPO_ROOT="$(pwd -P)"

"$AI_TEAMLEAD_BIN" internal bind-zellij-pane "$SESSION_UUID"
eval "$("$AI_TEAMLEAD_BIN" internal render-launch-agent-context "$ISSUE_URL")"

mkdir -p "$(dirname "$WORKTREE_ROOT")"
if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
    git worktree add "$WORKTREE_ROOT" "$BRANCH"
else
    git worktree add -b "$BRANCH" "$WORKTREE_ROOT" main
fi

cd "$WORKTREE_ROOT"
./init.sh
mkdir -p "$ANALYSIS_ARTIFACTS_DIR"

export AI_TEAMLEAD_SESSION_UUID="$SESSION_UUID"
export AI_TEAMLEAD_ISSUE_URL="$ISSUE_URL"
export AI_TEAMLEAD_ANALYSIS_BRANCH="$BRANCH"
export AI_TEAMLEAD_WORKTREE_ROOT="$WORKTREE_ROOT"
export AI_TEAMLEAD_ANALYSIS_ARTIFACTS_DIR="$ANALYSIS_ARTIFACTS_DIR"
export AI_TEAMLEAD_REPO_ROOT="$PRIMARY_REPO_ROOT"

PROMPT="$(cat ./.ai-teamlead/flows/issue-analysis-flow.md)

Issue URL: $ISSUE_URL
Session UUID: $SESSION_UUID
Analysis branch: $BRANCH
Analysis artifacts dir: $ANALYSIS_ARTIFACTS_DIR"

exec codex --cd "$WORKTREE_ROOT" --no-alt-screen "$PROMPT"
EOF
chmod +x "$REPO_ROOT/.ai-teamlead/launch-agent.sh"

cat > "$GH_SNAPSHOT" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-43","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":43,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

install_gh_stub "$STUB_BIN" "$GH_SNAPSHOT" "$GH_LOG"
install_complete_stage_agent_stub "$STUB_BIN" "$STUB_OUT"
export PATH="$STUB_BIN:$PATH"

(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" run 43
)

ISSUE_INDEX="$REPO_ROOT/.git/.ai-teamlead/issues/43.json"
if ! wait_for_file "$ISSUE_INDEX"; then
    echo "  FAIL: run created issue index for complete-stage flow"
    ((FAIL++)) || true
    return 0
fi

SESSION_UUID="$(jq -r '.session_uuid' "$ISSUE_INDEX")"
SESSION_MANIFEST="$REPO_ROOT/.git/.ai-teamlead/sessions/$SESSION_UUID/session.json"
if ! wait_for_file "$SESSION_MANIFEST"; then
    echo "  FAIL: run created session manifest for complete-stage flow"
    ((FAIL++)) || true
    return 0
fi

WORKTREE_ROOT="${HOME}/worktrees/example/analysis/issue-43"
ARTIFACTS_DIR="$WORKTREE_ROOT/specs/issues/43"
ARTIFACT_README="$ARTIFACTS_DIR/README.md"
ARTIFACT_WHAT="$ARTIFACTS_DIR/01-what-we-build.md"
ARTIFACT_HOW="$ARTIFACTS_DIR/02-how-we-build.md"
ARTIFACT_VERIFY="$ARTIFACTS_DIR/03-how-we-verify.md"

wait_for_dir "$WORKTREE_ROOT" 30 || true
wait_for_dir "$ARTIFACTS_DIR" 30 || true
wait_for_file "$ARTIFACT_README" 30 || true
wait_for_file "$ARTIFACT_WHAT" 30 || true
wait_for_file "$ARTIFACT_HOW" 30 || true
wait_for_file "$ARTIFACT_VERIFY" 30 || true
wait_for_file "$STUB_OUT/complete-stage.exit_code" 30 || true

PANE_ID="$(wait_for_json_field_not_value "$SESSION_MANIFEST" '.zellij.pane_id' 'pending' 30 || true)"
SESSION_STATUS="$(wait_for_json_field_not_value "$SESSION_MANIFEST" '.status' 'active' 30 || true)"
FLOW_STATUS="$(wait_for_json_field_not_value "$ISSUE_INDEX" '.last_known_flow_status' 'Analysis In Progress' 30 || true)"

assert_file_exists "$ISSUE_INDEX" "run created issue index for complete-stage flow"
assert_file_exists "$SESSION_MANIFEST" "run created session manifest for complete-stage flow"
assert_file_exists "$STUB_OUT/codex.invoked" "stub agent started inside zellij pane"
assert_file_exists "$ARTIFACT_README" "stub agent created issue README"
assert_file_exists "$ARTIFACT_WHAT" "stub agent created what-we-build artifact"
assert_file_exists "$ARTIFACT_HOW" "stub agent created how-we-build artifact"
assert_file_exists "$ARTIFACT_VERIFY" "stub agent created how-we-verify artifact"
assert_eq "$(cat "$STUB_OUT/complete-stage.exit_code")" "0" "complete-stage exited successfully"
assert_eq "$(cat "$STUB_OUT/issue_url")" "https://github.com/dapi/example/issues/43" "stub agent received issue URL"
assert_eq "$(cat "$STUB_OUT/session_uuid")" "$SESSION_UUID" "stub agent received session UUID"
assert_eq "$(cat "$STUB_OUT/analysis_branch")" "analysis/issue-43" "stub agent received analysis branch"
assert_eq "$(cat "$STUB_OUT/worktree_root")" "$WORKTREE_ROOT" "stub agent received worktree root"
assert_eq "$(cat "$STUB_OUT/analysis_artifacts_dir")" "specs/issues/43" "stub agent received artifacts dir"
assert_eq "$(cat "$STUB_OUT/codex.cwd")" "$WORKTREE_ROOT" "stub agent ran inside analysis worktree"
assert_ne "$PANE_ID" "" "complete-stage flow captured zellij pane id"
assert_eq "$SESSION_STATUS" "completed" "complete-stage marked session as completed"
assert_eq "$FLOW_STATUS" "Waiting for Plan Review" "complete-stage updated issue flow status"
assert_file_contains "$ARTIFACT_README" "generated from stub codex agent" "artifact README content was written before complete-stage"
assert_file_contains "$ARTIFACT_README" "01-what-we-build.md" "artifact README links what-we-build doc"
assert_file_contains "$ARTIFACT_WHAT" "## User Story" "feature artifact includes User Story section"
assert_file_contains "$ARTIFACT_WHAT" "## Use Cases" "feature artifact includes Use Cases section"
assert_file_contains "$ARTIFACT_HOW" "## Approach" "how-we-build artifact includes Approach section"
assert_file_contains "$ARTIFACT_VERIFY" "## Acceptance Criteria" "how-we-verify artifact includes Acceptance Criteria section"
assert_file_contains "$ARTIFACT_VERIFY" "## Verification Checklist" "how-we-verify artifact includes Verification Checklist section"
assert_eq "$(git -C "$WORKTREE_ROOT" log -1 --pretty=%s)" "analysis(#43): stub analysis ready" "complete-stage committed analysis artifacts"
assert_eq "$(git -C "$WORKTREE_ROOT" status --short)" "" "analysis worktree is clean after complete-stage commit"
if git --git-dir="$REMOTE_REPO" show-ref --verify --quiet refs/heads/analysis/issue-43; then
    echo "  PASS: complete-stage pushed analysis branch"
    ((PASS++)) || true
else
    echo "  FAIL: complete-stage pushed analysis branch"
    ((FAIL++)) || true
fi
assert_file_contains "$GH_LOG" "gh pr list --head analysis/issue-43 --json number --jq length" "complete-stage checked for existing PR"
assert_file_contains "$GH_LOG" "gh pr create --draft --title analysis(#43): stub analysis ready" "complete-stage created draft PR"
assert_file_contains "$GH_LOG" "itemId=ITEM-43" "complete-stage updated GitHub Project status"
assert_file_contains "$STUB_OUT/complete-stage.stdout" "complete-stage: created draft PR: https://github.com/dapi/example/pull/99" "complete-stage reported created draft PR"
assert_file_contains "$STUB_OUT/complete-stage.stdout" "complete-stage: issue=#43 outcome=plan-ready status=Waiting for Plan Review" "complete-stage reported final outcome"
