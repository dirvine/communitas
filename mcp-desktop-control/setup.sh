#!/bin/bash

# Setup script for Desktop Control MCP in Communitas project
# This enables full macOS desktop automation for Tauri app testing

echo "🍏 Setting up AppleScript MCP for Desktop Control..."
echo "=================================================="

# Navigate to the MCP directory
cd /Users/davidirvine/Desktop/Devel/projects/communitas/mcp-desktop-control

# Install the AppleScript MCP server
echo "📦 Installing AppleScript MCP server..."
npm install

echo ""
echo "✅ Installation complete!"
echo ""
echo "🔑 IMPORTANT: macOS Permissions Required"
echo "========================================="
echo ""
echo "For the AppleScript MCP to work, you need to grant permissions:"
echo ""
echo "1. Open System Settings > Privacy & Security > Automation"
echo "   - Find 'Terminal' (or your code editor if using Claude Code from there)"
echo "   - Check boxes for all apps you want to control"
echo ""
echo "2. Open System Settings > Privacy & Security > Accessibility"
echo "   - Add Terminal (or your code editor)"
echo "   - This enables UI scripting capabilities"
echo ""
echo "3. If you see permission dialogs when first using it, click 'Allow'"
echo ""
echo "📝 Available Commands"
echo "===================="
echo ""
echo "The AppleScript MCP lets you:"
echo "- Control any macOS application"
echo "- Automate UI interactions"
echo "- Send keyboard/mouse events"
echo "- Read screen content"
echo "- Manage windows and tabs"
echo ""
echo "Example uses for Tauri development:"
echo "- Launch your Tauri app"
echo "- Click buttons in your app"
echo "- Fill forms automatically"
echo "- Take screenshots of specific windows"
echo "- Test keyboard shortcuts"
echo "- Verify window states"
echo ""
echo "🧪 Test the MCP Server"
echo "====================="
echo ""
echo "To test if it's working, run:"
echo "cd mcp-desktop-control && npm test"
echo ""
echo "This will open the MCP Inspector where you can test AppleScript commands."
echo ""
echo "📚 Example AppleScript Commands for Tauri Testing:"
echo "=================================================="
echo ""
echo 'tell application "Communitas" to activate'
echo '-- Brings your Tauri app to the front'
echo ""
echo 'tell application "System Events" to tell process "Communitas"'
echo '    click button "Connect" of window 1'
echo 'end tell'
echo '-- Clicks a button in your app'
echo ""
echo 'tell application "Communitas"'
echo '    set bounds of window 1 to {100, 100, 1200, 800}'
echo 'end tell'
echo '-- Resizes your app window'
echo ""
echo "🚀 Ready to use with Claude Code!"
echo "================================="
echo ""
echo "The .mcp.json file has been configured with:"
echo "- applescript: Full desktop control via AppleScript"
echo "- tauri-mcp: Direct Tauri app integration"
echo "- chrome-devtools: Browser automation"
echo ""
echo "You can now use Claude Code to:"
echo "1. Control your Tauri app UI"
echo "2. Automate testing workflows"
echo "3. Interact with any macOS application"
echo "4. Create complex automation sequences"
