#!/usr/bin/env bash
set -euo pipefail

detect_default_branch() {
    local remote_head
    remote_head="$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's#refs/remotes/origin/##' || true)"
    if [[ -n "$remote_head" ]]; then
        printf '%s\n' "$remote_head"
        return 0
    fi

    if command -v gh >/dev/null 2>&1; then
        remote_head="$(gh repo view --json defaultBranchRef --jq '.defaultBranchRef.name' 2>/dev/null || true)"
        if [[ -n "$remote_head" ]]; then
            printf '%s\n' "$remote_head"
            return 0
        fi
    fi

    remote_head="$(git remote show -n origin 2>/dev/null | sed -n 's/.*HEAD branch: //p' | head -n 1 || true)"
    if [[ -n "$remote_head" && "$remote_head" != "(unknown)" && "$remote_head" != "(not queried)" ]]; then
        printf '%s\n' "$remote_head"
        return 0
    fi

    if git show-ref --verify --quiet refs/heads/main; then
        printf 'main\n'
        return 0
    fi

    if git show-ref --verify --quiet refs/heads/master; then
        printf 'master\n'
        return 0
    fi

    git rev-parse --abbrev-ref HEAD
}

find_primary_worktree() {
    local git_common_dir
    git_common_dir="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null || true)"
    if [[ -z "$git_common_dir" ]]; then
        return 1
    fi

    (
        cd "$git_common_dir/.." >/dev/null 2>&1 && pwd -P
    )
}

copy_env_files_from_primary_worktree() {
    local current_dir primary_worktree default_branch primary_branch source_env target_env
    current_dir="$(pwd -P)"
    primary_worktree="$(find_primary_worktree || true)"

    if [[ -z "$primary_worktree" ]]; then
        return 0
    fi

    if [[ "$primary_worktree" == "$current_dir" ]]; then
        return 0
    fi

    default_branch="$(detect_default_branch)"
    primary_branch="$(git -C "$primary_worktree" rev-parse --abbrev-ref HEAD 2>/dev/null || true)"

    if [[ -n "$default_branch" && -n "$primary_branch" && "$primary_branch" != "$default_branch" ]]; then
        printf 'init.sh: skipped .env* copy because primary worktree is on %s, expected %s\n' "$primary_branch" "$default_branch" >&2
        return 0
    fi

    shopt -s nullglob dotglob
    for source_env in "$primary_worktree"/.env*; do
        if [[ ! -f "$source_env" ]]; then
            continue
        fi

        target_env="./$(basename "$source_env")"
        if [[ -e "$target_env" ]]; then
            continue
        fi

        cp "$source_env" "$target_env"
        printf 'init.sh: copied %s from %s\n' "$(basename "$source_env")" "$primary_worktree"
    done
    shopt -u nullglob dotglob
}

if [[ -f "mise.toml" ]] || [[ -f ".mise.toml" ]]; then
    if command -v mise >/dev/null 2>&1; then
        mise trust >/dev/null 2>&1 || true
        mise install
    else
        printf 'init.sh: skipped mise setup because mise is not installed\n' >&2
    fi
fi

if [[ -f ".gitmodules" ]]; then
    git submodule update --init --recursive
fi

copy_env_files_from_primary_worktree

if [[ -f ".envrc" ]] && command -v direnv >/dev/null 2>&1; then
    direnv allow
fi
