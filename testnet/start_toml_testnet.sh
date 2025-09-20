#!/bin/bash

# Communitas TOML Testnet Launcher

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BINARY="${PROJECT_ROOT}/target/release/communitas-headless"
PID_FILE="${SCRIPT_DIR}/testnet.pids"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Stopping testnet...${NC}"
    if [ -f "$PID_FILE" ]; then
        while read pid; do
            if kill -0 $pid 2>/dev/null; then
                echo "Stopping node with PID $pid"
                kill $pid 2>/dev/null || true
            fi
        done < "$PID_FILE"
        rm "$PID_FILE"
    fi
    echo -e "${GREEN}Testnet stopped${NC}"
}

trap cleanup EXIT INT TERM

# Clear previous PID file
rm -f "$PID_FILE"

echo -e "${BLUE}🚀 Starting Communitas TOML Testnet${NC}"
echo "=================================="

# Start nodes

echo -e "${GREEN}Starting Node 1:${NC}"
echo "  Identity: bear-moon-owl-edge"
echo "  Port: 9000"

"$BINARY" \
    --config "${SCRIPT_DIR}/node1/config.toml" \
    --storage "${SCRIPT_DIR}/node1/data" \
    --listen "0.0.0.0:9000" \
     \
    > "${SCRIPT_DIR}/node1/node.log" 2>&1 &

PID=$!
echo $PID >> "$PID_FILE"
echo -e "  ${GREEN}✓${NC} Started with PID: $PID"

sleep 2  # Wait before starting next node

echo -e "${GREEN}Starting Node 2:${NC}"
echo "  Identity: center-sun-lake-moon"
echo "  Port: 9010"

"$BINARY" \
    --config "${SCRIPT_DIR}/node2/config.toml" \
    --storage "${SCRIPT_DIR}/node2/data" \
    --listen "0.0.0.0:9010" \
     \
    > "${SCRIPT_DIR}/node2/node.log" 2>&1 &

PID=$!
echo $PID >> "$PID_FILE"
echo -e "  ${GREEN}✓${NC} Started with PID: $PID"

sleep 2  # Wait before starting next node

echo -e "${GREEN}Starting Node 3:${NC}"
echo "  Identity: falcon-mountain-eta-desert"
echo "  Port: 9020"

"$BINARY" \
    --config "${SCRIPT_DIR}/node3/config.toml" \
    --storage "${SCRIPT_DIR}/node3/data" \
    --listen "0.0.0.0:9020" \
    -b bear-moon-owl-edge -b center-sun-lake-moon \
    > "${SCRIPT_DIR}/node3/node.log" 2>&1 &

PID=$!
echo $PID >> "$PID_FILE"
echo -e "  ${GREEN}✓${NC} Started with PID: $PID"

sleep 2  # Wait before starting next node

echo -e "${GREEN}Starting Node 4:${NC}"
echo "  Identity: sun-owl-theta-gamma"
echo "  Port: 9030"

"$BINARY" \
    --config "${SCRIPT_DIR}/node4/config.toml" \
    --storage "${SCRIPT_DIR}/node4/data" \
    --listen "0.0.0.0:9030" \
    -b bear-moon-owl-edge -b center-sun-lake-moon \
    > "${SCRIPT_DIR}/node4/node.log" 2>&1 &

PID=$!
echo $PID >> "$PID_FILE"
echo -e "  ${GREEN}✓${NC} Started with PID: $PID"

sleep 2  # Wait before starting next node

echo -e "${GREEN}Starting Node 5:${NC}"
echo "  Identity: desert-owl-bear-gamma"
echo "  Port: 9040"

"$BINARY" \
    --config "${SCRIPT_DIR}/node5/config.toml" \
    --storage "${SCRIPT_DIR}/node5/data" \
    --listen "0.0.0.0:9040" \
    -b bear-moon-owl-edge -b center-sun-lake-moon \
    > "${SCRIPT_DIR}/node5/node.log" 2>&1 &

PID=$!
echo $PID >> "$PID_FILE"
echo -e "  ${GREEN}✓${NC} Started with PID: $PID"


echo -e "\n${GREEN}✅ All nodes started!${NC}"
echo "=================================="
echo -e "${YELLOW}Commands:${NC}"
echo "  View logs:     tail -f ${SCRIPT_DIR}/node*/node.log"
echo "  Stop testnet:  Press Ctrl+C"
echo -e "${GREEN}Testnet is running. Press Ctrl+C to stop all nodes.${NC}"

# Keep running
while true; do
    sleep 10
done
