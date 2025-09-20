#!/bin/bash

# Launch Two Communitas App Instances for Testing
# Configured to connect to local testnet

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_BINARY="${PROJECT_ROOT}/target/debug/communitas-desktop"

echo -e "${BLUE}🚀 Launching Dual Communitas Apps${NC}"
echo "================================"
echo ""

# Check if app is built
if [ ! -f "$APP_BINARY" ]; then
    echo -e "${YELLOW}Building Communitas desktop app...${NC}"
    cd "$PROJECT_ROOT"
    cargo build -p communitas-desktop
fi

# Launch App Instance 1 - Alice
echo -e "${GREEN}Starting App Instance 1 (Alice)${NC}"
echo "  Identity: bike-in-porto-napkin"
echo "  Bootstrap nodes: Node 1 & 2"
echo ""

# Create data directory for Alice
mkdir -p "${SCRIPT_DIR}/alice_data"

# Set environment for Alice
export COMMUNITAS_DATA_DIR="${SCRIPT_DIR}/alice_data"
export COMMUNITAS_IDENTITY="bike-in-porto-napkin"
export COMMUNITAS_BOOTSTRAP_NODES="philosophy-truth-prevent-wound:9000,donna-jewish-scorpion-socrates:9010"
export RUST_LOG=info

# Launch Alice instance
"$APP_BINARY" &
ALICE_PID=$!
echo "  PID: $ALICE_PID"
echo ""

sleep 3

# Launch App Instance 2 - Bob
echo -e "${GREEN}Starting App Instance 2 (Bob)${NC}"
echo "  Identity: congratulate-twice-tonga-hurt"
echo "  Bootstrap nodes: Node 1 & 2"
echo ""

# Create data directory for Bob
mkdir -p "${SCRIPT_DIR}/bob_data"

# Set environment for Bob
export COMMUNITAS_DATA_DIR="${SCRIPT_DIR}/bob_data"
export COMMUNITAS_IDENTITY="congratulate-twice-tonga-hurt"
export COMMUNITAS_BOOTSTRAP_NODES="philosophy-truth-prevent-wound:9000,donna-jewish-scorpion-socrates:9010"

# Launch Bob instance
"$APP_BINARY" &
BOB_PID=$!
echo "  PID: $BOB_PID"
echo ""

echo -e "${GREEN}✅ Both app instances started!${NC}"
echo "================================"
echo ""
echo "Alice (bike-in-porto-napkin): PID $ALICE_PID"
echo "Bob (congratulate-twice-tonga-hurt): PID $BOB_PID"
echo ""
echo -e "${YELLOW}Testing Instructions:${NC}"
echo "1. In Alice's app: Create a group or channel"
echo "2. In Bob's app: Join the group using the invite code"
echo "3. Exchange messages between the two instances"
echo "4. Test file sharing and other features"
echo ""
echo "Press Ctrl+C to stop both app instances"

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Stopping app instances...${NC}"
    kill $ALICE_PID $BOB_PID 2>/dev/null || true
    echo -e "${GREEN}App instances stopped${NC}"
}

trap cleanup EXIT INT TERM

# Keep running
while true; do
    sleep 1
    # Check if processes are still running
    if ! kill -0 $ALICE_PID 2>/dev/null; then
        echo -e "${RED}Alice app stopped unexpectedly${NC}"
        exit 1
    fi
    if ! kill -0 $BOB_PID 2>/dev/null; then
        echo -e "${RED}Bob app stopped unexpectedly${NC}"
        exit 1
    fi
done