#!/usr/bin/env bash
set -euo pipefail

SCRIPT_TEMPLATE="/test/bin/templates/init/init.sh"

PRIMARY_REPO="$(mktemp -d /tmp/ai-teamlead-project-init-primary-XXXXXX)"
git init -q -b main "$PRIMARY_REPO"
git -C "$PRIMARY_REPO" remote add origin git@github.com:dapi/example.git
git -C "$PRIMARY_REPO" config user.name "AI Teamlead Test"
git -C "$PRIMARY_REPO" config user.email "ai-teamlead@example.com"
printf 'root\n' > "$PRIMARY_REPO/README.md"
git -C "$PRIMARY_REPO" add README.md
git -C "$PRIMARY_REPO" commit -q -m "initial"
printf 'DATABASE_URL=postgres://primary\n' > "$PRIMARY_REPO/.env"
printf 'API_TOKEN=primary-token\n' > "$PRIMARY_REPO/.env.local"

FEATURE_WORKTREE="$(mktemp -d /tmp/ai-teamlead-project-init-feature-XXXXXX)"
rmdir "$FEATURE_WORKTREE"
git -C "$PRIMARY_REPO" worktree add -q -b feature/test-env-copy "$FEATURE_WORKTREE"

FEATURE_OUTPUT="$(
    cd "$FEATURE_WORKTREE"
    bash "$SCRIPT_TEMPLATE"
)"

assert_file_exists "$FEATURE_WORKTREE/.env" "project init copied .env from primary worktree"
assert_file_exists "$FEATURE_WORKTREE/.env.local" "project init copied .env.local from primary worktree"
assert_eq "$(cat "$FEATURE_WORKTREE/.env")" "DATABASE_URL=postgres://primary" "copied .env content matches primary worktree"
assert_eq "$(cat "$FEATURE_WORKTREE/.env.local")" "API_TOKEN=primary-token" "copied .env.local content matches primary worktree"

if [[ "$FEATURE_OUTPUT" == *"copied .env from $PRIMARY_REPO"* ]] && [[ "$FEATURE_OUTPUT" == *"copied .env.local from $PRIMARY_REPO"* ]]; then
    echo "  PASS: project init reports copied env files"
    ((PASS++)) || true
else
    echo "  FAIL: project init reports copied env files"
    ((FAIL++)) || true
fi

printf 'API_TOKEN=custom-feature\n' > "$FEATURE_WORKTREE/.env.local"
(
    cd "$FEATURE_WORKTREE"
    bash "$SCRIPT_TEMPLATE" >/dev/null
)
assert_eq "$(cat "$FEATURE_WORKTREE/.env.local")" "API_TOKEN=custom-feature" "project init does not overwrite existing env files"

MISALIGNED_PRIMARY="$(mktemp -d /tmp/ai-teamlead-project-init-misaligned-XXXXXX)"
git init -q -b main "$MISALIGNED_PRIMARY"
git -C "$MISALIGNED_PRIMARY" remote add origin git@github.com:dapi/example.git
git -C "$MISALIGNED_PRIMARY" config user.name "AI Teamlead Test"
git -C "$MISALIGNED_PRIMARY" config user.email "ai-teamlead@example.com"
printf 'root\n' > "$MISALIGNED_PRIMARY/README.md"
git -C "$MISALIGNED_PRIMARY" add README.md
git -C "$MISALIGNED_PRIMARY" commit -q -m "initial"
printf 'SECRET=main\n' > "$MISALIGNED_PRIMARY/.env.secret"

MISALIGNED_WORKTREE="$(mktemp -d /tmp/ai-teamlead-project-init-misaligned-feature-XXXXXX)"
rmdir "$MISALIGNED_WORKTREE"
git -C "$MISALIGNED_PRIMARY" worktree add -q -b feature/misaligned "$MISALIGNED_WORKTREE"
git -C "$MISALIGNED_PRIMARY" switch -q -c scratch

MISALIGNED_OUTPUT="$(
    cd "$MISALIGNED_WORKTREE"
    bash "$SCRIPT_TEMPLATE" 2>&1
)"

if [[ -e "$MISALIGNED_WORKTREE/.env.secret" ]]; then
    echo "  FAIL: project init must skip env copy when primary worktree branch is not default"
    ((FAIL++)) || true
else
    echo "  PASS: project init skips env copy when primary worktree branch is not default"
    ((PASS++)) || true
fi

if [[ "$MISALIGNED_OUTPUT" == *"skipped .env* copy because primary worktree is on scratch, expected main"* ]]; then
    echo "  PASS: project init explains skipped env copy for misaligned primary worktree"
    ((PASS++)) || true
else
    echo "  FAIL: project init explains skipped env copy for misaligned primary worktree"
    ((FAIL++)) || true
fi
