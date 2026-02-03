#!/bin/bash
# Full E2E Test Runner for Communitas
#
# This script orchestrates comprehensive E2E testing including:
# - Single instance GUI tests via WebDriverIO
# - Dual instance P2P tests
# - MCP server integration tests
#
# Usage:
#   ./scripts/run-e2e-tests.sh [mode]
#
# Modes:
#   gui        - Run WebDriverIO GUI tests (default)
#   p2p        - Run P2P dual-instance tests
#   mcp        - Run MCP integration tests
#   full       - Run all tests
#   setup      - Setup test environment only

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOGS_DIR="$PROJECT_ROOT/tests/webdriverio/logs"
MCP_PORT_A=3100
MCP_PORT_B=3101

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Ensure logs directory exists
mkdir -p "$LOGS_DIR"

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for tauri-driver
    if ! command -v tauri-driver &> /dev/null; then
        log_warn "tauri-driver not found. Installing..."
        cargo install tauri-driver
    fi

    # Check for node/npm
    if ! command -v npm &> /dev/null; then
        log_error "npm is required but not installed"
        exit 1
    fi

    # Check if desktop app is built
    DESKTOP_BIN="$PROJECT_ROOT/target/debug/communitas-dioxus"
    if [ ! -f "$DESKTOP_BIN" ]; then
        log_warn "Desktop app not built. Building now..."
        cd "$PROJECT_ROOT/communitas-dioxus"
        cargo build
    fi

    log_success "Prerequisites checked"
}

# Setup WebDriverIO dependencies
setup_webdriverio() {
    log_info "Setting up WebDriverIO..."
    cd "$PROJECT_ROOT/tests/webdriverio"

    if [ ! -d "node_modules" ]; then
        npm install
    fi

    log_success "WebDriverIO setup complete"
}

# Run GUI tests via WebDriverIO
run_gui_tests() {
    log_info "Running GUI E2E tests..."
    cd "$PROJECT_ROOT/tests/webdriverio"

    # Run all specs or specific one
    if [ -n "$SPEC" ]; then
        npx wdio run wdio.conf.js --spec "$SPEC"
    else
        npx wdio run wdio.conf.js
    fi

    log_success "GUI tests complete"
}

# Run single spec (full E2E with login)
run_full_e2e() {
    log_info "Running full E2E test with login flow..."
    cd "$PROJECT_ROOT/tests/webdriverio"

    npx wdio run wdio.conf.js --spec specs/full-e2e.spec.js

    log_success "Full E2E test complete"
}

# Run P2P dual-instance tests
run_p2p_tests() {
    log_info "Running P2P dual-instance tests..."

    # Create temp data directories
    DATA_DIR_A="/tmp/communitas-e2e-alice"
    DATA_DIR_B="/tmp/communitas-e2e-bob"

    rm -rf "$DATA_DIR_A" "$DATA_DIR_B"
    mkdir -p "$DATA_DIR_A" "$DATA_DIR_B"

    # For full P2P testing, we would:
    # 1. Start two MCP servers
    # 2. Run the P2P spec

    log_warn "Full P2P testing requires two MCP server instances"
    log_info "See tests/webdriverio/specs/p2p-dual-instance.spec.js for implementation guide"

    cd "$PROJECT_ROOT/tests/webdriverio"
    npx wdio run wdio.conf.js --spec specs/p2p-dual-instance.spec.js

    log_success "P2P tests complete"
}

# Run MCP integration tests
run_mcp_tests() {
    log_info "Running MCP integration tests..."
    cd "$PROJECT_ROOT"

    # Run the comprehensive MCP E2E tests
    cargo test -p communitas-mcp --test comprehensive_e2e -- --nocapture
    cargo test -p communitas-mcp --test mcp_e2e -- --nocapture

    log_success "MCP tests complete"
}

# Start MCP server for testing
start_mcp_server() {
    local port=$1
    local data_dir=$2
    local log_file="$LOGS_DIR/mcp-$port.log"

    log_info "Starting MCP server on port $port..."

    COMMUNITAS_DATA_DIR="$data_dir" \
    cargo run -p communitas-mcp -- \
        --http \
        --port "$port" \
        --demo \
        > "$log_file" 2>&1 &

    echo $!
}

# Clean up test artifacts
cleanup() {
    log_info "Cleaning up..."

    # Kill any running MCP servers
    pkill -f "communitas-mcp" 2>/dev/null || true

    # Clean temp directories
    rm -rf /tmp/communitas-e2e-*

    log_success "Cleanup complete"
}

# Print help
print_help() {
    echo "Communitas E2E Test Runner"
    echo ""
    echo "Usage: $0 [mode]"
    echo ""
    echo "Modes:"
    echo "  gui        Run WebDriverIO GUI tests"
    echo "  full       Run full E2E test with login flow"
    echo "  p2p        Run P2P dual-instance tests"
    echo "  mcp        Run MCP integration tests"
    echo "  all        Run all tests"
    echo "  setup      Setup test environment only"
    echo "  clean      Clean up test artifacts"
    echo "  help       Show this help"
    echo ""
    echo "Environment variables:"
    echo "  SPEC       Specific spec file to run (for gui mode)"
    echo ""
    echo "Examples:"
    echo "  $0 full                                # Run full E2E with login"
    echo "  SPEC=specs/nav-auth.smoke.js $0 gui   # Run specific spec"
    echo "  $0 all                                 # Run everything"
}

# Main entry point
main() {
    local mode="${1:-gui}"

    case "$mode" in
        gui)
            check_prerequisites
            setup_webdriverio
            run_gui_tests
            ;;
        full)
            check_prerequisites
            setup_webdriverio
            run_full_e2e
            ;;
        p2p)
            check_prerequisites
            setup_webdriverio
            run_p2p_tests
            ;;
        mcp)
            check_prerequisites
            run_mcp_tests
            ;;
        all)
            check_prerequisites
            setup_webdriverio
            run_mcp_tests
            run_full_e2e
            run_p2p_tests
            ;;
        setup)
            check_prerequisites
            setup_webdriverio
            log_success "Setup complete"
            ;;
        clean)
            cleanup
            ;;
        help|--help|-h)
            print_help
            ;;
        *)
            log_error "Unknown mode: $mode"
            print_help
            exit 1
            ;;
    esac
}

# Trap cleanup on exit
trap cleanup EXIT

main "$@"
