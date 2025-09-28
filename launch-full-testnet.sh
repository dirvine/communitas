#!/bin/bash

# Comprehensive Test Network Orchestrator for Communitas
# Sets up 50 headless nodes and 2 Tauri app instances with MCP monitoring

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_NET_DIR="/tmp/communitas-test-network-50"
MCP_TEST_DIR="/tmp/communitas-mcp-tests"
LOGS_DIR="/tmp/communitas-test-logs"

# Configuration
NUM_NODES=50
BASE_PORT=20000
NUM_BOOTSTRAP=3
NUM_APPS=2

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Communitas Full Test Network Launcher   ║${NC}"
echo -e "${BLUE}║      50 Nodes + 2 Apps + MCP Monitor      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}\n"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up test network...${NC}"

    # Stop all headless nodes
    if [ -f "$TEST_NET_DIR/pids.txt" ]; then
        while read pid; do
            kill $pid 2>/dev/null || true
        done < "$TEST_NET_DIR/pids.txt"
    fi

    # Stop Tauri apps
    pkill -f "communitas-desktop" || true
    pkill -f "tauri dev" || true

    # Clean up MCP sockets
    rm -f /tmp/tauri-mcp-*.sock

    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

trap cleanup EXIT

# Step 1: Prepare directories
echo -e "${BLUE}[1/7] Preparing directories...${NC}"
rm -rf "$TEST_NET_DIR" "$MCP_TEST_DIR" "$LOGS_DIR"
mkdir -p "$TEST_NET_DIR" "$MCP_TEST_DIR" "$LOGS_DIR"
mkdir -p "$TEST_NET_DIR/configs"
mkdir -p "$TEST_NET_DIR/data"

# Step 2: Build the headless binary if needed
echo -e "${BLUE}[2/7] Building headless node binary...${NC}"
cd "$SCRIPT_DIR/communitas-headless"

if [ ! -f "target/release/communitas-headless" ]; then
    cargo build --release
fi
cp target/release/communitas-headless "$TEST_NET_DIR/"

# Step 3: Generate node configurations
echo -e "${BLUE}[3/7] Generating configurations for $NUM_NODES nodes...${NC}"

# Generate bootstrap nodes with fixed four-word identities
BOOTSTRAP_NODES=""
for i in $(seq 1 $NUM_BOOTSTRAP); do
    PORT=$((BASE_PORT + i - 1))

    # Generate consistent four-word identities for bootstrap nodes
    case $i in
        1) FOUR_WORDS="ocean-forest-moon-star" ;;
        2) FOUR_WORDS="mountain-river-sun-cloud" ;;
        3) FOUR_WORDS="valley-lake-earth-wind" ;;
    esac

    echo -e "Bootstrap $i: ${GREEN}$FOUR_WORDS${NC} on port $PORT"

    cat > "$TEST_NET_DIR/configs/node$i.toml" << EOF
# Bootstrap Node $i Configuration
[identity]
four_words = "$FOUR_WORDS"
display_name = "Bootstrap-$i"

[network]
listen_address = "0.0.0.0:$PORT"
bootstrap_nodes = []
enable_mdns = false
enable_relay = false
max_connections = 100

[storage]
data_dir = "$TEST_NET_DIR/data/node$i"
cache_size = 104857600

[logging]
level = "info"
file = "$LOGS_DIR/node$i.log"
EOF

    if [ ! -z "$BOOTSTRAP_NODES" ]; then
        BOOTSTRAP_NODES="$BOOTSTRAP_NODES,"
    fi
    BOOTSTRAP_NODES="${BOOTSTRAP_NODES}\"/ip4/127.0.0.1/udp/$PORT/quic-v1\""
done

# Generate regular nodes
echo -e "${CYAN}Generating regular nodes...${NC}"
for i in $(seq $((NUM_BOOTSTRAP + 1)) $NUM_NODES); do
    PORT=$((BASE_PORT + i - 1))

    # Generate random four-word identity for regular nodes
    # Using a simple approach - in production would use four-word-networking crate
    WORDS=("ocean" "forest" "moon" "star" "mountain" "river" "sun" "cloud"
           "valley" "lake" "earth" "wind" "storm" "desert" "jungle" "arctic")

    W1=${WORDS[$((RANDOM % ${#WORDS[@]}))]}
    W2=${WORDS[$((RANDOM % ${#WORDS[@]}))]}
    W3=${WORDS[$((RANDOM % ${#WORDS[@]}))]}
    W4=${WORDS[$((RANDOM % ${#WORDS[@]}))]}
    FOUR_WORDS="$W1-$W2-$W3-$W4"

    cat > "$TEST_NET_DIR/configs/node$i.toml" << EOF
# Node $i Configuration
[identity]
four_words = "$FOUR_WORDS"
display_name = "Node-$i"

[network]
listen_address = "0.0.0.0:$PORT"
bootstrap_nodes = [$BOOTSTRAP_NODES]
enable_mdns = true
enable_relay = true
max_connections = 50

[storage]
data_dir = "$TEST_NET_DIR/data/node$i"
cache_size = 52428800

[logging]
level = "info"
file = "$LOGS_DIR/node$i.log"
EOF

    # Show progress every 10 nodes
    if [ $((i % 10)) -eq 0 ]; then
        echo -e "  Generated configs for ${GREEN}$i/$NUM_NODES${NC} nodes"
    fi
done

# Step 4: Launch headless nodes
echo -e "\n${BLUE}[4/7] Launching $NUM_NODES headless nodes...${NC}"

echo -n "" > "$TEST_NET_DIR/pids.txt"

# Launch bootstrap nodes first
for i in $(seq 1 $NUM_BOOTSTRAP); do
    CONFIG="$TEST_NET_DIR/configs/node$i.toml"
    LOG="$LOGS_DIR/node$i.log"

    RUST_LOG=info "$TEST_NET_DIR/communitas-headless" \
        --config "$CONFIG" \
        > "$LOG" 2>&1 &

    PID=$!
    echo $PID >> "$TEST_NET_DIR/pids.txt"
    echo -e "  Bootstrap $i started (PID: ${GREEN}$PID${NC})"
done

# Wait for bootstrap nodes to initialize
echo -e "${YELLOW}Waiting for bootstrap nodes to initialize...${NC}"
sleep 5

# Launch regular nodes in batches to avoid overwhelming the system
BATCH_SIZE=10
for batch_start in $(seq $((NUM_BOOTSTRAP + 1)) $BATCH_SIZE $NUM_NODES); do
    batch_end=$((batch_start + BATCH_SIZE - 1))
    if [ $batch_end -gt $NUM_NODES ]; then
        batch_end=$NUM_NODES
    fi

    echo -e "${CYAN}Launching nodes $batch_start-$batch_end...${NC}"

    for i in $(seq $batch_start $batch_end); do
        CONFIG="$TEST_NET_DIR/configs/node$i.toml"
        LOG="$LOGS_DIR/node$i.log"

        RUST_LOG=info "$TEST_NET_DIR/communitas-headless" \
            --config "$CONFIG" \
            > "$LOG" 2>&1 &

        PID=$!
        echo $PID >> "$TEST_NET_DIR/pids.txt"
    done

    # Small delay between batches
    sleep 2
done

echo -e "${GREEN}✓ All $NUM_NODES nodes launched${NC}"

# Step 5: Verify network formation
echo -e "\n${BLUE}[5/7] Verifying network formation...${NC}"
sleep 10

# Check node connectivity by sampling logs
CONNECTED_NODES=0
for i in $(seq 1 5); do
    if [ -f "$LOGS_DIR/node$i.log" ]; then
        if grep -q "Connected to peer" "$LOGS_DIR/node$i.log"; then
            CONNECTED_NODES=$((CONNECTED_NODES + 1))
        fi
    fi
done

echo -e "  Sample check: ${GREEN}$CONNECTED_NODES/5${NC} nodes showing peer connections"

# Step 6: Launch Tauri applications
echo -e "\n${BLUE}[6/7] Launching $NUM_APPS Tauri applications...${NC}"

cd "$SCRIPT_DIR/communitas-desktop"

# Build if needed
if [ ! -d "node_modules" ]; then
    npm install
fi

# Launch App 1
echo -e "${CYAN}Launching App 1 (Alice)...${NC}"
export COMMUNITAS_DATA_DIR="/tmp/communitas-app1"
export COMMUNITAS_FOUR_WORDS="tree-mountain-river-sun"
export COMMUNITAS_DISPLAY_NAME="Alice"
export VITE_PORT=5173
export TAURI_MCP_SOCKET_PATH="/tmp/tauri-mcp-app1.sock"
export RUST_LOG=info,communitas=debug,tauri_plugin_mcp=debug

rm -rf "$COMMUNITAS_DATA_DIR"
mkdir -p "$COMMUNITAS_DATA_DIR"

# Add bootstrap nodes to app config
cat > "$COMMUNITAS_DATA_DIR/bootstrap.json" << EOF
{
  "bootstrap_nodes": [
    "/ip4/127.0.0.1/udp/$BASE_PORT/quic-v1",
    "/ip4/127.0.0.1/udp/$((BASE_PORT + 1))/quic-v1",
    "/ip4/127.0.0.1/udp/$((BASE_PORT + 2))/quic-v1"
  ]
}
EOF

npm run tauri dev -- --config '{"build":{"devUrl":"http://localhost:5173"}}' &
APP1_PID=$!
echo -e "  App 1 started (PID: ${GREEN}$APP1_PID${NC})"

sleep 5

# Launch App 2
echo -e "${CYAN}Launching App 2 (Bob)...${NC}"
export COMMUNITAS_DATA_DIR="/tmp/communitas-app2"
export COMMUNITAS_FOUR_WORDS="sky-valley-ocean-moon"
export COMMUNITAS_DISPLAY_NAME="Bob"
export VITE_PORT=5174
export TAURI_MCP_SOCKET_PATH="/tmp/tauri-mcp-app2.sock"

rm -rf "$COMMUNITAS_DATA_DIR"
mkdir -p "$COMMUNITAS_DATA_DIR"

# Add bootstrap nodes to app config
cp "/tmp/communitas-app1/bootstrap.json" "$COMMUNITAS_DATA_DIR/"

# Run frontend on different port
cd "$SCRIPT_DIR"
npm run dev -- --port 5174 &
FRONTEND2_PID=$!

cd "$SCRIPT_DIR/communitas-desktop"
npm run tauri dev -- --config '{"build":{"devUrl":"http://localhost:5174"}}' &
APP2_PID=$!
echo -e "  App 2 started (PID: ${GREEN}$APP2_PID${NC})"

# Step 7: Create MCP test harness
echo -e "\n${BLUE}[7/7] Setting up MCP test harness...${NC}"

cat > "$MCP_TEST_DIR/mcp-network-test.js" << 'EOF'
#!/usr/bin/env node

const net = require('net');
const fs = require('fs');

class MCPTester {
    constructor(socketPath) {
        this.socketPath = socketPath;
        this.socket = null;
    }

    async connect() {
        return new Promise((resolve, reject) => {
            this.socket = net.createConnection(this.socketPath, () => {
                console.log(`Connected to MCP at ${this.socketPath}`);
                resolve();
            });

            this.socket.on('error', reject);

            this.socket.on('data', (data) => {
                const response = JSON.parse(data.toString());
                console.log('MCP Response:', response);
            });
        });
    }

    async sendCommand(command, payload = {}) {
        return new Promise((resolve) => {
            const request = {
                command: command,
                payload: { window_label: 'main', ...payload }
            };

            this.socket.write(JSON.stringify(request) + '\n');

            this.socket.once('data', (data) => {
                const response = JSON.parse(data.toString());
                resolve(response);
            });
        });
    }

    async testPeerCache() {
        console.log('\n=== Testing Peer Cache ===');

        // Execute JS to check peer cache
        const result = await this.sendCommand('execute_js', {
            code: `
                // Access the network service
                const state = window.networkService?.getState();
                const peers = window.networkService?.getPeers?.() || [];

                JSON.stringify({
                    status: state?.status || 'unknown',
                    peerCount: peers.length,
                    peers: peers.slice(0, 5).map(p => ({
                        id: p.id,
                        address: p.address,
                        latency: p.latency
                    }))
                })
            `
        });

        return result;
    }

    async testNetworkConnectivity() {
        console.log('\n=== Testing Network Connectivity ===');

        const result = await this.sendCommand('execute_js', {
            code: `
                // Check network status
                const indicator = document.querySelector('[data-testid="network-status"]');
                const status = indicator?.getAttribute('data-status') || 'not-found';

                // Get connection details
                const connectionInfo = window.networkService?.getConnectionInfo?.() || {};

                JSON.stringify({
                    uiStatus: status,
                    bootstrapNodes: connectionInfo.bootstrapNodes || [],
                    connectedPeers: connectionInfo.connectedPeers || 0,
                    networkId: connectionInfo.networkId || null
                })
            `
        });

        return result;
    }

    async takeScreenshot() {
        console.log('\n=== Taking Screenshot ===');
        const result = await this.sendCommand('take_screenshot', {
            format: 'png'
        });

        if (result.success && result.data) {
            const filename = `screenshot-${Date.now()}.png`;
            fs.writeFileSync(filename, Buffer.from(result.data.value, 'base64'));
            console.log(`Screenshot saved to ${filename}`);
        }

        return result;
    }

    disconnect() {
        if (this.socket) {
            this.socket.end();
        }
    }
}

// Run tests for both apps
async function runTests() {
    console.log('Starting MCP Network Tests...\n');

    // Test App 1
    console.log('=== Testing App 1 (Alice) ===');
    const tester1 = new MCPTester('/tmp/tauri-mcp-app1.sock');

    try {
        await tester1.connect();
        await tester1.testNetworkConnectivity();
        await tester1.testPeerCache();
        await tester1.takeScreenshot();
    } catch (error) {
        console.error('App 1 test error:', error.message);
    } finally {
        tester1.disconnect();
    }

    // Test App 2
    console.log('\n=== Testing App 2 (Bob) ===');
    const tester2 = new MCPTester('/tmp/tauri-mcp-app2.sock');

    try {
        await tester2.connect();
        await tester2.testNetworkConnectivity();
        await tester2.testPeerCache();
        await tester2.takeScreenshot();
    } catch (error) {
        console.error('App 2 test error:', error.message);
    } finally {
        tester2.disconnect();
    }
}

// Wait for apps to initialize then run tests
setTimeout(() => {
    runTests().then(() => {
        console.log('\nMCP tests completed');
    }).catch(console.error);
}, 5000);
EOF

chmod +x "$MCP_TEST_DIR/mcp-network-test.js"

# Wait for apps to initialize
echo -e "${YELLOW}Waiting for applications to initialize...${NC}"
sleep 15

# Run MCP tests
echo -e "\n${BLUE}Running MCP network tests...${NC}"
node "$MCP_TEST_DIR/mcp-network-test.js"

# Generate final report
echo -e "\n${BLUE}═══════════════════════════════════════════${NC}"
echo -e "${BLUE} Test Network Successfully Deployed!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════${NC}\n"

echo -e "${GREEN}Network Topology:${NC}"
echo -e "  • ${CYAN}$NUM_NODES${NC} headless nodes running"
echo -e "  • ${CYAN}$NUM_BOOTSTRAP${NC} bootstrap nodes"
echo -e "  • ${CYAN}$NUM_APPS${NC} Tauri applications"

echo -e "\n${GREEN}Bootstrap Nodes:${NC}"
for i in $(seq 1 $NUM_BOOTSTRAP); do
    PORT=$((BASE_PORT + i - 1))
    case $i in
        1) FOUR_WORDS="ocean-forest-moon-star" ;;
        2) FOUR_WORDS="mountain-river-sun-cloud" ;;
        3) FOUR_WORDS="valley-lake-earth-wind" ;;
    esac
    echo -e "  • 127.0.0.1:${CYAN}$PORT${NC} - ${GREEN}$FOUR_WORDS${NC}"
done

echo -e "\n${GREEN}Applications:${NC}"
echo -e "  • App 1 (Alice): http://localhost:${CYAN}5173${NC}"
echo -e "    Four-words: ${GREEN}tree-mountain-river-sun${NC}"
echo -e "    MCP Socket: ${CYAN}/tmp/tauri-mcp-app1.sock${NC}"
echo -e "  • App 2 (Bob): http://localhost:${CYAN}5174${NC}"
echo -e "    Four-words: ${GREEN}sky-valley-ocean-moon${NC}"
echo -e "    MCP Socket: ${CYAN}/tmp/tauri-mcp-app2.sock${NC}"

echo -e "\n${GREEN}Log Files:${NC}"
echo -e "  • Node logs: ${CYAN}$LOGS_DIR/node*.log${NC}"
echo -e "  • PIDs: ${CYAN}$TEST_NET_DIR/pids.txt${NC}"

echo -e "\n${GREEN}MCP Test Harness:${NC}"
echo -e "  • Test script: ${CYAN}$MCP_TEST_DIR/mcp-network-test.js${NC}"
echo -e "  • Run again: ${CYAN}node $MCP_TEST_DIR/mcp-network-test.js${NC}"

echo -e "\n${YELLOW}Commands:${NC}"
echo -e "  • Monitor network: ${CYAN}tail -f $LOGS_DIR/node1.log${NC}"
echo -e "  • Check peer count: ${CYAN}grep 'Connected to peer' $LOGS_DIR/*.log | wc -l${NC}"
echo -e "  • Stop everything: ${CYAN}Ctrl+C${NC} or run cleanup"

echo -e "\n${YELLOW}Press Ctrl+C to stop the test network${NC}"

# Keep script running
wait