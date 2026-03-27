# communitas-dioxus

Cross-platform Dioxus + Tauri desktop application for Communitas.

## Requirements

- Rust 1.85+
- `dx` CLI 0.7.3 (run `scripts/install_dx.sh` from the repo root)
- Node.js 18+ (Tailwind/Vite asset bundling)
- x0xd daemon installed and running (the onboarding gate handles this on first launch)

## Development

```bash
# From the repo root, install the pinned dx CLI
scripts/install_dx.sh

# Run with hot reload
cd communitas-dioxus
dx serve --platform desktop --hotpatch
```

## Building

```bash
# Desktop bundle (macOS/Windows/Linux)
dx bundle --platform desktop

# Experimental mobile (stability varies)
dx bundle --platform android
dx bundle --platform ios
```

## Quality Checks

```bash
dx check --platform desktop
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used
cargo nextest run -p communitas-dioxus --lib
```

## Features

- Onboarding gate: auto-installs and starts x0xd on first launch
- Channel chat with message editing, deletion, and pinning
- Threading (replies in a side panel)
- Inline quotes/replies
- Emoji reactions with quick-reaction bar and full categorized emoji picker (with search)
- Markdown rendering in messages
- @mention autocomplete
- Typing indicators and presence badges
- In-channel message search
- Spaces/groups/DMs sidebar navigation
- Kanban boards, Drive, Canvas, Wiki, and Feed views
- Virtual list for large message history
