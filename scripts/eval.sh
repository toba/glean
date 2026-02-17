#!/usr/bin/env bash
# Run mini eval tasks against committed fixtures.
# Usage: ./scripts/eval.sh [bench eval flags...]
# Examples:
#   ./scripts/eval.sh                                    # all tasks, haiku, baseline+glean
#   ./scripts/eval.sh --tasks eval_ts_class_usage        # single task
#   ./scripts/eval.sh --model sonnet --modes glean_forced # override model & mode

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BENCH="${REPO_ROOT}/benchmark/target/release/bench"

if [[ ! -x "${BENCH}" ]]; then
    echo "Building benchmark crate (release)..."
    cargo build --release --manifest-path "${REPO_ROOT}/benchmark/Cargo.toml"
fi

exec "${BENCH}" eval "$@"
