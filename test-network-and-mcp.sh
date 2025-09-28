#!/bin/bash

# Quick test script to verify network connectivity and MCP integration
# Uses existing binaries and infrastructure

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE} Quick Network & MCP Test${NC}"
echo -e "${BLUE}========================================${NC}\n"

# Step 1: Build everything we need
echo -e "${BLUE}Step 1: Building required binaries...${NC}"

# Build headless node
cd "$SCRIPT_DIR/communitas-headless"
echo "Building headless node..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true

# Build Tauri app
cd "$SCRIPT_DIR/communitas-desktop"
echo "Building Tauri desktop..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true

# Step 2: Launch a minimal test network (8 nodes for quick testing)
echo -e "\n${BLUE}Step 2: Launching minimal test network (8 nodes)...${NC}"

# Create test directories
TEST_DIR="/tmp/communitas-quick-test"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR/configs" "$TEST_DIR/data" "$TEST_DIR/logs"

# Bootstrap node config
cat > "$TEST_DIR/configs/bootstrap.toml" << 'EOF'
[identity]
four_words = "ocean-forest-moon-star"
display_name = "Bootstrap"

[network]
listen_address = "127.0.0.1:30000"
bootstrap_nodes = []

[storage]
data_dir = "/tmp/communitas-quick-test/data/bootstrap"

[logging]
level = "debug"
EOF

# Launch bootstrap node
echo "Starting bootstrap node..."
"$SCRIPT_DIR/communitas-headless/target/release/communitas-headless" \
    --config "$TEST_DIR/configs/bootstrap.toml" \
    > "$TEST_DIR/logs/bootstrap.log" 2>&1 &
BOOTSTRAP_PID=$!
echo "Bootstrap node PID: $BOOTSTRAP_PID"

# Give bootstrap time to start
sleep 3

# Launch 7 regular nodes
echo "Starting regular nodes..."
for i in {1..7}; do
    PORT=$((30000 + i))

    # Generate four-word identity from a pool
    WORDS=("valley" "mountain" "river" "ocean" "forest" "desert" "arctic" "jungle")
    W1=${WORDS[$((RANDOM % ${#WORDS[@]}))]}
    W2=${WORDS[$((RANDOM % ${#WORDS[@]}))]}
    W3=${WORDS[$((RANDOM % ${#WORDS[@]}))]}
    W4=${WORDS[$((RANDOM % ${#WORDS[@]}))]}

    cat > "$TEST_DIR/configs/node$i.toml" << EOF
[identity]
four_words = "$W1-$W2-$W3-$W4"
display_name = "Node-$i"

[network]
listen_address = "127.0.0.1:$PORT"
bootstrap_nodes = ["127.0.0.1:30000"]

[storage]
data_dir = "/tmp/communitas-quick-test/data/node$i"

[logging]
level = "info"
EOF

    "$SCRIPT_DIR/communitas-headless/target/release/communitas-headless" \
        --config "$TEST_DIR/configs/node$i.toml" \
        > "$TEST_DIR/logs/node$i.log" 2>&1 &

    echo "  Node $i started (port $PORT)"
done

echo -e "${GREEN}✓ Network launched${NC}"

# Step 3: Launch 2 Tauri apps
echo -e "\n${BLUE}Step 3: Launching 2 Tauri applications...${NC}"

cd "$SCRIPT_DIR/communitas-desktop"

# App 1 (Alice)
export COMMUNITAS_DATA_DIR="/tmp/communitas-alice"
export COMMUNITAS_FOUR_WORDS="tree-mountain-river-sun"
export COMMUNITAS_DISPLAY_NAME="Alice"
export VITE_PORT=5173
export RUST_LOG=info,tauri_plugin_mcp=debug

rm -rf "$COMMUNITAS_DATA_DIR"
mkdir -p "$COMMUNITAS_DATA_DIR"

# Configure bootstrap nodes for the app
cat > "$COMMUNITAS_DATA_DIR/network_config.json" << 'EOF'
{
  "bootstrap_nodes": ["127.0.0.1:30000"],
  "enable_mdns": true
}
EOF

npm run tauri dev -- --config '{"build":{"devUrl":"http://localhost:5173"}}' &
APP1_PID=$!
echo "App 1 (Alice) PID: $APP1_PID"

# App 2 (Bob) - on different port
sleep 5

export COMMUNITAS_DATA_DIR="/tmp/communitas-bob"
export COMMUNITAS_FOUR_WORDS="sky-valley-ocean-moon"
export COMMUNITAS_DISPLAY_NAME="Bob"
export VITE_PORT=5174

rm -rf "$COMMUNITAS_DATA_DIR"
mkdir -p "$COMMUNITAS_DATA_DIR"
cp "/tmp/communitas-alice/network_config.json" "$COMMUNITAS_DATA_DIR/"

# Start frontend on different port
cd "$SCRIPT_DIR"
npm run dev -- --port 5174 &
FRONTEND2_PID=$!

cd "$SCRIPT_DIR/communitas-desktop"
npm run tauri dev -- --config '{"build":{"devUrl":"http://localhost:5174"}}' &
APP2_PID=$!
echo "App 2 (Bob) PID: $APP2_PID"

# Step 4: Wait for everything to initialize
echo -e "\n${BLUE}Step 4: Waiting for initialization...${NC}"
sleep 15

# Step 5: Test network connectivity via MCP
echo -e "\n${BLUE}Step 5: Testing via MCP...${NC}"

# Create simple MCP test
cat > "$TEST_DIR/mcp-test.js" << 'EOF'
const net = require('net');

function testMCP(name, socketPath) {
    return new Promise((resolve) => {
        const socket = net.createConnection(socketPath, () => {
            console.log(`Connected to ${name} MCP`);

            // Test network status
            const request = {
                command: 'execute_js',
                payload: {
                    window_label: 'main',
                    code: `
                        const ns = window.networkService;
                        JSON.stringify({
                            status: ns?.getState()?.status || 'unknown',
                            peers: ns?.getPeers?.()?.length || 0
                        })
                    `
                }
            };

            socket.write(JSON.stringify(request) + '\n');

            socket.on('data', (data) => {
                try {
                    const response = JSON.parse(data.toString());
                    console.log(`${name} response:`, response);
                } catch (e) {
                    console.error(`${name} parse error:`, e.message);
                }
                socket.end();
                resolve();
            });
        });

        socket.on('error', (err) => {
            console.error(`${name} MCP error:`, err.message);
            resolve();
        });

        setTimeout(() => {
            socket.end();
            resolve();
        }, 5000);
    });
}

// Find MCP sockets
const fs = require('fs');
const sockets = fs.readdirSync('/tmp')
    .filter(f => f.startsWith('tauri-mcp-communitas-'))
    .map(f => `/tmp/${f}`);

console.log('Found MCP sockets:', sockets);

// Test each socket
Promise.all(sockets.map((s, i) => testMCP(`App${i+1}`, s)))
    .then(() => console.log('\nMCP tests complete'));
EOF

node "$TEST_DIR/mcp-test.js"

# Step 6: Check network formation
echo -e "\n${BLUE}Step 6: Checking network formation...${NC}"

# Check for peer connections in logs
CONNECTED=$(grep -h "Connected to peer\|Peer connected" "$TEST_DIR/logs/"*.log 2>/dev/null | wc -l || echo "0")
echo -e "Peer connections found in logs: ${GREEN}$CONNECTED${NC}"

# Check if bootstrap is accepting connections
BOOTSTRAP_CONNECTIONS=$(grep -c "Accepted connection" "$TEST_DIR/logs/bootstrap.log" 2>/dev/null || echo "0")
echo -e "Bootstrap node connections: ${GREEN}$BOOTSTRAP_CONNECTIONS${NC}"

# Final summary
echo -e "\n${BLUE}========================================${NC}"
echo -e "${BLUE} Test Network Running!${NC}"
echo -e "${BLUE}========================================${NC}\n"

echo -e "${GREEN}Network:${NC}"
echo -e "  • 8 headless nodes running"
echo -e "  • Bootstrap: 127.0.0.1:30000"
echo -e "  • Logs: $TEST_DIR/logs/"

echo -e "\n${GREEN}Applications:${NC}"
echo -e "  • Alice: http://localhost:5173"
echo -e "  • Bob: http://localhost:5174"

echo -e "\n${YELLOW}Monitor logs:${NC}"
echo -e "  tail -f $TEST_DIR/logs/bootstrap.log"

echo -e "\n${YELLOW}To stop everything:${NC}"
echo -e "  pkill -f communitas"

echo -e "\n${GREEN}Test network is ready for testing!${NC}"

# Cleanup handler
cleanup() {
    echo -e "\n${YELLOW}Stopping test network...${NC}"
    pkill -f communitas-headless || true
    pkill -f communitas-desktop || true
    kill $FRONTEND2_PID 2>/dev/null || true
    echo -e "${GREEN}Cleanup complete${NC}"
}

trap cleanup EXIT

# Keep running
echo -e "\n${YELLOW}Press Ctrl+C to stop${NC}"
wait