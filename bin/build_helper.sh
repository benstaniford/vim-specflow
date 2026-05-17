#!/usr/bin/env bash
# Build the specflow-helper binary used by the Vim plugin.
#
# Run once after cloning the plugin (and after pulling updates to the Rust
# crate). The resulting binary lands at <plugin>/bin/specflow-helper, which
# is where the VimScript shim looks for it by default.

set -euo pipefail

PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_DIR="$PLUGIN_DIR/rust/specflow-helper"
OUT_BIN="$PLUGIN_DIR/bin/specflow-helper"

if ! command -v cargo >/dev/null 2>&1; then
    echo "specflow-helper: cargo not found in PATH" >&2
    echo "  install Rust from https://rustup.rs and re-run this script" >&2
    exit 1
fi

echo "specflow-helper: building (cargo build --release)…"
( cd "$CRATE_DIR" && cargo build --release )

cp "$CRATE_DIR/target/release/specflow-helper" "$OUT_BIN"
chmod +x "$OUT_BIN"

echo "specflow-helper: installed at $OUT_BIN"
"$OUT_BIN" --version
