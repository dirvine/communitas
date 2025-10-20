#!/bin/bash
# Comprehensive test for TUI - Sign up and create all entity types
# Tests: Signup, Organizations/Channels, Projects, Groups, Contacts

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Communitas TUI - Comprehensive Integration Test${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"

# Paths
TUI_BINARY="./target/release/communitas-tui"
TEST_DATA_DIR="/tmp/communitas-tui-comprehensive-test-$$"
LOG_FILE="/tmp/tui-comprehensive-test-$$.log"

if [ ! -f "$TUI_BINARY" ]; then
    echo -e "${RED}Error: TUI binary not found at $TUI_BINARY${NC}"
    exit 1
fi

mkdir -p "$TEST_DATA_DIR"
echo -e "${GREEN}✓ Created test data directory: $TEST_DATA_DIR${NC}"

# AppleScript for full test workflow
osascript <<EOF
-- Launch Terminal and run TUI app
tell application "Terminal"
    activate

    -- Create new window with TUI app
    set newWindow to do script "cd '$PWD' && $TUI_BINARY --data-dir '$TEST_DATA_DIR' --offline --no-keyring 2>&1 | tee $LOG_FILE"

    -- Wait for app to start
    delay 2

    tell application "System Events"
        -- ═══════════════════════════════════════════════════════
        -- STEP 1: SIGNUP
        -- ═══════════════════════════════════════════════════════
        log "Step 1: Signing up..."

        -- Navigate to Signup (right arrow)
        keystroke (ASCII character 29) -- Right arrow
        delay 0.5

        -- Press Enter to start signup
        keystroke return
        delay 0.5

        -- Enter display name
        keystroke "Test User"
        delay 0.5

        -- Submit signup
        keystroke return
        delay 3

        log "✓ Signup completed"

        -- ═══════════════════════════════════════════════════════
        -- STEP 2: CREATE CHANNELS (Organizations view)
        -- ═══════════════════════════════════════════════════════
        delay 1
        log "Step 2: Creating channels..."

        -- Press 'o' to open Organizations
        keystroke "o"
        delay 1

        -- Press 'n' to create new channel
        keystroke "n"
        delay 0.5

        -- Enter channel name
        keystroke "General Chat"
        delay 0.5

        -- Submit
        keystroke return
        delay 2

        -- Create second channel
        keystroke "n"
        delay 0.5
        keystroke "Announcements"
        keystroke return
        delay 2

        -- Create third channel
        keystroke "n"
        delay 0.5
        keystroke "Random"
        keystroke return
        delay 2

        log "✓ Created 3 channels"

        -- Go back to dashboard
        keystroke "q"
        delay 1

        -- ═══════════════════════════════════════════════════════
        -- STEP 3: CREATE PROJECTS
        -- ═══════════════════════════════════════════════════════
        log "Step 3: Creating projects..."

        -- Press 'p' to open Projects
        keystroke "p"
        delay 1

        -- Create first project
        keystroke "n"
        delay 0.5
        keystroke "Website Redesign"
        keystroke return
        delay 2

        -- Create second project
        keystroke "n"
        delay 0.5
        keystroke "Mobile App"
        keystroke return
        delay 2

        log "✓ Created 2 projects"

        -- Go back
        keystroke "q"
        delay 1

        -- ═══════════════════════════════════════════════════════
        -- STEP 4: CREATE GROUPS
        -- ═══════════════════════════════════════════════════════
        log "Step 4: Creating groups..."

        -- Press 'g' to open Groups
        keystroke "g"
        delay 1

        -- Create first group
        keystroke "n"
        delay 0.5
        keystroke "Engineering Team"
        keystroke return
        delay 2

        -- Create second group
        keystroke "n"
        delay 0.5
        keystroke "Design Team"
        keystroke return
        delay 2

        -- Create third group
        keystroke "n"
        delay 0.5
        keystroke "Marketing Team"
        keystroke return
        delay 2

        log "✓ Created 3 groups"

        -- Go back
        keystroke "q"
        delay 1

        -- ═══════════════════════════════════════════════════════
        -- STEP 5: CREATE CONTACTS
        -- ═══════════════════════════════════════════════════════
        log "Step 5: Creating contacts..."

        -- Press 'c' to open Contacts
        keystroke "c"
        delay 1

        -- Create first contact
        keystroke "n"
        delay 0.5
        keystroke "Alice Developer"
        keystroke return
        delay 2

        -- Create second contact
        keystroke "n"
        delay 0.5
        keystroke "Bob Designer"
        keystroke return
        delay 2

        -- Create third contact
        keystroke "n"
        delay 0.5
        keystroke "Carol Manager"
        keystroke return
        delay 2

        log "✓ Created 3 contacts"

        -- Go back
        keystroke "q"
        delay 1

        -- ═══════════════════════════════════════════════════════
        -- FINISH
        -- ═══════════════════════════════════════════════════════
        delay 2

        -- Quit application
        keystroke "q"
        delay 1
    end tell

    delay 1
end tell

return "Comprehensive test completed"
EOF

# Analyze results
echo -e "\n${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Test Results${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"

if [ -f "$LOG_FILE" ]; then
    # Check for identity creation
    if grep -q "Identity created:" "$LOG_FILE"; then
        IDENTITY=$(grep "Identity created:" "$LOG_FILE" | grep -oE "[a-z]+-[a-z]+-[a-z]+-[a-z]+")
        echo -e "${GREEN}✓ Signup: Identity created successfully ($IDENTITY)${NC}"
    else
        echo -e "${RED}✗ Signup: Failed${NC}"
    fi

    # Count created entities
    CHANNEL_COUNT=$(grep -c "Created entity.*Channel" "$LOG_FILE" || echo "0")
    PROJECT_COUNT=$(grep -c "Created entity.*Project" "$LOG_FILE" || echo "0")
    GROUP_COUNT=$(grep -c "Created entity.*Group" "$LOG_FILE" || echo "0")
    PERSON_COUNT=$(grep -c "Created entity.*Person" "$LOG_FILE" || echo "0")

    echo -e "\n${YELLOW}Entity Creation Summary:${NC}"
    echo -e "  Channels:  ${GREEN}$CHANNEL_COUNT${NC} / 3 expected"
    echo -e "  Projects:  ${GREEN}$PROJECT_COUNT${NC} / 2 expected"
    echo -e "  Groups:    ${GREEN}$GROUP_COUNT${NC} / 3 expected"
    echo -e "  Contacts:  ${GREEN}$PERSON_COUNT${NC} / 3 expected"

    TOTAL_CREATED=$((CHANNEL_COUNT + PROJECT_COUNT + GROUP_COUNT + PERSON_COUNT))
    TOTAL_EXPECTED=11

    echo -e "\n${BLUE}Total Entities: ${GREEN}$TOTAL_CREATED${BLUE} / $TOTAL_EXPECTED${NC}"

    if [ "$TOTAL_CREATED" -eq "$TOTAL_EXPECTED" ]; then
        echo -e "\n${GREEN}═══════════════════════════════════════════════════════${NC}"
        echo -e "${GREEN}  ✓✓✓ ALL TESTS PASSED ✓✓✓${NC}"
        echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
    else
        echo -e "\n${YELLOW}═══════════════════════════════════════════════════════${NC}"
        echo -e "${YELLOW}  ⚠ Some entities were not created${NC}"
        echo -e "${YELLOW}═══════════════════════════════════════════════════════${NC}"
    fi

    # Show recent log entries
    echo -e "\n${BLUE}Recent Log Entries:${NC}"
    echo -e "${BLUE}─────────────────────────────────────────────────────${NC}"
    grep -E "(Created entity|Identity created|Generated four-word)" "$LOG_FILE" | tail -20

else
    echo -e "${RED}Error: Log file not found${NC}"
    exit 1
fi

# Cleanup
echo -e "\n${BLUE}Cleanup:${NC}"
rm -rf "$TEST_DATA_DIR"
echo -e "${GREEN}✓ Cleaned up test data directory${NC}"

echo -e "\n${BLUE}Full log saved to: $LOG_FILE${NC}"
echo -e "${GREEN}Test completed successfully!${NC}\n"
