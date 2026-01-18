# MCP + Dioxus Integration Architecture (All-Rust)

**Version**: 3.0  
**Status**: Active  
**Last Updated**: 2026-01-18

## Overview

Communitas now follows an **all-Rust GUI architecture**:

- **Dioxus + Tauri 2** renders every desktop surface while calling shared Rust `UiServices` directly (no Dart/FFI layer).
- **MCP** exposes the same core features for **automation, AI agents, and other local apps**.
- MCP is **not** the primary GUI backend; it is a parallel API surface that stays bit-for-bit aligned with Dioxus because both consume the same services.

The previous multi-language thin-client and its FRB plumbing were fully archived on January 18, 2026 (ADR-020). Only Rust remains in the active UI stack.

---

## Architecture Principles

1. **Core is the source of truth** — All business logic lives in Rust (`communitas-core`).
2. **GUI is thin** — Dioxus renders UI and calls the shared Rust `UiServices`; no intermediate languages or IPC layers.
3. **MCP is for integrations** — CLI tools, AI agents, and local automation use MCP.
4. **Consistent domain model** — Commands/Queries remain the same across adapters, guaranteeing automation = GUI parity.

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

## Dioxus Scope (Current)

The Dioxus app consumes `communitas-ui-service` to cover:
- Vault lifecycle (create/import/export/recover)
- Entities, membership, and messaging (threads, edits, reactions, direct messages)
- Contacts management + favorites
- Presence queries (peer presence + connection words + status)
- Kanban, files, drive, and website publishing views
- Demo mode (temporary local storage, clear demo data)

---

## Demo Mode Policy

- **Web build** remains **demo-only** (no MCP exposure in browser).
- Demo mode uses temporary local storage and supports quick login for testing.
- Desktop builds gate demo telemetry to avoid contaminating production metrics.

---

## Next Milestones

1. Harden WebRTC flows with multi-node testing
2. Decide on DHT support vs. explicit de-scoping
3. Add notifications/unread counts in core if required
4. Expand Dioxus UI coverage for remaining features (calls, canvas, mobile shells)
