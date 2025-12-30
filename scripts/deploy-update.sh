#!/bin/bash
# Copyright (c) 2025 Saorsa Labs Limited
# Deploy and Update Script for Communitas
#
# Usage:
#   ./scripts/deploy-update.sh build [--release]
#   ./scripts/deploy-update.sh github-release <version>
#   ./scripts/deploy-update.sh direct [node]
#   ./scripts/deploy-update.sh trigger-check
#   ./scripts/deploy-update.sh full <version>

set -eo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
REPO_OWNER="saorsa-labs"
REPO_NAME="communitas"
BINARY_HEADLESS="communitas-headless"
BINARY_ICED="communitas-iced"
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=5"

# VPS nodes for direct deploy (bash 3.2 compatible)
ALL_NODES="saorsa-2 saorsa-3 saorsa-4 saorsa-5 saorsa-6 saorsa-7 saorsa-8 saorsa-9"

# Get node IP (bash 3.2 compatible - no associative arrays)
get_node_ip() {
    local node=$1
    case $node in
        saorsa-2) echo "142.93.199.50" ;;
        saorsa-3) echo "147.182.234.192" ;;
        saorsa-4) echo "206.189.7.117" ;;
        saorsa-5) echo "144.126.230.161" ;;
        saorsa-6) echo "65.21.157.229" ;;
        saorsa-7) echo "116.203.101.172" ;;
        saorsa-8) echo "149.28.156.231" ;;
        saorsa-9) echo "45.77.176.184" ;;
        *) echo "" ;;
    esac
}

# Check if node exists
node_exists() {
    local node=$1
    [[ -n "$(get_node_ip $node)" ]]
}

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

# Build release binaries
cmd_build() {
    local release_mode=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --release) release_mode=true; shift ;;
            *) shift ;;
        esac
    done
    
    echo -e "${BLUE}Building Communitas binaries...${NC}"
    
    local cargo_args=""
    if $release_mode; then
        cargo_args="--release"
        echo "Building in RELEASE mode"
    else
        echo "Building in DEBUG mode (use --release for release builds)"
    fi
    
    # Build headless
    print_status "info" "Building $BINARY_HEADLESS..."
    if cargo build -p communitas-headless $cargo_args; then
        print_status "ok" "$BINARY_HEADLESS built"
    else
        print_status "fail" "Failed to build $BINARY_HEADLESS"
        exit 1
    fi
    
    # Build Iced
    print_status "info" "Building $BINARY_ICED..."
    if cargo build -p communitas-iced $cargo_args; then
        print_status "ok" "$BINARY_ICED built"
    else
        print_status "fail" "Failed to build $BINARY_ICED"
        exit 1
    fi
    
    # Show binary locations
    echo -e "\n${GREEN}Build complete!${NC}"
    if $release_mode; then
        echo "Binaries:"
        ls -lh target/release/$BINARY_HEADLESS target/release/$BINARY_ICED 2>/dev/null || true
    else
        echo "Binaries:"
        ls -lh target/debug/$BINARY_HEADLESS target/debug/$BINARY_ICED 2>/dev/null || true
    fi
}

# Create GitHub release
cmd_github_release() {
    local version=${1:-}
    
    if [[ -z "$version" ]]; then
        echo "Usage: $0 github-release <version>"
        echo "Example: $0 github-release 0.2.0"
        exit 1
    fi
    
    # Ensure version starts with 'v'
    if [[ ! "$version" =~ ^v ]]; then
        version="v$version"
    fi
    
    echo -e "${BLUE}Creating GitHub release $version...${NC}"
    
    # Check if gh CLI is available
    if ! command -v gh &>/dev/null; then
        print_status "fail" "GitHub CLI (gh) not installed"
        echo "Install with: brew install gh"
        exit 1
    fi
    
    # Build release binaries
    cmd_build --release
    
    # Create release
    print_status "info" "Creating release $version on GitHub..."
    
    local release_notes="## Communitas $version

### Changes
- Auto-update capability for desktop apps
- Improved P2P connectivity
- Bug fixes and performance improvements

### Downloads
- \`communitas-headless-linux-x86_64\` - Headless node for Linux
- \`communitas-iced-linux-x86_64\` - Desktop app for Linux
- \`communitas-iced-macos-universal\` - Desktop app for macOS (Intel + Apple Silicon)
"
    
    if gh release create "$version" \
        --title "Communitas $version" \
        --notes "$release_notes" \
        --draft \
        target/release/$BINARY_HEADLESS \
        target/release/$BINARY_ICED; then
        print_status "ok" "Draft release created: $version"
        echo ""
        echo "Next steps:"
        echo "1. Review the release at: https://github.com/$REPO_OWNER/$REPO_NAME/releases"
        echo "2. Edit release notes if needed"
        echo "3. Publish the release to trigger auto-updates"
    else
        print_status "fail" "Failed to create release"
        exit 1
    fi
}

# Direct deploy to VPS nodes
cmd_direct() {
    local target_node=${1:-all}
    local binary_path="target/release/$BINARY_HEADLESS"

    # Check if binary exists
    if [[ ! -f "$binary_path" ]]; then
        echo -e "${YELLOW}Release binary not found. Building...${NC}"
        cmd_build --release
    fi

    local nodes_to_deploy=""
    if [[ "$target_node" == "all" ]]; then
        nodes_to_deploy="$ALL_NODES"
    else
        if ! node_exists $target_node; then
            echo "Unknown node: $target_node"
            echo "Available: $ALL_NODES"
            exit 1
        fi
        nodes_to_deploy="$target_node"
    fi

    echo -e "${BLUE}Direct deploy to VPS nodes...${NC}"

    local success=0
    local failed=0

    for node in $nodes_to_deploy; do
        local ip=$(get_node_ip $node)
        print_status "info" "Deploying to $node ($ip)..."

        # Stop service
        ssh $SSH_OPTS root@$ip "systemctl stop communitas-bootstrap 2>/dev/null || true" &>/dev/null

        # Copy binary
        if scp $SSH_OPTS "$binary_path" root@$ip:/opt/communitas/$BINARY_HEADLESS; then
            # Make executable and restart
            ssh $SSH_OPTS root@$ip "chmod +x /opt/communitas/$BINARY_HEADLESS && systemctl start communitas-bootstrap"
            print_status "ok" "$node deployed"
            ((success++)) || true
        else
            print_status "fail" "Failed to deploy to $node"
            ((failed++)) || true
        fi
    done

    echo -e "\n${GREEN}Deployment complete: $success success, $failed failed${NC}"
}

# Trigger update check on running apps
cmd_trigger_check() {
    echo -e "${BLUE}Triggering update check on fleet...${NC}"

    # For headless nodes, we can signal them to check for updates
    # This sends SIGUSR1 which the app can handle to trigger update check

    for node in $ALL_NODES; do
        local ip=$(get_node_ip $node)
        print_status "info" "Triggering update check on $node..."

        # Send signal to process
        if ssh $SSH_OPTS root@$ip "pkill -USR1 -f $BINARY_HEADLESS 2>/dev/null"; then
            print_status "ok" "$node signaled"
        else
            print_status "warn" "$node - no process found or signal failed"
        fi
    done

    echo -e "\n${GREEN}Update check triggered.${NC}"
    echo "Apps will check GitHub releases and update if newer version available."
}

# Full release cycle
cmd_full() {
    local version=${1:-}
    
    if [[ -z "$version" ]]; then
        echo "Usage: $0 full <version>"
        echo "This will: build -> create GitHub release -> deploy direct -> trigger check"
        exit 1
    fi
    
    echo -e "${BLUE}Full release cycle for version $version${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Step 1: Build
    echo -e "\n${BLUE}Step 1/4: Build${NC}"
    cmd_build --release
    
    # Step 2: Create GitHub release
    echo -e "\n${BLUE}Step 2/4: GitHub Release${NC}"
    cmd_github_release "$version"
    
    # Step 3: Direct deploy (immediate update)
    echo -e "\n${BLUE}Step 3/4: Direct Deploy${NC}"
    cmd_direct all
    
    # Step 4: Trigger update check
    echo -e "\n${BLUE}Step 4/4: Trigger Update Check${NC}"
    cmd_trigger_check
    
    echo -e "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}Full release cycle complete for $version${NC}"
    echo ""
    echo "What happened:"
    echo "1. Built release binaries locally"
    echo "2. Created draft GitHub release (publish to enable auto-update)"
    echo "3. Deployed directly to all VPS nodes"
    echo "4. Triggered update check on running apps"
}

# Version info
cmd_version() {
    echo "Deploy Update Script v1.0"
    echo "Repository: $REPO_OWNER/$REPO_NAME"
    
    # Get current version from Cargo.toml
    local current_version=$(grep -m1 'version = ' Cargo.toml | cut -d'"' -f2)
    echo "Current version: $current_version"
    
    # Check latest GitHub release
    if command -v gh &>/dev/null; then
        local latest=$(gh release list --limit 1 --json tagName -q '.[0].tagName' 2>/dev/null || echo "unknown")
        echo "Latest release: $latest"
    fi
}

# Main
case ${1:-help} in
    build)          shift; cmd_build "$@" ;;
    github-release) shift; cmd_github_release "$@" ;;
    direct)         shift; cmd_direct "$@" ;;
    trigger-check)  cmd_trigger_check ;;
    full)           shift; cmd_full "$@" ;;
    version)        cmd_version ;;
    help|*)
        echo "Communitas Deploy & Update Script"
        echo ""
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Commands:"
        echo "  build [--release]        Build binaries (debug or release)"
        echo "  github-release <ver>     Create GitHub release with binaries"
        echo "  direct [node]            Deploy directly to VPS (all or specific)"
        echo "  trigger-check            Signal apps to check for updates"
        echo "  full <version>           Full cycle: build -> release -> deploy -> trigger"
        echo "  version                  Show version info"
        echo ""
        echo "Examples:"
        echo "  $0 build --release"
        echo "  $0 github-release 0.2.0"
        echo "  $0 direct saorsa-2"
        echo "  $0 full 0.2.0"
        ;;
esac
