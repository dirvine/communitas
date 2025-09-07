#!/usr/bin/env bash
set -euo pipefail

MCP_HTTP_URL=${MCP_HTTP_URL:-http://127.0.0.1:8765}
MCP_BROWSER_URL=${MCP_BROWSER_URL:-http://localhost:1420}

echo "Using MCP_HTTP_URL=${MCP_HTTP_URL}"
echo "Using MCP_BROWSER_URL=${MCP_BROWSER_URL}"

node servers/mcp-puppeteer/demo-client-http.js

