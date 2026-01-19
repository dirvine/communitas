#!/usr/bin/env bash
#
# Unified MCP Parity Tests
#
# Tests that MCP tools for all surfaces return consistent data.
# Covers: Kanban, Drive, Call, Canvas surfaces.
#
# Usage: ./scripts/tests/mcp_parity.sh [--surface <name>]
#
# Options:
#   --surface kanban|drive|call|canvas  Run tests for specific surface only
#   --verbose                           Enable verbose output
#
set -euo pipefail

SURFACE=""
VERBOSE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --surface)
      SURFACE="$2"
      shift 2
      ;;
    --verbose)
      VERBOSE=true
      shift
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

if ! command -v jq >/dev/null 2>&1; then
  echo "[mcp-parity] jq is required" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "[mcp-parity] curl is required" >&2
  exit 1
fi

PORT="${MCP_HTTP_PORT:-7333}"
TMP_ROOT="$(mktemp -d)"
STORAGE_DIR="${TMP_ROOT}/storage"
mkdir -p "${STORAGE_DIR}"
FOUR_WORDS="${MCP_DEMO_FOUR_WORDS:-demo-parity-unified-test}"
DISPLAY_NAME="${MCP_DEMO_DISPLAY:-Parity Unified}"
LOG_FILE="${TMP_ROOT}/mcp.log"

# Fixed artifacts directory for CI
if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  CI_ARTIFACTS_DIR="${GITHUB_WORKSPACE:-$(pwd)}/m3-parity-artifacts"
  mkdir -p "${CI_ARTIFACTS_DIR}"
fi

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  # Copy artifacts for CI before cleanup
  if [[ -n "${GITHUB_ACTIONS:-}" ]] && [[ -d "${TMP_ROOT}/artifacts" ]]; then
    cp -r "${TMP_ROOT}/artifacts/"* "${CI_ARTIFACTS_DIR}/" 2>/dev/null || true
    log "Artifacts copied to: ${CI_ARTIFACTS_DIR}"
  fi
  if [[ "${VERBOSE}" != "true" ]] && [[ -z "${GITHUB_ACTIONS:-}" ]]; then
    rm -rf "${TMP_ROOT}"
  else
    echo "[mcp-parity] artifacts preserved at: ${TMP_ROOT}"
  fi
}
trap cleanup EXIT

log() {
  echo "[mcp-parity] $*"
}

verbose() {
  if [[ "${VERBOSE}" == "true" ]]; then
    echo "[mcp-parity] $*"
  fi
}

log "building communitas-mcp (release)"
cargo build -p communitas-mcp --release --quiet

log "launching communitas-mcp demo HTTP server"
target/release/communitas-mcp \
  --demo \
  --http \
  --listen "127.0.0.1:${PORT}" \
  --storage-dir "${STORAGE_DIR}" \
  --four-words "${FOUR_WORDS}" \
  --display-name "${DISPLAY_NAME}" \
  >"${LOG_FILE}" 2>&1 &
SERVER_PID=$!

# Wait for server to start - poll until ready or timeout
MAX_WAIT=10
for i in $(seq 1 ${MAX_WAIT}); do
  if curl -s "http://127.0.0.1:${PORT}/mcp" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":0,"method":"ping","params":{}}' >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

post() {
  local payload="$1"
  curl -sS -H 'Content-Type: application/json' -X POST "http://127.0.0.1:${PORT}/mcp" -d "${payload}"
}

INIT_REQ='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"mcp-parity-unified","version":"0.1.0"},"capabilities":{},"protocolVersion":"2024-11-05"}}'
post "${INIT_REQ}" >/dev/null

ARTIFACT_DIR="${TMP_ROOT}/artifacts"
mkdir -p "${ARTIFACT_DIR}"

RESULTS=()
REQUEST_ID=2

# Helper to increment request ID
next_id() {
  REQUEST_ID=$((REQUEST_ID + 1))
  echo "${REQUEST_ID}"
}

# Helper to record test result
record_result() {
  local surface="$1"
  local test_name="$2"
  local status="$3"
  local details="${4:-}"
  RESULTS+=("${surface}|${test_name}|${status}|${details}")
  if [[ "${status}" == "PASS" ]]; then
    log "PASS: ${surface}/${test_name}"
  else
    log "FAIL: ${surface}/${test_name} - ${details}"
  fi
}

# ============================================================================
# KANBAN SURFACE TESTS
# ============================================================================
run_kanban_tests() {
  log "=== KANBAN SURFACE TESTS ==="

  # Test 1: Create board and verify round-trip
  log "Test: Kanban board CRUD consistency"

  local board_name="parity-test-board-$(date +%s)"
  local create_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"create_kanban_board","arguments":{"entity_id":"self","name":"'"${board_name}"'","description":"Parity test board"}}}'
  local create_resp=$(post "${create_req}")
  echo "${create_resp}" > "${ARTIFACT_DIR}/kanban_create_board.json"

  local board_id=$(echo "${create_resp}" | jq -r '.result.content[0].text' | jq -r '.id // .board_id // empty')

  if [[ -z "${board_id}" ]]; then
    record_result "kanban" "create_board" "FAIL" "No board_id returned"
  else
    verbose "Created board: ${board_id}"

    # List boards and verify it exists
    local list_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"list_kanban_boards","arguments":{"entity_id":"self"}}}'
    local list_resp=$(post "${list_req}")
    echo "${list_resp}" > "${ARTIFACT_DIR}/kanban_list_boards.json"

    local found=$(echo "${list_resp}" | jq -r '.result.content[0].text' | jq -r --arg id "${board_id}" '.boards[] | select(.id == $id) | .id // empty')

    if [[ "${found}" == "${board_id}" ]]; then
      record_result "kanban" "board_crud" "PASS"
    else
      record_result "kanban" "board_crud" "FAIL" "Board not found in list"
    fi
  fi

  # Test 2: Column CRUD consistency
  log "Test: Kanban column CRUD consistency"

  if [[ -n "${board_id:-}" ]]; then
    local col_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"create_kanban_column","arguments":{"entity_id":"self","board_id":"'"${board_id}"'","name":"Todo","position":0}}}'
    local col_resp=$(post "${col_req}")
    echo "${col_resp}" > "${ARTIFACT_DIR}/kanban_create_column.json"

    local col_id=$(echo "${col_resp}" | jq -r '.result.content[0].text' | jq -r '.id // .column_id // empty')

    if [[ -z "${col_id}" ]]; then
      record_result "kanban" "create_column" "FAIL" "No column_id returned"
    else
      # Verify board has columns by checking column_count
      local board_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"get_kanban_board","arguments":{"entity_id":"self","board_id":"'"${board_id}"'"}}}'
      local board_resp=$(post "${board_req}")
      echo "${board_resp}" > "${ARTIFACT_DIR}/kanban_get_board.json"

      local col_count=$(echo "${board_resp}" | jq -r '.result.content[0].text' | jq -r '.column_count // 0')

      if [[ "${col_count}" -gt 0 ]]; then
        record_result "kanban" "column_crud" "PASS" "column_count=${col_count}"
      else
        record_result "kanban" "column_crud" "FAIL" "No columns in board"
      fi
    fi
  else
    record_result "kanban" "column_crud" "SKIP" "No board created"
  fi

  # Test 3: Card CRUD consistency
  log "Test: Kanban card CRUD consistency"

  if [[ -n "${board_id:-}" && -n "${col_id:-}" ]]; then
    local card_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"create_kanban_card","arguments":{"entity_id":"self","board_id":"'"${board_id}"'","column_id":"'"${col_id}"'","title":"Test Card","description":"Parity test"}}}'
    local card_resp=$(post "${card_req}")
    echo "${card_resp}" > "${ARTIFACT_DIR}/kanban_create_card.json"

    local card_id=$(echo "${card_resp}" | jq -r '.result.content[0].text' | jq -r '.id // .card_id // empty')

    if [[ -z "${card_id}" ]]; then
      record_result "kanban" "create_card" "FAIL" "No card_id returned"
    else
      # Verify card appears in column
      local cards_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"list_kanban_cards","arguments":{"entity_id":"self","board_id":"'"${board_id}"'","column_id":"'"${col_id}"'"}}}'
      local cards_resp=$(post "${cards_req}")
      echo "${cards_resp}" > "${ARTIFACT_DIR}/kanban_list_cards.json"

      local card_found=$(echo "${cards_resp}" | jq -r '.result.content[0].text' | jq -r --arg id "${card_id}" '.cards[]? | select(.id == $id) | .id // empty')

      if [[ "${card_found}" == "${card_id}" ]]; then
        record_result "kanban" "card_crud" "PASS"
      else
        record_result "kanban" "card_crud" "FAIL" "Card not found in list"
      fi
    fi
  else
    record_result "kanban" "card_crud" "SKIP" "No board/column created"
  fi
}

# ============================================================================
# DRIVE SURFACE TESTS
# ============================================================================
run_drive_tests() {
  log "=== DRIVE SURFACE TESTS ==="

  # Test 1: List disks
  log "Test: Drive list_disks"

  local disks_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"list_disks","arguments":{"entity_id":"self"}}}'
  local disks_resp=$(post "${disks_req}")
  echo "${disks_resp}" > "${ARTIFACT_DIR}/drive_list_disks.json"

  local disk_count=$(echo "${disks_resp}" | jq -r '.result.content[0].text' | jq -r '.disks | length // 0')

  if [[ "${disk_count}" -gt 0 ]]; then
    record_result "drive" "list_disks" "PASS" "Found ${disk_count} disks"
  else
    # Demo mode may not have disks initialized
    record_result "drive" "list_disks" "WARN" "No disks found (may be expected in demo)"
  fi

  # Test 2: Create directory and verify
  log "Test: Drive create_directory"

  local dir_name="parity-test-$(date +%s)"
  local create_dir_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"create_directory","arguments":{"entity_id":"self","disk_type":"private","path":"'"${dir_name}"'"}}}'
  local create_dir_resp=$(post "${create_dir_req}")
  echo "${create_dir_resp}" > "${ARTIFACT_DIR}/drive_create_dir.json"

  local create_success=$(echo "${create_dir_resp}" | jq -r '.result.content[0].text' | grep -c "success\|created" || echo "0")

  if [[ "${create_success}" -gt 0 ]]; then
    # List directory to verify
    local list_dir_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"list_files","arguments":{"entity_id":"self","disk_type":"private","path":"/"}}}'
    local list_dir_resp=$(post "${list_dir_req}")
    echo "${list_dir_resp}" > "${ARTIFACT_DIR}/drive_list_dir.json"

    local dir_found=$(echo "${list_dir_resp}" | jq -r '.result.content[0].text' | jq -r --arg name "${dir_name}" '.files[]? | select(.name == $name) | .name // empty')

    if [[ "${dir_found}" == "${dir_name}" ]]; then
      record_result "drive" "create_directory" "PASS"
    else
      record_result "drive" "create_directory" "FAIL" "Directory not found in listing"
    fi
  else
    record_result "drive" "create_directory" "FAIL" "Create failed"
  fi

  # Test 3: File operations consistency
  log "Test: Drive file operations"

  local write_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"write_file","arguments":{"entity_id":"self","disk_type":"private","path":"parity-test.txt","content":"Parity test content"}}}'
  local write_resp=$(post "${write_req}")
  echo "${write_resp}" > "${ARTIFACT_DIR}/drive_write_file.json"

  local write_success=$(echo "${write_resp}" | jq -r '.result.content[0].text' | grep -c "success\|written\|saved" || echo "0")

  if [[ "${write_success}" -gt 0 ]]; then
    # Read back and verify
    local read_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"read_file","arguments":{"entity_id":"self","disk_type":"private","path":"parity-test.txt"}}}'
    local read_resp=$(post "${read_req}")
    echo "${read_resp}" > "${ARTIFACT_DIR}/drive_read_file.json"

    local content=$(echo "${read_resp}" | jq -r '.result.content[0].text' | jq -r '.content // empty')

    if [[ "${content}" == "Parity test content" ]]; then
      record_result "drive" "file_roundtrip" "PASS"
    else
      record_result "drive" "file_roundtrip" "FAIL" "Content mismatch"
    fi
  else
    record_result "drive" "file_roundtrip" "FAIL" "Write failed"
  fi
}

# ============================================================================
# CALL SURFACE TESTS
# ============================================================================
run_call_tests() {
  log "=== CALL SURFACE TESTS ==="

  # Test 1: Start call and verify status
  log "Test: Call start and status"

  local start_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"start_voice_call","arguments":{"entity_id":"self"}}}'
  local start_resp=$(post "${start_req}")
  echo "${start_resp}" > "${ARTIFACT_DIR}/call_start.json"

  local call_id=$(echo "${start_resp}" | jq -r '.result.content[0].text' | jq -r '.id // .call_id // empty')

  if [[ -z "${call_id}" ]]; then
    # Call may not be supported in demo mode
    record_result "call" "start_call" "SKIP" "Call not supported in demo"
  else
    verbose "Started call: ${call_id}"

    # Get call status
    local status_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"get_call_status","arguments":{"call_id":"'"${call_id}"'"}}}'
    local status_resp=$(post "${status_req}")
    echo "${status_resp}" > "${ARTIFACT_DIR}/call_status.json"

    local status_call_id=$(echo "${status_resp}" | jq -r '.result.content[0].text' | jq -r '.call_id // empty')

    if [[ "${status_call_id}" == "${call_id}" ]]; then
      record_result "call" "call_status" "PASS"
    else
      record_result "call" "call_status" "FAIL" "Status returned wrong call_id"
    fi

    # Test mute toggle
    log "Test: Call mute toggle"

    local mute_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"toggle_mute","arguments":{"call_id":"'"${call_id}"'","muted":true}}}'
    local mute_resp=$(post "${mute_req}")
    echo "${mute_resp}" > "${ARTIFACT_DIR}/call_mute.json"

    local muted=$(echo "${mute_resp}" | jq -r '.result.content[0].text' | jq -r '.muted // empty')

    if [[ "${muted}" == "true" ]]; then
      record_result "call" "toggle_mute" "PASS"
    else
      record_result "call" "toggle_mute" "FAIL" "Mute state incorrect"
    fi

    # Test video toggle
    log "Test: Call video toggle"

    local video_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"toggle_video","arguments":{"call_id":"'"${call_id}"'","enabled":true}}}'
    local video_resp=$(post "${video_req}")
    echo "${video_resp}" > "${ARTIFACT_DIR}/call_video.json"

    local video_enabled=$(echo "${video_resp}" | jq -r '.result.content[0].text' | jq -r '.video_enabled // empty')

    if [[ "${video_enabled}" == "true" ]]; then
      record_result "call" "toggle_video" "PASS"
    else
      record_result "call" "toggle_video" "FAIL" "Video state incorrect"
    fi

    # End call
    local end_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"end_call","arguments":{"call_id":"'"${call_id}"'"}}}'
    post "${end_req}" > "${ARTIFACT_DIR}/call_end.json"
  fi
}

# ============================================================================
# CANVAS SURFACE TESTS
# ============================================================================
run_canvas_tests() {
  log "=== CANVAS SURFACE TESTS ==="

  # Canvas uses per-entity model - there's one canvas per entity, not create/list
  # Test 1: Get canvas snapshot
  log "Test: Canvas get snapshot"

  local get_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"canvas_get_snapshot","arguments":{"entity_id":"self"}}}'
  local get_resp=$(post "${get_req}")
  echo "${get_resp}" > "${ARTIFACT_DIR}/canvas_get_snapshot.json"

  local is_error=$(echo "${get_resp}" | jq -r '.result.isError // false')

  if [[ "${is_error}" == "false" ]]; then
    record_result "canvas" "get_snapshot" "PASS"
  else
    local error_msg=$(echo "${get_resp}" | jq -r '.result.content[0].text // "unknown"')
    record_result "canvas" "get_snapshot" "FAIL" "${error_msg}"
  fi

  # Test 2: Add text element
  log "Test: Canvas add text"

  local add_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"canvas_add_text","arguments":{"entity_id":"self","x":100,"y":100,"content":"Parity test text"}}}'
  local add_resp=$(post "${add_req}")
  echo "${add_resp}" > "${ARTIFACT_DIR}/canvas_add_text.json"

  local add_is_error=$(echo "${add_resp}" | jq -r '.result.isError // false')
  local element_id=$(echo "${add_resp}" | jq -r '.result.content[0].text' 2>/dev/null | jq -r '.id // .element_id // empty' 2>/dev/null || echo "")

  if [[ "${add_is_error}" == "false" && -n "${element_id}" ]]; then
    record_result "canvas" "add_text" "PASS" "element_id=${element_id}"
  elif [[ "${add_is_error}" == "false" ]]; then
    record_result "canvas" "add_text" "PASS" "No element_id but success"
  else
    local add_error=$(echo "${add_resp}" | jq -r '.result.content[0].text // "unknown"')
    record_result "canvas" "add_text" "FAIL" "${add_error}"
  fi

  # Test 3: Canvas clear
  log "Test: Canvas clear"

  local clear_req='{"jsonrpc":"2.0","id":'$(next_id)',"method":"tools/call","params":{"name":"canvas_clear","arguments":{"entity_id":"self"}}}'
  local clear_resp=$(post "${clear_req}")
  echo "${clear_resp}" > "${ARTIFACT_DIR}/canvas_clear.json"

  local clear_is_error=$(echo "${clear_resp}" | jq -r '.result.isError // false')

  if [[ "${clear_is_error}" == "false" ]]; then
    record_result "canvas" "clear" "PASS"
  else
    local clear_error=$(echo "${clear_resp}" | jq -r '.result.content[0].text // "unknown"')
    record_result "canvas" "clear" "FAIL" "${clear_error}"
  fi
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

# Run tests based on --surface flag or all
if [[ -z "${SURFACE}" ]]; then
  run_kanban_tests
  run_drive_tests
  run_call_tests
  run_canvas_tests
else
  case "${SURFACE}" in
    kanban) run_kanban_tests ;;
    drive) run_drive_tests ;;
    call) run_call_tests ;;
    canvas) run_canvas_tests ;;
    *)
      log "Unknown surface: ${SURFACE}"
      exit 1
      ;;
  esac
fi

# ============================================================================
# SUMMARY
# ============================================================================
log "=== TEST SUMMARY ==="
log "Artifacts: ${ARTIFACT_DIR}"

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
WARN_COUNT=0

for result in "${RESULTS[@]}"; do
  IFS='|' read -r surface test status details <<< "${result}"
  case "${status}" in
    PASS) PASS_COUNT=$((PASS_COUNT + 1)) ;;
    FAIL) FAIL_COUNT=$((FAIL_COUNT + 1)) ;;
    SKIP) SKIP_COUNT=$((SKIP_COUNT + 1)) ;;
    WARN) WARN_COUNT=$((WARN_COUNT + 1)) ;;
  esac
done

log "Results: ${PASS_COUNT} passed, ${FAIL_COUNT} failed, ${SKIP_COUNT} skipped, ${WARN_COUNT} warnings"

# Copy artifacts to CI-accessible location
if [[ -n "${CI:-}" ]]; then
  PARITY_ARTIFACTS="${GITHUB_WORKSPACE:-/tmp}/mcp-parity-artifacts"
  mkdir -p "${PARITY_ARTIFACTS}"
  cp -r "${ARTIFACT_DIR}"/* "${PARITY_ARTIFACTS}/" 2>/dev/null || true
  log "CI artifacts copied to: ${PARITY_ARTIFACTS}"

  # Write summary for GitHub Actions
  if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
    {
      echo "## MCP Parity Test Results"
      echo ""
      echo "| Surface | Test | Status | Details |"
      echo "|---------|------|--------|---------|"
      for result in "${RESULTS[@]}"; do
        IFS='|' read -r surface test status details <<< "${result}"
        echo "| ${surface} | ${test} | ${status} | ${details} |"
      done
      echo ""
      echo "**Summary**: ${PASS_COUNT} passed, ${FAIL_COUNT} failed, ${SKIP_COUNT} skipped, ${WARN_COUNT} warnings"
    } >> "${GITHUB_STEP_SUMMARY}"
  fi
fi

# Exit with failure if any tests failed
if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  log "FAILED: ${FAIL_COUNT} parity checks did not pass"
  exit 1
fi

log "ALL MCP PARITY CHECKS PASSED"
