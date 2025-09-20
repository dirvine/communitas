#!/bin/bash

# Launch two Communitas app instances for testing
# These will run in local/offline mode initially but can connect via the network

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}🚀 Launching Communitas App Test Instances${NC}"
echo "=================================="
echo ""

# Create data directories for each instance
echo -e "${YELLOW}Creating data directories...${NC}"
mkdir -p "${SCRIPT_DIR}/app1_data"
mkdir -p "${SCRIPT_DIR}/app2_data"

# Build the app if needed
echo -e "${YELLOW}Building app (if needed)...${NC}"
cd "$PROJECT_ROOT"
npm run build:desktop 2>/dev/null || npm run build 2>/dev/null || true

echo ""
echo -e "${GREEN}Launching App Instance 1${NC}"
echo "  Data: ${SCRIPT_DIR}/app1_data"
echo "  Port: 1420 (default)"
echo ""

# Launch first instance with its own data directory
COMMUNITAS_DATA_DIR="${SCRIPT_DIR}/app1_data" \
COMMUNITAS_APP_INSTANCE="1" \
npm run tauri dev &

APP1_PID=$!
echo -e "  ${GREEN}✓${NC} App 1 started with PID: $APP1_PID"

# Wait a bit before launching second instance
echo ""
echo -e "${YELLOW}Waiting 10 seconds before launching second instance...${NC}"
sleep 10

echo ""
echo -e "${GREEN}Launching App Instance 2${NC}"
echo "  Data: ${SCRIPT_DIR}/app2_data"
echo "  Port: 1421 (alternate)"
echo ""

# Launch second instance with different data directory and port
COMMUNITAS_DATA_DIR="${SCRIPT_DIR}/app2_data" \
COMMUNITAS_APP_INSTANCE="2" \
VITE_DEV_SERVER_PORT=1421 \
npm run tauri dev &

APP2_PID=$!
echo -e "  ${GREEN}✓${NC} App 2 started with PID: $APP2_PID"

echo ""
echo "=================================="
echo -e "${GREEN}✅ Both app instances launched!${NC}"
echo ""
echo -e "${YELLOW}Testing Instructions:${NC}"
echo ""
echo "1. App 1 window should open automatically"
echo "2. App 2 window should open on a different port"
echo "3. Initialize both apps with different four-word identities:"
echo "   - App 1: Use any valid four-word identity (e.g., from the app's generator)"
echo "   - App 2: Use a different four-word identity"
echo ""
echo "4. Test features:"
echo "   - Create groups in each app"
echo "   - Send messages"
echo "   - Share files via virtual disks"
echo "   - Test offline/online sync"
echo ""
echo -e "${YELLOW}Network Status:${NC}"
echo "   - Apps will start in local/offline mode"
echo "   - Click the network indicator to attempt connection"
echo "   - If no bootstrap nodes available, apps work offline"
echo ""
echo -e "${RED}To stop both apps:${NC} Press Ctrl+C"
echo ""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Stopping app instances...${NC}"

    if kill -0 $APP1_PID 2>/dev/null; then
        echo "Stopping App 1 (PID $APP1_PID)"
        kill $APP1_PID 2>/dev/null || true
    fi

    if kill -0 $APP2_PID 2>/dev/null; then
        echo "Stopping App 2 (PID $APP2_PID)"
        kill $APP2_PID 2>/dev/null || true
    fi

    echo -e "${GREEN}Apps stopped${NC}"
}

trap cleanup EXIT INT TERM

# Keep running
wait $APP1_PID $APP2_PID