# Communitas MCP Production Readiness Report

**Date**: 2026-01-16  
**Version**: 2.1  
**Status**: PARTIAL (Not production ready)

---

## Executive Summary

The `communitas-mcp` server provides solid coverage of **core collaboration workflows** (identity, entities, messaging, Kanban, files, contacts, websites, and networking commands). However, several advanced features previously claimed as complete are **not implemented** in the MCP layer or are **still experimental** in the core stack. MCP is currently best suited for **automation, testing, and local integrations**, not as a production-grade primary backend.

### Key Facts (Current State)

| Area | Status | Notes |
|------|--------|-------|
| Identity & Vaults | ✅ Implemented | Create/import/export/recover, delegate tokens |
| Entities & Members | ✅ Implemented | Full CRUD and membership |
| Messaging | ✅ Implemented | Send/edit/delete, threads, reactions |
| Kanban | ✅ Implemented | Full CRUD coverage |
| File Ops | ✅ Implemented | Read/write/delete/list + metadata |
| Contacts | ✅ Implemented | CRUD + favorites + search |
| Website | ✅ Implemented | Publish/update/remove |
| Networking (Gossip) | ✅ Implemented | Start/stop/connect + presence queries |
| Presence (MCP status) | ⚠️ Limited | In-memory status only (not network-synced) |
| WebRTC Calls | ⚠️ Experimental | Wired to core; depends on gossip + webrtc stack |
| DHT Storage & Metrics | ❌ Removed | Not implemented; tooling removed |
| Social (Polls/Stories/Location) | ❌ Removed | Not implemented |
| Presentations/Slides | ❌ Removed | Not implemented |
| Notifications/Unread Counts | ❌ Not implemented | No core support |
| Per-Peer Disconnect | ❌ Not implemented | Core lacks API |

---

## 1. What MCP Currently Delivers

### 1.1 Core Collaboration (Stable)
- ✅ Identity + vault management
- ✅ Entities, membership, and invites
- ✅ Messaging (including threads & reactions)
- ✅ Kanban boards/cards/tags/steps/comments
- ✅ File read/write/delete/list + metadata
- ✅ Contacts + favorites + search
- ✅ Website publishing

### 1.2 Networking (Functional, still maturing)
- ✅ Start/stop networking
- ✅ Connect by connection words (IP:port encoding)
- ✅ External address discovery
- ✅ Presence announcements and peer presence queries

### 1.3 WebRTC Calls (Experimental)
- ✅ Start/Join/End call tools wired to core WebRTC service
- ⚠️ Requires networking to be active
- ⚠️ Multi-device call flows not yet validated in MCP E2E

---

## 2. Gaps & Risks Blocking Production Readiness

1. **No DHT or distributed storage** in MCP tooling
2. **Social features (polls/stories/location)** are not implemented
3. **Presentation/screen sharing workflow** only supports core screen share toggle (no slides)
4. **Unread count + notifications** not available (core lacks APIs)
5. **Presence is local-only** for MCP status tools
6. **WebRTC flows** need real multi-node verification

---

## 3. Recommendation

MCP is ready for **developer automation and integration testing**, but **not yet production-ready** for full collaboration parity or advanced features. The next readiness milestone requires:

- Multi-device validation of WebRTC + presence
- Core APIs for unread counts + notifications
- Decisions on DHT support vs. deliberate de-scoping
- Updated API documentation reflecting the scoped tool set

---

## 4. Conclusion

**Current MCP status: reliable for core workflows, not complete for advanced collaboration.**

Use MCP now for automation, scripted testing, and AI agent integrations. Treat advanced collaboration features as **out of scope** until core support is implemented.
