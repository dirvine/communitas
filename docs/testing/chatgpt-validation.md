# ChatGPT Widget Validation

## Overview

This document covers testing Communitas MCP Apps widgets in ChatGPT.

## Current Status

**Status: Pending MCP Apps Support**

As of January 2026, ChatGPT's MCP Apps support is not yet publicly available. This document will be updated when ChatGPT adds support for MCP interactive UI widgets.

## Expected Integration

When ChatGPT adds MCP Apps support, the integration should work similarly to Claude Desktop:

1. Configure MCP server in ChatGPT settings
2. ChatGPT will render `ui://` resources in iframes
3. postMessage communication enables interactivity

## Test Plan (When Available)

### Prerequisites
- ChatGPT Plus or Enterprise account (expected)
- MCP server configuration access
- Communitas MCP server running

### Configuration
```json
{
  "mcp_servers": {
    "communitas": {
      "url": "https://mcp.saorsalabs.com/mcp"
    }
  }
}
```

### Validation Steps

1. Connect MCP server to ChatGPT
2. Verify tool discovery
3. Test each widget (same as Claude Desktop)
4. Document any differences in behavior

## Compatibility Notes

### Expected Differences from Claude Desktop

| Feature | Claude Desktop | ChatGPT (Expected) |
|---------|---------------|-------------------|
| iframe sandbox | Yes | Yes |
| postMessage | Yes | Yes |
| CSP headers | Respected | Respected |
| Widget size | Fixed | TBD |
| Theme | Dark/Light | TBD |

### Potential Issues

1. **iframe dimensions** - ChatGPT may use different default sizes
2. **Theme detection** - May need explicit theme parameter
3. **Resource caching** - Different cache behavior
4. **Security policy** - May have stricter CSP

## Resources

- ChatGPT MCP Documentation: (Link when available)
- MCP Apps Specification: https://modelcontextprotocol.io/specification/ui

## Updates

| Date | Status | Notes |
|------|--------|-------|
| 2026-01-27 | Pending | Initial documentation created |

---

*This document will be updated when ChatGPT MCP Apps support becomes available.*
