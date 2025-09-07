#!/usr/bin/env bash
set -euo pipefail

echo "== Offline Stop =="
if [ -f /tmp/communitas-mcp-http.pid ]; then
  kill $(cat /tmp/communitas-mcp-http.pid) 2>/dev/null || true
  rm -f /tmp/communitas-mcp-http.pid
  echo "Stopped MCP HTTP server"
fi
if [ -f /tmp/communitas-vite.pid ]; then
  kill $(cat /tmp/communitas-vite.pid) 2>/dev/null || true
  rm -f /tmp/communitas-vite.pid
  echo "Stopped Vite dev server"
fi
if [ -f /tmp/communitas-ngrok.pid ]; then
  kill $(cat /tmp/communitas-ngrok.pid) 2>/dev/null || true
  rm -f /tmp/communitas-ngrok.pid
  echo "Stopped ngrok tunnel"
fi
rm -f /tmp/communitas-mcp-http-url
echo "Done."

