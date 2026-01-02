#!/bin/bash
# Cleanup after distributed MCP tests
# Usage: ./cleanup.sh [--all] [--local] [--remote]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Node configuration
declare -A NODES=(
    ["saorsa-4"]="206.189.7.117"
    ["saorsa-5"]="144.126.230.161"
    ["saorsa-6"]="65.21.157.229"
)

SSH_USER="root"
BINARY_NAME="communitas-mcp"

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

# Parse arguments
CLEAN_LOCAL=false
CLEAN_REMOTE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            CLEAN_LOCAL=true
            CLEAN_REMOTE=true
            shift
            ;;
        --local)
            CLEAN_LOCAL=true
            shift
            ;;
        --remote)
            CLEAN_REMOTE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--all] [--local] [--remote]"
            echo ""
            echo "Options:"
            echo "  --all     Clean up both local and remote resources"
            echo "  --local   Clean up local processes and temp files"
            echo "  --remote  Stop MCP servers on VPS nodes"
            echo ""
            echo "If no options specified, defaults to --all"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Default to all if nothing specified
if [[ "$CLEAN_LOCAL" == "false" ]] && [[ "$CLEAN_REMOTE" == "false" ]]; then
    CLEAN_LOCAL=true
    CLEAN_REMOTE=true
fi

log_info "=== Communitas MCP Cleanup ==="
echo ""

# Clean local processes
if [[ "$CLEAN_LOCAL" == "true" ]]; then
    log_info "Cleaning local processes..."

    # Kill local MCP processes
    if pgrep -f "communitas-mcp" > /dev/null 2>&1; then
        pkill -f "communitas-mcp" || true
        log_success "Stopped local communitas-mcp processes"
    else
        log_info "No local communitas-mcp processes running"
    fi

    # Kill orchestrator if running
    if pgrep -f "distributed-test-orchestrator" > /dev/null 2>&1; then
        pkill -f "distributed-test-orchestrator" || true
        log_success "Stopped orchestrator processes"
    else
        log_info "No orchestrator processes running"
    fi

    # Clean temp files (optional)
    TEMP_FILES=(/tmp/communitas-* /tmp/mcp-test-*)
    for pattern in "${TEMP_FILES[@]}"; do
        if compgen -G "$pattern" > /dev/null 2>&1; then
            rm -rf "$pattern" 2>/dev/null || true
            log_info "Cleaned temp files: $pattern"
        fi
    done

    echo ""
fi

# Clean remote nodes
if [[ "$CLEAN_REMOTE" == "true" ]]; then
    log_info "Cleaning remote nodes..."

    for name in "${!NODES[@]}"; do
        ip="${NODES[$name]}"
        log_info "  Stopping MCP on $name ($ip)..."

        ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 "$SSH_USER@$ip" \
            "pkill -f '$BINARY_NAME' || true" 2>/dev/null && \
            log_success "  $name: MCP stopped" || \
            log_warn "  $name: Could not connect or process not running"
    done

    echo ""
fi

log_success "Cleanup complete"
