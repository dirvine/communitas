#!/bin/bash

# Modern Shell Testing Script
# This script opens Chrome with remote debugging and captures screenshots

set -e

# Configuration
URL="http://localhost:5173/prototype/modern-shell"
SCREENSHOTS_DIR="./screenshots/modern-shell-test"
CHROME_DEBUG_PORT=9222

# Create screenshots directory
mkdir -p "$SCREENSHOTS_DIR"

echo "📸 Modern Shell Testing Script"
echo "================================"
echo ""
echo "URL: $URL"
echo "Screenshots will be saved to: $SCREENSHOTS_DIR"
echo ""

# Check if dev server is running
if ! curl -s "$URL" > /dev/null; then
  echo "❌ Dev server not running on port 5173"
  echo "Please run: npm run dev"
  exit 1
fi

echo "✅ Dev server is running"
echo ""

# Kill any existing Chrome remote debugging instances
echo "🔧 Cleaning up existing Chrome instances..."
pkill -f "remote-debugging-port=$CHROME_DEBUG_PORT" || true
sleep 2

# Launch Chrome with remote debugging
echo "🚀 Launching Chrome with remote debugging on port $CHROME_DEBUG_PORT..."
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --remote-debugging-port=$CHROME_DEBUG_PORT \
  --new-window \
  --user-data-dir=/tmp/chrome-devtools-test \
  "$URL" &

CHROME_PID=$!
sleep 3

echo "✅ Chrome launched (PID: $CHROME_PID)"
echo ""
echo "📋 MANUAL TESTING INSTRUCTIONS:"
echo "================================"
echo ""
echo "1. The page should now be open in Chrome"
echo "2. Open DevTools (Cmd+Option+I)"
echo "3. Check the Console tab for any errors"
echo "4. Follow the test cases in TEST_MODERN_SHELL.md"
echo ""
echo "Quick Test Checklist:"
echo "  ✓ Click different conversation types (Group, Org, Channel, Project)"
echo "  ✓ Click view mode chips (Chat, Threads, Files, Board, Tasks)"
echo "  ✓ Hover over messages to see action toolbar"
echo "  ✓ Click info icon to toggle drawer"
echo "  ✓ Click filter chips (All, Unread, Favourites)"
echo "  ✓ Hover over org members/projects for Edit/Remove buttons"
echo ""
echo "Screenshot Capture:"
echo "  - Press Cmd+Shift+3 to capture full screen"
echo "  - Press Cmd+Shift+4 to capture selection"
echo "  - Screenshots will be saved to Desktop by default"
echo ""
echo "Console Monitoring:"
echo "  - Look for RED error messages"
echo "  - Check for yellow warnings about missing props"
echo "  - Verify navigation events log correctly"
echo ""
echo "Press Ctrl+C to stop Chrome when done testing..."
echo ""

# Wait for user to finish testing
wait $CHROME_PID

echo ""
echo "✅ Testing complete!"
echo "📁 Review screenshots in: $SCREENSHOTS_DIR"
