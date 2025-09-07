#!/usr/bin/env bash
set -euo pipefail

# Config (override with env)
APP_PORT=${APP_PORT:-1420}
MCP_PORT=${MCP_PORT:-8765}
MCP_HOST=${MCP_HOST:-127.0.0.1}
MCP_BROWSER_URL=${MCP_BROWSER_URL:-http://localhost:${APP_PORT}}
MCP_BROWSER_HEADLESS=${MCP_BROWSER_HEADLESS:-0}

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)

echo "== Killing existing listeners on ${APP_PORT} and ${MCP_PORT} =="
"$ROOT_DIR/scripts/kill-ports.sh" "$APP_PORT" "$MCP_PORT"

echo "== Starting Vite dev server on :${APP_PORT} =="
(cd "$ROOT_DIR" && npm run dev:browser -- --port "$APP_PORT" --strictPort >/tmp/communitas-vite.log 2>&1 & echo $! > /tmp/communitas-vite.pid)

echo "Waiting for :${APP_PORT} to be ready..."
for i in {1..60}; do
  if lsof -iTCP:"${APP_PORT}" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Vite listening on :${APP_PORT}"
    break
  fi
  sleep 0.5
  if [ "$i" -eq 60 ]; then
    echo "Vite failed to bind to :${APP_PORT}. See /tmp/communitas-vite.log" >&2
    exit 1
  fi
done

echo "== Starting MCP HTTP server on ${MCP_HOST}:${MCP_PORT} (headless=${MCP_BROWSER_HEADLESS}) =="
(
  cd "$ROOT_DIR" && \
  MCP_BROWSER_URL="$MCP_BROWSER_URL" \
  MCP_HTTP_HOST="$MCP_HOST" \
  MCP_HTTP_PORT="$MCP_PORT" \
  MCP_BROWSER_HEADLESS="$MCP_BROWSER_HEADLESS" \
  node servers/mcp-playwright/server-http.js >/tmp/communitas-mcp-http.log 2>&1 & echo $! > /tmp/communitas-mcp-http.pid
)

echo "Waiting for MCP HTTP to be ready..."
for i in {1..60}; do
  if lsof -iTCP:"${MCP_PORT}" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "MCP HTTP listening on ${MCP_HOST}:${MCP_PORT}"
    break
  fi
  sleep 0.5
  if [ "$i" -eq 60 ]; then
    echo "MCP HTTP failed to bind to ${MCP_HOST}:${MCP_PORT}. See /tmp/communitas-mcp-http.log" >&2
    exit 1
  fi
done

echo "\nAll set. Open the app at ${MCP_BROWSER_URL}"
echo "MCP HTTP server: http://${MCP_HOST}:${MCP_PORT}"
echo "Logs:"
echo "  Vite: /tmp/communitas-vite.log (PID $(cat /tmp/communitas-vite.pid))"
echo "  MCP:  /tmp/communitas-mcp-http.log (PID $(cat /tmp/communitas-mcp-http.pid))"
echo "\nTo stop both: \n  kill \$(cat /tmp/communitas-mcp-http.pid) ; kill \$(cat /tmp/communitas-vite.pid)"

