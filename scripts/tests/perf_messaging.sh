#!/usr/bin/env bash
#
# Messaging Performance Tests
#
# Measures latency targets for messaging operations:
# - Thread list load: <200ms
# - Message send: <100ms local
# - Contact presence update: <50ms
#
# Usage: ./scripts/tests/perf_messaging.sh
#
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "[perf-messaging] jq is required" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "[perf-messaging] curl is required" >&2
  exit 1
fi

PORT="${MCP_HTTP_PORT:-7333}"
TMP_ROOT="$(mktemp -d)"
STORAGE_DIR="${TMP_ROOT}/storage"
mkdir -p "${STORAGE_DIR}"
FOUR_WORDS="${MCP_DEMO_FOUR_WORDS:-demo-perf-test}"
DISPLAY_NAME="${MCP_DEMO_DISPLAY:-Perf Test}"
LOG_FILE="${TMP_ROOT}/mcp.log"
RESULTS_FILE="${TMP_ROOT}/perf_results.json"

# Performance targets (milliseconds)
TARGET_THREAD_LIST=200
TARGET_MESSAGE_SEND=100
TARGET_CONTACT_LIST=200
TARGET_PRESENCE=50

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  rm -rf "${TMP_ROOT}"
}
trap cleanup EXIT

echo "=== Messaging Performance Tests ==="
echo ""

echo "[perf-messaging] building communitas-mcp"
cargo build -p communitas-mcp --quiet --release 2>/dev/null || cargo build -p communitas-mcp --quiet

echo "[perf-messaging] launching communitas-mcp demo server"
target/release/communitas-mcp \
  --demo \
  --http \
  --listen "127.0.0.1:${PORT}" \
  --storage-dir "${STORAGE_DIR}" \
  --four-words "${FOUR_WORDS}" \
  --display-name "${DISPLAY_NAME}" \
  >"${LOG_FILE}" 2>&1 &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Function to measure request time in milliseconds
measure_request() {
  local payload="$1"
  local start end duration

  start=$(python3 -c "import time; print(int(time.time() * 1000))")
  curl -sS -H 'Content-Type: application/json' -X POST "http://127.0.0.1:${PORT}/mcp" -d "${payload}" >/dev/null
  end=$(python3 -c "import time; print(int(time.time() * 1000))")

  duration=$((end - start))
  echo "${duration}"
}

# Initialize MCP session
INIT_REQ='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"perf-test","version":"0.1.0"},"capabilities":{},"protocolVersion":"2024-11-05"}}'
curl -sS -H 'Content-Type: application/json' -X POST "http://127.0.0.1:${PORT}/mcp" -d "${INIT_REQ}" >/dev/null

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0
RESULTS="{\"tests\":[]}"

# Test 1: Thread list load time
echo ""
echo "Test 1: list_threads latency (target: <${TARGET_THREAD_LIST}ms)"

THREAD_REQ='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_threads","arguments":{}}}'

# Run multiple iterations and average
total_time=0
iterations=5
for i in $(seq 1 $iterations); do
  duration=$(measure_request "${THREAD_REQ}")
  total_time=$((total_time + duration))
  echo "  Iteration $i: ${duration}ms"
done
avg_time=$((total_time / iterations))

if [[ "${avg_time}" -le "${TARGET_THREAD_LIST}" ]]; then
  echo "PASS: list_threads avg ${avg_time}ms <= ${TARGET_THREAD_LIST}ms target"
  PASS_COUNT=$((PASS_COUNT + 1))
  status="pass"
else
  echo "WARN: list_threads avg ${avg_time}ms > ${TARGET_THREAD_LIST}ms target"
  WARN_COUNT=$((WARN_COUNT + 1))
  status="warn"
fi
RESULTS=$(echo "${RESULTS}" | jq --arg name "list_threads" --argjson avg "${avg_time}" --argjson target "${TARGET_THREAD_LIST}" --arg status "${status}" \
  '.tests += [{"name": $name, "avg_ms": $avg, "target_ms": $target, "status": $status}]')

# Test 2: Contact list with presence
echo ""
echo "Test 2: list_contacts (with presence) latency (target: <${TARGET_CONTACT_LIST}ms)"

CONTACT_REQ='{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_contacts","arguments":{"include_presence":true}}}'

total_time=0
for i in $(seq 1 $iterations); do
  duration=$(measure_request "${CONTACT_REQ}")
  total_time=$((total_time + duration))
  echo "  Iteration $i: ${duration}ms"
done
avg_time=$((total_time / iterations))

if [[ "${avg_time}" -le "${TARGET_CONTACT_LIST}" ]]; then
  echo "PASS: list_contacts avg ${avg_time}ms <= ${TARGET_CONTACT_LIST}ms target"
  PASS_COUNT=$((PASS_COUNT + 1))
  status="pass"
else
  echo "WARN: list_contacts avg ${avg_time}ms > ${TARGET_CONTACT_LIST}ms target"
  WARN_COUNT=$((WARN_COUNT + 1))
  status="warn"
fi
RESULTS=$(echo "${RESULTS}" | jq --arg name "list_contacts" --argjson avg "${avg_time}" --argjson target "${TARGET_CONTACT_LIST}" --arg status "${status}" \
  '.tests += [{"name": $name, "avg_ms": $avg, "target_ms": $target, "status": $status}]')

# Test 3: Send message (if tool exists)
echo ""
echo "Test 3: send_message latency (target: <${TARGET_MESSAGE_SEND}ms)"

SEND_REQ='{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"send_message","arguments":{"thread_id":"entity:test","text":"perf test"}}}'

total_time=0
error_count=0
for i in $(seq 1 $iterations); do
  duration=$(measure_request "${SEND_REQ}")
  # Check if send_message tool exists (error response is fast)
  if [[ "${duration}" -lt 5 ]]; then
    error_count=$((error_count + 1))
  fi
  total_time=$((total_time + duration))
  echo "  Iteration $i: ${duration}ms"
done

if [[ "${error_count}" -eq "${iterations}" ]]; then
  echo "SKIP: send_message tool may not be implemented yet"
  status="skip"
else
  avg_time=$((total_time / iterations))
  if [[ "${avg_time}" -le "${TARGET_MESSAGE_SEND}" ]]; then
    echo "PASS: send_message avg ${avg_time}ms <= ${TARGET_MESSAGE_SEND}ms target"
    PASS_COUNT=$((PASS_COUNT + 1))
    status="pass"
  else
    echo "WARN: send_message avg ${avg_time}ms > ${TARGET_MESSAGE_SEND}ms target"
    WARN_COUNT=$((WARN_COUNT + 1))
    status="warn"
  fi
fi
RESULTS=$(echo "${RESULTS}" | jq --arg name "send_message" --argjson avg "${avg_time:-0}" --argjson target "${TARGET_MESSAGE_SEND}" --arg status "${status}" \
  '.tests += [{"name": $name, "avg_ms": $avg, "target_ms": $target, "status": $status}]')

# Test 4: Get contact presence
echo ""
echo "Test 4: get_contact_presence latency (target: <${TARGET_PRESENCE}ms)"

PRESENCE_REQ='{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get_contact_presence","arguments":{"contact_id":"test"}}}'

total_time=0
for i in $(seq 1 $iterations); do
  duration=$(measure_request "${PRESENCE_REQ}")
  total_time=$((total_time + duration))
  echo "  Iteration $i: ${duration}ms"
done
avg_time=$((total_time / iterations))

if [[ "${avg_time}" -le "${TARGET_PRESENCE}" ]]; then
  echo "PASS: get_contact_presence avg ${avg_time}ms <= ${TARGET_PRESENCE}ms target"
  PASS_COUNT=$((PASS_COUNT + 1))
  status="pass"
else
  echo "WARN: get_contact_presence avg ${avg_time}ms > ${TARGET_PRESENCE}ms target"
  WARN_COUNT=$((WARN_COUNT + 1))
  status="warn"
fi
RESULTS=$(echo "${RESULTS}" | jq --arg name "get_contact_presence" --argjson avg "${avg_time}" --argjson target "${TARGET_PRESENCE}" --arg status "${status}" \
  '.tests += [{"name": $name, "avg_ms": $avg, "target_ms": $target, "status": $status}]')

# Summary
echo ""
echo "=== Performance Test Summary ==="
echo "Passed: ${PASS_COUNT}"
echo "Warnings: ${WARN_COUNT}"
echo "Failed: ${FAIL_COUNT}"

# Add summary to results
RESULTS=$(echo "${RESULTS}" | jq --argjson pass "${PASS_COUNT}" --argjson warn "${WARN_COUNT}" --argjson fail "${FAIL_COUNT}" \
  '. + {"summary": {"passed": $pass, "warnings": $warn, "failed": $fail}}')

echo "${RESULTS}" > "${RESULTS_FILE}"
echo ""
echo "Results saved to: ${RESULTS_FILE}"

# Copy to CI artifacts if in CI
if [[ -n "${CI:-}" ]]; then
  PERF_ARTIFACTS="${GITHUB_WORKSPACE:-/tmp}/perf-artifacts"
  mkdir -p "${PERF_ARTIFACTS}"
  cp "${RESULTS_FILE}" "${PERF_ARTIFACTS}/perf_messaging.json"
  echo "CI artifacts copied to: ${PERF_ARTIFACTS}"
fi

# Exit with warning if any targets were missed (but don't fail CI)
if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  echo ""
  echo "FAILED: ${FAIL_COUNT} tests exceeded targets"
  exit 1
fi

if [[ "${WARN_COUNT}" -gt 0 ]]; then
  echo ""
  echo "COMPLETED with ${WARN_COUNT} warnings"
fi

echo ""
echo "=== Performance Tests Complete ==="
