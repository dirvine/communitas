#!/bin/bash

# Launch two Communitas app instances for P2P communication testing
# Each instance will have its own data directory and user identity

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE} Communitas Two-User Test Setup${NC}"
echo -e "${BLUE}========================================${NC}\n"

# Check if test network is running
NODES=$(ps aux | grep communitas-headless | grep -v grep | wc -l)
if [ "$NODES" -lt 8 ]; then
    echo -e "${RED}Error: Test network not fully running (found $NODES nodes, expected 8)${NC}"
    echo "Please run: cd /tmp/communitas-test-network && ./setup-local-network.sh"
    exit 1
fi
echo -e "${GREEN}✓ Test network running with $NODES nodes${NC}"

# Kill any existing Tauri processes
echo -e "\n${YELLOW}Cleaning up existing Tauri instances...${NC}"
pkill -f "communitas-desktop" || true
pkill -f "tauri dev" || true
sleep 2

# Create data directories for two users
USER1_DATA="/tmp/communitas-user1"
USER2_DATA="/tmp/communitas-user2"

rm -rf "$USER1_DATA" "$USER2_DATA"
mkdir -p "$USER1_DATA" "$USER2_DATA"

echo -e "${GREEN}✓ Created data directories${NC}"

# User identities (using valid four-word dictionary words)
USER1_FOURWORDS="tree-mountain-river-sun"
USER1_NAME="Alice"

USER2_FOURWORDS="sky-valley-ocean-moon"
USER2_NAME="Bob"

# Launch User 1 (Alice) app
echo -e "\n${BLUE}Launching User 1 (Alice) - Port 5173${NC}"
echo -e "Four-word identity: ${GREEN}$USER1_FOURWORDS${NC}"

cd "$SCRIPT_DIR/communitas-desktop"

# Set environment for User 1
export COMMUNITAS_DATA_DIR="$USER1_DATA"
export COMMUNITAS_FOUR_WORDS="$USER1_FOURWORDS"
export COMMUNITAS_DISPLAY_NAME="$USER1_NAME"
export VITE_PORT=5173
export TAURI_MCP_SOCKET_PATH="/tmp/tauri-mcp-user1.sock"
export RUST_LOG=info,communitas=debug

# Launch first instance in background
npm run tauri dev -- --config '{"build":{"devUrl":"http://localhost:5173"}}' &
USER1_PID=$!

echo -e "${GREEN}✓ User 1 app launched (PID: $USER1_PID)${NC}"
sleep 5

# Launch User 2 (Bob) app
echo -e "\n${BLUE}Launching User 2 (Bob) - Port 5174${NC}"
echo -e "Four-word identity: ${GREEN}$USER2_FOURWORDS${NC}"

# Set environment for User 2
export COMMUNITAS_DATA_DIR="$USER2_DATA"
export COMMUNITAS_FOUR_WORDS="$USER2_FOURWORDS"
export COMMUNITAS_DISPLAY_NAME="$USER2_NAME"
export VITE_PORT=5174
export TAURI_MCP_SOCKET_PATH="/tmp/tauri-mcp-user2.sock"

# Need to run frontend on different port
cd "$SCRIPT_DIR"
npm run dev -- --port 5174 &
FRONTEND2_PID=$!

sleep 3

cd "$SCRIPT_DIR/communitas-desktop"
npm run tauri dev -- --config '{"build":{"devUrl":"http://localhost:5174"}}' &
USER2_PID=$!

echo -e "${GREEN}✓ User 2 app launched (PID: $USER2_PID)${NC}"

# Wait for both apps to initialize
echo -e "\n${YELLOW}Waiting for apps to initialize...${NC}"
sleep 10

# Display connection info
echo -e "\n${BLUE}========================================${NC}"
echo -e "${BLUE} Two-User Setup Complete!${NC}"
echo -e "${BLUE}========================================${NC}\n"

echo -e "${GREEN}User 1 (Alice):${NC}"
echo -e "  URL: http://localhost:5173"
echo -e "  Four-words: $USER1_FOURWORDS"
echo -e "  Data: $USER1_DATA"
echo -e "  PID: $USER1_PID"
echo -e "  MCP Socket: /tmp/tauri-mcp-user1.sock"

echo -e "\n${GREEN}User 2 (Bob):${NC}"
echo -e "  URL: http://localhost:5174"
echo -e "  Four-words: $USER2_FOURWORDS"
echo -e "  Data: $USER2_DATA"
echo -e "  PID: $USER2_PID"
echo -e "  MCP Socket: /tmp/tauri-mcp-user2.sock"

echo -e "\n${BLUE}Bootstrap Nodes:${NC}"
echo -e "  127.0.0.1:10000 - ocean-forest-moon-star"
echo -e "  127.0.0.1:10001 - lake-valley-earth-fire"
echo -e "  127.0.0.1:10002 - river-sky-cloud-tree"

echo -e "\n${YELLOW}Instructions:${NC}"
echo -e "1. Open both URLs in separate browser windows"
echo -e "2. Login with the four-word identities shown above"
echo -e "3. Add each other as contacts using the four-word addresses"
echo -e "4. Start chatting!"

echo -e "\n${YELLOW}To stop all apps, run:${NC}"
echo -e "pkill -f communitas-desktop"

# Save PIDs for cleanup
cat > /tmp/communitas-two-apps.pids << EOF
$USER1_PID
$USER2_PID
$FRONTEND2_PID
EOF

echo -e "\n${GREEN}PIDs saved to /tmp/communitas-two-apps.pids${NC}"

# Keep script running and show logs
echo -e "\n${YELLOW}Press Ctrl+C to stop all apps${NC}"

# Trap to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Stopping all apps...${NC}"
    kill $USER1_PID 2>/dev/null || true
    kill $USER2_PID 2>/dev/null || true
    kill $FRONTEND2_PID 2>/dev/null || true
    pkill -f "communitas-desktop" || true
    echo -e "${GREEN}✓ All apps stopped${NC}"
}

trap cleanup EXIT

# Wait and show any errors
wait