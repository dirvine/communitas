#!/usr/bin/env bash
set -euo pipefail

APP_PORT=${APP_PORT:-1420}
MCP_PORT=${MCP_PORT:-8899}
MCP_BROWSER_URL=${MCP_BROWSER_URL:-http://localhost:${APP_PORT}}

if [ -f /tmp/communitas-mcp-http-url ]; then
  MCP_HTTP_URL=$(cat /tmp/communitas-mcp-http-url)
else
  MCP_HTTP_URL=${MCP_HTTP_URL:-http://127.0.0.1:${MCP_PORT}}
fi

echo "== Offline Test =="
echo " MCP_HTTP_URL=${MCP_HTTP_URL}"
echo " MCP_BROWSER_URL=${MCP_BROWSER_URL}"

# Small grace period in case the server just bound
sleep 0.5

attempts=${ATTEMPTS:-3}
delay=${DELAY_SECS:-1}
ok=0
for i in $(seq 1 "$attempts"); do
  echo "Attempt $i/${attempts}..."
  set +e
  MCP_HTTP_URL="$MCP_HTTP_URL" MCP_BROWSER_URL="$MCP_BROWSER_URL" \
    node servers/mcp-puppeteer/full-check.js
  status=$?
  set -e
  if [ $status -eq 0 ]; then
    ok=1; break
  fi
  echo "Retrying in ${delay}s..."
  sleep "$delay"
done

if [ $ok -ne 1 ]; then
  echo "Offline test failed after ${attempts} attempts. See logs: /tmp/communitas-mcp-http.log /tmp/communitas-vite.log" >&2
  exit 1
fi
