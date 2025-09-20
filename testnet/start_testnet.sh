#!/bin/bash

# Communitas Local Testnet Launcher
# Starts multiple headless nodes for testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BINARY="${PROJECT_ROOT}/target/release/communitas-headless"
LOG_DIR="${SCRIPT_DIR}/logs"
PID_FILE="${SCRIPT_DIR}/testnet.pids"

# Ensure binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Error: Binary not found at ${BINARY}${NC}"
    echo "Please run: cd communitas-headless && cargo build --release"
    exit 1
fi

# Create log directory
mkdir -p "$LOG_DIR"

# Function to cleanup on exit
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

# Set trap for cleanup
trap cleanup EXIT INT TERM

# Clear previous PID file
rm -f "$PID_FILE"

echo -e "${BLUE}🚀 Starting Communitas Local Testnet${NC}"
echo "=================================="

# Function to start a node
start_node() {
    local node_num=$1
    local config_file="${SCRIPT_DIR}/node${node_num}/config/node.json"
    local log_file="${LOG_DIR}/node${node_num}.log"
    local data_dir="${SCRIPT_DIR}/node${node_num}/data"

    # Create data directory
    mkdir -p "$data_dir"

    # Extract node info from config
    local four_words=$(jq -r '.identity.four_words' "$config_file")
    local api_port=$(jq -r '.network.api_listen' "$config_file" | cut -d: -f2)

    echo -e "${GREEN}Starting Node ${node_num}:${NC}"
    echo "  Identity: ${four_words}"
    echo "  API Port: ${api_port}"
    echo "  Log: ${log_file}"

    # Parse listen address from config
    local listen_addr=$(jq -r '.network.quic_listen' "$config_file")

    # Get bootstrap nodes if any
    local bootstrap_args=""
    if [ $i -gt 2 ]; then
        # Nodes 3-5 bootstrap from nodes 1 and 2
        bootstrap_args="-b meadow-sun-river-lake -b desert-meadow-south-raven"
    fi

    # Start node with proper command line arguments
    RUST_LOG=info \
    "$BINARY" \
        --config "$config_file" \
        --storage "$data_dir" \
        --listen "$listen_addr" \
        $bootstrap_args \
        > "$log_file" 2>&1 &

    local pid=$!
    echo $pid >> "$PID_FILE"

    echo -e "  ${GREEN}✓${NC} Started with PID: $pid"
    echo ""
}

# Start all nodes with a delay between each
echo -e "${YELLOW}Starting 5 nodes...${NC}\n"

for i in {1..5}; do
    start_node $i

    # Wait a bit between starting nodes (except for the last one)
    if [ $i -lt 5 ]; then
        echo "Waiting 2 seconds before starting next node..."
        sleep 2
    fi
done

echo -e "${GREEN}✅ All nodes started!${NC}\n"
echo "=================================="
echo -e "${BLUE}Testnet Status:${NC}"
echo ""

# Wait a moment for nodes to initialize
sleep 5

# Show node status
echo "Active Nodes:"
for i in {1..5}; do
    api_port=$((9002 + (i-1)*10))
    config_file="${SCRIPT_DIR}/node${i}/config/node.json"
    four_words=$(jq -r '.identity.four_words' "$config_file")

    echo -e "  Node $i (${four_words}): http://127.0.0.1:${api_port}"
done

echo ""
echo "=================================="
echo -e "${YELLOW}Commands:${NC}"
echo "  View logs:     tail -f ${LOG_DIR}/node*.log"
echo "  Check status:  curl http://127.0.0.1:9002/health"
echo "  Stop testnet:  Press Ctrl+C"
echo ""
echo -e "${GREEN}Testnet is running. Press Ctrl+C to stop all nodes.${NC}"

# Keep script running - wait for any signal
while true; do
    sleep 10
    # Optional: Check if processes are still running
    if [ -f "$PID_FILE" ]; then
        alive_count=0
        while read pid; do
            if kill -0 $pid 2>/dev/null; then
                ((alive_count++))
            fi
        done < "$PID_FILE"

        if [ $alive_count -eq 0 ]; then
            echo -e "${RED}All nodes have stopped unexpectedly${NC}"
            exit 1
        fi
    fi
done