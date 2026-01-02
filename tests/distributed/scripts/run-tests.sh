#!/bin/bash
# Run distributed MCP tests
# Usage: ./run-tests.sh [SUITE] [OPTIONS]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
TESTS_DIR="$SCRIPT_DIR/.."

# Default configuration
SUITE="${1:-all}"
shift || true

LOCAL_MCP_PORT=3041
REPORT_DIR="$TESTS_DIR/reports/$(date +%Y%m%d_%H%M%S)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    if [[ -n "${LOCAL_MCP_PID:-}" ]]; then
        kill "$LOCAL_MCP_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Parse additional options
ANTHROPIC_KEY="${ANTHROPIC_API_KEY:-}"
MODEL="claude-3-haiku-20240307"

while [[ $# -gt 0 ]]; do
    case $1 in
        --key)
            ANTHROPIC_KEY="$2"
            shift 2
            ;;
        --model)
            MODEL="$2"
            shift 2
            ;;
        --output)
            REPORT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [SUITE] [OPTIONS]"
            echo ""
            echo "Suites:"
            echo "  all         Run all test phases (default)"
            echo "  identity    Phase 1: Identity & Authentication"
            echo "  entities    Phase 2: Entity Lifecycle"
            echo "  members     Phase 3: Member Management"
            echo "  messaging   Phase 4: Messaging"
            echo "  files       Phase 5: Virtual Disks"
            echo "  kanban      Phase 6: Kanban"
            echo "  contacts    Phase 7: Contacts"
            echo "  networking  Phase 8: P2P Networking"
            echo "  websites    Phase 9: Websites"
            echo "  invites     Phase 10: Invites"
            echo "  sync        Phase 11: Sync & Reconnection"
            echo ""
            echo "Options:"
            echo "  --key KEY       Anthropic API key (default: \$ANTHROPIC_API_KEY)"
            echo "  --model MODEL   Model to use (default: claude-3-haiku-20240307)"
            echo "  --output DIR    Report output directory"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate Anthropic key
if [[ -z "$ANTHROPIC_KEY" ]]; then
    log_error "ANTHROPIC_API_KEY not set. Use --key or export ANTHROPIC_API_KEY"
    exit 1
fi

# Map suite to config file
get_config_file() {
    case $1 in
        all)       echo "$TESTS_DIR/scenarios/all_tools.yaml" ;;
        identity)  echo "$TESTS_DIR/scenarios/phase_01_identity.yaml" ;;
        entities)  echo "$TESTS_DIR/scenarios/phase_02_entities.yaml" ;;
        members)   echo "$TESTS_DIR/scenarios/phase_03_members.yaml" ;;
        messaging) echo "$TESTS_DIR/scenarios/phase_04_messaging.yaml" ;;
        files)     echo "$TESTS_DIR/scenarios/phase_05_files.yaml" ;;
        kanban)    echo "$TESTS_DIR/scenarios/phase_06_kanban.yaml" ;;
        contacts)  echo "$TESTS_DIR/scenarios/phase_07_contacts.yaml" ;;
        networking) echo "$TESTS_DIR/scenarios/phase_08_networking.yaml" ;;
        websites)  echo "$TESTS_DIR/scenarios/phase_09_websites.yaml" ;;
        invites)   echo "$TESTS_DIR/scenarios/phase_10_invites.yaml" ;;
        sync)      echo "$TESTS_DIR/scenarios/phase_11_sync.yaml" ;;
        *)
            log_error "Unknown suite: $1"
            exit 1
            ;;
    esac
}

CONFIG_FILE=$(get_config_file "$SUITE")
if [[ ! -f "$CONFIG_FILE" ]]; then
    log_error "Config file not found: $CONFIG_FILE"
    exit 1
fi

# Create report directory
mkdir -p "$REPORT_DIR"
log_info "Reports will be saved to: $REPORT_DIR"

# Build orchestrator if needed
ORCHESTRATOR="$PROJECT_ROOT/target/release/distributed-test-orchestrator"
if [[ ! -f "$ORCHESTRATOR" ]]; then
    log_info "Building test orchestrator..."
    cd "$PROJECT_ROOT"
    cargo build --release -p distributed-test-orchestrator
fi

# Start local MCP server
log_info "Starting local MCP server on port $LOCAL_MCP_PORT..."
cd "$PROJECT_ROOT"
cargo run --release -p communitas-mcp -- \
    --http --demo --listen "127.0.0.1:$LOCAL_MCP_PORT" &
LOCAL_MCP_PID=$!

# Wait for local server to be ready
log_info "Waiting for local MCP server..."
for i in {1..30}; do
    if curl -sf "http://127.0.0.1:$LOCAL_MCP_PORT/health" > /dev/null 2>&1; then
        log_success "Local MCP server is ready"
        break
    fi
    if [[ $i -eq 30 ]]; then
        log_error "Local MCP server failed to start"
        exit 1
    fi
    sleep 1
done

# Node configuration
NODES="alice:206.189.7.117:3040,bob:144.126.230.161:3040,charlie:65.21.157.229:3040,dave:127.0.0.1:$LOCAL_MCP_PORT"

# Run orchestrator
log_info "=== Running Test Suite: $SUITE ==="
echo ""

"$ORCHESTRATOR" \
    --config "$CONFIG_FILE" \
    --nodes "$NODES" \
    --output "$REPORT_DIR" \
    --anthropic-key "$ANTHROPIC_KEY" \
    --model "$MODEL" \
    "$@"

RESULT=$?

echo ""
log_info "=== Test Complete ==="
log_info "Report: $REPORT_DIR/report.html"
log_info "JSON: $REPORT_DIR/report.json"

exit $RESULT
