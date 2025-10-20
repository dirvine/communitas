#!/bin/bash
# Test script for TUI signup automation using AppleScript

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting TUI Signup Test${NC}"

# Path to the TUI binary
TUI_BINARY="./target/release/communitas-tui"

if [ ! -f "$TUI_BINARY" ]; then
    echo -e "${RED}Error: TUI binary not found at $TUI_BINARY${NC}"
    exit 1
fi

# Test data directory
TEST_DATA_DIR="/tmp/communitas-tui-test-$$"
mkdir -p "$TEST_DATA_DIR"
echo -e "${GREEN}Created test data directory: $TEST_DATA_DIR${NC}"

# AppleScript to automate the signup flow
osascript <<EOF
-- Launch Terminal and run TUI app
tell application "Terminal"
    activate

    -- Create a new window with the TUI app
    set newWindow to do script "cd '$PWD' && $TUI_BINARY --data-dir '$TEST_DATA_DIR' --offline --no-keyring 2>&1 | tee /tmp/tui-test-output-$$.log"

    -- Wait for the app to start
    delay 2

    -- Navigate to Signup (right arrow)
    tell application "System Events"
        keystroke (ASCII character 29) -- Right arrow key
        delay 0.5

        -- Press Enter to select Signup
        keystroke return
        delay 0.5

        -- Type display name
        keystroke "Test User"
        delay 0.5

        -- Press Enter to create identity
        keystroke return
        delay 2

        -- Wait for identity creation
        delay 3

        -- Press 'q' to quit
        keystroke "q"
    end tell

    delay 1
end tell

-- Return success
return "Test completed"
EOF

# Check the output log
echo -e "${BLUE}Checking test output...${NC}"
if [ -f "/tmp/tui-test-output-$$.log" ]; then
    echo -e "${GREEN}Test output:${NC}"
    cat "/tmp/tui-test-output-$$.log"

    # Check if identity was created (look for four-word pattern)
    if grep -E "[a-z]+-[a-z]+-[a-z]+-[a-z]+" "/tmp/tui-test-output-$$.log" > /dev/null; then
        echo -e "${GREEN}✓ Identity created successfully!${NC}"
    else
        echo -e "${RED}✗ Identity creation failed${NC}"
        exit 1
    fi
else
    echo -e "${RED}No output log found${NC}"
    exit 1
fi

# Cleanup
rm -rf "$TEST_DATA_DIR"
echo -e "${GREEN}Test completed successfully!${NC}"
