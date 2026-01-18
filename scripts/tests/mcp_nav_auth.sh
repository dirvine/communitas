#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "[mcp-parity] jq is required" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "[mcp-parity] curl is required" >&2
  exit 1
fi

PORT="${MCP_HTTP_PORT:-7331}"
TMP_ROOT="$(mktemp -d)"
STORAGE_DIR="${TMP_ROOT}/storage"
mkdir -p "${STORAGE_DIR}"
FOUR_WORDS="${MCP_DEMO_FOUR_WORDS:-demo-parity-harness-node}"
DISPLAY_NAME="${MCP_DEMO_DISPLAY:-Parity Harness}"
LOG_FILE="${TMP_ROOT}/mcp.log"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  rm -rf "${TMP_ROOT}"
}
trap cleanup EXIT

echo "[mcp-parity] building communitas-mcp"
cargo build -p communitas-mcp --quiet

echo "[mcp-parity] launching communitas-mcp demo HTTP server"
target/debug/communitas-mcp \
  --demo \
  --http \
  --listen "127.0.0.1:${PORT}" \
  --storage-dir "${STORAGE_DIR}" \
  --four-words "${FOUR_WORDS}" \
  --display-name "${DISPLAY_NAME}" \
  >"${LOG_FILE}" 2>&1 &
SERVER_PID=$!

sleep 3

post() {
  local payload="$1"
  curl -sS -H 'Content-Type: application/json' -X POST "http://127.0.0.1:${PORT}/mcp" -d "${payload}"
}

INIT_REQ='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"mcp-parity-harness","version":"0.1.0"},"capabilities":{},"protocolVersion":"2024-11-05"}}'
post "${INIT_REQ}" >/dev/null

ENTITY_REQ='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_entities","arguments":{}}}'
MCP_RESPONSE=$(post "${ENTITY_REQ}")
MCP_ENTITIES=$(echo "${MCP_RESPONSE}" | jq -r '.result.content[0].text' | jq -S '.entities // []')

echo "[mcp-parity] exporting snapshot via Communitas Core helper"
CLI_SNAPSHOT=$(cargo run -q -p communitas-core --bin export_directory -- "${FOUR_WORDS}" "${DISPLAY_NAME}" "${STORAGE_DIR}" "parity-cli")
CLI_ENTITIES=$(echo "${CLI_SNAPSHOT}" | jq -S '.entities // []')

if [[ "${CLI_ENTITIES}" != "${MCP_ENTITIES}" ]]; then
  echo "[mcp-parity] entity snapshots diverge!"
  diff <(echo "${CLI_ENTITIES}") <(echo "${MCP_ENTITIES}") || true
  exit 1
fi

echo "[mcp-parity] entity list parity verified"
