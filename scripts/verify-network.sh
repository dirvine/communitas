#!/bin/bash
# Copyright (c) 2025 Saorsa Labs Limited
# Network Verification Script for Communitas
#
# Usage:
#   ./scripts/verify-network.sh connectivity
#   ./scripts/verify-network.sh services
#   ./scripts/verify-network.sh gossip
#   ./scripts/verify-network.sh nat
#   ./scripts/verify-network.sh full
#   ./scripts/verify-network.sh report

set -eo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
SERVICE_NAME="communitas-bootstrap"
SERVICE_PORT=11000
BINARY_NAME="communitas-headless"
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=5"

# VPS Fleet Configuration (bash 3.2 compatible)
ALL_NODES="saorsa-1 saorsa-2 saorsa-3 saorsa-4 saorsa-5 saorsa-6 saorsa-7 saorsa-8 saorsa-9"
BOOTSTRAP_NODES="saorsa-2 saorsa-3"
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

# Check if node exists
node_exists() {
    local node=$1
    [[ -n "$(get_node_info $node)" ]]
}

# Helper functions
get_ip() { get_node_info $1 | cut -d: -f1; }
get_role() { get_node_info $1 | cut -d: -f2; }
get_provider() { get_node_info $1 | cut -d: -f3; }
get_location() { get_node_info $1 | cut -d: -f4; }

print_status() {
    local status=$1
    local message=$2
    case $status in
        "ok")     echo -e "${GREEN}✓${NC} $message" ;;
        "fail")   echo -e "${RED}✗${NC} $message" ;;
        "warn")   echo -e "${YELLOW}!${NC} $message" ;;
        "info")   echo -e "${BLUE}→${NC} $message" ;;
        "check")  echo -e "${CYAN}○${NC} $message" ;;
    esac
}

# Check SSH connectivity
check_ssh() {
    local node=$1
    local ip=$(get_ip $node)
    if timeout 5 ssh $SSH_OPTS root@$ip "echo ok" &>/dev/null; then
        return 0
    fi
    return 1
}

# Check UDP port connectivity
check_udp_port() {
    local ip=$1
    local port=$2
    if timeout 2 nc -zu $ip $port 2>/dev/null; then
        return 0
    fi
    return 1
}

# Get service status
get_service_status() {
    local node=$1
    local ip=$(get_ip $node)
    ssh $SSH_OPTS root@$ip "systemctl is-active $SERVICE_NAME 2>/dev/null || echo 'inactive'" 2>/dev/null
}

# Get process info
get_process_info() {
    local node=$1
    local ip=$(get_ip $node)
    ssh $SSH_OPTS root@$ip "pgrep -af $BINARY_NAME 2>/dev/null | head -1 || echo 'not running'" 2>/dev/null
}

# Get memory usage
get_memory_usage() {
    local node=$1
    local ip=$(get_ip $node)
    ssh $SSH_OPTS root@$ip "ps aux | grep $BINARY_NAME | grep -v grep | awk '{print \$6}'" 2>/dev/null || echo "0"
}

# Get uptime
get_uptime() {
    local node=$1
    local ip=$(get_ip $node)
    ssh $SSH_OPTS root@$ip "systemctl show $SERVICE_NAME --property=ActiveEnterTimestamp | cut -d= -f2" 2>/dev/null || echo "unknown"
}

# Connectivity checks
cmd_connectivity() {
    echo -e "${BLUE}Network Connectivity Verification${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    local ssh_ok=0
    local ssh_fail=0
    local udp_ok=0
    local udp_fail=0

    echo -e "\n${CYAN}SSH Connectivity:${NC}"
    for node in $ALL_NODES; do
        local ip=$(get_ip $node)
        local role=$(get_role $node)

        if [[ "$role" == "dashboard" ]]; then
            continue
        fi

        if check_ssh $node; then
            print_status "ok" "$node ($ip) - SSH reachable"
            ((ssh_ok++))
        else
            print_status "fail" "$node ($ip) - SSH unreachable"
            ((ssh_fail++))
        fi
    done

    echo -e "\n${CYAN}UDP Port $SERVICE_PORT Connectivity:${NC}"
    for node in $BOOTSTRAP_NODES $TEST_NODES; do
        local ip=$(get_ip $node)

        if check_udp_port $ip $SERVICE_PORT; then
            print_status "ok" "$node:$SERVICE_PORT - UDP reachable"
            ((udp_ok++))
        else
            print_status "fail" "$node:$SERVICE_PORT - UDP unreachable"
            ((udp_fail++))
        fi
    done

    echo -e "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "SSH: ${GREEN}$ssh_ok OK${NC}, ${RED}$ssh_fail FAILED${NC}"
    echo -e "UDP: ${GREEN}$udp_ok OK${NC}, ${RED}$udp_fail FAILED${NC}"
}

# Service checks
cmd_services() {
    echo -e "${BLUE}Service Status Verification${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    printf "%-12s %-16s %-10s %-12s %s\n" "NODE" "IP" "ROLE" "STATUS" "UPTIME"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    local running=0
    local stopped=0

    for node in $ALL_NODES; do
        local ip=$(get_ip $node)
        local role=$(get_role $node)

        if [[ "$role" == "dashboard" ]]; then
            printf "%-12s %-16s %-10s %-12b %s\n" "$node" "$ip" "$role" "${BLUE}monitoring${NC}" "-"
            continue
        fi

        if check_ssh $node; then
            local status=$(get_service_status $node)
            local uptime=$(get_uptime $node)

            case $status in
                "active")
                    printf "%-12s %-16s %-10s %-12b %s\n" "$node" "$ip" "$role" "${GREEN}running${NC}" "$uptime"
                    ((running++))
                    ;;
                "inactive")
                    printf "%-12s %-16s %-10s %-12b %s\n" "$node" "$ip" "$role" "${YELLOW}stopped${NC}" "-"
                    ((stopped++))
                    ;;
                *)
                    printf "%-12s %-16s %-10s %-12b %s\n" "$node" "$ip" "$role" "${RED}$status${NC}" "-"
                    ((stopped++))
                    ;;
            esac
        else
            printf "%-12s %-16s %-10s %-12b %s\n" "$node" "$ip" "$role" "${RED}unreachable${NC}" "-"
            ((stopped++))
        fi
    done | sort

    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "Services: ${GREEN}$running running${NC}, ${YELLOW}$stopped stopped${NC}"
}

# Gossip health check
cmd_gossip() {
    echo -e "${BLUE}Gossip Mesh Health${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    local active_nodes=""

    # Get list of active nodes
    for node in $BOOTSTRAP_NODES $TEST_NODES; do
        if check_ssh $node; then
            local status=$(get_service_status $node)
            if [[ "$status" == "active" ]]; then
                active_nodes="$active_nodes $node"
            fi
        fi
    done

    # Count active nodes
    local node_count=0
    for node in $active_nodes; do
        ((node_count++)) || true
    done
    echo -e "Active nodes: $node_count"

    if [[ $node_count -lt 2 ]]; then
        print_status "fail" "Insufficient nodes for gossip mesh (need at least 2)"
        return 1
    fi

    # Check inter-node connectivity
    echo -e "\n${CYAN}Inter-Node UDP Connectivity Matrix:${NC}"

    # Header
    printf "%-12s" "FROM \\ TO"
    for target in $active_nodes; do
        printf "%-10s" "$target"
    done
    echo ""

    # Matrix
    for source in $active_nodes; do
        local source_ip=$(get_ip $source)
        printf "%-12s" "$source"

        for target in $active_nodes; do
            if [[ "$source" == "$target" ]]; then
                printf "${BLUE}%-10s${NC}" "-"
            else
                local target_ip=$(get_ip $target)
                # Check from source node if it can reach target
                if ssh $SSH_OPTS root@$source_ip "timeout 2 nc -zu $target_ip $SERVICE_PORT" &>/dev/null; then
                    printf "${GREEN}%-10s${NC}" "✓"
                else
                    printf "${RED}%-10s${NC}" "✗"
                fi
            fi
        done
        echo ""
    done

    # Expected mesh size for gossip
    local expected_connections=$((node_count * (node_count - 1)))
    echo -e "\nExpected full mesh: $expected_connections connections"

    # Check bootstrap health
    echo -e "\n${CYAN}Bootstrap Node Health:${NC}"
    for node in $BOOTSTRAP_NODES; do
        if [[ " $active_nodes " =~ " $node " ]]; then
            print_status "ok" "$node is active (bootstrap)"
        else
            print_status "fail" "$node is NOT active (bootstrap down!)"
        fi
    done
}

# NAT emulation status
cmd_nat() {
    echo -e "${BLUE}NAT Emulation Status${NC}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Check local Docker NAT emulation
    echo -e "${CYAN}Local Docker NAT Emulation:${NC}"

    if command -v docker &>/dev/null; then
        local nat_containers=$(docker ps --filter "name=nat-" --format "{{.Names}}" 2>/dev/null | wc -l)
        if [[ $nat_containers -gt 0 ]]; then
            print_status "ok" "$nat_containers NAT containers running"
            docker ps --filter "name=nat-" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null
        else
            print_status "info" "No NAT emulation containers running locally"
            echo "Start with: docker-compose -f docker/nat-emulation/docker-compose.yml up -d"
        fi
    else
        print_status "warn" "Docker not available"
    fi

    # Check iptables NAT rules on VPS nodes
    echo -e "\n${CYAN}VPS iptables NAT Rules:${NC}"
    for node in $TEST_NODES; do
        local ip=$(get_ip $node)
        if check_ssh $node; then
            local nat_rules=$(ssh $SSH_OPTS root@$ip "iptables -t nat -L -n 2>/dev/null | grep -c MASQUERADE" 2>/dev/null || echo "0")
            if [[ $nat_rules -gt 0 ]]; then
                print_status "info" "$node: $nat_rules NAT rules configured"
            else
                print_status "check" "$node: No NAT rules (direct connectivity)"
            fi
        else
            print_status "warn" "$node: unreachable"
        fi
    done
}

# Full verification
cmd_full() {
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}                    ${CYAN}COMMUNITAS NETWORK VERIFICATION${NC}                       ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    cmd_connectivity
    echo ""
    cmd_services
    echo ""
    cmd_gossip
    echo ""
    cmd_nat

    echo -e "\n${BLUE}╔══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}                         ${CYAN}VERIFICATION COMPLETE${NC}                            ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════╝${NC}"
}

# JSON health report
cmd_report() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    echo "{"
    echo "  \"timestamp\": \"$timestamp\","
    echo "  \"network\": \"communitas\","
    echo "  \"port\": $SERVICE_PORT,"
    echo "  \"nodes\": ["

    local first=true
    for node in $ALL_NODES; do
        local ip=$(get_ip $node)
        local role=$(get_role $node)
        local provider=$(get_provider $node)
        local location=$(get_location $node)

        if [[ "$first" != "true" ]]; then
            echo ","
        fi
        first=false

        local ssh_status="false"
        local service_status="unknown"
        local udp_status="false"
        local memory_kb="0"

        if [[ "$role" != "dashboard" ]]; then
            if check_ssh $node 2>/dev/null; then
                ssh_status="true"
                service_status=$(get_service_status $node 2>/dev/null || echo "unknown")
                memory_kb=$(get_memory_usage $node 2>/dev/null || echo "0")
            fi

            if check_udp_port $ip $SERVICE_PORT 2>/dev/null; then
                udp_status="true"
            fi
        else
            ssh_status="true"
            service_status="dashboard"
        fi

        echo -n "    {"
        echo -n "\"name\": \"$node\", "
        echo -n "\"ip\": \"$ip\", "
        echo -n "\"role\": \"$role\", "
        echo -n "\"provider\": \"$provider\", "
        echo -n "\"location\": \"$location\", "
        echo -n "\"ssh_reachable\": $ssh_status, "
        echo -n "\"service_status\": \"$service_status\", "
        echo -n "\"udp_reachable\": $udp_status, "
        echo -n "\"memory_kb\": $memory_kb"
        echo -n "}"
    done | sort

    echo ""
    echo "  ],"

    # Summary
    local total_nodes=0
    for node in $ALL_NODES; do
        ((total_nodes++)) || true
    done
    local active_count=0
    local bootstrap_healthy=true

    for node in $BOOTSTRAP_NODES $TEST_NODES; do
        if check_ssh $node 2>/dev/null; then
            local status=$(get_service_status $node 2>/dev/null)
            if [[ "$status" == "active" ]]; then
                ((active_count++))
            fi
        fi
    done

    for node in $BOOTSTRAP_NODES; do
        if ! check_ssh $node 2>/dev/null; then
            bootstrap_healthy=false
        else
            local status=$(get_service_status $node 2>/dev/null)
            if [[ "$status" != "active" ]]; then
                bootstrap_healthy=false
            fi
        fi
    done

    echo "  \"summary\": {"
    echo "    \"total_nodes\": $total_nodes,"
    echo "    \"active_nodes\": $active_count,"
    echo "    \"bootstrap_healthy\": $bootstrap_healthy,"
    echo "    \"mesh_expected_connections\": $((active_count * (active_count - 1)))"
    echo "  }"
    echo "}"
}

# Quick check (exit code only)
cmd_quick() {
    local failed=0

    # Check bootstrap nodes are up
    for node in $BOOTSTRAP_NODES; do
        if ! check_ssh $node; then
            ((failed++))
            continue
        fi

        local status=$(get_service_status $node)
        if [[ "$status" != "active" ]]; then
            ((failed++))
        fi
    done

    if [[ $failed -gt 0 ]]; then
        exit 1
    fi

    exit 0
}

# Main
case ${1:-help} in
    connectivity) cmd_connectivity ;;
    services)     cmd_services ;;
    gossip)       cmd_gossip ;;
    nat)          cmd_nat ;;
    full)         cmd_full ;;
    report)       cmd_report ;;
    quick)        cmd_quick ;;
    help|*)
        echo "Communitas Network Verification"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  connectivity   Check SSH and UDP port connectivity"
        echo "  services       Show systemd service status"
        echo "  gossip         Verify gossip mesh health"
        echo "  nat            Check NAT emulation status"
        echo "  full           Run all verification checks"
        echo "  report         Output JSON health report"
        echo "  quick          Quick health check (exit code only)"
        echo ""
        echo "Nodes: $ALL_NODES"
        ;;
esac
