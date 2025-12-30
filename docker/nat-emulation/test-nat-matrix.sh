#!/bin/bash
# NAT Connectivity Matrix Test for Communitas
# Copyright (c) 2025 Saorsa Labs Limited
#
# Tests connectivity between all NAT type combinations
#
# Usage:
#   ./test-nat-matrix.sh           # Full matrix test
#   ./test-nat-matrix.sh quick     # Quick connectivity check
#   ./test-nat-matrix.sh pair <a> <b>  # Test specific pair

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
COMMUNITAS_PORT=11000
TIMEOUT=5

# NAT node containers
NAT_NODES=(
    "communitas-node-public:172.20.100.1:Public"
    "communitas-node-fullcone:10.100.1.10:FullCone"
    "communitas-node-restricted:10.100.2.10:Restricted"
    "communitas-node-portrestricted:10.100.3.10:PortRestricted"
    "communitas-node-symmetric:10.100.4.10:Symmetric"
    "communitas-node-cgnat:10.100.5.10:CGNAT"
    "communitas-node-doublenat:10.100.6.10:DoubleNAT"
    "communitas-node-hairpin:10.100.7.10:Hairpin"
)

# External IPs for each NAT type (for testing through NAT)
declare -A EXTERNAL_IPS=(
    ["communitas-node-public"]="172.20.100.1"
    ["communitas-node-fullcone"]="172.20.1.1"
    ["communitas-node-restricted"]="172.20.2.1"
    ["communitas-node-portrestricted"]="172.20.3.1"
    ["communitas-node-symmetric"]="172.20.4.1"
    ["communitas-node-cgnat"]="172.20.5.1"
    ["communitas-node-doublenat"]="172.20.6.1"
    ["communitas-node-hairpin"]="172.20.7.1"
)

print_status() {
    local status=$1
    local message=$2
    case $status in
        "ok")     echo -e "${GREEN}✓${NC} $message" ;;
        "fail")   echo -e "${RED}✗${NC} $message" ;;
        "warn")   echo -e "${YELLOW}!${NC} $message" ;;
        "info")   echo -e "${BLUE}→${NC} $message" ;;
    esac
}

# Check if containers are running
check_containers() {
    echo -e "${BLUE}Checking container status...${NC}"
    local all_running=true

    for node_info in "${NAT_NODES[@]}"; do
        local container=$(echo "$node_info" | cut -d: -f1)
        if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
            print_status "ok" "$container running"
        else
            print_status "fail" "$container not running"
            all_running=false
        fi
    done

    if ! $all_running; then
        echo ""
        print_status "warn" "Start containers with: docker-compose up -d"
        exit 1
    fi
}

# Test UDP connectivity from source to target
test_udp_connectivity() {
    local source_container=$1
    local target_ip=$2
    local target_port=$3

    # Use nc (netcat) to test UDP connectivity
    # Send a test packet and check if we can reach the endpoint
    docker exec "$source_container" sh -c \
        "echo 'test' | timeout $TIMEOUT nc -u -w1 $target_ip $target_port 2>/dev/null && echo 'ok'" 2>/dev/null | grep -q "ok"
}

# Quick connectivity check
cmd_quick() {
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}            ${CYAN}NAT Quick Connectivity Check${NC}                      ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    check_containers
    echo ""

    echo -e "${CYAN}Testing connectivity to public node (172.20.100.1:$COMMUNITAS_PORT)${NC}"
    echo ""

    for node_info in "${NAT_NODES[@]}"; do
        local container=$(echo "$node_info" | cut -d: -f1)
        local nat_type=$(echo "$node_info" | cut -d: -f3)

        if [[ "$container" == "communitas-node-public" ]]; then
            continue
        fi

        # Test if container can reach public node
        if docker exec "$container" sh -c "ping -c 1 -W 2 172.20.100.1" &>/dev/null; then
            print_status "ok" "$nat_type -> Public (ICMP)"
        else
            print_status "fail" "$nat_type -> Public (ICMP blocked)"
        fi
    done
}

# Full matrix test
cmd_full() {
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}            ${CYAN}NAT Connectivity Matrix Test${NC}                      ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    check_containers
    echo ""

    # Print header
    echo -e "${CYAN}Connectivity Matrix (port $COMMUNITAS_PORT)${NC}"
    echo -e "Legend: ${GREEN}✓${NC} = reachable, ${RED}✗${NC} = blocked, ${YELLOW}?${NC} = conditional"
    echo ""

    # Extract NAT type names
    local nat_types=()
    for node_info in "${NAT_NODES[@]}"; do
        nat_types+=($(echo "$node_info" | cut -d: -f3))
    done

    # Print column headers
    printf "%-16s" "FROM \\ TO"
    for nat_type in "${nat_types[@]}"; do
        printf "%-14s" "$nat_type"
    done
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Test matrix
    for source_info in "${NAT_NODES[@]}"; do
        local source_container=$(echo "$source_info" | cut -d: -f1)
        local source_type=$(echo "$source_info" | cut -d: -f3)

        printf "%-16s" "$source_type"

        for target_info in "${NAT_NODES[@]}"; do
            local target_container=$(echo "$target_info" | cut -d: -f1)
            local target_type=$(echo "$target_info" | cut -d: -f3)
            local target_external=${EXTERNAL_IPS[$target_container]}

            if [[ "$source_container" == "$target_container" ]]; then
                printf "${BLUE}%-14s${NC}" "-"
            else
                # Test ICMP (basic connectivity)
                if docker exec "$source_container" sh -c "ping -c 1 -W 1 $target_external" &>/dev/null; then
                    printf "${GREEN}%-14s${NC}" "✓"
                else
                    printf "${RED}%-14s${NC}" "✗"
                fi
            fi
        done
        echo ""
    done

    echo ""
    echo -e "${CYAN}Expected Results:${NC}"
    echo "- Public can reach all nodes through their NAT"
    echo "- Full Cone allows bidirectional (easy hole-punch)"
    echo "- Restricted/Port-Restricted need coordination"
    echo "- Symmetric-to-Symmetric typically fails (needs relay)"
    echo "- Double NAT very restrictive"
}

# Test specific pair
cmd_pair() {
    local source_type=${1:-}
    local target_type=${2:-}

    if [[ -z "$source_type" || -z "$target_type" ]]; then
        echo "Usage: $0 pair <source-nat> <target-nat>"
        echo ""
        echo "NAT types: Public, FullCone, Restricted, PortRestricted, Symmetric, CGNAT, DoubleNAT, Hairpin"
        exit 1
    fi

    # Find containers
    local source_container=""
    local target_container=""
    local target_external=""

    for node_info in "${NAT_NODES[@]}"; do
        local container=$(echo "$node_info" | cut -d: -f1)
        local nat_type=$(echo "$node_info" | cut -d: -f3)

        if [[ "$nat_type" == "$source_type" ]]; then
            source_container=$container
        fi
        if [[ "$nat_type" == "$target_type" ]]; then
            target_container=$container
            target_external=${EXTERNAL_IPS[$container]}
        fi
    done

    if [[ -z "$source_container" ]]; then
        echo "Unknown source NAT type: $source_type"
        exit 1
    fi
    if [[ -z "$target_container" ]]; then
        echo "Unknown target NAT type: $target_type"
        exit 1
    fi

    echo -e "${BLUE}Testing: $source_type -> $target_type${NC}"
    echo "Source: $source_container"
    echo "Target: $target_container ($target_external)"
    echo ""

    # ICMP test
    print_status "info" "Testing ICMP..."
    if docker exec "$source_container" ping -c 3 -W 2 "$target_external" 2>/dev/null; then
        print_status "ok" "ICMP reachable"
    else
        print_status "fail" "ICMP blocked"
    fi

    # UDP test
    print_status "info" "Testing UDP port $COMMUNITAS_PORT..."
    if docker exec "$source_container" sh -c "echo test | nc -u -w1 $target_external $COMMUNITAS_PORT" &>/dev/null; then
        print_status "ok" "UDP reachable"
    else
        print_status "warn" "UDP test inconclusive (no listener)"
    fi

    # Route trace
    print_status "info" "Route:"
    docker exec "$source_container" ip route 2>/dev/null || true
}

# Main
case ${1:-full} in
    quick)     cmd_quick ;;
    full)      cmd_full ;;
    pair)      shift; cmd_pair "$@" ;;
    help|*)
        echo "NAT Connectivity Matrix Test for Communitas"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  full           Run full connectivity matrix (default)"
        echo "  quick          Quick connectivity check to public node"
        echo "  pair <a> <b>   Test specific NAT pair"
        echo ""
        echo "NAT types: Public, FullCone, Restricted, PortRestricted, Symmetric, CGNAT, DoubleNAT, Hairpin"
        echo ""
        echo "Prerequisites:"
        echo "  docker-compose up -d"
        ;;
esac
