# MCP Configuration for Communitas

## Overview
This directory contains MCP (Model Context Protocol) configuration to enable Claude Code to interact with the Communitas Tauri application and its development environment.

## Active MCP Servers

### 1. **Tauri MCP** (`tauri-mcp`)
**Purpose:** Direct interaction with Tauri applications
**Capabilities:**
- Launch and control Tauri apps
- Take screenshots
- Send keyboard/mouse inputs
- Execute JavaScript in webview
- Monitor resource usage
- Call IPC commands

### 2. **Chrome DevTools** (`chrome-devtools-mcp`)
**Purpose:** Debug and inspect web content in Tauri webview
**Capabilities:**
- Inspect DOM elements
- Monitor network requests
- Debug JavaScript
- Profile performance

### 3. **Filesystem** (`@modelcontextprotocol/server-filesystem`)
**Purpose:** File operations within the Communitas project
**Scope:** `/Users/davidirvine/Desktop/Devel/projects/communitas`

### 4. **AppleScript** (`@peakmojo/applescript-mcp`)
**Purpose:** macOS system integration
**Use Cases:**
- Launch development tools
- Control system settings
- Automate workflows

### 5. **Playwright** (`@playwright/mcp`)
**Purpose:** E2E testing and browser automation
**Use Cases:**
- Run automated tests
- Generate test scenarios
- Capture test screenshots

### 6. **GitHub** (`@modelcontextprotocol/server-github`)
**Purpose:** Repository management
**Token:** Already configured with your GitHub token
**Capabilities:**
- Create/manage issues
- Handle pull requests
- Access repository data

### 7. **Memory** (`@modelcontextprotocol/server-memory`)
**Purpose:** Persistent context across sessions
**Storage:** `.mcp-memory` directory

### 8. **Slack** (`@modelcontextprotocol/server-slack`)
**Status:** Needs configuration
**Note:** Add your Slack tokens if needed

### 9. **Puppeteer** (`@modelcontextprotocol/server-puppeteer`)
**Purpose:** Alternative browser automation
**Mode:** Non-headless for debugging

## Setup Instructions

### Quick Setup
```bash
# Run from the communitas directory
cd /Users/davidirvine/Desktop/Devel/projects/communitas
./scripts/setup-mcp.sh
```

### Manual Installation
```bash
# Install all MCP servers
npm install -g tauri-mcp
npm install -g chrome-devtools-mcp@latest
npm install -g @modelcontextprotocol/server-filesystem
npm install -g @peakmojo/applescript-mcp
npm install -g @playwright/mcp@latest
npm install -g @modelcontextprotocol/server-github
npm install -g @modelcontextprotocol/server-memory
```

## Using MCP with Communitas

### Launch and Debug Tauri App
```
"Launch the Communitas desktop app and take a screenshot"
"Execute JavaScript in the Communitas webview to check the current state"
"Monitor CPU and memory usage of the running Communitas app"
```

### Development Workflows
```
"Run the Communitas E2E tests with Playwright"
"Check for any TypeScript errors in the src directory"
"Build the Tauri app for production"
```

### Git Operations
```
"Check the status of the Communitas repository"
"Create a new branch for feature development"
"Review recent commits and changes"
```

## Project-Specific Commands

### Build Commands
```bash
# Development build
cargo tauri dev

# Production build
cargo tauri build

# Run tests
cargo test
npm test
```

### Tauri IPC Commands
The MCP can interact with these IPC commands:
- Entity management
- Membership operations
- P2P networking functions
- CRDT synchronization

## Troubleshooting

### MCP Not Connecting
1. Ensure Claude Desktop is using the project directory
2. Check that all npm packages are installed
3. Verify paths in `.mcp.json` are correct

### Tauri App Not Launching
1. Check that Tauri dependencies are installed:
   ```bash
   cargo tauri info
   ```
2. Ensure the app builds successfully:
   ```bash
   cargo build
   ```

### Permission Issues
Grant necessary permissions in System Preferences:
- Accessibility (for AppleScript)
- Screen Recording (for screenshots)
- Automation (for app control)

## Environment Variables
The following are configured in `.mcp.json`:
- `PROJECT_ROOT`: Points to Communitas directory
- `MEMORY_STORE_PATH`: MCP memory storage location
- `GITHUB_PERSONAL_ACCESS_TOKEN`: Your GitHub token
- `PUPPETEER_HEADLESS`: Set to false for debugging

## Related Files
- `.mcp.json` - MCP server configuration
- `.mcp.json.backup` - Backup of previous configuration
- `.mcp-memory/` - Memory storage directory
- `scripts/setup-mcp.sh` - Setup script

## Notes
- The MCP configuration is project-specific
- Changes to `.mcp.json` require Claude Desktop restart
- Use `npx -y` flag to auto-accept package installation
- Memory MCP provides context persistence across sessions

## Security Considerations
- GitHub token is included - ensure `.mcp.json` is in `.gitignore`
- Filesystem access is restricted to project directory
- Slack tokens need to be added manually if needed

## Support
For issues with MCP servers, check:
- [MCP Documentation](https://modelcontextprotocol.io)
- [Tauri MCP GitHub](https://github.com/tauri-apps/tauri-mcp)
- Individual MCP server repositories for specific issues
