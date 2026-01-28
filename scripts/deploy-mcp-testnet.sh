#!/usr/bin/env bash
# Copyright (c) 2025 Saorsa Labs Limited
# Licensed under the AGPL-3.0 license

# MCP Testnet Deployment Script
#
# Deploys communitas-mcp to Saorsa Labs VPS infrastructure for E2E testing.
#
# Usage:
#   ./scripts/deploy-mcp-testnet.sh [options] [nodes...]
#
# Options:
#   -b, --build       Build locally before deploying
#   -r, --release     Use GitHub release instead of local build
#   -c, --clean       Clean existing deployment before deploying
#   -s, --status      Show status of all nodes
#   -h, --help        Show this help message
#
# Examples:
#   ./scripts/deploy-mcp-testnet.sh                    # Deploy to all nodes
#   ./scripts/deploy-mcp-testnet.sh -b                 # Build and deploy
#   ./scripts/deploy-mcp-testnet.sh -s                 # Show status only
#   ./scripts/deploy-mcp-testnet.sh saorsa-2 saorsa-3  # Deploy to specific nodes
#   ./scripts/deploy-mcp-testnet.sh -c                 # Clean and redeploy

set -eo pipefail

# =============================================================================
# SECURITY WARNING
# =============================================================================
# This script is for TESTNET ONLY in controlled environments.
#
# WARNING: The deployed service:
#   - Runs as root (elevated privileges)
#   - Listens on 0.0.0.0 (ALL network interfaces)
#   - Uses HTTP without TLS encryption
#   - Runs in demo mode without authentication
#
# DO NOT use this script:
#   - In production environments
#   - On public networks
#   - With sensitive data
#   - Without proper network isolation
#
# Usage: Only on trusted, isolated test networks with proper firewall rules.
# =============================================================================


# =============================================================================
# Configuration
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BINARY_NAME="communitas-mcp"
SERVICE_NAME="communitas-mcp-test"
INSTALL_DIR="/opt/communitas-test"
SERVICE_PORT=3040

# Node inventory (using parallel arrays for bash 3 compatibility)
NODE_NAMES=("saorsa-2" "saorsa-3" "saorsa-7")
NODE_IPS=("142.93.199.50" "147.182.234.192" "116.203.101.172")

# Default nodes for deployment (primary testnet)
DEFAULT_NODES=("saorsa-2" "saorsa-3" "saorsa-7")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# =============================================================================
# Helper Functions
# =============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Get IP for a node name
get_node_ip() {
    local node="$1"
    local i=0
    for n in "${NODE_NAMES[@]}"; do
        if [[ "$n" == "$node" ]]; then
            echo "${NODE_IPS[$i]}"
            return 0
        fi
        i=$((i + 1))
    done
    return 1
}

# Check if a node name is valid
is_valid_node() {
    local node="$1"
    for n in "${NODE_NAMES[@]}"; do
        if [[ "$n" == "$node" ]]; then
            return 0
        fi
    done
    return 1
}

usage() {
    cat << EOF
MCP Testnet Deployment Script

Usage: $0 [options] [nodes...]

Options:
  -b, --build       Build locally before deploying (using cargo zigbuild)
  -r, --release     Download from GitHub releases instead of local build
  -c, --clean       Clean existing deployment before deploying
  -s, --status      Show status of all testnet nodes
  -t, --teardown    Stop services and clean up on all nodes
  -h, --help        Show this help message

Nodes:
  Specify one or more node names (saorsa-2, saorsa-3, saorsa-7)
  If none specified, deploys to all nodes in the default set.

Examples:
  $0                     # Deploy existing binary to all nodes
  $0 -b                  # Build and deploy to all nodes
  $0 -s                  # Show status of all nodes
  $0 -c saorsa-2         # Clean and deploy to saorsa-2 only
  $0 -t                  # Tear down testnet on all nodes

Available Nodes:
EOF
    local i=0
    for node in "${NODE_NAMES[@]}"; do
        echo "  $node (${NODE_IPS[$i]})"
        i=$((i + 1))
    done
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for cargo zigbuild if building locally
    if [[ "${BUILD_LOCAL:-false}" == "true" ]]; then
        if ! command -v cargo-zigbuild &> /dev/null; then
            log_error "cargo-zigbuild not found. Install with: cargo install cargo-zigbuild"
            exit 1
        fi
        if ! command -v zig &> /dev/null; then
            log_error "zig not found. Install with: brew install zig"
            exit 1
        fi
    fi

    # Check for SSH access
    if ! command -v ssh &> /dev/null; then
        log_error "ssh not found"
        exit 1
    fi

    log_success "Prerequisites check passed"
}

# =============================================================================
# Build Functions
# =============================================================================

build_local() {
    log_info "Building communitas-mcp for Linux (x86_64)..."
    cd "$PROJECT_ROOT"

    cargo zigbuild --release --target x86_64-unknown-linux-gnu -p communitas-mcp

    BINARY_PATH="$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/$BINARY_NAME"
    if [[ ! -f "$BINARY_PATH" ]]; then
        log_error "Build failed: binary not found at $BINARY_PATH"
        exit 1
    fi

    log_success "Build complete: $BINARY_PATH"
}

download_release() {
    log_info "Downloading latest release from GitHub..."

    if ! command -v gh &> /dev/null; then
        log_error "GitHub CLI (gh) not found. Install with: brew install gh"
        exit 1
    fi

    RELEASE_DIR="$PROJECT_ROOT/target/release-download"
    mkdir -p "$RELEASE_DIR"

    gh release download --repo saorsa-labs/communitas -p "*linux*x86_64*" --dir "$RELEASE_DIR" --clobber

    # Find the binary
    BINARY_PATH=$(find "$RELEASE_DIR" -name "$BINARY_NAME" -o -name "$BINARY_NAME.gz" | head -1)

    if [[ -z "$BINARY_PATH" ]]; then
        log_error "Could not find $BINARY_NAME in downloaded release"
        exit 1
    fi

    # Decompress if needed
    if [[ "$BINARY_PATH" == *.gz ]]; then
        gunzip -f "$BINARY_PATH"
        BINARY_PATH="${BINARY_PATH%.gz}"
    fi

    log_success "Downloaded: $BINARY_PATH"
}

# =============================================================================
# Deployment Functions
# =============================================================================

create_systemd_service() {
    local node="$1"
    local ip
    ip=$(get_node_ip "$node")

    log_info "Creating systemd service on $node..."

    ssh "root@$ip" "cat > /etc/systemd/system/$SERVICE_NAME.service" << EOF
[Unit]
Description=Communitas MCP Server (Testnet)
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/$BINARY_NAME --http --demo --listen 0.0.0.0:$SERVICE_PORT
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

# Resource limits
LimitNOFILE=65535
MemoryMax=512M

# Environment
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

[Install]
WantedBy=multi-user.target
EOF

    ssh "root@$ip" "systemctl daemon-reload"
    log_success "Systemd service created on $node"
}

deploy_to_node() {
    local node="$1"
    local binary="$2"
    local ip
    ip=$(get_node_ip "$node")

    log_info "Deploying to $node ($ip)..."

    # Create install directory
    ssh "root@$ip" "mkdir -p $INSTALL_DIR"

    # Stop existing service
    ssh "root@$ip" "systemctl stop $SERVICE_NAME 2>/dev/null || true"

    # Copy binary
    scp "$binary" "root@$ip:$INSTALL_DIR/$BINARY_NAME"
    ssh "root@$ip" "chmod +x $INSTALL_DIR/$BINARY_NAME"

    # Create systemd service
    create_systemd_service "$node"

    # Start service
    ssh "root@$ip" "systemctl enable $SERVICE_NAME && systemctl start $SERVICE_NAME"

    # Verify
    sleep 2
    if ssh "root@$ip" "systemctl is-active $SERVICE_NAME" > /dev/null 2>&1; then
        log_success "Deployed and running on $node"
    else
        log_error "Failed to start service on $node"
        ssh "root@$ip" "journalctl -u $SERVICE_NAME -n 20 --no-pager" || true
        return 1
    fi
}

clean_node() {
    local node="$1"
    local ip
    ip=$(get_node_ip "$node")

    log_info "Cleaning $node..."

    ssh "root@$ip" << 'EOF'
        systemctl stop communitas-mcp-test 2>/dev/null || true
        systemctl disable communitas-mcp-test 2>/dev/null || true
        rm -rf /opt/communitas-test
        rm -f /etc/systemd/system/communitas-mcp-test.service
        systemctl daemon-reload
        journalctl --vacuum-time=1d
EOF

    log_success "Cleaned $node"
}

# =============================================================================
# Status Functions
# =============================================================================

show_status() {
    echo ""
    echo "=============================================="
    echo "  MCP Testnet Status"
    echo "=============================================="
    echo ""

    printf "%-12s %-18s %-10s %-10s\n" "NODE" "IP" "SERVICE" "PORT"
    printf "%-12s %-18s %-10s %-10s\n" "----" "--" "-------" "----"

    local i=0
    for node in "${NODE_NAMES[@]}"; do
        local ip="${NODE_IPS[$i]}"
        local status="UNKNOWN"
        local port_status="N/A"

        # Check SSH connectivity
        if ssh -o ConnectTimeout=5 "root@$ip" "true" 2>/dev/null; then
            # Check service status
            if ssh "root@$ip" "systemctl is-active $SERVICE_NAME" > /dev/null 2>&1; then
                status="${GREEN}RUNNING${NC}"
                # Check port
                if ssh "root@$ip" "curl -s -o /dev/null -w '%{http_code}' http://localhost:$SERVICE_PORT/health 2>/dev/null" | grep -q "200"; then
                    port_status="${GREEN}OPEN${NC}"
                else
                    port_status="${YELLOW}CLOSED${NC}"
                fi
            else
                status="${RED}STOPPED${NC}"
            fi
        else
            status="${RED}OFFLINE${NC}"
        fi

        printf "%-12s %-18s %-20b %-20b\n" "$node" "$ip" "$status" "$port_status"
        i=$((i + 1))
    done

    echo ""
}

teardown_all() {
    log_info "Tearing down testnet on all nodes..."

    for node in "${NODE_NAMES[@]}"; do
        clean_node "$node"
    done

    log_success "Testnet teardown complete"
}

# =============================================================================
# Main
# =============================================================================

main() {
    local build_local=false
    local use_release=false
    local clean=false
    local status_only=false
    local teardown=false
    local target_nodes=()

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -b|--build)
                build_local=true
                shift
                ;;
            -r|--release)
                use_release=true
                shift
                ;;
            -c|--clean)
                clean=true
                shift
                ;;
            -s|--status)
                status_only=true
                shift
                ;;
            -t|--teardown)
                teardown=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            saorsa-*)
                if is_valid_node "$1"; then
                    target_nodes+=("$1")
                else
                    log_error "Unknown node: $1"
                    exit 1
                fi
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    # Use default nodes if none specified
    if [[ ${#target_nodes[@]} -eq 0 ]]; then
        target_nodes=("${DEFAULT_NODES[@]}")
    fi

    # Status only
    if [[ "$status_only" == "true" ]]; then
        show_status
        exit 0
    fi

    # Teardown
    if [[ "$teardown" == "true" ]]; then
        teardown_all
        exit 0
    fi

    # Check prerequisites
    BUILD_LOCAL="$build_local"
    check_prerequisites

    # Determine binary path
    if [[ "$build_local" == "true" ]]; then
        build_local
        BINARY_PATH="$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/$BINARY_NAME"
    elif [[ "$use_release" == "true" ]]; then
        download_release
    else
        # Use existing local build
        BINARY_PATH="$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/$BINARY_NAME"
        if [[ ! -f "$BINARY_PATH" ]]; then
            log_warn "No local build found. Building..."
            build_local
            BINARY_PATH="$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/$BINARY_NAME"
        fi
    fi

    # Clean if requested
    if [[ "$clean" == "true" ]]; then
        for node in "${target_nodes[@]}"; do
            clean_node "$node"
        done
    fi

    # Deploy to each node
    log_info "Deploying to ${#target_nodes[@]} node(s): ${target_nodes[*]}"

    for node in "${target_nodes[@]}"; do
        deploy_to_node "$node" "$BINARY_PATH"
    done

    echo ""
    log_success "Deployment complete!"
    echo ""

    # Show final status
    show_status
}

main "$@"
