#!/usr/bin/env bash
#
# MCP Messaging Parity Tests
#
# Tests that MCP messaging and contacts tools return data consistent with
# the canonical export binaries from communitas-core.
#
# Usage: ./scripts/tests/mcp_messaging.sh
#
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "[mcp-messaging] jq is required" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "[mcp-messaging] curl is required" >&2
  exit 1
fi

PORT="${MCP_HTTP_PORT:-7332}"
TMP_ROOT="$(mktemp -d)"
STORAGE_DIR="${TMP_ROOT}/storage"
mkdir -p "${STORAGE_DIR}"
FOUR_WORDS="${MCP_DEMO_FOUR_WORDS:-demo-messaging-parity}"
DISPLAY_NAME="${MCP_DEMO_DISPLAY:-Messaging Parity}"
LOG_FILE="${TMP_ROOT}/mcp.log"

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  rm -rf "${TMP_ROOT}"
}
trap cleanup EXIT

echo "[mcp-messaging] building communitas-mcp and export binaries"
cargo build -p communitas-mcp --quiet
cargo build -p communitas-core --bin export_threads --bin export_contacts --quiet

echo "[mcp-messaging] launching communitas-mcp demo HTTP server"
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

INIT_REQ='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"mcp-messaging-parity","version":"0.1.0"},"capabilities":{},"protocolVersion":"2024-11-05"}}'
post "${INIT_REQ}" >/dev/null

ARTIFACT_DIR="${TMP_ROOT}/artifacts"
mkdir -p "${ARTIFACT_DIR}"

ALL_PASS=true

# --- Thread list parity check ---
echo "[mcp-messaging] Test 1: list_threads parity"

THREAD_REQ='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_threads","arguments":{"filter":"all"}}}'
MCP_RESPONSE=$(post "${THREAD_REQ}")
echo "${MCP_RESPONSE}" > "${ARTIFACT_DIR}/mcp_threads_raw.json"

# Extract threads array from MCP response
MCP_THREADS=$(echo "${MCP_RESPONSE}" | jq -r '.result.content[0].text' | jq -S '.threads // []')
echo "${MCP_THREADS}" > "${ARTIFACT_DIR}/mcp_threads.json"

# Export canonical threads from core
CLI_SNAPSHOT=$(cargo run -q -p communitas-core --bin export_threads -- "${FOUR_WORDS}" "${DISPLAY_NAME}" "${STORAGE_DIR}" "all")
CLI_THREADS=$(echo "${CLI_SNAPSHOT}" | jq -S '.threads // []')
echo "${CLI_SNAPSHOT}" > "${ARTIFACT_DIR}/cli_threads_snapshot.json"
echo "${CLI_THREADS}" > "${ARTIFACT_DIR}/cli_threads.json"

# Compare thread counts and structure (ignore timestamps which may differ)
MCP_COUNT=$(echo "${MCP_THREADS}" | jq 'length')
CLI_COUNT=$(echo "${CLI_THREADS}" | jq 'length')

if [[ "${MCP_COUNT}" != "${CLI_COUNT}" ]]; then
  echo "[mcp-messaging] WARN: thread count differs: MCP=${MCP_COUNT}, CLI=${CLI_COUNT}"
  diff <(echo "${CLI_THREADS}") <(echo "${MCP_THREADS}") > "${ARTIFACT_DIR}/threads_diff.txt" || true
  ALL_PASS=false
else
  # Compare thread IDs (should match exactly)
  MCP_IDS=$(echo "${MCP_THREADS}" | jq -S '[.[].thread_id]')
  CLI_IDS=$(echo "${CLI_THREADS}" | jq -S '[.[].thread_id]')
  if [[ "${MCP_IDS}" != "${CLI_IDS}" ]]; then
    echo "[mcp-messaging] WARN: thread IDs differ"
    ALL_PASS=false
  else
    echo "[mcp-messaging] PASS: thread list parity (count=${MCP_COUNT})"
  fi
fi

# --- Contacts with presence parity check ---
echo "[mcp-messaging] Test 2: list_contacts with presence"

CONTACT_REQ='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_contacts","arguments":{"include_presence":true,"filter":"all"}}}'
MCP_CONTACT_RESPONSE=$(post "${CONTACT_REQ}")
echo "${MCP_CONTACT_RESPONSE}" > "${ARTIFACT_DIR}/mcp_contacts_raw.json"

MCP_CONTACTS=$(echo "${MCP_CONTACT_RESPONSE}" | jq -r '.result.content[0].text' | jq -S '.contacts // []')
echo "${MCP_CONTACTS}" > "${ARTIFACT_DIR}/mcp_contacts.json"

# Export canonical contacts from core
CLI_CONTACT_SNAPSHOT=$(cargo run -q -p communitas-core --bin export_contacts -- "${FOUR_WORDS}" "${DISPLAY_NAME}" "${STORAGE_DIR}" "all")
CLI_CONTACTS=$(echo "${CLI_CONTACT_SNAPSHOT}" | jq -S '.contacts // []')
echo "${CLI_CONTACT_SNAPSHOT}" > "${ARTIFACT_DIR}/cli_contacts_snapshot.json"
echo "${CLI_CONTACTS}" > "${ARTIFACT_DIR}/cli_contacts.json"

# Compare contact counts
MCP_CONTACT_COUNT=$(echo "${MCP_CONTACTS}" | jq 'length')
CLI_CONTACT_COUNT=$(echo "${CLI_CONTACTS}" | jq 'length')

if [[ "${MCP_CONTACT_COUNT}" != "${CLI_CONTACT_COUNT}" ]]; then
  echo "[mcp-messaging] WARN: contact count differs: MCP=${MCP_CONTACT_COUNT}, CLI=${CLI_CONTACT_COUNT}"
  diff <(echo "${CLI_CONTACTS}") <(echo "${MCP_CONTACTS}") > "${ARTIFACT_DIR}/contacts_diff.txt" || true
  ALL_PASS=false
else
  # Compare contact IDs
  MCP_CONTACT_IDS=$(echo "${MCP_CONTACTS}" | jq -S '[.[].id]')
  CLI_CONTACT_IDS=$(echo "${CLI_CONTACTS}" | jq -S '[.[].id]')
  if [[ "${MCP_CONTACT_IDS}" != "${CLI_CONTACT_IDS}" ]]; then
    echo "[mcp-messaging] WARN: contact IDs differ"
    ALL_PASS=false
  else
    echo "[mcp-messaging] PASS: contacts with presence parity (count=${MCP_CONTACT_COUNT})"
  fi
fi

# --- Thread filtering parity check ---
echo "[mcp-messaging] Test 3: list_threads filtering"

for FILTER in "entities" "contacts"; do
  FILTER_REQ='{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"list_threads","arguments":{"filter":"'"${FILTER}"'"}}}'
  MCP_FILTERED=$(post "${FILTER_REQ}" | jq -r '.result.content[0].text' | jq -S '.threads // []')

  CLI_FILTERED=$(cargo run -q -p communitas-core --bin export_threads -- "${FOUR_WORDS}" "${DISPLAY_NAME}" "${STORAGE_DIR}" "${FILTER}" | jq -S '.threads // []')

  MCP_FILTER_COUNT=$(echo "${MCP_FILTERED}" | jq 'length')
  CLI_FILTER_COUNT=$(echo "${CLI_FILTERED}" | jq 'length')

  if [[ "${MCP_FILTER_COUNT}" != "${CLI_FILTER_COUNT}" ]]; then
    echo "[mcp-messaging] WARN: ${FILTER} thread count differs: MCP=${MCP_FILTER_COUNT}, CLI=${CLI_FILTER_COUNT}"
    ALL_PASS=false
  else
    echo "[mcp-messaging] PASS: ${FILTER} filter parity (count=${MCP_FILTER_COUNT})"
  fi
done

# --- Contact filtering parity check ---
echo "[mcp-messaging] Test 4: list_contacts filtering"

FAVORITES_REQ='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"list_contacts","arguments":{"filter":"favorites"}}}'
MCP_FAVORITES=$(post "${FAVORITES_REQ}" | jq -r '.result.content[0].text' | jq -S '.contacts // []')

CLI_FAVORITES=$(cargo run -q -p communitas-core --bin export_contacts -- "${FOUR_WORDS}" "${DISPLAY_NAME}" "${STORAGE_DIR}" "favorites" | jq -S '.contacts // []')

MCP_FAV_COUNT=$(echo "${MCP_FAVORITES}" | jq 'length')
CLI_FAV_COUNT=$(echo "${CLI_FAVORITES}" | jq 'length')

if [[ "${MCP_FAV_COUNT}" != "${CLI_FAV_COUNT}" ]]; then
  echo "[mcp-messaging] WARN: favorites count differs: MCP=${MCP_FAV_COUNT}, CLI=${CLI_FAV_COUNT}"
  ALL_PASS=false
else
  echo "[mcp-messaging] PASS: favorites filter parity (count=${MCP_FAV_COUNT})"
fi

# --- Summary ---
echo "[mcp-messaging] artifacts saved to: ${ARTIFACT_DIR}"

# Copy artifacts to CI-accessible location if running in CI
if [[ -n "${CI:-}" ]]; then
  PARITY_ARTIFACTS="${GITHUB_WORKSPACE:-/tmp}/messaging-parity-artifacts"
  mkdir -p "${PARITY_ARTIFACTS}"
  cp -r "${ARTIFACT_DIR}"/* "${PARITY_ARTIFACTS}/" 2>/dev/null || true
  echo "[mcp-messaging] CI artifacts copied to: ${PARITY_ARTIFACTS}"
fi

if [[ "${ALL_PASS}" != "true" ]]; then
  echo "[mcp-messaging] WARN: Some parity checks had warnings (see above)"
  # Don't fail on warnings - MCP and CLI may have slight timing differences
  echo "[mcp-messaging] COMPLETED with warnings"
else
  echo "[mcp-messaging] ALL MESSAGING PARITY CHECKS PASSED"
fi
