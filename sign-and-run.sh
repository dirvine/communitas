#!/bin/bash
# Auto-sign the binary with entitlements after Tauri compiles it

cd "$(dirname "$0")"

# Run Tauri dev in background
npm run tauri dev 2>&1 | tee /tmp/tauri-signed-dev.log &
TAURI_PID=$!

# Wait for compilation
echo "Waiting for compilation..."
sleep 15

# Sign the binary
echo "Signing binary with entitlements..."
codesign --sign - \
  --entitlements communitas-desktop/Communitas.entitlements \
  --force \
  target/debug/communitas-desktop

echo "Binary signed. App should restart automatically."
echo "Logs: /tmp/tauri-signed-dev.log"

# Keep script running
wait $TAURI_PID
