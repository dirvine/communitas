#!/bin/bash
#
# Multi-Peer CRDT Testing Script
#
# This script helps launch two Tauri instances for testing P2P CRDT messaging.
# Each instance runs with a separate data directory and peer identity.
#
# NOTE: This script builds the frontend once and runs Tauri in production mode
# to avoid port conflicts between multiple dev servers.
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
ALICE_PEER_ID="ocean-forest-moon-star"
BOB_PEER_ID="river-mountain-sun-cloud"
ALICE_PORT=8080
BOB_PORT=8081
DATA_DIR_BASE="$HOME/.communitas-data"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    pkill -f "communitas.*alice" || true
    pkill -f "communitas.*bob" || true
    echo -e "${GREEN}Cleanup complete${NC}"
}

# Help message
show_help() {
    cat << EOF
Usage: $0 [OPTION]

Launch two Tauri instances for multi-peer CRDT testing.

Options:
  alice          Launch Alice's instance (peer: ${ALICE_PEER_ID})
  bob            Launch Bob's instance (peer: ${BOB_PEER_ID})
  clean          Clean data directories and kill running instances
  help           Show this help message

Examples:
  # Terminal 1: Launch Alice
  $0 alice

  # Terminal 2: Launch Bob
  $0 bob

  # Clean up after testing
  $0 clean

Peer Configuration:
  Alice:
    - Four-word ID: ${ALICE_PEER_ID}
    - Data dir: ${DATA_DIR_BASE}-alice
    - Port: ${ALICE_PORT}

  Bob:
    - Four-word ID: ${BOB_PEER_ID}
    - Data dir: ${DATA_DIR_BASE}-bob
    - Port: ${BOB_PORT}

Testing Steps:
  1. Run '$0 alice' in terminal 1
  2. Run '$0 bob' in terminal 2
  3. In Alice's app: Register with four-word ID '${ALICE_PEER_ID}'
  4. In Bob's app: Register with four-word ID '${BOB_PEER_ID}'
  5. Wait for both to show "Connected to network"
  6. Test messaging between Alice and Bob
  7. When done, run '$0 clean' to cleanup

For detailed testing instructions, see:
  docs/CRDT_MULTI_PEER_TEST.md
EOF
}

# Ensure frontend is built
ensure_build() {
    if [ ! -d "dist" ] || [ -z "$(ls -A dist 2>/dev/null)" ]; then
        echo -e "${YELLOW}Frontend not built. Building now...${NC}"
        npm run build
        echo -e "${GREEN}Build complete!${NC}\n"
    else
        echo -e "${GREEN}Using existing frontend build${NC}\n"
    fi
}

# Launch Alice's instance
launch_alice() {
    echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Launching Alice's Instance (Production Mode)${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo -e "  Peer ID:   ${YELLOW}${ALICE_PEER_ID}${NC}"
    echo -e "  Data Dir:  ${YELLOW}${DATA_DIR_BASE}-alice${NC}"
    echo -e "  Port:      ${YELLOW}${ALICE_PORT}${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo ""

    # Ensure frontend is built
    ensure_build

    # Set environment variables
    export COMMUNITAS_DATA_DIR="${DATA_DIR_BASE}-alice"
    export COMMUNITAS_PORT="${ALICE_PORT}"
    export COMMUNITAS_PEER_ID="${ALICE_PEER_ID}"
    export COMMUNITAS_USER_NAME="Alice"

    # Create data directory if it doesn't exist
    mkdir -p "${COMMUNITAS_DATA_DIR}"

    echo -e "${YELLOW}Starting Tauri (production mode to avoid port conflicts)...${NC}"
    echo -e "Press ${RED}Ctrl+C${NC} to stop\n"

    # Run Tauri backend directly with built frontend
    cd communitas-desktop
    cargo run --release
}

# Launch Bob's instance
launch_bob() {
    echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Launching Bob's Instance (Production Mode)${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo -e "  Peer ID:   ${YELLOW}${BOB_PEER_ID}${NC}"
    echo -e "  Data Dir:  ${YELLOW}${DATA_DIR_BASE}-bob${NC}"
    echo -e "  Port:      ${YELLOW}${BOB_PORT}${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo ""

    # Ensure frontend is built
    ensure_build

    # Set environment variables
    export COMMUNITAS_DATA_DIR="${DATA_DIR_BASE}-bob"
    export COMMUNITAS_PORT="${BOB_PORT}"
    export COMMUNITAS_PEER_ID="${BOB_PEER_ID}"
    export COMMUNITAS_USER_NAME="Bob"

    # Create data directory if it doesn't exist
    mkdir -p "${COMMUNITAS_DATA_DIR}"

    echo -e "${YELLOW}Starting Tauri (production mode to avoid port conflicts)...${NC}"
    echo -e "Press ${RED}Ctrl+C${NC} to stop\n"

    # Run Tauri backend directly with built frontend
    cd communitas-desktop
    cargo run --release
}

# Clean data directories
clean_data() {
    echo -e "${YELLOW}Cleaning data directories...${NC}"

    # Kill running processes
    cleanup

    # Remove data directories
    if [ -d "${DATA_DIR_BASE}-alice" ]; then
        echo -e "  Removing ${DATA_DIR_BASE}-alice"
        rm -rf "${DATA_DIR_BASE}-alice"
    fi

    if [ -d "${DATA_DIR_BASE}-bob" ]; then
        echo -e "  Removing ${DATA_DIR_BASE}-bob"
        rm -rf "${DATA_DIR_BASE}-bob"
    fi

    echo -e "${GREEN}Data directories cleaned${NC}"
}

# Main script logic
case "${1:-help}" in
    alice)
        launch_alice
        ;;
    bob)
        launch_bob
        ;;
    clean)
        clean_data
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo -e "${RED}Error: Unknown command '${1}'${NC}\n"
        show_help
        exit 1
        ;;
esac
