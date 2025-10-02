# Desktop Control MCP for Communitas Tauri App

This MCP server provides full macOS desktop control capabilities using AppleScript, perfect for testing and automating your Tauri application.

## 🚀 Quick Setup

### 1. Install the MCP Server

```bash
cd ~/Desktop/Devel/projects/communitas/mcp-desktop-control
npm install
```

### 2. Grant Required Permissions

**IMPORTANT**: For AppleScript to control applications, macOS requires explicit permissions:

1. **Automation Permissions**
   - Open: System Settings > Privacy & Security > Automation
   - Find "Terminal" (or "Claude" if using Claude Desktop app)
   - Check boxes for all apps you want to control (especially your Tauri app)

2. **Accessibility Permissions**
   - Open: System Settings > Privacy & Security > Accessibility
   - Click the lock to make changes
   - Add Terminal (or your code editor)
   - This enables UI scripting (clicking buttons, typing, etc.)

3. **First-Use Dialogs**
   - When you first use the MCP, macOS may show permission dialogs
   - Always click "Allow" to grant access

## 🎯 What This Enables

With this MCP server, Claude Code can:

- **Launch and control your Tauri app**
- **Automate UI testing** (click buttons, fill forms, navigate)
- **Verify app behavior** (check window states, read content)
- **Take screenshots** of specific windows
- **Send keyboard/mouse events**
- **Control any macOS application** for integration testing

## 📝 Configuration

The `.mcp.json` file in the project root is already configured with three MCP servers:

```json
{
  "mcpServers": {
    "applescript": {
      "command": "npx",
      "args": ["@peakmojo/applescript-mcp"]
    },
    "tauri-mcp": {
      "command": "node",
      "args": ["/Users/davidirvine/mcp-servers/tauri-plugin-mcp/mcp-server-ts/build/index.js"],
      "env": {
        "TAURI_MCP_CONNECTION_TYPE": "tcp",
        "TAURI_MCP_TCP_HOST": "127.0.0.1",
        "TAURI_MCP_TCP_PORT": "4000"
      }
    },
    "chrome-devtools": {
      "command": "npx",
      "args": ["chrome-devtools-mcp@latest"],
      "env": {
        "CHROME_DEBUG_PORT": "9222"
      }
    }
  }
}
```

## 🧪 Testing the Setup

### Test with MCP Inspector

```bash
cd mcp-desktop-control
npx @modelcontextprotocol/inspector npx @peakmojo/applescript-mcp
```

This opens a web interface where you can test AppleScript commands directly.

### Example Test Commands

Try these in the MCP Inspector:

```applescript
// Show a notification
display notification "MCP is working!" with title "Test"

// Get list of running apps
tell application "System Events"
    return name of every process
end tell

// Open Finder
tell application "Finder" to activate
```

## 💡 Usage Examples for Tauri Testing

### Example 1: Launch and Position Your App

```applescript
tell application "Communitas"
    activate
    set bounds of window 1 to {100, 100, 1200, 800}
end tell
```

### Example 2: Automated UI Testing

```applescript
-- Click a button
tell application "System Events" to tell process "Communitas"
    click button "Connect" of window 1
end tell

-- Fill a form field
tell application "System Events" to tell process "Communitas"
    set value of text field 1 of window 1 to "test@example.com"
    set value of text field 2 of window 1 to "password123"
    click button "Login" of window 1
end tell
```

### Example 3: Verify App State

```applescript
tell application "System Events" to tell process "Communitas"
    -- Get all button names
    set buttonNames to name of every button of window 1
    
    -- Check if a specific element exists
    if exists button "Connect" of window 1 then
        return "Connect button found"
    else
        return "Connect button not found"
    end if
end tell
```

### Example 4: Integration Testing with Browser

```applescript
-- Open a URL in Chrome
tell application "Google Chrome"
    activate
    open location "http://localhost:5173"
end tell

-- Wait for page to load
delay 2

-- Switch back to your Tauri app
tell application "Communitas" to activate
```

## 🔧 Troubleshooting

### "Not allowed to send keystrokes"
- Add Terminal/Claude to System Settings > Privacy & Security > Accessibility

### "Can't get window 1 of application"
- Make sure the app is running
- Check the app name matches exactly (case-sensitive)

### Commands don't work
1. Check permissions in System Settings
2. Restart Terminal/Claude after granting permissions
3. Try running a simple test like `display notification "test"`

## 📚 Resources

- [AppleScript Language Guide](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/introduction/ASLR_intro.html)
- [UI Element Inspector](https://support.apple.com/guide/accessibility-inspector/welcome/mac) - Helps find UI elements to automate
- Check `tauri-test-examples.applescript` for more examples

## 🎯 Next Steps

1. Start Claude Code in this directory
2. Ask Claude to test your Tauri app using AppleScript
3. Create automated test workflows
4. Build integration tests that span multiple applications

## 🔐 Security Note

The AppleScript MCP gives full control over your Mac. Only use it with trusted AI models and be aware of the commands being executed. The server only runs locally and doesn't expose any network endpoints.
