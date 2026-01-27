#!/bin/bash
# Deploy Communitas MCP Server to Saorsa Labs infrastructure
#
# Usage: ./deploy.sh <node> [--skip-build]
#
# Examples:
#   ./deploy.sh saorsa-1           # Deploy to primary node
#   ./deploy.sh saorsa-7           # Deploy to secondary node
#   ./deploy.sh saorsa-1 --skip-build  # Deploy without rebuilding

set -e

# Configuration
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BINARY_NAME="communitas-mcp"
TARGET="x86_64-unknown-linux-gnu"
REMOTE_DIR="/opt/communitas-mcp"
REMOTE_USER="root"

# Node IP mapping
declare -A NODE_IPS=(
    ["saorsa-1"]="77.42.75.115"
    ["saorsa-7"]="116.203.101.172"
)

# Parse arguments
NODE="${1:-}"
SKIP_BUILD="${2:-}"

if [ -z "$NODE" ]; then
    echo "Usage: $0 <node> [--skip-build]"
    echo ""
    echo "Available nodes:"
    for n in "${!NODE_IPS[@]}"; do
        echo "  - $n (${NODE_IPS[$n]})"
    done
    exit 1
fi

IP="${NODE_IPS[$NODE]}"
if [ -z "$IP" ]; then
    echo "Error: Unknown node '$NODE'"
    exit 1
fi

echo "=== Communitas MCP Deployment ==="
echo "Node: $NODE"
echo "IP: $IP"
echo "Project: $PROJECT_DIR"
echo ""

# Step 1: Build binary (unless --skip-build)
if [ "$SKIP_BUILD" != "--skip-build" ]; then
    echo "=== Building binary ==="
    cd "$PROJECT_DIR"

    # Check if cargo-zigbuild is available
    if ! command -v cargo-zigbuild &> /dev/null; then
        echo "Installing cargo-zigbuild..."
        cargo install cargo-zigbuild
    fi

    echo "Cross-compiling for $TARGET..."
    cargo zigbuild --release --target "$TARGET" -p "$BINARY_NAME"

    BINARY="$PROJECT_DIR/target/$TARGET/release/$BINARY_NAME"
    if [ ! -f "$BINARY" ]; then
        echo "Error: Binary not found at $BINARY"
        exit 1
    fi
    echo "Binary built: $BINARY"
else
    BINARY="$PROJECT_DIR/target/$TARGET/release/$BINARY_NAME"
    if [ ! -f "$BINARY" ]; then
        echo "Error: Binary not found. Run without --skip-build first."
        exit 1
    fi
    echo "Using existing binary: $BINARY"
fi

# Step 2: Create remote directory and backup
echo ""
echo "=== Preparing remote ==="
ssh "${REMOTE_USER}@${IP}" "
    mkdir -p ${REMOTE_DIR}/backup
    if [ -f ${REMOTE_DIR}/${BINARY_NAME} ]; then
        cp ${REMOTE_DIR}/${BINARY_NAME} ${REMOTE_DIR}/backup/${BINARY_NAME}.\$(date +%Y%m%d_%H%M%S)
        echo 'Previous binary backed up'
    fi
"

# Step 3: Stop service if running
echo ""
echo "=== Stopping service ==="
ssh "${REMOTE_USER}@${IP}" "
    systemctl stop ${BINARY_NAME} 2>/dev/null || echo 'Service not running'
"

# Step 4: Copy binary
echo ""
echo "=== Copying binary ==="
scp "$BINARY" "${REMOTE_USER}@${IP}:${REMOTE_DIR}/${BINARY_NAME}"

# Step 5: Copy service file
echo ""
echo "=== Installing service ==="
scp "$PROJECT_DIR/deployment/communitas-mcp.service" "${REMOTE_USER}@${IP}:/etc/systemd/system/"

# Step 6: Create user if needed and set permissions
echo ""
echo "=== Setting up permissions ==="
ssh "${REMOTE_USER}@${IP}" "
    # Create user if not exists
    id communitas &>/dev/null || useradd -r -s /bin/false communitas

    # Set permissions
    chmod +x ${REMOTE_DIR}/${BINARY_NAME}
    chown -R communitas:communitas ${REMOTE_DIR}

    # Reload systemd
    systemctl daemon-reload
"

# Step 7: Start service
echo ""
echo "=== Starting service ==="
ssh "${REMOTE_USER}@${IP}" "
    systemctl enable ${BINARY_NAME}
    systemctl start ${BINARY_NAME}
    sleep 2
    systemctl status ${BINARY_NAME} --no-pager
"

# Step 8: Verify deployment
echo ""
echo "=== Verifying deployment ==="
echo "Checking health endpoint..."
sleep 3

# Try to curl health endpoint (may need TLS setup first)
if curl -sk --connect-timeout 5 "https://${IP}:8443/health" 2>/dev/null; then
    echo ""
    echo "✓ Health check passed"
else
    echo ""
    echo "! Health check failed (may need TLS setup)"
    echo "  Run: ssh root@${IP} '${REMOTE_DIR}/setup-tls.sh'"
fi

echo ""
echo "=== Deployment Complete ==="
echo ""
echo "Next steps:"
echo "  1. Setup TLS: ssh root@${IP} '/opt/communitas-mcp/setup-tls.sh <domain>'"
echo "  2. View logs: ssh root@${IP} 'journalctl -u ${BINARY_NAME} -f'"
echo "  3. Test MCP: curl -X POST https://${IP}:8443/mcp -H 'Content-Type: application/json' -d '{...}'"
