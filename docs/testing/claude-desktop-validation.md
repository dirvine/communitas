# Claude Desktop Widget Validation

## Overview

This document covers testing Communitas MCP Apps widgets in Claude Desktop.

## Prerequisites

- Claude Desktop installed
- Communitas MCP server configured (see [Claude Desktop Setup](../guides/claude-desktop-setup.md))

## Test Procedure

### 1. Start MCP Server

```bash
cd communitas
cargo run -p communitas-mcp -- --demo
```

### 2. Verify Connection

In Claude Desktop, start a conversation and ask:

```
What MCP tools are available from Communitas?
```

Claude should list 187 tools including contacts, messaging, kanban, etc.

### 3. Test Each Widget

#### Contacts Widget

1. Ask Claude: "Show me my contacts"
2. Verify interactive contact list appears
3. Test search functionality by typing in search box
4. Test favorite toggle on a contact
5. Click a contact to see details

**Expected Behavior:**
- Contact list renders with names and avatars
- Presence indicators show (online/away/offline)
- Search filters contacts in real-time
- Favorites toggle updates immediately

#### Messages Widget

1. Ask Claude: "Show my message threads"
2. Verify thread list appears
3. Click a thread to view messages
4. Type a message and send

**Expected Behavior:**
- Thread list shows with unread counts
- Messages display with timestamps
- Compose area accepts input
- Sent messages appear in thread

#### Kanban Widget

1. Ask Claude: "Show my Kanban boards"
2. Verify board list appears
3. Click a board to view columns and cards
4. Try dragging a card between columns

**Expected Behavior:**
- Board overview shows card counts
- Columns display with cards
- Cards show title, tags, due dates
- Drag-drop updates card position

#### Drive Widget

1. Ask Claude: "Show my files"
2. Verify file list appears
3. Click folders to navigate
4. Preview a file (image or text)

**Expected Behavior:**
- Files and folders display with icons
- File sizes shown
- Preview panel shows file content
- Quota meter displays storage usage

#### Canvas Widget

1. Ask Claude: "Show my canvas"
2. Verify canvas elements render
3. Toggle layer visibility
4. Navigate history with undo/redo

**Expected Behavior:**
- Canvas elements render correctly
- Layer panel shows all layers
- Layer toggles work
- History controls navigate states

#### Settings Widget

1. Invoke settings through a tool or ask Claude
2. Verify settings panel appears
3. Toggle theme (dark/light)
4. Update notification preferences

**Expected Behavior:**
- Settings sections display
- Theme toggle works
- Preferences update and persist

#### Search Widget

1. Ask Claude to search across content
2. Verify search results appear
3. Click a result to navigate

**Expected Behavior:**
- Search input accepts queries
- Results show across all content types
- Results link to original items

#### Notifications Widget

1. Ask Claude to show notifications
2. Verify notification list appears
3. Mark notifications as read
4. Clear all notifications

**Expected Behavior:**
- Notifications show with timestamps
- Mark read updates status
- Clear all removes notifications

## Validation Checklist

| Widget | Renders | Interactive | Data Updates | Notes |
|--------|---------|-------------|--------------|-------|
| Contacts | [ ] | [ ] | [ ] | |
| Messages | [ ] | [ ] | [ ] | |
| Kanban | [ ] | [ ] | [ ] | |
| Drive | [ ] | [ ] | [ ] | |
| Canvas | [ ] | [ ] | [ ] | |
| Settings | [ ] | [ ] | [ ] | |
| Search | [ ] | [ ] | [ ] | |
| Notifications | [ ] | [ ] | [ ] | |

## Known Issues

| Issue | Status | Workaround |
|-------|--------|------------|
| Drag-drop requires click-hold | Known | Use click-drag-release |
| Large files may not preview | Known | Download file instead |

## Screenshots

Capture screenshots during validation for documentation:

1. Each widget in initial state
2. Interactive elements in use
3. Error states if any

Save to: `docs/testing/screenshots/claude-desktop/`

## Troubleshooting

### Widget doesn't render

1. Check MCP server is running
2. Verify Claude Desktop config
3. Check Claude Desktop logs

### Widget renders but no data

1. Verify `--demo` flag is used
2. Check tool call in MCP server logs
3. Try refreshing conversation

### Widget interaction not working

1. Check browser console for errors
2. Verify postMessage communication
3. Check CSP headers
