# Communitas MCP Puppeteer Server

This MCP server provides browser automation capabilities for the Communitas application, allowing you to see and interact with browser windows.

## Features

- **Visible Browser Windows**: Run with visible browser windows by default
- **Connect to Existing Chrome**: Attach to running Chrome instances for debugging
- **Communitas App Integration**: Specialized tools for testing Communitas features
- **Screenshot Capture**: Take screenshots of pages and elements
- **Form Interaction**: Fill forms, click elements, and navigate pages

## Quick Start

### Run with Visible Browser Window

```bash
npm run mcp:puppeteer:visible
```

This will launch a visible Chrome browser window that you can see and interact with.

### Run Headless (Traditional)

```bash
npm run mcp:puppeteer:headless
```

### Connect to Existing Chrome Instance

1. **Launch Chrome with debugging enabled:**

   **Windows:**
   ```cmd
   "C:\Program Files\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9222
   ```

   **macOS:**
   ```bash
   /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222
   ```

   **Linux:**
   ```bash
   google-chrome --remote-debugging-port=9222
   ```

2. **Navigate to your desired page in Chrome**

3. **Use the MCP tool to connect:**
   ```
   browser_connect_active_chrome
   ```

## Environment Variables

- `MCP_BROWSER_HEADLESS=1` - Run in headless mode (default: visible)
- `MCP_BROWSER_URL` - Default URL to navigate to (default: http://localhost:1420)
- `MCP_CHROME_DEBUG_PORT` - Chrome debugging port (default: 9222)
- `PUPPETEER_EXECUTABLE_PATH` - Custom Chrome/Chromium executable path

## Available Tools

### Browser Control
- `browser_navigate` - Navigate to a URL
- `browser_click` - Click elements by CSS selector
- `browser_click_text` - Click elements by text content
- `browser_fill` - Fill input fields
- `browser_fill_by_label` - Fill inputs by associated label
- `browser_type` - Type text into elements
- `browser_snapshot` - Take screenshots
- `browser_eval` - Execute JavaScript
- `browser_wait_for` - Wait for selectors
- `browser_wait_text` - Wait for text to appear
- `browser_query_all_texts` - Get text content from multiple elements

### Chrome Connection
- `browser_connect_active_chrome` - Connect to existing Chrome instance

### Communitas App Tools
- `app_test_identity` - Test identity functionality
- `app_setup_workspace` - Setup workspace
- `app_test_groups` - Test group functionality
- `app_test_group_messaging` - Test group messaging
- `app_offline_simulate` - Simulate offline mode
- `app_list_groups` - List groups
- `app_offline_stats` - Get offline storage stats
- `app_click_tab` - Click top-level tabs
- `app_offline_full_flow` - Run complete offline flow

## MCP Configuration

Add this to your MCP client configuration:

```json
{
  "mcpServers": {
    "communitas-puppeteer": {
      "command": "node",
      "args": ["/path/to/communitas/servers/mcp-puppeteer/server.js"],
      "env": {
        "MCP_BROWSER_HEADLESS": "0"
      }
    }
  }
}
```

## Usage Examples

### Basic Navigation
```
browser_navigate: { "url": "http://localhost:1420" }
```

### Connect to Existing Chrome
```
browser_connect_active_chrome: { "targetUrl": "http://localhost:1420" }
```

### Take Screenshot
```
browser_snapshot: { "fullPage": true }
```

### Test Communitas Features
```
app_test_identity: {}
app_setup_workspace: {}
```

## Troubleshooting

### Browser Won't Launch
- Ensure Chrome/Chromium is installed
- Set `PUPPETEER_EXECUTABLE_PATH` if using custom Chrome location
- Check that no other Chrome instances are using the debugging port

### Cannot Connect to Existing Chrome
- Verify Chrome is running with `--remote-debugging-port=9222`
- Check that the port isn't blocked by firewall
- Ensure you're not trying to connect to an extension page

### Communitas App Tools Not Working
- Make sure the Communitas app is running on the expected URL
- Check that the app has loaded the required test utilities
- Verify the page has finished loading before using app tools

## Development

The server is built with:
- **Puppeteer** - Browser automation
- **@modelcontextprotocol/sdk** - MCP protocol implementation
- **Zod** - Schema validation

To modify the server, edit `server.js` and restart the MCP server.</content>
</xai:function_call">Now let me update the todo list to mark the configuration task as completed and add a final testing task. 

<xai:function_call name="todowrite">
<parameter name="todos">[{"content":"Modify existing MCP Puppeteer server to show browser windows instead of running headless","status":"completed","priority":"high","id":"modify-headless-mode"},{"content":"Add option to connect to existing Chrome browser instances for debugging","status":"completed","priority":"medium","id":"add-chrome-debugging"},{"content":"Update MCP server configuration to support both headless and visible modes","status":"completed","priority":"medium","id":"update-server-config"},{"content":"Test the browser window visibility with the Communitas app","status":"in_progress","priority":"low","id":"test-browser-visibility"}]