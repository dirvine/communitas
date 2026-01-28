#!/bin/bash
# Launch two Communitas instances for local testing
#
# Instance 1: Port 11000, data in ~/Communitas-Instance1
# Instance 2: Port 11001, data in ~/Communitas-Instance2
#
# Each instance uses the other as bootstrap, creating a peer-to-peer network.


# =============================================================================
# SECURITY NOTE
# =============================================================================
# This script is for LOCAL TESTING ONLY on your local machine.
# It creates two instances connecting to each other via localhost.
# All communication is local and not exposed to external networks.
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Create data directories
INSTANCE1_DATA="$HOME/Communitas-Instance1"
INSTANCE2_DATA="$HOME/Communitas-Instance2"

mkdir -p "$INSTANCE1_DATA"
mkdir -p "$INSTANCE2_DATA"

echo "=== Communitas Local Testnet ==="
echo ""
echo "Instance 1: Port 11000, Data: $INSTANCE1_DATA"
echo "Instance 2: Port 11001, Data: $INSTANCE2_DATA"
echo ""

# Build the app first (if needed)
echo "Building Communitas..."
cd "$PROJECT_DIR/communitas-dioxus"
dx build --platform desktop --release 2>/dev/null || {
    echo "Building in debug mode..."
    dx build --platform desktop
}

# Find the binary
if [[ "$OSTYPE" == "darwin"* ]]; then
    BINARY="$PROJECT_DIR/target/dx/communitas-dioxus/debug/macos/communitas-dioxus.app/Contents/MacOS/communitas-dioxus"
    if [[ ! -f "$BINARY" ]]; then
        BINARY="$PROJECT_DIR/target/dx/communitas-dioxus/release/macos/communitas-dioxus.app/Contents/MacOS/communitas-dioxus"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    BINARY="$PROJECT_DIR/target/dx/communitas-dioxus/debug/linux/communitas-dioxus"
    if [[ ! -f "$BINARY" ]]; then
        BINARY="$PROJECT_DIR/target/dx/communitas-dioxus/release/linux/communitas-dioxus"
    fi
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Could not find binary at $BINARY"
    echo "Please build the app first: cd communitas-dioxus && dx build --platform desktop"
    exit 1
fi

echo "Using binary: $BINARY"
echo ""

# For local testing, each instance bootstraps from the other
# We use a special local mode where instances discover each other via multicast/local
# Set COMMUNITAS_BOOTSTRAP to point to localhost ports

echo "Starting Instance 1 (port 11000)..."
COMMUNITAS_DATA_DIR="$INSTANCE1_DATA" \
COMMUNITAS_PORT=11000 \
COMMUNITAS_BOOTSTRAP="127.0.0.1:11001" \
RUST_LOG=info \
"$BINARY" &
PID1=$!
echo "Instance 1 PID: $PID1"

# Wait a moment for first instance to start
sleep 3

echo "Starting Instance 2 (port 11001)..."
COMMUNITAS_DATA_DIR="$INSTANCE2_DATA" \
COMMUNITAS_PORT=11001 \
COMMUNITAS_BOOTSTRAP="127.0.0.1:11000" \
RUST_LOG=info \
"$BINARY" &
PID2=$!
echo "Instance 2 PID: $PID2"

echo ""
echo "=== Both instances started ==="
echo ""
echo "Instance 1 PID: $PID1 (port 11000)"
echo "Instance 2 PID: $PID2 (port 11001)"
echo ""
echo "To stop both: kill $PID1 $PID2"
echo ""
echo "Press Ctrl+C to stop both instances..."

# Wait and handle cleanup
cleanup() {
    echo ""
    echo "Stopping instances..."
    kill $PID1 2>/dev/null || true
    kill $PID2 2>/dev/null || true
    echo "Done."
}

trap cleanup EXIT

wait
