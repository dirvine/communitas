#!/usr/bin/env bash
set -euo pipefail

# Defaults (override via env)
APP_PORT=${APP_PORT:-1420}
MCP_PORT=${MCP_PORT:-8899}
MCP_HOST=${MCP_HOST:-127.0.0.1}
MCP_BROWSER_URL=${MCP_BROWSER_URL:-http://localhost:${APP_PORT}}
MCP_HTTP_URL=${MCP_HTTP_URL:-http://${MCP_HOST}:${MCP_PORT}}
MCP_BROWSER_HEADLESS=1
OFFLINE_TUNNEL=${OFFLINE_TUNNEL:-0}

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)

echo "== Offline Start =="
echo " APP_PORT=${APP_PORT}"
echo " MCP_HOST=${MCP_HOST}"
echo " MCP_PORT=${MCP_PORT}"
echo " MCP_BROWSER_URL=${MCP_BROWSER_URL}"
echo " MCP_HTTP_URL=${MCP_HTTP_URL}"
echo " TUNNEL=${OFFLINE_TUNNEL}"

# Kill any listeners first
"$ROOT_DIR/scripts/kill-ports.sh" "${APP_PORT}" "${MCP_PORT}" >/dev/null || true

# Start Vite
(cd "$ROOT_DIR" && npm run dev:browser -- --port "$APP_PORT" --strictPort >/tmp/communitas-vite.log 2>&1 & echo $! > /tmp/communitas-vite.pid)

for i in {1..60}; do
  if lsof -iTCP:"${APP_PORT}" -sTCP:LISTEN >/dev/null 2>&1; then break; fi
  sleep 0.5
done
echo "Vite listening on :${APP_PORT}"

# Start MCP HTTP server (headless)
(
  cd "$ROOT_DIR" && \
  MCP_BROWSER_URL="$MCP_BROWSER_URL" \
  MCP_HTTP_HOST="$MCP_HOST" \
  MCP_HTTP_PORT="$MCP_PORT" \
  MCP_BROWSER_HEADLESS="$MCP_BROWSER_HEADLESS" \
  node servers/mcp-puppeteer/server-http.js >/tmp/communitas-mcp-http.log 2>&1 & echo $! > /tmp/communitas-mcp-http.pid
)

for i in {1..60}; do
  if lsof -iTCP:"${MCP_PORT}" -sTCP:LISTEN >/dev/null 2>&1; then break; fi
  sleep 0.5
done
echo "MCP HTTP listening on ${MCP_HOST}:${MCP_PORT}"

echo -n "$MCP_HTTP_URL" > /tmp/communitas-mcp-http-url

# Optional tunnel via ngrok
if [ "$OFFLINE_TUNNEL" = "1" ]; then
  if command -v ngrok >/dev/null 2>&1; then
    (ngrok http "$MCP_PORT" >/tmp/communitas-ngrok.log 2>&1 & echo $! > /tmp/communitas-ngrok.pid)
    # Wait a moment for ngrok to start
    sleep 2
    # Query ngrok local API for the https URL
    if command -v curl >/dev/null 2>&1; then
      URL=$(curl -s http://127.0.0.1:4040/api/tunnels | sed -n 's/.*"public_url":"\(https:[^"]*\)".*/\1/p' | head -n1 || true)
      if [ -n "$URL" ]; then
        echo "$URL" | tee /tmp/communitas-mcp-http-url
        echo "Tunnel: $URL"
      else
        echo "Tunnel started, but could not retrieve public URL (check /tmp/communitas-ngrok.log)"
      fi
    fi
  else
    echo "ngrok not found; skipping tunnel"
  fi
fi

echo "\nReady."
echo " App: $MCP_BROWSER_URL"
echo " MCP: $(cat /tmp/communitas-mcp-http-url)"
echo " Logs: /tmp/communitas-vite.log, /tmp/communitas-mcp-http.log"

