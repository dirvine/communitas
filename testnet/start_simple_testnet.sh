#!/bin/bash

# Simple Communitas Testnet Launcher
# Uses command-line arguments only, no config files

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
LOG_DIR="${SCRIPT_DIR}/logs"

# Ensure binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Error: Binary not found at ${BINARY}${NC}"
    echo "Please run: cd communitas-headless && cargo build --release"
    exit 1
fi

# Create directories
mkdir -p "$LOG_DIR"
for i in {1..5}; do
    mkdir -p "${SCRIPT_DIR}/node${i}/storage"
done

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

echo -e "${BLUE}🚀 Starting Simple Communitas Testnet${NC}"
echo "=================================="
echo ""

# Node identities (valid four-word addresses from saorsa-core)
NODE1_ID="philosophy-truth-prevent-wound"
NODE2_ID="donna-jewish-scorpion-socrates"
NODE3_ID="bike-in-porto-napkin"
NODE4_ID="congratulate-twice-tonga-hurt"
NODE5_ID="sponsor-biker-simon-leipzig"

# Base ports
BASE_PORT=9000

# Start nodes
start_node() {
    local node_num=$1
    local node_id=$2
    local bootstrap_args=$3

    local port=$((BASE_PORT + (node_num - 1) * 10))
    local storage_dir="${SCRIPT_DIR}/node${node_num}/storage"
    local log_file="${LOG_DIR}/node${node_num}.log"

    echo -e "${GREEN}Starting Node ${node_num}:${NC}"
    echo "  Identity: ${node_id}"
    echo "  Port: ${port}"
    echo "  Storage: ${storage_dir}"
    echo "  Log: ${log_file}"

    # Start the node - use minimal config and override with command line
    RUST_LOG=info \
    "$BINARY" \
        --config "${SCRIPT_DIR}/minimal_config.toml" \
        --storage "${storage_dir}" \
        --listen "0.0.0.0:${port}" \
        $bootstrap_args \
        > "${log_file}" 2>&1 &

    local pid=$!
    echo $pid >> "$PID_FILE"
    echo -e "  ${GREEN}✓${NC} Started with PID: $pid"
    echo ""
}

# Start first two nodes without bootstrap
echo -e "${YELLOW}Starting bootstrap nodes...${NC}\n"
start_node 1 "$NODE1_ID" ""
sleep 2
start_node 2 "$NODE2_ID" ""
sleep 3

# Start remaining nodes with bootstrap to first two
echo -e "${YELLOW}Starting remaining nodes with bootstrap...${NC}\n"
start_node 3 "$NODE3_ID" "-b $NODE1_ID -b $NODE2_ID"
sleep 2
start_node 4 "$NODE4_ID" "-b $NODE1_ID -b $NODE2_ID"
sleep 2
start_node 5 "$NODE5_ID" "-b $NODE1_ID -b $NODE2_ID"

echo -e "\n${GREEN}✅ All nodes started!${NC}"
echo "=================================="
echo ""

# Check if nodes are actually running
sleep 3
echo -e "${BLUE}Checking node status...${NC}"
alive_count=0
if [ -f "$PID_FILE" ]; then
    while read pid; do
        if kill -0 $pid 2>/dev/null; then
            ((alive_count++))
            echo -e "  Node PID $pid: ${GREEN}Running${NC}"
        else
            echo -e "  Node PID $pid: ${RED}Stopped${NC}"
        fi
    done < "$PID_FILE"
fi

echo ""
echo "Total nodes running: $alive_count/5"
echo ""

if [ $alive_count -eq 0 ]; then
    echo -e "${RED}⚠️  All nodes have stopped. Checking logs...${NC}"
    echo ""
    for i in {1..5}; do
        echo "Last error from node $i:"
        tail -n 3 "${LOG_DIR}/node${i}.log" | grep -E "ERROR|WARN" || echo "  No errors found"
        echo ""
    done
    exit 1
fi

echo "=================================="
echo -e "${YELLOW}Commands:${NC}"
echo "  View all logs:     tail -f ${LOG_DIR}/node*.log"
echo "  View node 1 log:   tail -f ${LOG_DIR}/node1.log"
echo "  Check processes:   ps aux | grep communitas-headless"
echo "  Stop testnet:      Press Ctrl+C"
echo ""
echo -e "${GREEN}Testnet is running. Press Ctrl+C to stop all nodes.${NC}"

# Keep running and periodically check status
while true; do
    sleep 30
    # Check if processes are still running
    if [ -f "$PID_FILE" ]; then
        alive_count=0
        while read pid; do
            if kill -0 $pid 2>/dev/null; then
                ((alive_count++))
            fi
        done < "$PID_FILE"

        if [ $alive_count -eq 0 ]; then
            echo -e "\n${RED}All nodes have stopped unexpectedly${NC}"
            exit 1
        fi
    fi
done