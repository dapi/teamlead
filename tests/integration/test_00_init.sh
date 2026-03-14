#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(mktemp -d /tmp/ai-teamlead-init-XXXXXX)"
git init -q "$REPO_ROOT"
git -C "$REPO_ROOT" remote add origin git@github.com:dapi/example.git

AI_TEAMLEAD_BIN="/test/bin/ai-teamlead"

OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" init
)"

SETTINGS_FILE="$REPO_ROOT/.ai-teamlead/settings.yml"
README_FILE="$REPO_ROOT/.ai-teamlead/README.md"
PROJECT_INIT_FILE="$REPO_ROOT/.ai-teamlead/init.sh"
LAUNCH_AGENT_FILE="$REPO_ROOT/.ai-teamlead/launch-agent.sh"
ANALYSIS_TAB_TEMPLATE_FILE="$REPO_ROOT/.ai-teamlead/zellij/analysis-tab.kdl"
FLOW_FILE="$REPO_ROOT/.ai-teamlead/flows/issue-analysis-flow.md"
FLOW_README_FILE="$REPO_ROOT/.ai-teamlead/flows/issue-analysis/README.md"
FLOW_WHAT_FILE="$REPO_ROOT/.ai-teamlead/flows/issue-analysis/01-what-we-build.md"
FLOW_HOW_FILE="$REPO_ROOT/.ai-teamlead/flows/issue-analysis/02-how-we-build.md"
FLOW_VERIFY_FILE="$REPO_ROOT/.ai-teamlead/flows/issue-analysis/03-how-we-verify.md"
CLAUDE_README_FILE="$REPO_ROOT/.claude/README.md"
CODEX_README_FILE="$REPO_ROOT/.codex/README.md"
ROOT_INIT_LINK="$REPO_ROOT/init.sh"
RUNTIME_DIR="$REPO_ROOT/.git/.ai-teamlead"

assert_file_exists "$SETTINGS_FILE" "init created settings.yml"
assert_file_exists "$README_FILE" "init created .ai-teamlead README"
assert_file_exists "$PROJECT_INIT_FILE" "init created project-local init.sh"
assert_file_exists "$LAUNCH_AGENT_FILE" "init created project-local launch-agent.sh"
assert_file_exists "$ANALYSIS_TAB_TEMPLATE_FILE" "init created analysis tab template"
assert_file_exists "$FLOW_FILE" "init created issue-analysis-flow.md"
assert_file_exists "$FLOW_README_FILE" "init created issue-analysis staged README"
assert_file_exists "$FLOW_WHAT_FILE" "init created issue-analysis stage 1"
assert_file_exists "$FLOW_HOW_FILE" "init created issue-analysis stage 2"
assert_file_exists "$FLOW_VERIFY_FILE" "init created issue-analysis stage 3"
assert_file_exists "$CLAUDE_README_FILE" "init created .claude README"
assert_file_exists "$CODEX_README_FILE" "init created .codex README"
assert_file_exists "$ROOT_INIT_LINK" "init created root init.sh symlink"

if [[ -L "$ROOT_INIT_LINK" ]] && [[ "$(readlink "$ROOT_INIT_LINK")" == ".ai-teamlead/init.sh" ]]; then
    echo "  PASS: init created expected root init.sh symlink"
    ((PASS++)) || true
else
    echo "  FAIL: init created expected root init.sh symlink"
    ((FAIL++)) || true
fi

if [[ -d "$RUNTIME_DIR" ]]; then
    echo "  FAIL: init must not create runtime directory"
    ((FAIL++)) || true
else
    echo "  PASS: init does not create runtime directory"
    ((PASS++)) || true
fi

if [[ "$OUTPUT" == *"created: $SETTINGS_FILE"* ]] && [[ "$OUTPUT" == *"created: $README_FILE"* ]] && [[ "$OUTPUT" == *"created: $PROJECT_INIT_FILE"* ]] && [[ "$OUTPUT" == *"created: $LAUNCH_AGENT_FILE"* ]] && [[ "$OUTPUT" == *"created: $ANALYSIS_TAB_TEMPLATE_FILE"* ]] && [[ "$OUTPUT" == *"created: $FLOW_FILE"* ]] && [[ "$OUTPUT" == *"created: $FLOW_README_FILE"* ]] && [[ "$OUTPUT" == *"created: $FLOW_WHAT_FILE"* ]] && [[ "$OUTPUT" == *"created: $FLOW_HOW_FILE"* ]] && [[ "$OUTPUT" == *"created: $FLOW_VERIFY_FILE"* ]] && [[ "$OUTPUT" == *"created: $CLAUDE_README_FILE"* ]] && [[ "$OUTPUT" == *"created: $CODEX_README_FILE"* ]] && [[ "$OUTPUT" == *"created: $ROOT_INIT_LINK"* ]]; then
    echo "  PASS: init reports created files"
    ((PASS++)) || true
else
    echo "  FAIL: init reports created files"
    ((FAIL++)) || true
fi

SECOND_OUTPUT="$(
    cd "$REPO_ROOT"
    "$AI_TEAMLEAD_BIN" init
)"

if [[ "$SECOND_OUTPUT" == *"skipped: $SETTINGS_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $README_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $PROJECT_INIT_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $LAUNCH_AGENT_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $ANALYSIS_TAB_TEMPLATE_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $FLOW_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $FLOW_README_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $FLOW_WHAT_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $FLOW_HOW_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $FLOW_VERIFY_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $CLAUDE_README_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $CODEX_README_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $ROOT_INIT_LINK"* ]]; then
    echo "  PASS: init is idempotent"
    ((PASS++)) || true
else
    echo "  FAIL: init is idempotent"
    ((FAIL++)) || true
fi

if grep -Fq 'layout: "compact"' "$SETTINGS_FILE"; then
    echo "  PASS: init bootstraps zellij.layout as compact"
    ((PASS++)) || true
else
    echo "  FAIL: init bootstraps zellij.layout as compact"
    ((FAIL++)) || true
fi

if grep -Fq 'plugin location="compact-bar"' "$ANALYSIS_TAB_TEMPLATE_FILE"; then
    echo "  PASS: init bootstraps analysis tab with compact-bar"
    ((PASS++)) || true
else
    echo "  FAIL: init bootstraps analysis tab with compact-bar"
    ((FAIL++)) || true
fi

if grep -Fq 'README.md' "$FLOW_FILE" && \
   grep -Fq '01-what-we-build.md' "$FLOW_FILE" && \
   grep -Fq '02-how-we-build.md' "$FLOW_FILE" && \
   grep -Fq '03-how-we-verify.md' "$FLOW_FILE"; then
    echo "  PASS: init bootstraps minimal SDD artifact contract"
    ((PASS++)) || true
else
    echo "  FAIL: init bootstraps minimal SDD artifact contract"
    ((FAIL++)) || true
fi

if grep -Fq 'User Story' "$FLOW_WHAT_FILE" && \
   grep -Fq 'Use Cases' "$FLOW_WHAT_FILE" && \
   grep -Fq 'Observed Behavior' "$FLOW_WHAT_FILE" && \
   grep -Fq 'Operational Goal' "$FLOW_WHAT_FILE"; then
    echo "  PASS: init bootstraps rule-based task sections for what-we-build"
    ((PASS++)) || true
else
    echo "  FAIL: init bootstraps rule-based task sections for what-we-build"
    ((FAIL++)) || true
fi

if grep -Fq 'Acceptance Criteria' "$FLOW_VERIFY_FILE" && \
   grep -Fq 'Happy Path' "$FLOW_VERIFY_FILE" && \
   grep -Fq 'Regression Checks' "$FLOW_VERIFY_FILE" && \
   grep -Fq 'Operational Validation' "$FLOW_VERIFY_FILE"; then
    echo "  PASS: init bootstraps rule-based task sections for how-we-verify"
    ((PASS++)) || true
else
    echo "  FAIL: init bootstraps rule-based task sections for how-we-verify"
    ((FAIL++)) || true
fi

NO_GIT_DIR="$(mktemp -d /tmp/ai-teamlead-init-no-git-XXXXXX)"
NO_GIT_OUTPUT_FILE="$(mktemp /tmp/ai-teamlead-init-no-git-output-XXXXXX)"

if (
    cd "$NO_GIT_DIR"
    "$AI_TEAMLEAD_BIN" init
) >"$NO_GIT_OUTPUT_FILE" 2>&1; then
    echo "  FAIL: init must fail outside git repository"
    ((FAIL++)) || true
else
    echo "  PASS: init fails outside git repository"
    ((PASS++)) || true
fi

if [[ -e "$NO_GIT_DIR/.ai-teamlead/settings.yml" ]]; then
    echo "  FAIL: init must not create files outside git repository"
    ((FAIL++)) || true
else
    echo "  PASS: init does not create files outside git repository"
    ((PASS++)) || true
fi

NO_ORIGIN_REPO="$(mktemp -d /tmp/ai-teamlead-init-no-origin-XXXXXX)"
git init -q "$NO_ORIGIN_REPO"
NO_ORIGIN_OUTPUT_FILE="$(mktemp /tmp/ai-teamlead-init-no-origin-output-XXXXXX)"

if (
    cd "$NO_ORIGIN_REPO"
    "$AI_TEAMLEAD_BIN" init
) >"$NO_ORIGIN_OUTPUT_FILE" 2>&1; then
    echo "  FAIL: init must fail when origin is missing"
    ((FAIL++)) || true
else
    echo "  PASS: init fails when origin is missing"
    ((PASS++)) || true
fi

if [[ -e "$NO_ORIGIN_REPO/.ai-teamlead/settings.yml" ]]; then
    echo "  FAIL: init must not create files when origin is missing"
    ((FAIL++)) || true
else
    echo "  PASS: init does not create files when origin is missing"
    ((PASS++)) || true
fi

EXISTING_INIT_REPO="$(mktemp -d /tmp/ai-teamlead-init-existing-init-XXXXXX)"
git init -q "$EXISTING_INIT_REPO"
git -C "$EXISTING_INIT_REPO" remote add origin git@github.com:dapi/example.git
printf '#!/usr/bin/env bash\necho custom\n' > "$EXISTING_INIT_REPO/init.sh"
chmod +x "$EXISTING_INIT_REPO/init.sh"

(
    cd "$EXISTING_INIT_REPO"
    "$AI_TEAMLEAD_BIN" init >/dev/null
)

if [[ -L "$EXISTING_INIT_REPO/init.sh" ]]; then
    echo "  FAIL: init must not replace existing root init.sh with symlink"
    ((FAIL++)) || true
else
    echo "  PASS: init does not replace existing root init.sh"
    ((PASS++)) || true
fi
