#!/bin/bash

# Test Connectivity Script
# Verifies connection between local instances and remote testnet

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}🔍 Testing Communitas Network Connectivity${NC}"

# Test local instances
echo -e "${YELLOW}Testing local instances...${NC}"
for i in {1..3}; do
    port=$((1420 + i - 1))
    if curl -s "http://localhost:$port/health" > /dev/null; then
        echo -e "${GREEN}✅ Instance $i (port $port) is healthy${NC}"
    else
        echo -e "${RED}❌ Instance $i (port $port) is not responding${NC}"
    fi
done

# Test remote bootstrap nodes
echo -e "${YELLOW}Testing remote bootstrap nodes...${NC}"
if [ -f "bootstrap.toml" ]; then
    # Extract seed addresses
    seeds=$(grep -o '"[^"]*"' bootstrap.toml | tr -d '"')

    for seed in $seeds; do
        addr=$(echo "$seed" | cut -d: -f1)
        port=$(echo "$seed" | cut -d: -f2)

        echo -e "${YELLOW}Testing $addr:$port...${NC}"
        # Note: This is a basic connectivity test
        # In production, you'd use proper peer discovery
        if timeout 5 bash -c "echo > /dev/tcp/$addr/$port" 2>/dev/null; then
            echo -e "${GREEN}✅ $addr:$port is reachable${NC}"
        else
            echo -e "${RED}❌ $addr:$port is not reachable${NC}"
        fi
    done
else
    echo -e "${RED}❌ bootstrap.toml not found${NC}"
fi

echo -e "${GREEN}Connectivity test complete${NC}"