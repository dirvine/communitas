#!/bin/bash
# Copyright (c) 2025 Saorsa Labs Limited
# Test Fleet Orchestration Script for Communitas
#
# Usage:
#   ./scripts/test-fleet.sh start [--nat-emulation]
#   ./scripts/test-fleet.sh stop
#   ./scripts/test-fleet.sh status
#   ./scripts/test-fleet.sh logs <node>
#   ./scripts/test-fleet.sh deploy [node]
#   ./scripts/test-fleet.sh run [test-name]
#   ./scripts/test-fleet.sh ssh <node>

set -eo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVICE_NAME="communitas-bootstrap"
SERVICE_PORT=11000
BINARY_NAME="communitas-headless"
REMOTE_PATH="/opt/communitas"
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=5"

# VPS Fleet Configuration (bash 3.2 compatible)
ALL_NODES="saorsa-1 saorsa-2 saorsa-3 saorsa-4 saorsa-5 saorsa-6 saorsa-7 saorsa-8 saorsa-9"

# Bootstrap nodes for communitas (port 11000)
BOOTSTRAP_NODES="saorsa-2 saorsa-3"

# Test nodes
TEST_NODES="saorsa-4 saorsa-5 saorsa-6 saorsa-7 saorsa-8 saorsa-9"

# Get node info (bash 3.2 compatible - no associative arrays)
get_node_info() {
    local node=$1
    case $node in
        saorsa-1) echo "77.42.75.115:dashboard:Hetzner:Helsinki" ;;
        saorsa-2) echo "142.93.199.50:bootstrap:DigitalOcean:NYC" ;;
        saorsa-3) echo "147.182.234.192:bootstrap:DigitalOcean:SFO" ;;
        saorsa-4) echo "206.189.7.117:test:DigitalOcean:AMS" ;;
        saorsa-5) echo "144.126.230.161:test:DigitalOcean:LON" ;;
        saorsa-6) echo "65.21.157.229:test:Hetzner:Helsinki" ;;
        saorsa-7) echo "116.203.101.172:test:Hetzner:Nuremberg" ;;
        saorsa-8) echo "149.28.156.231:test:Vultr:Singapore" ;;
        saorsa-9) echo "45.77.176.184:test:Vultr:Tokyo" ;;
        *) echo "" ;;
    esac
}

# Get node IP
get_ip() {
    local node=$1
    get_node_info $node | cut -d: -f1
}

# Get node role
get_role() {
    local node=$1
    get_node_info $node | cut -d: -f2
}

# Get node provider
get_provider() {
    local node=$1
    get_node_info $node | cut -d: -f3
}

# Get node location
get_location() {
    local node=$1
    get_node_info $node | cut -d: -f4
}

# Check if node exists
node_exists() {
    local node=$1
    [[ -n "$(get_node_info $node)" ]]
}

# Print status with color
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

# Check if node is reachable
check_node() {
    local node=$1
    local ip=$(get_ip $node)
    if timeout 3 ssh $SSH_OPTS root@$ip "echo ok" &>/dev/null; then
        return 0
    else
        return 1
    fi
}

# Get service status on node
get_service_status() {
    local node=$1
    local ip=$(get_ip $node)
    local role=$(get_role $node)
    
    if [[ "$role" == "dashboard" ]]; then
        echo "dashboard"
        return
    fi
    
    local status=$(ssh $SSH_OPTS root@$ip "systemctl is-active $SERVICE_NAME 2>/dev/null || echo 'inactive'")
    echo "$status"
}

# Start services
cmd_start() {
    local nat_emulation=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --nat-emulation) nat_emulation=true; shift ;;
            *) shift ;;
        esac
    done

    echo -e "${BLUE}Starting Communitas fleet...${NC}"

    # Start bootstrap nodes first
    for node in $BOOTSTRAP_NODES; do
        local ip=$(get_ip $node)
        print_status "info" "Starting $node ($ip)..."

        if ssh $SSH_OPTS root@$ip "systemctl start $SERVICE_NAME" 2>/dev/null; then
            print_status "ok" "$node started"
        else
            print_status "fail" "Failed to start $node"
        fi
    done

    # Wait for bootstrap nodes
    sleep 2

    # Start test nodes
    for node in $TEST_NODES; do
        local ip=$(get_ip $node)
        print_status "info" "Starting $node ($ip)..."

        if ssh $SSH_OPTS root@$ip "systemctl start $SERVICE_NAME" 2>/dev/null; then
            print_status "ok" "$node started"
        else
            print_status "fail" "Failed to start $node"
        fi
    done

    if $nat_emulation; then
        print_status "warn" "NAT emulation requested - run 'docker-compose up' in docker/nat-emulation/"
    fi

    echo -e "\n${GREEN}Fleet start complete.${NC}"
}

# Stop services
cmd_stop() {
    echo -e "${BLUE}Stopping Communitas fleet...${NC}"

    # Stop test nodes first
    for node in $TEST_NODES; do
        local ip=$(get_ip $node)
        print_status "info" "Stopping $node..."
        ssh $SSH_OPTS root@$ip "systemctl stop $SERVICE_NAME" 2>/dev/null || true
    done

    # Stop bootstrap nodes
    for node in $BOOTSTRAP_NODES; do
        local ip=$(get_ip $node)
        print_status "info" "Stopping $node..."
        ssh $SSH_OPTS root@$ip "systemctl stop $SERVICE_NAME" 2>/dev/null || true
    done

    echo -e "\n${GREEN}Fleet stopped.${NC}"
}

# Show status
cmd_status() {
    echo -e "${BLUE}Communitas Fleet Status${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    printf "%-12s %-16s %-10s %-12s %-12s %s\n" "NODE" "IP" "ROLE" "PROVIDER" "LOCATION" "STATUS"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    for node in $ALL_NODES; do
        local ip=$(get_ip $node)
        local role=$(get_role $node)
        local provider=$(get_provider $node)
        local location=$(get_location $node)

        if check_node $node; then
            local service_status=$(get_service_status $node)
            case $service_status in
                "active")    status="${GREEN}running${NC}" ;;
                "inactive")  status="${YELLOW}stopped${NC}" ;;
                "dashboard") status="${BLUE}dashboard${NC}" ;;
                *)           status="${RED}$service_status${NC}" ;;
            esac
        else
            status="${RED}unreachable${NC}"
        fi

        printf "%-12s %-16s %-10s %-12s %-12s %b\n" "$node" "$ip" "$role" "$provider" "$location" "$status"
    done | sort

    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Port check
    echo -e "\n${BLUE}Port $SERVICE_PORT Connectivity:${NC}"
    for node in $BOOTSTRAP_NODES $TEST_NODES; do
        local ip=$(get_ip $node)
        if timeout 2 nc -zu $ip $SERVICE_PORT 2>/dev/null; then
            print_status "ok" "$node:$SERVICE_PORT UDP reachable"
        else
            print_status "fail" "$node:$SERVICE_PORT UDP unreachable"
        fi
    done
}

# Show logs
cmd_logs() {
    local node=${1:-}

    if [[ -z "$node" ]]; then
        echo "Usage: $0 logs <node>"
        echo "Available nodes: $ALL_NODES"
        exit 1
    fi

    if ! node_exists $node; then
        echo "Unknown node: $node"
        exit 1
    fi

    local ip=$(get_ip $node)
    echo -e "${BLUE}Logs from $node ($ip):${NC}"
    ssh $SSH_OPTS root@$ip "journalctl -u $SERVICE_NAME -n 100 --no-pager"
}

# Deploy binary to nodes
cmd_deploy() {
    local target_node=${1:-all}
    local binary_path="target/release/$BINARY_NAME"

    # Check if binary exists
    if [[ ! -f "$binary_path" ]]; then
        echo -e "${YELLOW}Binary not found. Building...${NC}"
        cargo build -p communitas-headless --release
    fi

    local nodes_to_deploy=""
    if [[ "$target_node" == "all" ]]; then
        nodes_to_deploy="$BOOTSTRAP_NODES $TEST_NODES"
    else
        if ! node_exists $target_node; then
            echo "Unknown node: $target_node"
            exit 1
        fi
        nodes_to_deploy="$target_node"
    fi

    echo -e "${BLUE}Deploying $BINARY_NAME to fleet...${NC}"

    for node in $nodes_to_deploy; do
        local ip=$(get_ip $node)
        print_status "info" "Deploying to $node ($ip)..."

        # Stop service
        ssh $SSH_OPTS root@$ip "systemctl stop $SERVICE_NAME 2>/dev/null || true"

        # Create directory if needed
        ssh $SSH_OPTS root@$ip "mkdir -p $REMOTE_PATH"

        # Copy binary
        if scp $SSH_OPTS "$binary_path" root@$ip:$REMOTE_PATH/; then
            # Make executable
            ssh $SSH_OPTS root@$ip "chmod +x $REMOTE_PATH/$BINARY_NAME"

            # Start service
            ssh $SSH_OPTS root@$ip "systemctl start $SERVICE_NAME"

            print_status "ok" "$node deployed and started"
        else
            print_status "fail" "Failed to deploy to $node"
        fi
    done

    echo -e "\n${GREEN}Deployment complete.${NC}"
}

# Run E2E tests
cmd_run() {
    local test_name=${1:-infrastructure_e2e}

    echo -e "${BLUE}Running E2E test: $test_name${NC}"

    # Ensure fleet is running
    echo "Checking fleet status..."
    cmd_status

    echo -e "\n${BLUE}Executing test...${NC}"
    cargo test -p communitas-core --test $test_name -- --nocapture
}

# SSH to node
cmd_ssh() {
    local node=${1:-}

    if [[ -z "$node" ]]; then
        echo "Usage: $0 ssh <node>"
        echo "Available nodes: $ALL_NODES"
        exit 1
    fi

    if ! node_exists $node; then
        echo "Unknown node: $node"
        exit 1
    fi

    local ip=$(get_ip $node)
    echo -e "${BLUE}Connecting to $node ($ip)...${NC}"
    ssh $SSH_OPTS root@$ip
}

# Main
case ${1:-help} in
    start)  shift; cmd_start "$@" ;;
    stop)   cmd_stop ;;
    status) cmd_status ;;
    logs)   shift; cmd_logs "$@" ;;
    deploy) shift; cmd_deploy "$@" ;;
    run)    shift; cmd_run "$@" ;;
    ssh)    shift; cmd_ssh "$@" ;;
    help|*)
        echo "Communitas Test Fleet Orchestration"
        echo ""
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Commands:"
        echo "  start [--nat-emulation]  Start the test fleet"
        echo "  stop                     Stop the test fleet"
        echo "  status                   Show fleet status"
        echo "  logs <node>              Show logs for a node"
        echo "  deploy [node]            Deploy binary (all or specific node)"
        echo "  run [test]               Run E2E tests"
        echo "  ssh <node>               SSH to a node"
        echo ""
        echo "Nodes: $ALL_NODES"
        ;;
esac
