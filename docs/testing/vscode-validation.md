# VS Code Copilot Widget Validation

## Overview

This document covers testing Communitas MCP Apps widgets in VS Code with GitHub Copilot.

## Current Status

**Status: Limited Support**

As of January 2026, VS Code Copilot has limited MCP support through Copilot Extensions. Full MCP Apps support (interactive UI widgets) may be available through future updates.

## Current Integration Options

### Option 1: Copilot Extensions

VS Code Copilot supports tool use through Extensions, but interactive UI rendering is limited.

**Configuration:**
```json
// .vscode/settings.json
{
  "github.copilot.advanced": {
    "mcpServers": {
      "communitas": {
        "command": "cargo",
        "args": ["run", "-p", "communitas-mcp", "--", "--demo"],
        "cwd": "${workspaceFolder}"
      }
    }
  }
}
```

### Option 2: MCP for VS Code Extension

Third-party extensions may provide MCP Apps support. Check VS Code Marketplace for:
- "MCP Client"
- "Model Context Protocol"

## Test Plan

### With Tool Support Only

1. Configure MCP server in VS Code settings
2. Ask Copilot: "@workspace list my contacts from Communitas"
3. Verify tool is called and response returned
4. Note: Widget may not render, only text response

### With Full MCP Apps Support (When Available)

1. Install MCP Apps extension
2. Configure Communitas server
3. Verify widget renders in panel or sidebar
4. Test interactivity

## Compatibility Notes

### VS Code Environment

| Feature | Status | Notes |
|---------|--------|-------|
| Tool discovery | Supported | Via Copilot Extensions |
| Tool execution | Supported | Text responses |
| UI resources | Limited | May require extension |
| Widget rendering | TBD | Depends on extension |

### Differences from Claude Desktop

1. **Rendering context**: VS Code uses webview panels, not conversation UI
2. **Interaction model**: May use sidebar or panel instead of inline
3. **Theme**: Automatically uses VS Code theme
4. **State persistence**: Webview state may persist across sessions

## Workarounds

### Text-Only Mode

When interactive widgets aren't available, use text-based tools:

```
@workspace Use Communitas to list contacts as JSON
```

### Webview Panel

Extensions may provide a dedicated panel:

```
Command Palette > Communitas: Open Widget Panel
```

## Resources

- VS Code Copilot Extensions: https://docs.github.com/en/copilot/using-github-copilot/using-extensions-to-integrate-external-tools-with-copilot-chat
- VS Code Webview API: https://code.visualstudio.com/api/extension-guides/webview

## Updates

| Date | Status | Notes |
|------|--------|-------|
| 2026-01-27 | Limited | Tool support only, UI widgets pending |

---

*This document will be updated as VS Code MCP Apps support evolves.*
