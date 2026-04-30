#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Formatting check"
cargo fmt --manifest-path "$ROOT_DIR/Cargo.toml" --check

echo "==> Tests"
cargo test --manifest-path "$ROOT_DIR/Cargo.toml"

echo "==> Clippy"
cargo clippy --manifest-path "$ROOT_DIR/Cargo.toml" -- -D warnings

echo "==> Release build"
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" --release

echo "==> Check complete"
