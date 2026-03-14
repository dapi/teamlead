#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

cleanup_zellij
trap cleanup_zellij EXIT

for test_script in "$SCRIPT_DIR"/test_*.sh; do
    cleanup_zellij
    echo ""
    echo "--- Running: $(basename "$test_script") ---"
    source "$test_script"
    cleanup_zellij
done

print_summary
