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

ARTIFACT_DIR="${TMP_ROOT}/artifacts"
mkdir -p "${ARTIFACT_DIR}"

# --- Entity parity check ---
echo "[mcp-parity] comparing entity lists..."

ENTITY_REQ='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_entities","arguments":{}}}'
MCP_RESPONSE=$(post "${ENTITY_REQ}")
MCP_ENTITIES=$(echo "${MCP_RESPONSE}" | jq -r '.result.content[0].text' | jq -S '.entities // []')

echo "${MCP_RESPONSE}" > "${ARTIFACT_DIR}/mcp_entities_raw.json"
echo "${MCP_ENTITIES}" > "${ARTIFACT_DIR}/mcp_entities.json"

echo "[mcp-parity] exporting snapshot via Communitas Core helper"
CLI_SNAPSHOT=$(cargo run -q -p communitas-core --bin export_directory -- "${FOUR_WORDS}" "${DISPLAY_NAME}" "${STORAGE_DIR}" "parity-cli")
CLI_ENTITIES=$(echo "${CLI_SNAPSHOT}" | jq -S '.entities // []')

echo "${CLI_SNAPSHOT}" > "${ARTIFACT_DIR}/cli_snapshot.json"
echo "${CLI_ENTITIES}" > "${ARTIFACT_DIR}/cli_entities.json"

ENTITY_PARITY_PASS=true
if [[ "${CLI_ENTITIES}" != "${MCP_ENTITIES}" ]]; then
  echo "[mcp-parity] entity snapshots diverge!"
  diff <(echo "${CLI_ENTITIES}") <(echo "${MCP_ENTITIES}") > "${ARTIFACT_DIR}/entity_diff.txt" || true
  ENTITY_PARITY_PASS=false
else
  echo "[mcp-parity] entity list parity verified"
fi

# --- Contact parity check ---
echo "[mcp-parity] comparing contact lists..."

CONTACT_REQ='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_contacts","arguments":{}}}'
MCP_CONTACT_RESPONSE=$(post "${CONTACT_REQ}")
MCP_CONTACTS=$(echo "${MCP_CONTACT_RESPONSE}" | jq -r '.result.content[0].text' | jq -S '.contacts // []')

echo "${MCP_CONTACT_RESPONSE}" > "${ARTIFACT_DIR}/mcp_contacts_raw.json"
echo "${MCP_CONTACTS}" > "${ARTIFACT_DIR}/mcp_contacts.json"

CLI_CONTACTS=$(echo "${CLI_SNAPSHOT}" | jq -S '.contacts // []')
echo "${CLI_CONTACTS}" > "${ARTIFACT_DIR}/cli_contacts.json"

CONTACT_PARITY_PASS=true
if [[ "${CLI_CONTACTS}" != "${MCP_CONTACTS}" ]]; then
  echo "[mcp-parity] contact snapshots diverge!"
  diff <(echo "${CLI_CONTACTS}") <(echo "${MCP_CONTACTS}") > "${ARTIFACT_DIR}/contact_diff.txt" || true
  CONTACT_PARITY_PASS=false
else
  echo "[mcp-parity] contact list parity verified"
fi

# --- Summary ---
echo "[mcp-parity] artifacts saved to: ${ARTIFACT_DIR}"

# Copy artifacts to CI-accessible location if running in CI
if [[ -n "${CI:-}" ]]; then
  PARITY_ARTIFACTS="${GITHUB_WORKSPACE:-/tmp}/parity-artifacts"
  mkdir -p "${PARITY_ARTIFACTS}"
  cp -r "${ARTIFACT_DIR}"/* "${PARITY_ARTIFACTS}/" 2>/dev/null || true
  echo "[mcp-parity] CI artifacts copied to: ${PARITY_ARTIFACTS}"
fi

# Exit with error if either parity check failed
if [[ "${ENTITY_PARITY_PASS}" != "true" || "${CONTACT_PARITY_PASS}" != "true" ]]; then
  echo "[mcp-parity] FAILED: parity checks did not pass"
  exit 1
fi

echo "[mcp-parity] ALL PARITY CHECKS PASSED"
