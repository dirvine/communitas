#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_BINARY="${APP_BINARY:-$ROOT_DIR/target/debug/communitas-dioxus}"
DRIVER_PORT="${TAURI_DRIVER_PORT:-4444}"
NATIVE_PORT="${TAURI_DRIVER_NATIVE_PORT:-4445}"

OS="$(uname -s)"

# Platform-specific setup
if [[ "$OS" == "Darwin" ]]; then
  echo "[tauri-driver] macOS detected - using SafariDriver backend"
  # Ensure safaridriver is enabled (requires sudo in CI, already done in workflow)
  if ! pgrep -x "safaridriver" > /dev/null 2>&1; then
    # Try to enable if not already (will fail silently if no sudo)
    sudo safaridriver --enable 2>/dev/null || true
  fi
elif [[ "$OS" == "Linux" ]]; then
  echo "[tauri-driver] Linux detected - using WebKitGTK driver"
else
  echo "[tauri-driver] Unsupported platform: $OS" >&2
  exit 1
fi

echo "[tauri-driver] building Communitas Dioxus desktop binary"
cargo build -p communitas-dioxus --quiet

pushd "$ROOT_DIR/tests/webdriverio" >/dev/null
if [[ ! -d node_modules ]]; then
  echo "[tauri-driver] installing WebdriverIO harness dependencies"
  npm install >/dev/null
fi

echo "[tauri-driver] launching smoke test via WebdriverIO"
TAURI_APP_BINARY="$APP_BINARY" \
TAURI_DRIVER_PORT="$DRIVER_PORT" \
TAURI_DRIVER_NATIVE_PORT="$NATIVE_PORT" \
npx wdio run wdio.conf.js
popd >/dev/null
