#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" == "Darwin" ]]; then
  echo "[tauri-driver] Skipping: upstream driver does not support macOS yet (requires Linux or Windows)." >&2
  exit 0
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_BINARY="${APP_BINARY:-$ROOT_DIR/target/debug/communitas-dioxus}"
DRIVER_PORT="${TAURI_DRIVER_PORT:-4444}"
NATIVE_PORT="${TAURI_DRIVER_NATIVE_PORT:-4445}"

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
