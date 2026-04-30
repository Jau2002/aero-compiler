#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Checking Rust toolchain"
cargo --version >/dev/null
rustc --version >/dev/null

echo "==> Building project"
cargo build --manifest-path "$ROOT_DIR/Cargo.toml"

echo "==> Running tests"
cargo test --manifest-path "$ROOT_DIR/Cargo.toml"

echo "==> Bootstrap complete"
