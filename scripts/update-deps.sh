#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "Updating Rust toolchain..."
rustup update

echo ""
echo "Updating root crate dependencies..."
cargo update

echo ""
echo "Updating benchmark dependencies..."
cargo update --manifest-path benchmark/Cargo.toml

echo ""
echo "Done."
