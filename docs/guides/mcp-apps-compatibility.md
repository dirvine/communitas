# MCP Apps Browser/Host Compatibility Matrix

## Overview

This document details the compatibility of Communitas MCP Apps widgets across different MCP hosts and browser engines. Use this as a reference when testing or deploying widgets.

## MCP Host Compatibility

### Support Status Legend

| Status | Meaning |
|--------|---------|
| Full | All features work as expected |
| Partial | Core features work, some limitations |
| Limited | Basic functionality only |
| Planned | Support expected in future |
| N/A | Not applicable |

### Host Compatibility Matrix

| Feature | Claude Desktop | ChatGPT | VS Code Copilot | Custom MCP |
|---------|---------------|---------|-----------------|------------|
| **Tool Discovery** | Full | Planned | Partial | Full |
| **Tool Execution** | Full | Planned | Full | Full |
| **UI Resources** | Full | Planned | Limited | Full |
| **Widget Rendering** | Full | Planned | Limited | Full |
| **postMessage API** | Full | Planned | Limited | Full |
| **Iframe Sandbox** | Full | Planned | Full | Full |
| **Theme Support** | Full | Planned | Full | Varies |
| **Resize Handling** | Full | Planned | Partial | Full |

### Claude Desktop

**Status: Full Support**

Claude Desktop is the primary target for MCP Apps and provides complete support.

| Feature | Status | Notes |
|---------|--------|-------|
| Widget Rendering | Full | Renders in conversation UI |
| postMessage | Full | Bidirectional JSON-RPC |
| CSP Enforcement | Full | Respects resource CSP |
| Theme | Full | Dark/light mode sync |
| Interaction | Full | Click, drag, scroll |
| History | Full | Widgets persist in conversation |

**Configuration:**
```json
{
  "mcpServers": {
    "communitas": {
      "command": "path/to/communitas-mcp",
      "args": ["--demo"]
    }
  }
}
```

**Supported Versions:**
- Claude Desktop 1.0+ (macOS)
- Claude Desktop 1.0+ (Windows)
- Claude Desktop 1.0+ (Linux)

### ChatGPT

**Status: Planned Support**

As of January 2026, ChatGPT's MCP Apps support is not yet publicly available.

| Feature | Status | Notes |
|---------|--------|-------|
| Tool Discovery | Planned | Via MCP server config |
| Tool Execution | Planned | Text responses |
| UI Resources | Planned | iframe rendering |
| Widget Rendering | Planned | TBD |
| postMessage | Planned | Expected similar to Claude |

**Expected Configuration:**
```json
{
  "mcp_servers": {
    "communitas": {
      "url": "https://mcp.saorsalabs.com/mcp"
    }
  }
}
```

### VS Code Copilot

**Status: Limited Support**

VS Code Copilot has limited MCP support through Copilot Extensions.

| Feature | Status | Notes |
|---------|--------|-------|
| Tool Discovery | Full | Via Extensions |
| Tool Execution | Full | Text responses |
| UI Resources | Limited | May require extension |
| Widget Rendering | Limited | Webview panel |
| postMessage | Limited | Different model |

**Configuration:**
```json
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

**Workarounds:**
- Use text-only tool responses for maximum compatibility
- Third-party MCP extensions may provide better UI support

### Custom MCP Hosts

**Status: Full Support (Implementation Dependent)**

Custom MCP implementations can fully support Communitas widgets by:

1. Implementing `resources/read` for `ui://` URIs
2. Rendering HTML content in sandboxed iframe
3. Implementing postMessage JSON-RPC bridge
4. Respecting CSP from resource metadata

## Browser Engine Compatibility

### Rendering Engine Matrix

| Engine | Version | Status | Notes |
|--------|---------|--------|-------|
| Chromium | 120+ | Full | Claude Desktop, Electron |
| WebKit | 17+ | Full | macOS Safari, iOS |
| Gecko | 120+ | Full | Firefox |
| WebView2 | Edge 120+ | Full | Windows native |

### JavaScript Feature Requirements

| Feature | Required | Fallback |
|---------|----------|----------|
| ES2020 | Yes | No transpilation |
| async/await | Yes | - |
| fetch API | No | MCP bridge |
| CSS Grid | Yes | - |
| CSS Variables | Yes | - |
| postMessage | Yes | - |
| structuredClone | No | JSON parse/stringify |

### CSS Feature Requirements

| Feature | Required | Notes |
|---------|----------|-------|
| Flexbox | Yes | Layout |
| Grid | Yes | Widget layouts |
| Custom Properties | Yes | Theming |
| :focus-visible | Yes | Accessibility |
| prefers-color-scheme | Yes | Theme detection |

## Widget Feature Matrix

### Widget vs Host Compatibility

| Widget | Claude Desktop | ChatGPT | VS Code | Notes |
|--------|---------------|---------|---------|-------|
| Contacts | Full | Planned | Limited | |
| Messages | Full | Planned | Limited | |
| Kanban | Full | Planned | Limited | Drag-drop requires full support |
| Drive | Full | Planned | Limited | Preview may be text-only |
| Canvas | Full | Planned | Limited | SVG rendering |
| Settings | Full | Planned | Full | Simple form |
| Search | Full | Planned | Full | Text input/output |
| Notifications | Full | Planned | Full | Simple list |

### Feature Support by Widget

| Feature | Contacts | Messages | Kanban | Drive | Canvas | Settings | Search | Notifications |
|---------|----------|----------|--------|-------|--------|----------|--------|---------------|
| Search | Yes | Yes | Yes | Yes | No | No | Yes | Yes |
| Drag-drop | No | No | Yes | Yes | No | No | No | No |
| Real-time | No | Yes | Yes | No | Yes | No | No | Yes |
| Pagination | Yes | Yes | No | Yes | No | No | Yes | Yes |
| Actions | Yes | Yes | Yes | Yes | Yes | Yes | No | Yes |

## Theme Compatibility

### Theme Detection

| Host | Detection Method | Notes |
|------|------------------|-------|
| Claude Desktop | CSS media query | prefers-color-scheme |
| ChatGPT | TBD | Expected similar |
| VS Code | Host theme class | .vscode-dark/.vscode-light |
| Custom | CSS variable | --theme: dark/light |

### Theme Variables

Widgets use these CSS custom properties for consistent theming:

```css
:root {
  /* Light theme */
  --bg-primary: #ffffff;
  --bg-secondary: #f5f5f5;
  --text-primary: #1a1a1a;
  --text-secondary: #666666;
  --border-color: #e0e0e0;
  --accent-color: #007bff;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --text-primary: #ffffff;
    --text-secondary: #999999;
    --border-color: #404040;
    --accent-color: #4da3ff;
  }
}
```

## Security Compatibility

### Sandbox Attributes

All MCP hosts should render widgets with these sandbox attributes:

```html
<iframe
  sandbox="allow-scripts allow-forms"
  src="widget.html"
></iframe>
```

**Allowed:**
- `allow-scripts` - JavaScript execution
- `allow-forms` - Form submission (via postMessage)

**Blocked:**
- `allow-same-origin` - Cross-origin restrictions maintained
- `allow-popups` - No new windows
- `allow-top-navigation` - Cannot navigate parent

### CSP Compatibility

| Directive | Required | Notes |
|-----------|----------|-------|
| default-src 'self' | Yes | Block external resources |
| script-src 'self' | Yes | No inline scripts |
| style-src 'self' 'unsafe-inline' | Yes | Allow style tags |
| img-src 'self' data: | Yes | Allow data URIs |
| connect-src 'none' | Yes | Use postMessage only |
| frame-src 'none' | Yes | No nested iframes |

## Accessibility Compatibility

### Screen Reader Support

| Screen Reader | Platform | Status |
|---------------|----------|--------|
| VoiceOver | macOS | Full |
| NVDA | Windows | Full |
| JAWS | Windows | Full |
| Orca | Linux | Full |
| ChromeVox | Chrome | Full |

### WCAG Compliance

All widgets target WCAG 2.1 AA compliance:

| Criterion | Status | Notes |
|-----------|--------|-------|
| 1.4.3 Contrast | Pass | 4.5:1 minimum |
| 2.1.1 Keyboard | Pass | Full keyboard access |
| 2.4.7 Focus Visible | Pass | Visible focus indicators |
| 4.1.2 Name, Role, Value | Pass | ARIA attributes |

## Known Limitations

### Cross-Host Differences

| Issue | Claude Desktop | ChatGPT | VS Code |
|-------|---------------|---------|---------|
| Widget size | Fixed | TBD | Variable |
| Scroll behavior | Native | TBD | Webview |
| Keyboard shortcuts | Host captures some | TBD | Host captures some |
| Copy/paste | Works | TBD | Works |

### Platform-Specific Issues

| Platform | Issue | Workaround |
|----------|-------|------------|
| Windows | High DPI scaling | CSS zoom detection |
| Linux | Font rendering | System font fallbacks |
| macOS | Rubber-band scroll | overflow: hidden on body |

## Testing Recommendations

### Minimum Test Matrix

For each release, test on:

1. **Claude Desktop** (macOS) - Primary
2. **Claude Desktop** (Windows) - Cross-platform
3. **VS Code** (any) - Alternative host

### Full Test Matrix

For major releases, additionally test:

4. **Claude Desktop** (Linux)
5. **ChatGPT** (when available)
6. **Custom MCP host** (reference implementation)

### Automated Testing

Use the test harness at `communitas-mcp/ui-bundles/test/` for:

- Widget render tests (all widgets)
- MCP bridge tests (postMessage)
- Security tests (CSP, XSS)
- Accessibility tests (keyboard, ARIA)

## Version History

| Date | Version | Changes |
|------|---------|---------|
| 2026-01-27 | 1.0 | Initial compatibility matrix |

---

*This document is updated as new MCP hosts add support and compatibility issues are discovered.*
