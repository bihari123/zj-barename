#!/usr/bin/env bash
# Build the plugin to wasm and (optionally) install it into your Zellij plugins dir.
set -euo pipefail

TARGET="wasm32-wasip1"
OUT="target/${TARGET}/release/tab-bar.wasm"
INSTALL_DIR="${HOME}/.config/zellij/plugins"

cd "$(dirname "$0")"

# Ensure the wasm target is available (no-op if already installed).
rustup target add "${TARGET}" >/dev/null 2>&1 || true

echo "Building ${OUT} ..."
cargo build --release --target "${TARGET}"

echo "Built: ${OUT} ($(stat -c%s "${OUT}") bytes)"

if [[ "${1:-}" == "--install" ]]; then
    mkdir -p "${INSTALL_DIR}"
    cp "${OUT}" "${INSTALL_DIR}/tab-bar.wasm"
    echo "Installed to ${INSTALL_DIR}/tab-bar.wasm"
    echo "Start a new Zellij session to see it (existing sessions keep the old bar)."
fi
