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
FLOW_FILE="$REPO_ROOT/.ai-teamlead/flows/issue-analysis-flow.md"
RUNTIME_DIR="$REPO_ROOT/.git/.ai-teamlead"

assert_file_exists "$SETTINGS_FILE" "init created settings.yml"
assert_file_exists "$README_FILE" "init created .ai-teamlead README"
assert_file_exists "$FLOW_FILE" "init created issue-analysis-flow.md"

if [[ -d "$RUNTIME_DIR" ]]; then
    echo "  FAIL: init must not create runtime directory"
    ((FAIL++)) || true
else
    echo "  PASS: init does not create runtime directory"
    ((PASS++)) || true
fi

if [[ "$OUTPUT" == *"created: $SETTINGS_FILE"* ]] && [[ "$OUTPUT" == *"created: $README_FILE"* ]] && [[ "$OUTPUT" == *"created: $FLOW_FILE"* ]]; then
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

if [[ "$SECOND_OUTPUT" == *"skipped: $SETTINGS_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $README_FILE"* ]] && [[ "$SECOND_OUTPUT" == *"skipped: $FLOW_FILE"* ]]; then
    echo "  PASS: init is idempotent"
    ((PASS++)) || true
else
    echo "  FAIL: init is idempotent"
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
