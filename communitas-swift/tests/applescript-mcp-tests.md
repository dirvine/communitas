# AppleScript MCP Test Plan for Communitas macOS

This document outlines comprehensive testing scenarios for the Communitas macOS application using AppleScript MCP.

## Prerequisites

1. **Build the macOS app**:
   ```bash
   cd communitas-swift
   make                    # Build Rust bindings + generate Swift code
   open CommunitasKit/Package.swift  # Open in Xcode
   # Build the app target (Cmd+B)
   ```

2. **Launch the app** before running tests

3. **AppleScript MCP server** must be running

## Test Categories

### 1. Application Lifecycle Tests

#### Test 1.1: App Launch Verification
```applescript
-- Verify app launches successfully
tell application "Communitas"
    activate
    delay 2
end tell

tell application "System Events"
    tell process "Communitas"
        return exists window 1
    end tell
end tell
```

**Expected**: Returns `true`

#### Test 1.2: Window Properties
```applescript
tell application "System Events"
    tell process "Communitas"
        tell window 1
            return {name, size, position}
        end tell
    end tell
end tell
```

**Expected**: Window name contains "Communitas", valid size/position

### 2. Initialization Flow Tests

#### Test 2.1: Loading State Detection
```applescript
-- Check for loading indicator during initialization
tell application "System Events"
    tell process "Communitas"
        tell window 1
            -- Look for progress indicator or "Initializing" text
            return exists (first static text whose value contains "Initializing")
        end tell
    end tell
end tell
```

#### Test 2.2: Profile Display After Init
```applescript
-- Wait for initialization, then check profile display
tell application "System Events"
    tell process "Communitas"
        tell window 1
            delay 3  -- Wait for init
            -- Check for four-word display
            return exists (first static text whose value contains "-")
        end tell
    end tell
end tell
```

**Expected**: Four-word identity displayed (e.g., "ocean-forest-moon-star")

### 3. Identity & Profile Tests

#### Test 3.1: Display Name Verification
```applescript
tell application "System Events"
    tell process "Communitas"
        tell window 1
            -- Find the display name text
            set displayNameTexts to every static text whose value is not ""
            repeat with t in displayNameTexts
                if (value of t) contains "User" then
                    return value of t
                end if
            end repeat
            return "not found"
        end tell
    end tell
end tell
```

#### Test 3.2: Four-Word Identity Format
```applescript
-- Verify four-word format (word-word-word-word)
tell application "System Events"
    tell process "Communitas"
        tell window 1
            set allTexts to value of every static text
            repeat with t in allTexts
                if t contains "-" then
                    -- Count hyphens - should be 3 for four words
                    set hyphenCount to 0
                    repeat with c in characters of t
                        if c is "-" then set hyphenCount to hyphenCount + 1
                    end repeat
                    if hyphenCount is 3 then return t
                end if
            end repeat
            return "no four-word found"
        end tell
    end tell
end tell
```

### 4. Network Status Tests

#### Test 4.1: Network Status Indicator
```applescript
-- Check network status indicator presence
tell application "System Events"
    tell process "Communitas"
        tell window 1
            -- Look for Online/Offline text or status indicator
            if exists (first static text whose value is "Online") then
                return "online"
            else if exists (first static text whose value is "Offline") then
                return "offline"
            else
                return "status unknown"
            end if
        end tell
    end tell
end tell
```

#### Test 4.2: Network Status Color Indicator
```applescript
-- Check for status indicator (colored dot)
tell application "System Events"
    tell process "Communitas"
        tell window 1
            return exists image 1  -- Status indicator image/shape
        end tell
    end tell
end tell
```

### 5. Error Handling Tests

#### Test 5.1: Error Message Display
```applescript
-- Simulate error condition and check for error UI
tell application "System Events"
    tell process "Communitas"
        tell window 1
            -- Check for error icon or text
            if exists (first static text whose value contains "Error") then
                return value of first static text whose value contains "Error"
            else
                return "no error displayed"
            end if
        end tell
    end tell
end tell
```

### 6. UI Navigation Tests

#### Test 6.1: Navigation View Presence
```applescript
tell application "System Events"
    tell process "Communitas"
        tell window 1
            -- Check for NavigationView sidebar or navigation elements
            return count of groups
        end tell
    end tell
end tell
```

#### Test 6.2: Title Bar Verification
```applescript
tell application "System Events"
    tell process "Communitas"
        tell window 1
            return title
        end tell
    end tell
end tell
```

**Expected**: "Communitas"

### 7. Accessibility Tests

#### Test 7.1: VoiceOver Compatibility
```applescript
-- Check accessibility labels are present
tell application "System Events"
    tell process "Communitas"
        tell window 1
            set accessibleElements to every UI element whose description is not ""
            return count of accessibleElements
        end tell
    end tell
end tell
```

**Expected**: Count > 0 (accessible elements exist)

#### Test 7.2: Element Descriptions
```applescript
tell application "System Events"
    tell process "Communitas"
        tell window 1
            set descriptions to {}
            repeat with elem in every UI element
                try
                    set end of descriptions to description of elem
                end try
            end repeat
            return descriptions
        end tell
    end tell
end tell
```

### 8. Integration Test Scenarios

#### Test 8.1: Full Initialization Flow
```applescript
-- Complete init flow test
tell application "Communitas"
    activate
end tell

tell application "System Events"
    tell process "Communitas"
        tell window 1
            -- Step 1: Check loading
            delay 1
            set loadingVisible to exists (first static text whose value contains "Initializing")

            -- Step 2: Wait for profile
            delay 3
            set profileVisible to exists (first static text whose value contains "-")

            -- Step 3: Check network status
            set networkStatus to "unknown"
            if exists (first static text whose value is "Online") then
                set networkStatus to "online"
            else if exists (first static text whose value is "Offline") then
                set networkStatus to "offline"
            end if

            return {loadingVisible, profileVisible, networkStatus}
        end tell
    end tell
end tell
```

**Expected**: `{true, true, "offline"}` (starts offline until networking implemented)

## Running Tests with Claude Code

To run these tests using the AppleScript MCP from Claude Code:

1. **Invoke the mcp-applescript skill**:
   ```
   /skill mcp-applescript
   ```

2. **Execute individual tests**:
   ```
   Use the AppleScript MCP to run test 1.1 (App Launch Verification)
   ```

3. **Run full test suite**:
   ```
   Run all AppleScript MCP tests for Communitas macOS app
   ```

## Test Results Template

| Test ID | Description | Expected | Actual | Status |
|---------|-------------|----------|--------|--------|
| 1.1 | App Launch | true | | |
| 1.2 | Window Props | valid | | |
| 2.1 | Loading State | true | | |
| 2.2 | Profile Display | four-word | | |
| 3.1 | Display Name | "User" | | |
| 3.2 | Four-Word Format | word-word-word-word | | |
| 4.1 | Network Status | offline | | |
| 6.2 | Title Bar | "Communitas" | | |

## Automated Test Script

Save this as `run_all_tests.scpt`:

```applescript
-- Automated test runner for Communitas macOS

on runTest(testName, testScript)
    try
        set result to run script testScript
        return {testName, "PASS", result}
    on error errMsg
        return {testName, "FAIL", errMsg}
    end try
end runTest

-- Launch app
tell application "Communitas" to activate
delay 3

-- Run tests
set testResults to {}

set end of testResults to runTest("App Launch", "tell application \"System Events\" to tell process \"Communitas\" to return exists window 1")

set end of testResults to runTest("Window Title", "tell application \"System Events\" to tell process \"Communitas\" to return title of window 1")

-- Output results
return testResults
```

## Notes

- Tests assume the app bundle is named "Communitas"
- Adjust `delay` values based on system performance
- Some tests may need modification based on final UI implementation
- Network status tests depend on networking being implemented
