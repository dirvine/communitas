# MCP + Flutter Integration Architecture (FFI-first)

**Version**: 2.0  
**Status**: Active  
**Last Updated**: 2026-01-16

## Overview

Communitas now follows an **FFI-first GUI architecture**:

- **Flutter GUI** talks directly to `communitas-core` via **Flutter FFI** (flutter_rust_bridge).
- **MCP** exposes the same core features for **automation, AI agents, and other local apps**.
- MCP is **not** the primary GUI backend; it is a parallel API surface.

This keeps the GUI thin while avoiding extra IPC hops for mobile/desktop UI.

---

## Architecture Principles

1. **Core is the source of truth** — All business logic lives in Rust (`communitas-core`).
2. **GUI is thin** — Flutter renders UI and calls FFI; minimal app logic in Dart.
3. **MCP is for integrations** — CLI tools, AI agents, and local automation use MCP.
4. **Consistent domain model** — Commands/Queries remain the same across adapters.

---

## MCP Coverage (Current)

### ✅ Implemented
- Identity & vaults (create/import/export/recover)
- Entities & membership
- Messaging (threads + reactions)
- Kanban (full CRUD)
- Files + metadata
- Contacts + favorites + search
- Website publishing
- Networking (gossip start/stop/connect, presence queries)

### ⚠️ Experimental
- WebRTC call tools (start/join/end + screen share toggle)

### ❌ Not Implemented (Removed from MCP scope)
- DHT storage + metrics
- Polls / stories / location sharing
- Presentation/slide synchronization
- Notifications + unread counts
- Per-peer disconnect

---

## Flutter FFI Scope (Current)

The Flutter app uses FFI to cover:
- Vault lifecycle (create/import/export/recover)
- Entities, membership, and messaging (threads, edits, reactions, direct messages)
- Contacts management
- Presence queries (peer presence + connection words + status)
- Demo mode (temporary local storage, clear demo data)

---

## Demo Mode Policy

- **Web build** is **demo-only** (no FFI in browser).
- Demo mode uses temporary local storage and supports quick login for testing.

---

## Next Milestones (If Needed)

1. Harden WebRTC flows with multi-node testing
2. Decide on DHT support vs. explicit de-scoping
3. Add notifications/unread counts in core if required
4. Expand Flutter UI coverage for remaining features
