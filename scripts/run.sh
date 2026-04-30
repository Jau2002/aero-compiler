#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

RULES_FILE="${1:-$ROOT_DIR/restricciones.aero}"
DATA_DIR="${2:-$ROOT_DIR/data}"
OUTPUT_FORMAT="${3:-text}"

cargo run --manifest-path "$ROOT_DIR/Cargo.toml" -- validate --rules "$RULES_FILE" --data "$DATA_DIR" --output "$OUTPUT_FORMAT"
