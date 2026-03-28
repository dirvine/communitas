# communitas-apple

Native macOS application for Communitas, built with SwiftUI.

## Requirements

- macOS 14+
- Xcode 15+ (or Swift 5.10 toolchain)
- x0xd daemon installed and running

## Structure

```
communitas-apple/
  Package.swift
  Sources/
    Communitas/       # Main app target (executable)
      Views/          # SwiftUI views
      Models/         # AppState, ChannelManager, NavigationItem
      Services/       # NotificationService
    X0xClient/        # Reusable daemon client library
      X0xClient.swift
      X0xWebSocket.swift
      DaemonManager.swift
      Models/
```

## x0x Daemon Integration

The app discovers the running x0xd daemon from:
- `~/Library/Application Support/x0x/api.port` — `host:port` the API listens on
- `~/Library/Application Support/x0x/api-token` — 64-character hex Bearer token

`X0xClient.discover()` reads these files at startup. If either is missing, the onboarding view prompts the user to install or start x0xd.

## Development

```bash
# Open in Xcode
open Package.swift

# Build from command line
swift build

# Run tests
swift test
```

## Features

- Onboarding view: installs x0xd via `curl -sfL https://x0x.md | sh` if not present
- Channel chat with message editing, deletion, pinning, threading, and inline quotes
- Emoji picker (categorized) and quick-reaction bar
- Markdown message rendering
- @mention autocomplete
- Typing indicators
- Message search
- Spaces/groups/DMs sidebar navigation
- Board, Files, Feed, Wiki, Web Publish, Network, and Swarm views
- Daemon status and settings panels
