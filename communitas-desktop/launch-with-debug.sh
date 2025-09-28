#!/bin/bash

# Launch Tauri app with Chrome DevTools Protocol enabled for testing
# This enables Playwright and Chrome DevTools MCP to connect

echo "Starting Communitas with remote debugging enabled..."
echo "Remote debugging port: 9222"
echo "MCP socket will be at: /tmp/tauri-mcp-communitas-$$.sock"

# Set environment variables for debugging
export RUST_LOG=info,communitas=debug,tauri_plugin_mcp=debug
export WEBKIT_INSPECTOR_SERVER=127.0.0.1:9222  # For Linux/WebKit
export TAURI_WEBVIEW_REMOTE_DEBUGGING_PORT=9222  # For Windows/Edge

# Launch Tauri in development mode with remote debugging
cd "$(dirname "$0")"
npm run tauri dev -- --remote-debugging-port=9222