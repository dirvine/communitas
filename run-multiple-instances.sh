#!/bin/bash

# Run Multiple Communitas Instances for Testing
# This script sets up multiple local instances connected to the remote testnet

set -e

# Configuration
BOOTSTRAP_FILE="bootstrap.toml"
INSTANCES=${1:-3}  # Default to 3 instances

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}🚀 Starting $INSTANCES Communitas instances${NC}"

# Check if bootstrap file exists
if [ ! -f "$BOOTSTRAP_FILE" ]; then
    echo -e "${YELLOW}⚠️  Bootstrap file not found. Using default configuration.${NC}"
    cat > "$BOOTSTRAP_FILE" << EOF
seeds = [
  "ocean-forest-moon-star:443",
  "river-mountain-sun-cloud:443",
]
EOF
fi

# Function to run a single instance
run_instance() {
    local instance_num=$1
    local port_offset=$((instance_num - 1))
    local dev_port=$((1420 + port_offset))
    local p2p_port=$((9000 + port_offset))

    echo -e "${BLUE}📱 Starting Instance $instance_num (Port: $dev_port, P2P: $p2p_port)${NC}"

    # Create instance-specific data directory
    mkdir -p "instance-$instance_num-data"

    # Set environment variables
    export COMMUNITAS_LOCAL_BOOTSTRAP="$(pwd)/$BOOTSTRAP_FILE"
    export COMMUNITAS_P2P_PORT="$p2p_port"
    export COMMUNITAS_DATA_DIR="$(pwd)/instance-$instance_num-data"
    export RUST_LOG="info,communitas=debug,saorsa_core=info"

    # Run in background
    npm run tauri dev -- --port "$dev_port" &
    echo $! > "instance-$instance_num.pid"
}

# Start instances
for i in $(seq 1 "$INSTANCES"); do
    run_instance "$i"
    sleep 2  # Stagger startup
done

echo -e "${GREEN}✅ All instances started!${NC}"
echo ""
echo "Instance URLs:"
for i in $(seq 1 "$INSTANCES"); do
    port=$((1420 + i - 1))
    echo "  Instance $i: http://localhost:$port"
done

echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all instances${NC}"

# Wait for Ctrl+C
trap 'echo -e "\n${GREEN}🛑 Stopping all instances...${NC}"; kill_instances; exit 0' INT

kill_instances() {
    for i in $(seq 1 "$INSTANCES"); do
        if [ -f "instance-$i.pid" ]; then
            kill "$(cat "instance-$i.pid")" 2>/dev/null || true
            rm "instance-$i.pid"
        fi
    done
    echo "All instances stopped."
}

# Keep script running
wait