#!/bin/bash

# Unified Test Runner for Communitas with Chrome DevTools MCP
# This orchestrates the complete test environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_NET_DIR="/tmp/communitas-test-network"
TAURI_PID=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE} Communitas Unified Test Runner${NC}"
echo -e "${BLUE}========================================${NC}\n"

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"

    # Kill Tauri app if running
    if [ ! -z "$TAURI_PID" ]; then
        echo "Stopping Tauri app (PID: $TAURI_PID)..."
        kill $TAURI_PID 2>/dev/null || true
    fi

    # Stop test network
    if [ -f "$TEST_NET_DIR/pids.txt" ]; then
        echo "Stopping test network nodes..."
        while read pid; do
            kill $pid 2>/dev/null || true
        done < "$TEST_NET_DIR/pids.txt"
    fi

    echo -e "${GREEN}Cleanup complete${NC}"
}

trap cleanup EXIT

# Step 1: Check prerequisites
echo -e "${BLUE}Step 1: Checking prerequisites...${NC}"

# Check if test network is running
if [ ! -f "$TEST_NET_DIR/pids.txt" ]; then
    echo -e "${YELLOW}Test network not running. Starting it now...${NC}"
    cd "$TEST_NET_DIR" && ./setup-local-network.sh
    sleep 5
fi

# Verify nodes are running
RUNNING_NODES=$(ps aux | grep communitas-headless | grep -v grep | wc -l)
echo -e "Found ${GREEN}$RUNNING_NODES${NC} running nodes"

if [ "$RUNNING_NODES" -lt 8 ]; then
    echo -e "${RED}Error: Expected 8 nodes but found $RUNNING_NODES${NC}"
    echo "Please restart the test network"
    exit 1
fi

# Step 2: Build Tauri app
echo -e "\n${BLUE}Step 2: Building Tauri application...${NC}"
cd "$SCRIPT_DIR/communitas-desktop"

# Check if dependencies are installed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

# Build the app
cargo build --release

# Step 3: Launch Tauri with remote debugging
echo -e "\n${BLUE}Step 3: Launching Tauri with remote debugging...${NC}"

# Kill any existing Tauri processes
pkill -f communitas-desktop || true

# Launch with debug port
export RUST_LOG=info,communitas=debug,tauri_plugin_mcp=debug
export WEBKIT_INSPECTOR_SERVER=127.0.0.1:9222
export TAURI_WEBVIEW_REMOTE_DEBUGGING_PORT=9222

# Start Tauri in background
./launch-with-debug.sh &
TAURI_PID=$!

echo "Tauri launched with PID: $TAURI_PID"
echo "Waiting for app to start..."
sleep 10

# Check if MCP socket exists
MCP_SOCKET="/tmp/tauri-mcp-communitas-${TAURI_PID}.sock"
if [ -S "$MCP_SOCKET" ]; then
    echo -e "${GREEN}✓ MCP socket found at $MCP_SOCKET${NC}"
else
    echo -e "${YELLOW}⚠ MCP socket not found at $MCP_SOCKET${NC}"
fi

# Step 4: Run Chrome DevTools MCP tests
echo -e "\n${BLUE}Step 4: Running Chrome DevTools MCP tests...${NC}"

# Install Chrome DevTools MCP if needed
if ! command -v chrome-devtools-mcp &> /dev/null; then
    echo "Installing Chrome DevTools MCP..."
    npm install -g @modelcontextprotocol/server-chrome-devtools
fi

# Run the test suite
cd "$SCRIPT_DIR/communitas-desktop"
node test-chrome-devtools-mcp.js $TAURI_PID

TEST_RESULT=$?

# Step 5: Run Playwright tests
echo -e "\n${BLUE}Step 5: Running Playwright tests...${NC}"

# Install Playwright if needed
if [ ! -d "node_modules/@playwright" ]; then
    echo "Installing Playwright..."
    npm install --save-dev @playwright/test
    npx playwright install chromium
fi

# Create test directory if it doesn't exist
mkdir -p tests/integration

# Create a basic Playwright test
cat > tests/integration/network.spec.ts << 'EOF'
import { test, expect } from '@playwright/test';

test.describe('Communitas Network Integration', () => {
  test('should connect to P2P network', async ({ page }) => {
    await page.goto('http://localhost:5173');

    // Wait for network indicator
    const networkStatus = page.locator('[data-testid="network-status"]');
    await expect(networkStatus).toBeVisible({ timeout: 10000 });

    // Check connection status
    const status = await networkStatus.getAttribute('data-status');
    expect(['connected', 'connecting']).toContain(status);
  });

  test('should authenticate with four-word identity', async ({ page }) => {
    await page.goto('http://localhost:5173');

    // Fill in four-word identity
    await page.fill('#four-words-input', 'ocean-forest-moon-star');

    // Click login
    await page.click('#login-button');

    // Wait for authentication
    await expect(page.locator('[data-testid="user-profile"]')).toBeVisible({ timeout: 5000 });
  });

  test('should send P2P message', async ({ page }) => {
    // Assumes already logged in from previous test
    await page.goto('http://localhost:5173');

    // Send a test message
    const testMessage = `Test message ${Date.now()}`;
    await page.fill('#message-input', testMessage);
    await page.click('#send-button');

    // Verify message appears
    await expect(page.locator(`text="${testMessage}"`)).toBeVisible({ timeout: 5000 });
  });
});
EOF

# Run Playwright tests
npx playwright test --reporter=list

PLAYWRIGHT_RESULT=$?

# Step 6: Generate report
echo -e "\n${BLUE}Step 6: Generating test report...${NC}"

REPORT_FILE="$SCRIPT_DIR/test-report-$(date +%Y%m%d-%H%M%S).txt"

cat > "$REPORT_FILE" << EOF
================================================================================
COMMUNITAS INTEGRATION TEST REPORT
Generated: $(date)
================================================================================

TEST ENVIRONMENT:
- Test Network Nodes: $RUNNING_NODES
- Tauri PID: $TAURI_PID
- MCP Socket: $MCP_SOCKET
- Remote Debug Port: 9222

TEST RESULTS:
- Chrome DevTools MCP Tests: $([ $TEST_RESULT -eq 0 ] && echo "PASSED" || echo "FAILED")
- Playwright Tests: $([ $PLAYWRIGHT_RESULT -eq 0 ] && echo "PASSED" || echo "FAILED")

NETWORK TOPOLOGY:
EOF

# Add network info
for i in {1..8}; do
    if [ -f "$TEST_NET_DIR/node$i.log" ]; then
        IDENTITY=$(grep "Node identity:" "$TEST_NET_DIR/node$i.log" | tail -1 | cut -d: -f2 | xargs)
        PORT=$((9999 + i))
        echo "- Node $i: $IDENTITY (port $PORT)" >> "$REPORT_FILE"
    fi
done

echo "" >> "$REPORT_FILE"
echo "Log files available at: $TEST_NET_DIR/*.log" >> "$REPORT_FILE"
echo "=================================================================================" >> "$REPORT_FILE"

echo -e "${GREEN}Report saved to: $REPORT_FILE${NC}"

# Final summary
echo -e "\n${BLUE}========================================${NC}"
echo -e "${BLUE} Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"

if [ $TEST_RESULT -eq 0 ] && [ $PLAYWRIGHT_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ All tests PASSED${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests FAILED${NC}"
    echo -e "Chrome DevTools MCP: $([ $TEST_RESULT -eq 0 ] && echo "${GREEN}PASS${NC}" || echo "${RED}FAIL${NC}")"
    echo -e "Playwright: $([ $PLAYWRIGHT_RESULT -eq 0 ] && echo "${GREEN}PASS${NC}" || echo "${RED}FAIL${NC}")"
    exit 1
fi