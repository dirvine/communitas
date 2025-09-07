#!/usr/bin/env bash
set -euo pipefail

# Defaults (override as needed)
APP_PORT=${APP_PORT:-1420}
MCP_PORT=${MCP_PORT:-8765}
MCP_HOST=${MCP_HOST:-127.0.0.1}
MCP_BROWSER_URL=${MCP_BROWSER_URL:-http://localhost:${APP_PORT}}
MCP_HTTP_URL=${MCP_HTTP_URL:-http://${MCP_HOST}:${MCP_PORT}}
MCP_BROWSER_HEADLESS=${MCP_BROWSER_HEADLESS:-1}

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)

echo "== MCP Smoke Test =="
echo " APP_PORT=${APP_PORT}"
echo " MCP_HOST=${MCP_HOST}"
echo " MCP_PORT=${MCP_PORT}"
echo " MCP_BROWSER_URL=${MCP_BROWSER_URL}"
echo " MCP_HTTP_URL=${MCP_HTTP_URL}"
echo " HEADLESS=${MCP_BROWSER_HEADLESS}"

# Clean ports to avoid collisions
"$ROOT_DIR/scripts/kill-ports.sh" "${APP_PORT}" "${MCP_PORT}" >/dev/null || true

# Start services
APP_PORT="${APP_PORT}" MCP_PORT="${MCP_PORT}" MCP_HOST="${MCP_HOST}" MCP_BROWSER_URL="${MCP_BROWSER_URL}" MCP_BROWSER_HEADLESS="${MCP_BROWSER_HEADLESS}" \
  "$ROOT_DIR/scripts/run-browser-and-mcp-http.sh"

echo "\n== Running MCP HTTP demo client =="
set +e
MCP_HTTP_URL="${MCP_HTTP_URL}" MCP_BROWSER_URL="${MCP_BROWSER_URL}" \
  node "$ROOT_DIR/servers/mcp-puppeteer/demo-client-http.js"
status=$?
set -e

echo "\n== Stopping services =="
if [ -f /tmp/communitas-mcp-http.pid ]; then
  kill $(cat /tmp/communitas-mcp-http.pid) 2>/dev/null || true
fi
if [ -f /tmp/communitas-vite.pid ]; then
  kill $(cat /tmp/communitas-vite.pid) 2>/dev/null || true
fi

if [ $status -eq 0 ]; then
  echo "\n✅ Smoke test PASSED"
else
  echo "\n❌ Smoke test FAILED (exit ${status})"
  echo "  - Check logs:"
  echo "    Vite: /tmp/communitas-vite.log"
  echo "    MCP : /tmp/communitas-mcp-http.log"
fi

exit $status

