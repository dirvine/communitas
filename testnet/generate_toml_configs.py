#!/usr/bin/env python3
"""Generate TOML testnet configuration for Communitas nodes"""

import random
import os
from pathlib import Path

# Word lists for generating four-word identities
WORD_LIST = [
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
    "ocean", "forest", "mountain", "river", "valley", "desert", "lake", "meadow",
    "star", "moon", "sun", "comet", "nebula", "galaxy", "meteor", "planet",
    "eagle", "falcon", "hawk", "owl", "raven", "wolf", "bear", "lion",
    "north", "south", "east", "west", "center", "edge", "peak", "base"
]

def generate_four_words():
    """Generate a unique four-word identity"""
    return "-".join(random.sample(WORD_LIST, 4))

def create_node_toml_config(node_num, bootstrap_nodes=None):
    """Create TOML configuration for a single node"""

    # Generate unique identity
    four_words = generate_four_words()

    # Base port assignments (each node gets a range)
    base_port = 9000 + (node_num - 1) * 10
    quic_port = base_port

    # Create TOML config
    config = f'''# Communitas Headless Node Configuration
# Node {node_num}: {four_words}

# Identity configuration
[identity]
four_words = "{four_words}"
display_name = "TestNode{node_num}"

# Network configuration
[network]
listen_address = "0.0.0.0:{quic_port}"
bootstrap_nodes = ['''

    # Add bootstrap nodes if provided
    if bootstrap_nodes:
        bootstrap_list = ', '.join([f'"{node}"' for node in bootstrap_nodes])
        config += f"\n    {bootstrap_list}\n"
    else:
        config += "\n"

    config += f''']

# Storage configuration
[storage]
path = "./data"
max_size_gb = 10

# Logging configuration
[logging]
level = "info"
file = "./logs/node{node_num}.log"

# API configuration
[api]
enabled = true
address = "127.0.0.1:{quic_port + 2}"
'''

    return config, four_words, quic_port

def main():
    """Generate TOML testnet configuration"""

    print("🚀 Generating Communitas TOML testnet configuration...")

    nodes = {}
    bootstrap_nodes = []

    # Generate configurations for 5 nodes
    for i in range(1, 6):
        config_toml, four_words, quic_port = create_node_toml_config(i, bootstrap_nodes.copy())

        nodes[f"node{i}"] = {
            "four_words": four_words,
            "quic_port": quic_port,
            "api_port": quic_port + 2
        }

        # Write TOML configuration
        config_path = Path(f"testnet/node{i}/config.toml")
        config_path.parent.mkdir(parents=True, exist_ok=True)

        with open(config_path, 'w') as f:
            f.write(config_toml)

        print(f"✅ Created TOML config for node{i}:")
        print(f"   Identity: {four_words}")
        print(f"   QUIC Port: {quic_port}")
        print(f"   API Port: {quic_port + 2}")
        print(f"   Config: {config_path}")

        # Add first two nodes as bootstrap for others
        if i <= 2:
            bootstrap_nodes.append(four_words)

    # Create a simple startup script specific to TOML configs
    startup_script = '''#!/bin/bash

# Communitas TOML Testnet Launcher

set -e

# Colors for output
RED='\\033[0;31m'
GREEN='\\033[0;32m'
YELLOW='\\033[1;33m'
BLUE='\\033[0;34m'
NC='\\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BINARY="${PROJECT_ROOT}/target/release/communitas-headless"
PID_FILE="${SCRIPT_DIR}/testnet.pids"

# Cleanup function
cleanup() {
    echo -e "\\n${YELLOW}Stopping testnet...${NC}"
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
'''

    for i in range(1, 6):
        node_data = nodes[f"node{i}"]
        bootstrap_args = ""
        if i > 2:
            # Nodes 3-5 bootstrap from first 2 nodes
            bootstrap_args = f"-b {nodes['node1']['four_words']} -b {nodes['node2']['four_words']}"

        startup_script += f'''
echo -e "${{GREEN}}Starting Node {i}:${{NC}}"
echo "  Identity: {node_data['four_words']}"
echo "  Port: {node_data['quic_port']}"

"$BINARY" \\
    --config "${{SCRIPT_DIR}}/node{i}/config.toml" \\
    --storage "${{SCRIPT_DIR}}/node{i}/data" \\
    --listen "0.0.0.0:{node_data['quic_port']}" \\
    {bootstrap_args} \\
    > "${{SCRIPT_DIR}}/node{i}/node.log" 2>&1 &

PID=$!
echo $PID >> "$PID_FILE"
echo -e "  ${{GREEN}}✓${{NC}} Started with PID: $PID"

'''
        if i < 5:
            startup_script += 'sleep 2  # Wait before starting next node\n'

    startup_script += '''
echo -e "\\n${GREEN}✅ All nodes started!${NC}"
echo "=================================="
echo -e "${YELLOW}Commands:${NC}"
echo "  View logs:     tail -f ${SCRIPT_DIR}/node*/node.log"
echo "  Stop testnet:  Press Ctrl+C"
echo -e "${GREEN}Testnet is running. Press Ctrl+C to stop all nodes.${NC}"

# Keep running
while true; do
    sleep 10
done
'''

    # Write startup script
    script_path = Path("testnet/start_toml_testnet.sh")
    with open(script_path, 'w') as f:
        f.write(startup_script)

    os.chmod(script_path, 0o755)

    print("\n📋 TOML Testnet configuration complete!")
    print(f"📊 Total nodes: {len(nodes)}")
    print(f"🔗 Bootstrap nodes: 2 (nodes 1 and 2)")
    print(f"🚀 Run: ./testnet/start_toml_testnet.sh")

if __name__ == "__main__":
    main()