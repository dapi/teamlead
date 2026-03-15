#!/usr/bin/env bash
set -euo pipefail

cargo test
cargo run -- test agent-flow --scenario run-happy-path --agent stub --mode stub
cargo run -- test agent-flow --scenario live-codex-smoke --agent codex --mode live
cargo run -- test agent-flow --scenario live-claude-smoke --agent claude --mode live
