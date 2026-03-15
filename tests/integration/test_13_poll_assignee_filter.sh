#!/usr/bin/env bash
set -euo pipefail

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"

run_poll_case() {
    local filter_yaml="$1"
    local expected_issue="$2"
    local expected_item="$3"
    local gh_user="${4:-}"
    local expect_user_lookup="$5"
    local label="$6"

    local repo_root stub_bin stub_out gh_log gh_snapshot
    repo_root="$(mktemp -d /tmp/ai-teamlead-poll-assignee-real-XXXXXX)"
    stub_bin="$(mktemp -d /tmp/ai-teamlead-poll-assignee-stub-bin-XXXXXX)"
    stub_out="$(mktemp -d /tmp/ai-teamlead-poll-assignee-stub-out-XXXXXX)"
    gh_log="$(mktemp /tmp/ai-teamlead-poll-assignee-gh-log-XXXXXX)"
    gh_snapshot="$(mktemp /tmp/ai-teamlead-poll-assignee-gh-snapshot-XXXXXX)"

    create_initialized_repo "$repo_root" "$AI_TEAMLEAD_BIN"
    perl -0pi -e 's/session_name: "example"/session_name: "example-'"$expected_issue"'"/' \
        "$repo_root/.ai-teamlead/settings.yml"
    cat >> "$repo_root/.ai-teamlead/settings.yml" <<EOF

poll:
  assignee_filter: $filter_yaml
EOF

    cat > "$gh_snapshot" <<'EOF'
{"data":{"node":{"id":"PVT_test_project","title":"Test Project","field":{"id":"STATUS_FIELD","options":[{"id":"OPT_BACKLOG","name":"Backlog"},{"id":"OPT_ANALYSIS","name":"Analysis In Progress"},{"id":"OPT_CLARIFY","name":"Waiting for Clarification"},{"id":"OPT_PLAN","name":"Waiting for Plan Review"},{"id":"OPT_READY","name":"Ready for Implementation"},{"id":"OPT_BLOCKED","name":"Analysis Blocked"}]},"items":{"nodes":[{"id":"ITEM-7","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":7,"state":"OPEN","assignees":{"nodes":[{"login":"alice"}]},"repository":{"name":"example","owner":{"login":"dapi"}}}},{"id":"ITEM-42","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":42,"state":"OPEN","assignees":{"nodes":[{"login":"bob"},{"login":"carol"}]},"repository":{"name":"example","owner":{"login":"dapi"}}}},{"id":"ITEM-99","fieldValueByName":{"name":"Backlog","optionId":"OPT_BACKLOG"},"content":{"number":99,"state":"OPEN","assignees":{"nodes":[]},"repository":{"name":"example","owner":{"login":"dapi"}}}}]}}}}
EOF

    install_gh_stub "$stub_bin" "$gh_snapshot" "$gh_log"
    install_agent_stubs "$stub_bin" "$stub_out"
    export PATH="$stub_bin:$PATH"
    export AI_TEAMLEAD_STUB_AGENT_SLEEP=8

    if [[ -n "$gh_user" ]]; then
        export AI_TEAMLEAD_TEST_GH_USER_LOGIN="$gh_user"
    else
        unset AI_TEAMLEAD_TEST_GH_USER_LOGIN || true
    fi

    (
        cd "$repo_root"
        env \
            -u ZELLIJ \
            -u ZELLIJ_SESSION_NAME \
            -u ZELLIJ_PANE_ID \
            "$AI_TEAMLEAD_BIN" poll
    )

    local issue_index session_uuid session_manifest launch_log
    issue_index="$repo_root/.git/.ai-teamlead/issues/$expected_issue.json"
    if ! wait_for_file "$issue_index"; then
        echo "  FAIL: $label created issue index"
        ((FAIL++)) || true
        return 0
    fi

    session_uuid="$(issue_session_uuid "$issue_index")"
    session_manifest="$repo_root/.git/.ai-teamlead/sessions/$session_uuid/session.json"
    launch_log="$repo_root/.git/.ai-teamlead/sessions/$session_uuid/launch.log"
    wait_for_file "$session_manifest" 30 || true
    wait_for_file "$stub_out/codex.invoked" 60 || true
    wait_for_file "$stub_out/issue_url" 60 || true
    if [[ ! -f "$stub_out/issue_url" && -f "$launch_log" ]]; then
        echo "  DIAG: $label launch log"
        sed 's/^/    /' "$launch_log"
    fi

    assert_file_exists "$issue_index" "$label created expected issue index"
    assert_file_exists "$session_manifest" "$label created session manifest"
    assert_file_exists "$stub_out/codex.invoked" "$label started stub agent"
    assert_eq "$(cat "$stub_out/issue_url")" "https://github.com/dapi/example/issues/$expected_issue" "$label launched expected issue"
    assert_file_contains "$gh_log" "itemId=$expected_item" "$label updated expected project item"
    assert_eq "$(grep -Fc 'gh api user --jq .login' "$gh_log")" "$expect_user_lookup" "$label resolved current user expected number of times"

    if [[ "$expected_issue" == "42" && -e "$repo_root/.git/.ai-teamlead/issues/7.json" ]]; then
        echo "  FAIL: $label should not claim alice issue for bob filter"
        ((FAIL++)) || true
    else
        echo "  PASS: $label did not claim unrelated backlog issue"
        ((PASS++)) || true
    fi
}

run_poll_case '"$me"' "7" "ITEM-7" "alice" "1" 'poll with "$me"'
run_poll_case '"bob"' "42" "ITEM-42" "" "0" 'poll with literal username'
