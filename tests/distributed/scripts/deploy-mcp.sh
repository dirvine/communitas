#!/bin/bash
# Deploy communitas-mcp to VPS nodes
# Usage: ./deploy-mcp.sh [--build] [--node NODE_NAME]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Node configuration
declare -A NODES=(
    ["saorsa-4"]="206.189.7.117"
    ["saorsa-5"]="144.126.230.161"
    ["saorsa-6"]="65.21.157.229"
)

MCP_PORT=3040
SSH_USER="root"
REMOTE_DIR="/opt/communitas"
BINARY_NAME="communitas-mcp"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Parse arguments
BUILD=false
TARGET_NODE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --build)
            BUILD=true
            shift
            ;;
        --node)
            TARGET_NODE="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--build] [--node NODE_NAME]"
            echo ""
            echo "Options:"
            echo "  --build       Build the binary before deploying"
            echo "  --node NAME   Deploy only to specified node (saorsa-4, saorsa-5, saorsa-6)"
            echo ""
            echo "Nodes:"
            for name in "${!NODES[@]}"; do
                echo "  $name: ${NODES[$name]}"
            done
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Build if requested or binary doesn't exist
BINARY_PATH="$PROJECT_ROOT/target/release/$BINARY_NAME"
if [[ "$BUILD" == "true" ]] || [[ ! -f "$BINARY_PATH" ]]; then
    log_info "Building $BINARY_NAME..."
    cd "$PROJECT_ROOT"
    cargo build --release -p communitas-mcp
    log_success "Build complete"
fi

if [[ ! -f "$BINARY_PATH" ]]; then
    log_error "Binary not found at $BINARY_PATH"
    exit 1
fi

# Deploy function
deploy_to_node() {
    local name=$1
    local ip=$2

    log_info "Deploying to $name ($ip)..."

    # Create remote directory if needed
    ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 "$SSH_USER@$ip" \
        "mkdir -p $REMOTE_DIR" 2>/dev/null || {
        log_error "Failed to connect to $name"
        return 1
    }

    # Copy binary
    log_info "  Copying binary..."
    scp -o StrictHostKeyChecking=no "$BINARY_PATH" "$SSH_USER@$ip:$REMOTE_DIR/" || {
        log_error "Failed to copy binary to $name"
        return 1
    }

    # Stop existing process, start new one
    log_info "  Restarting service..."
    ssh -o StrictHostKeyChecking=no "$SSH_USER@$ip" << EOF
        # Stop existing process
        pkill -f '$BINARY_NAME' || true
        sleep 2

        # Make executable
        chmod +x $REMOTE_DIR/$BINARY_NAME

        # Start with nohup
        nohup $REMOTE_DIR/$BINARY_NAME \
            --http --demo --listen 0.0.0.0:$MCP_PORT \
            > /var/log/communitas-mcp.log 2>&1 &

        # Wait for startup
        sleep 3

        # Health check
        if curl -sf http://127.0.0.1:$MCP_PORT/health > /dev/null 2>&1; then
            echo "HEALTH_OK"
        else
            echo "HEALTH_FAIL"
        fi
EOF

    # Check result
    local result
    result=$(ssh -o StrictHostKeyChecking=no "$SSH_USER@$ip" \
        "curl -sf http://127.0.0.1:$MCP_PORT/health > /dev/null 2>&1 && echo OK || echo FAIL")

    if [[ "$result" == "OK" ]]; then
        log_success "$name is healthy"
        return 0
    else
        log_error "$name health check failed"
        return 1
    fi
}

# Main deployment
log_info "=== Communitas MCP Deployment ==="
log_info "Binary: $BINARY_PATH"
echo ""

FAILED_NODES=()

if [[ -n "$TARGET_NODE" ]]; then
    # Deploy to specific node
    if [[ -z "${NODES[$TARGET_NODE]:-}" ]]; then
        log_error "Unknown node: $TARGET_NODE"
        exit 1
    fi
    deploy_to_node "$TARGET_NODE" "${NODES[$TARGET_NODE]}" || FAILED_NODES+=("$TARGET_NODE")
else
    # Deploy to all nodes
    for name in "${!NODES[@]}"; do
        deploy_to_node "$name" "${NODES[$name]}" || FAILED_NODES+=("$name")
        echo ""
    done
fi

# Summary
echo ""
log_info "=== Deployment Summary ==="
if [[ ${#FAILED_NODES[@]} -eq 0 ]]; then
    log_success "All nodes deployed successfully"
    exit 0
else
    log_error "Failed nodes: ${FAILED_NODES[*]}"
    exit 1
fi
