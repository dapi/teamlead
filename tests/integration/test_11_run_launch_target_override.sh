#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"

run_override_scenario() {
    local repo_root stub_bin stub_out gh_log gh_snapshot settings_target cli_target expected_tab_name run_output issue_index session_uuid session_manifest settings_file
    repo_root="$1"
    settings_target="$2"
    cli_target="$3"
    expected_tab_name="$4"
    stub_bin="$(mktemp -d /tmp/ai-teamlead-launch-target-override-bin-XXXXXX)"
    stub_out="$(mktemp -d /tmp/ai-teamlead-launch-target-override-out-XXXXXX)"
    gh_log="$(mktemp /tmp/ai-teamlead-launch-target-override-gh-log-XXXXXX)"
    gh_snapshot="$(mktemp /tmp/ai-teamlead-launch-target-override-gh-snapshot-XXXXXX)"
    settings_file="$repo_root/.ai-teamlead/settings.yml"

    cat > "$gh_snapshot" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-42","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":42,"state":"OPEN","repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

    sed -i "s/^  launch_target: \".*\"$/  launch_target: \"$settings_target\"/" "$settings_file"
    sed -i '/^  tab_name: "issue-analysis"$/a\  tab_name_template: "#${ISSUE_NUMBER}"' "$settings_file"

    install_gh_stub "$stub_bin" "$gh_snapshot" "$gh_log"
    install_agent_stubs "$stub_bin" "$stub_out"
    export PATH="$stub_bin:$PATH"
    export AI_TEAMLEAD_STUB_AGENT_SLEEP=8

    run_output="$(
        cd "$repo_root"
        "$AI_TEAMLEAD_BIN" run -d 42 --launch-target "$cli_target" 2>&1
    )"

    issue_index="$repo_root/.git/.ai-teamlead/issues/42.json"
    wait_for_file "$issue_index" 30 || true
    session_uuid="$(issue_session_uuid "$issue_index")"
    session_manifest="$repo_root/.git/.ai-teamlead/sessions/$session_uuid/session.json"
    wait_for_file "$session_manifest" 30 || true

    assert_eq "$(jq -r '.zellij.launch_target' "$session_manifest")" "$cli_target" "run stores CLI launch target override '$cli_target'"
    assert_eq "$(jq -r '.zellij.tab_name' "$session_manifest")" "$expected_tab_name" "run resolves effective tab name for '$cli_target'"
    assert_text_contains "$run_output" "launch_target=$cli_target" "run prints effective launch target '$cli_target'"
    assert_file_contains "$settings_file" "launch_target: \"$settings_target\"" "run override '$cli_target' does not mutate settings.yml"

    cleanup_zellij
}

REPO_ROOT_TAB="$(mktemp -d /tmp/ai-teamlead-run-launch-target-tab-default-XXXXXX)"
create_initialized_repo "$REPO_ROOT_TAB" "$AI_TEAMLEAD_BIN"
run_override_scenario "$REPO_ROOT_TAB" "tab" "pane" "issue-analysis"

REPO_ROOT_PANE="$(mktemp -d /tmp/ai-teamlead-run-launch-target-pane-default-XXXXXX)"
create_initialized_repo "$REPO_ROOT_PANE" "$AI_TEAMLEAD_BIN"
run_override_scenario "$REPO_ROOT_PANE" "pane" "tab" "#42"
