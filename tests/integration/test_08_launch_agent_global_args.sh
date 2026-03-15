#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"
BASE_TEST_PATH="/usr/local/bin:/usr/bin:/bin"

run_codex_override_scenario() {
    local repo_root worktree_base stub_bin stub_out gh_log gh_snapshot run_output
    repo_root="$(mktemp -d /tmp/ai-teamlead-codex-override-XXXXXX)"
    worktree_base="$(mktemp -d /tmp/ai-teamlead-codex-worktrees-XXXXXX)"
    stub_bin="$(mktemp -d /tmp/ai-teamlead-codex-override-bin-XXXXXX)"
    stub_out="$(mktemp -d /tmp/ai-teamlead-codex-override-out-XXXXXX)"
    gh_log="$(mktemp /tmp/ai-teamlead-codex-override-gh-log-XXXXXX)"
    gh_snapshot="$(mktemp /tmp/ai-teamlead-codex-override-gh-snapshot-XXXXXX)"

    create_initialized_repo "$repo_root" "$AI_TEAMLEAD_BIN"
    cat > "$repo_root/.ai-teamlead/settings.yml" <<EOF
github:
  project_id: "PVT_test_project"

issue_analysis_flow:
  statuses:
    backlog: "Backlog"
    analysis_in_progress: "Analysis In Progress"
    waiting_for_clarification: "Waiting for Clarification"
    waiting_for_plan_review: "Waiting for Plan Review"
    ready_for_implementation: "Ready for Implementation"
    analysis_blocked: "Analysis Blocked"

issue_implementation_flow:
  statuses:
    ready_for_implementation: "Ready for Implementation"
    implementation_in_progress: "Implementation In Progress"
    waiting_for_ci: "Waiting for CI"
    waiting_for_code_review: "Waiting for Code Review"
    implementation_blocked: "Implementation Blocked"

runtime:
  max_parallel: 1
  poll_interval_seconds: 3600

zellij:
  session_name: "\${REPO}"
  tab_name: "issue-analysis"

launch_agent:
  analysis_branch_template: "analysis/issue-\${ISSUE_NUMBER}"
  worktree_root_template: "${worktree_base}/\${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/\${ISSUE_NUMBER}"
  global_args:
    claude:
      - "--permission-mode"
      - "auto"
    codex:
      - "--sandbox"
      - "workspace-write"
  implementation_branch_template: "implementation/issue-\${ISSUE_NUMBER}"
  implementation_worktree_root_template: "${worktree_base}/\${BRANCH}"
  implementation_artifacts_dir_template: "specs/issues/\${ISSUE_NUMBER}"
EOF

    cat > "$gh_snapshot" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

    install_gh_stub "$stub_bin" "$gh_snapshot" "$gh_log"
    install_agent_stubs "$stub_bin" "$stub_out"
    export PATH="$stub_bin:$BASE_TEST_PATH"
    export AI_TEAMLEAD_STUB_AGENT_SLEEP=2

    if ! run_output="$(
        cd "$repo_root"
        "$AI_TEAMLEAD_BIN" run -d 42 2>&1
    )"; then
        printf '%s\n' "$run_output"
        return 1
    fi

    wait_for_file "$stub_out/codex.invoked" 30 || true
    assert_file_exists "$stub_out/codex.invoked" "codex override scenario invoked codex"
    assert_file_contains "$stub_out/codex.args" "--sandbox" "codex override scenario passes custom flag"
    assert_file_contains "$stub_out/codex.args" "workspace-write" "codex override scenario passes custom value"
    if [[ -f "$stub_out/codex.args" ]] && grep -Fxq -- "--ask-for-approval" "$stub_out/codex.args"; then
        echo "  FAIL: codex override scenario must not keep default approval args"
        ((FAIL++)) || true
    else
        echo "  PASS: codex override scenario replaces default approval args"
        ((PASS++)) || true
    fi
}

run_claude_override_scenario() {
    local repo_root worktree_base stub_bin stub_out gh_log gh_snapshot run_output
    repo_root="$(mktemp -d /tmp/ai-teamlead-claude-override-XXXXXX)"
    worktree_base="$(mktemp -d /tmp/ai-teamlead-claude-override-worktrees-XXXXXX)"
    stub_bin="$(mktemp -d /tmp/ai-teamlead-claude-override-bin-XXXXXX)"
    stub_out="$(mktemp -d /tmp/ai-teamlead-claude-override-out-XXXXXX)"
    gh_log="$(mktemp /tmp/ai-teamlead-claude-override-gh-log-XXXXXX)"
    gh_snapshot="$(mktemp /tmp/ai-teamlead-claude-override-gh-snapshot-XXXXXX)"

    create_initialized_repo "$repo_root" "$AI_TEAMLEAD_BIN"
    cat > "$repo_root/.ai-teamlead/settings.yml" <<EOF
github:
  project_id: "PVT_test_project"

issue_analysis_flow:
  statuses:
    backlog: "Backlog"
    analysis_in_progress: "Analysis In Progress"
    waiting_for_clarification: "Waiting for Clarification"
    waiting_for_plan_review: "Waiting for Plan Review"
    ready_for_implementation: "Ready for Implementation"
    analysis_blocked: "Analysis Blocked"

issue_implementation_flow:
  statuses:
    ready_for_implementation: "Ready for Implementation"
    implementation_in_progress: "Implementation In Progress"
    waiting_for_ci: "Waiting for CI"
    waiting_for_code_review: "Waiting for Code Review"
    implementation_blocked: "Implementation Blocked"

runtime:
  max_parallel: 1
  poll_interval_seconds: 3600

zellij:
  session_name: "\${REPO}"
  tab_name: "issue-analysis"

launch_agent:
  analysis_branch_template: "analysis/issue-\${ISSUE_NUMBER}"
  worktree_root_template: "${worktree_base}/\${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/\${ISSUE_NUMBER}"
  global_args:
    claude:
      - "--dangerously-skip-permissions"
    codex:
      - "--ask-for-approval"
      - "never"
      - "--sandbox"
      - "workspace-write"
  implementation_branch_template: "implementation/issue-\${ISSUE_NUMBER}"
  implementation_worktree_root_template: "${worktree_base}/\${BRANCH}"
  implementation_artifacts_dir_template: "specs/issues/\${ISSUE_NUMBER}"
EOF

    cat > "$gh_snapshot" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

    install_gh_stub "$stub_bin" "$gh_snapshot" "$gh_log"
    install_agent_stubs "$stub_bin" "$stub_out"
    rm -f "$stub_bin/codex"
    export PATH="$stub_bin:$BASE_TEST_PATH"
    export AI_TEAMLEAD_STUB_AGENT_SLEEP=2

    if ! run_output="$(
        cd "$repo_root"
        AI_TEAMLEAD_AGENT_BIN="$stub_bin/claude" \
        AI_TEAMLEAD_AGENT_KIND="claude" \
        "$AI_TEAMLEAD_BIN" run -d 42 2>&1
    )"; then
        printf '%s\n' "$run_output"
        return 1
    fi

    wait_for_file "$stub_out/claude.invoked" 30 || true
    assert_file_exists "$stub_out/claude.invoked" "claude override scenario invoked claude"
    assert_file_contains "$stub_out/claude.args" "--dangerously-skip-permissions" "claude override scenario passes configured args"
    if [[ -f "$stub_out/claude.args" ]] && grep -Fxq -- "--permission-mode" "$stub_out/claude.args"; then
        echo "  FAIL: claude override scenario must not force default permission mode"
        ((FAIL++)) || true
    else
        echo "  PASS: claude override scenario does not force default permission mode"
        ((PASS++)) || true
    fi
}

run_claude_default_scenario() {
    local repo_root worktree_base stub_bin stub_out gh_log gh_snapshot run_output
    repo_root="$(mktemp -d /tmp/ai-teamlead-claude-default-XXXXXX)"
    worktree_base="$(mktemp -d /tmp/ai-teamlead-claude-worktrees-XXXXXX)"
    stub_bin="$(mktemp -d /tmp/ai-teamlead-claude-default-bin-XXXXXX)"
    stub_out="$(mktemp -d /tmp/ai-teamlead-claude-default-out-XXXXXX)"
    gh_log="$(mktemp /tmp/ai-teamlead-claude-default-gh-log-XXXXXX)"
    gh_snapshot="$(mktemp /tmp/ai-teamlead-claude-default-gh-snapshot-XXXXXX)"

    create_initialized_repo "$repo_root" "$AI_TEAMLEAD_BIN"
    cat > "$repo_root/.ai-teamlead/settings.yml" <<EOF
github:
  project_id: "PVT_test_project"

issue_analysis_flow:
  statuses:
    backlog: "Backlog"
    analysis_in_progress: "Analysis In Progress"
    waiting_for_clarification: "Waiting for Clarification"
    waiting_for_plan_review: "Waiting for Plan Review"
    ready_for_implementation: "Ready for Implementation"
    analysis_blocked: "Analysis Blocked"

issue_implementation_flow:
  statuses:
    ready_for_implementation: "Ready for Implementation"
    implementation_in_progress: "Implementation In Progress"
    waiting_for_ci: "Waiting for CI"
    waiting_for_code_review: "Waiting for Code Review"
    implementation_blocked: "Implementation Blocked"

runtime:
  max_parallel: 1
  poll_interval_seconds: 3600

zellij:
  session_name: "\${REPO}"
  tab_name: "issue-analysis"

launch_agent:
  analysis_branch_template: "analysis/issue-\${ISSUE_NUMBER}"
  worktree_root_template: "${worktree_base}/\${BRANCH}"
  analysis_artifacts_dir_template: "specs/issues/\${ISSUE_NUMBER}"
  global_args:
    claude:
      - "--permission-mode"
      - "auto"
    codex:
      - "--ask-for-approval"
      - "never"
      - "--sandbox"
      - "workspace-write"
  implementation_branch_template: "implementation/issue-\${ISSUE_NUMBER}"
  implementation_worktree_root_template: "${worktree_base}/\${BRANCH}"
  implementation_artifacts_dir_template: "specs/issues/\${ISSUE_NUMBER}"
EOF
    cat > "$gh_snapshot" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

    install_gh_stub "$stub_bin" "$gh_snapshot" "$gh_log"
    install_agent_stubs "$stub_bin" "$stub_out"
    rm -f "$stub_bin/codex"
    export PATH="$stub_bin:$BASE_TEST_PATH"
    export AI_TEAMLEAD_STUB_AGENT_SLEEP=2

    if ! run_output="$(
        cd "$repo_root"
        "$AI_TEAMLEAD_BIN" run -d 42 2>&1
    )"; then
        printf '%s\n' "$run_output"
        return 1
    fi

    wait_for_file "$stub_out/claude.invoked" 30 || true
    assert_file_exists "$stub_out/claude.invoked" "claude default scenario invoked claude"
    assert_file_contains "$stub_out/claude.args" "--permission-mode" "claude default scenario passes permission flag"
    assert_file_contains "$stub_out/claude.args" "auto" "claude default scenario passes auto mode"
    if [[ -e "$stub_out/codex.invoked" ]]; then
        echo "  FAIL: claude default scenario must not invoke codex"
        ((FAIL++)) || true
    else
        echo "  PASS: claude default scenario does not invoke codex"
        ((PASS++)) || true
    fi
}

run_degraded_fallback_scenario() {
    local repo_root stub_bin stub_out fake_bin fake_shell
    repo_root="$(mktemp -d /tmp/ai-teamlead-degraded-launcher-XXXXXX)"
    stub_bin="$(mktemp -d /tmp/ai-teamlead-degraded-launcher-bin-XXXXXX)"
    stub_out="$(mktemp -d /tmp/ai-teamlead-degraded-launcher-out-XXXXXX)"

    create_initialized_repo "$repo_root" "$AI_TEAMLEAD_BIN"
    fake_bin="$stub_bin/ai-teamlead"
    fake_shell="$stub_bin/fallback-shell"

    cat > "$fake_bin" <<EOF
#!/usr/bin/env bash
set -euo pipefail

if [[ "\${1:-}" == "internal" && "\${2:-}" == "bind-zellij-pane" ]]; then
  exit 0
fi

if [[ "\${1:-}" == "internal" && "\${2:-}" == "render-launch-agent-context" ]]; then
  cat <<'CTX'
ISSUE_NUMBER='42'
REPO='example'
FLOW_STAGE='analysis'
BRANCH='analysis/issue-42'
WORKTREE_ROOT='${repo_root}/.tmp/analysis-issue-42'
ARTIFACTS_DIR='specs/issues/42'
CLAUDE_GLOBAL_ARGS=('--permission-mode' 'auto')
CODEX_GLOBAL_ARGS=('--ask-for-approval' 'never' '--sandbox' 'workspace-write')
CTX
  exit 0
fi

echo "unexpected fake ai-teamlead invocation: \$*" >&2
exit 1
EOF
    chmod +x "$fake_bin"

    cat > "$fake_shell" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${AI_TEAMLEAD_STUB_OUT_DIR:?}"
printf '%s\n' "$PWD" > "$OUT_DIR/fallback.cwd"
printf '%s\n' "${AI_TEAMLEAD_WORKTREE_ROOT:-}" > "$OUT_DIR/fallback.worktree_root"
exit 0
EOF
    chmod +x "$fake_shell"

    (
        cd "$repo_root"
        PATH="$stub_bin:/usr/bin:/bin" \
        AI_TEAMLEAD_BIN="$fake_bin" \
        AI_TEAMLEAD_STUB_OUT_DIR="$stub_out" \
        SHELL="$fake_shell" \
        ./.ai-teamlead/launch-agent.sh test-session https://github.com/dapi/example/issues/42 >/dev/null 2>"$stub_out/fallback.stderr"
    )

    assert_file_exists "$stub_out/fallback.cwd" "degraded fallback scenario entered shell"
    assert_eq "$(cat "$stub_out/fallback.cwd")" "${repo_root}/.tmp/analysis-issue-42" "degraded fallback enters analysis worktree"
    assert_dir_exists "${repo_root}/.tmp/analysis-issue-42/specs/issues/42" "degraded fallback still creates artifacts dir"
    assert_file_contains "$stub_out/fallback.stderr" "no supported agent found" "degraded fallback reports agent fallback before shell handoff"
}

run_codex_override_scenario
cleanup_zellij
run_claude_override_scenario
cleanup_zellij
run_claude_default_scenario
cleanup_zellij
run_degraded_fallback_scenario
