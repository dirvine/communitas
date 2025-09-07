#!/bin/bash

# Collect Node IP Addresses Script
# Run this on a Linux machine with SSH access to the Digital Ocean containers

set -e

# Configuration
APP_ID="18ccbf3a-7111-4897-a0b6-f215910bcf1a"
SSH_KEY_PATH="$HOME/.ssh/id_rsa"

# Node configuration
NODES=(
    "bootstrap-coordinator:8080"
    "node-nyc:8081"
    "node-sfo:8082"
    "node-lon:8083"
    "node-fra:8084"
    "node-sgp:8085"
    "node-syd:8086"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Collecting IP addresses from Communitas testnet nodes${NC}"
echo "======================================================"

# Function to get IP from a node
get_node_ip() {
    local component=$1
    local port=$2

    echo -e "${YELLOW}📡 Connecting to $component...${NC}"

    # Try to SSH into the container and get IP
    if ssh -i "$SSH_KEY_PATH" -o ConnectTimeout=10 -o StrictHostKeyChecking=no "root@app-$APP_ID-$component.ondigitalocean.app" "hostname -i | awk '{print \$1}'" 2>/dev/null; then
        echo -e "${GREEN}✅ Got IP from $component${NC}"
    else
        echo -e "${RED}❌ Failed to connect to $component${NC}"
        echo "   Make sure SSH key is set up and container is running"
        return 1
    fi
}

# Function to test connectivity
test_connectivity() {
    local component=$1
    local port=$2

    echo -e "${YELLOW}🔗 Testing connectivity to $component...${NC}"

    if ssh -i "$SSH_KEY_PATH" -o ConnectTimeout=5 "root@app-$APP_ID-$component.ondigitalocean.app" "echo 'Connection successful'" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $component is accessible${NC}"
        return 0
    else
        echo -e "${RED}❌ $component is not accessible${NC}"
        return 1
    fi
}

# Main collection process
main() {
    echo -e "${BLUE}📋 Node Configuration:${NC}"
    for node_info in "${NODES[@]}"; do
        IFS=':' read -r component port <<< "$node_info"
        echo "  $component (port: $port)"
    done
    echo ""

    # Test connectivity first
    echo -e "${BLUE}🔗 Testing connectivity to all nodes...${NC}"
    CONNECTED_NODES=()
    for node_info in "${NODES[@]}"; do
        IFS=':' read -r component port <<< "$node_info"
        if test_connectivity "$component" "$port"; then
            CONNECTED_NODES+=("$node_info")
        fi
    done
    echo ""

    # Collect IPs
    echo -e "${BLUE}📊 Collecting IP addresses...${NC}"
    NODE_IPS=()

    for node_info in "${CONNECTED_NODES[@]}"; do
        IFS=':' read -r component port <<< "$node_info"
        ip=$(get_node_ip "$component" "$port")
        if [ $? -eq 0 ] && [ -n "$ip" ]; then
            NODE_IPS+=("$component:$ip:$port")
        fi
    done

    # Display results
    echo ""
    echo -e "${BLUE}📋 Collected Node Information:${NC}"
    echo "=============================="

    for node_data in "${NODE_IPS[@]}"; do
        IFS=':' read -r component ip port <<< "$node_data"
        echo -e "${GREEN}$component${NC}: $ip:$port"
    done

    # Generate bootstrap config snippet
    echo ""
    echo -e "${BLUE}📝 Bootstrap Configuration Snippet:${NC}"
    echo "====================================="

    for i in "${!NODE_IPS[@]}"; do
        node_data="${NODE_IPS[$i]}"
        IFS=':' read -r component ip port <<< "$node_data"

        if [ "$component" = "bootstrap-coordinator" ]; then
            echo "[[nodes]]"
            echo "name = \"$component\""
            echo "internal_ip = \"$ip\""
            echo "port = $port"
            echo "region = \"nyc\""
            echo "role = \"bootstrap\""
            echo ""
        else
            region=$(echo "$component" | sed 's/node-//')
            echo "[[nodes]]"
            echo "name = \"$component\""
            echo "internal_ip = \"$ip\""
            echo "port = $port"
            echo "region = \"$region\""
            echo "role = \"worker\""
            echo ""
        fi
    done

    # Summary
    echo -e "${GREEN}✅ Collection complete!${NC}"
    echo -e "${YELLOW}📊 Summary:${NC}"
    echo "  Connected nodes: ${#CONNECTED_NODES[@]}/${#NODES[@]}"
    echo "  IPs collected: ${#NODE_IPS[@]}/${#NODES[@]}"

    if [ ${#NODE_IPS[@]} -eq ${#NODES[@]} ]; then
        echo -e "${GREEN}🎉 All nodes are ready for deployment!${NC}"
    else
        echo -e "${YELLOW}⚠️  Some nodes may need attention${NC}"
    fi
}

# Check if SSH key exists
if [ ! -f "$SSH_KEY_PATH" ]; then
    echo -e "${RED}❌ SSH key not found at $SSH_KEY_PATH${NC}"
    echo -e "${YELLOW}💡 Make sure your SSH key is set up for Digital Ocean access${NC}"
    exit 1
fi

# Run main function
main "$@"