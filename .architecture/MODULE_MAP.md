# Communitas Module Map

## Overview

This document provides a factual inventory of the Communitas codebase, explicitly marking what exists today versus what is aspirational or required for future expansion.

**Generated**: 2026-04-05
**Version**: 0.11.5

---

## Module Status Legend

| Status | Meaning |
|--------|---------|
| **STABLE/SHIPPING** | Production-ready, actively used in Dioxus app |
| **EMERGING** | Functional but under active development |
| **EXPERIMENTAL** | Proof-of-concept, API unstable |
| **MISSING/REQUIRED** | Does not exist but needed for future architecture |

---

## 1. Workspace Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| `communitas-core` | Cross-platform business logic, crypto, storage | **STABLE** |
| `communitas-ui-api` | Serializable view models (shared types) | **STABLE** |
| `communitas-ui-service` | Shared service layer for all UI surfaces | **STABLE** |
| `communitas-dioxus` | Desktop application (Dioxus + Tauri) | **STABLE** |
| `communitas-kanban` | CRDT-based project boards | **STABLE** |
| `communitas-x0x-client` | x0xd daemon discovery, HTTP client, WebSocket | **STABLE** |
| `communitas-bench` | Performance benchmarks | **EMERGING** |
| `communitas-workspace-hack` | Workspace dependency unification | **STABLE** |
| ~~`communitas-mcp`~~ | ~~AI agent interface~~ | **REMOVED** (networking delegated to x0xd) |
| ~~`communitas-headless`~~ | ~~Bootstrap/seed node daemon~~ | **REMOVED** (replaced by x0xd) |
| ~~`communitas-p2p-test`~~ | ~~P2P testing utilities~~ | **REMOVED** (networking delegated to x0xd) |

---

## 2. communitas-core Modules

### 2.1 Core Infrastructure

| Module | Purpose | Status |
|--------|---------|--------|
| `app.rs` | CommunitasApp entry point | **STABLE** |
| `command.rs` | Command/Event/Query architecture | **STABLE** |
| `core_context.rs` | Service orchestrator | **STABLE** |
| `ui_core.rs` | UI-facing API surface | **STABLE** |
| `error.rs` | Error types | **STABLE** |
| `types.rs` | Core type definitions | **STABLE** |
| `services.rs` | CoreServices aggregator | **STABLE** |

### 2.2 Identity & Authentication

| Module | Purpose | Status |
|--------|---------|--------|
| `identity.rs` | Four-word encoding, identity helpers | **STABLE** |
| `auth_service.rs` | SessionInfo, AuthService traits | **STABLE** |
| `keystore.rs` | Platform keyring integration | **STABLE** |
| `recovery/keys.rs` | BIP39 deterministic key derivation | **STABLE** |
| `recovery/mod.rs` | Vault recovery operations | **STABLE** |

### 2.3 Entity Management

| Module | Purpose | Status |
|--------|---------|--------|
| `entity_service.rs` | Organization/Group/Channel/Project CRUD | **STABLE** |
| `message_service.rs` | Message operations | **STABLE** |
| `presence_service.rs` | Online/offline tracking | **STABLE** |
| `invite_service.rs` | Cross-org collaboration invites | **STABLE** |
| `invite.rs` | Invite domain model | **STABLE** |
| `linking_service.rs` | Entity-to-network linking | **STABLE** |

### 2.4 Storage & Persistence

| Module | Purpose | Status |
|--------|---------|--------|
| `disk_service.rs` | Virtual disk management per entity | **STABLE** |
| `doc_replicator.rs` | Document synchronization | **STABLE** |
| `storage/` | File system persistence | **STABLE** |
| `encrypted_storage/` | Vault management | **STABLE** |

### 2.5 CRDT Infrastructure

| Module | Purpose | Status |
|--------|---------|--------|
| `crdt/mod.rs` | CRDT re-exports, usage examples | **STABLE** |
| `crdt/documents.rs` | CrdtDocument types | **STABLE** |
| `crdt/operations.rs` | LWWRegister, ORSet, Counter | **STABLE** |
| `crdt/conflict_resolution.rs` | ConflictResolver, AutoResolution | **STABLE** |
| `crdt/offline_queue.rs` | OfflineQueue for offline-first | **STABLE** |
| `crdt_manager/` | CRDT lifecycle management | **STABLE** |
| `legacy_crdt.rs` | Legacy vector clock (deprecated) | **DEPRECATED** |

### 2.6 Gossip Overlay Network (REMOVED — ADR-028)

> **Note**: The `gossip/` module tree was removed when networking was delegated to the x0x daemon (ADR-028, 2026-03). All P2P networking now goes through `communitas-x0x-client`.

| Module | Purpose | Status |
|--------|---------|--------|
| ~~`gossip/context.rs`~~ | ~~GossipContext orchestrator~~ | **REMOVED** |
| ~~`gossip/boot.rs`~~ | ~~GossipBootSequence~~ | **REMOVED** |
| ~~`gossip/presence.rs`~~ | ~~PresenceInfo, PresenceWrapper~~ | **REMOVED** |
| ~~`gossip/discovery.rs`~~ | ~~FoafDiscovery, IntroducerConfig~~ | **REMOVED** |
| ~~`gossip/sites.rs`~~ | ~~SitePublisher, SiteFetcher~~ | **REMOVED** |
| ~~`gossip/contact_storage.rs`~~ | ~~ContactStore, ContactRecord~~ | **REMOVED** |
| ~~`gossip/peer_cache.rs`~~ | ~~PeerCache~~ | **REMOVED** |
| ~~`gossip/backup.rs`~~ | ~~BackupManager~~ | **REMOVED** |
| ~~`gossip/name_record.rs`~~ | ~~NameRegistry~~ | **REMOVED** |

### 2.7 Security Infrastructure

| Module | Purpose | Status |
|--------|---------|--------|
| `security/rate_limiter.rs` | RateLimiter | **STABLE** |
| `security/audit_log.rs` | Security audit logging | **STABLE** |
| `security/auth_middleware.rs` | Authentication middleware | **STABLE** |
| `security/input_validation.rs` | Input sanitization | **STABLE** |
| `security/secure_storage.rs` | Keychain integration | **STABLE** |
| `security/device.rs` | Device identification | **EMERGING** |
| `validation.rs` | ValidationService | **STABLE** |

### 2.8 WebRTC (REMOVED — ADR-028)

> **Note**: The `webrtc/` module tree was removed when networking was delegated to the x0x daemon (ADR-028, 2026-03).

| Module | Purpose | Status |
|--------|---------|--------|
| ~~`webrtc/service.rs`~~ | ~~CommunitasWebRtcService~~ | **REMOVED** |
| ~~`webrtc/gossip_signaling.rs`~~ | ~~GossipSignalingTransport~~ | **REMOVED** |
| ~~`webrtc/identity.rs`~~ | ~~CommunitasIdentity wrapper~~ | **REMOVED** |
| ~~`webrtc/recorder.rs`~~ | ~~Recording sessions~~ | **REMOVED** |

### 2.9 Resilience & Observability

| Module | Purpose | Status |
|--------|---------|--------|
| `connectivity_watchdog.rs` | Internet collapse detection | **STABLE** |
| `resource_limits.rs` | Rate limiting, connection caps | **STABLE** |
| `retry_utils.rs` | Exponential backoff | **STABLE** |
| `metrics.rs` | Observability hooks | **EMERGING** |
| `telemetry.rs` | Tracing integration | **STABLE** |

### 2.10 Permissions

| Module | Purpose | Status |
|--------|---------|--------|
| `permissions/` | Role-based access control | **EMERGING** |

---

## 3. communitas-ui-service Modules

| Module | Purpose | Status |
|--------|---------|--------|
| `lib.rs` | UiServices aggregator | **STABLE** |
| `auth.rs` | AuthController | **STABLE** |
| `messaging.rs` | MessagingService | **STABLE** |
| `directory.rs` | DirectoryService | **STABLE** |
| `presence.rs` | PresenceService | **STABLE** |
| `navigation.rs` | NavigationStore | **STABLE** |
| `kanban.rs` | KanbanService | **STABLE** |
| `canvas.rs` | CanvasService | **STABLE** |
| `drive.rs` | DriveService | **STABLE** |
| `call.rs` | CallService | **STABLE** |
| `audit.rs` | AuditService | **STABLE** |
| `storage.rs` | UiStorage (paths) | **STABLE** |

---

## 4. communitas-mcp Modules (REMOVED)

> **Note**: The `communitas-mcp` crate was removed when networking was delegated to x0xd (ADR-028). MCP functionality is now provided by x0xd's built-in MCP server.

---

## 5. communitas-dioxus Modules

| Module | Purpose | Status |
|--------|---------|--------|
| `main.rs` | App entry, lifecycle | **STABLE** |
| `app.rs` | Route definitions | **STABLE** |
| `pages.rs` | Page components | **STABLE** |
| `components/` | 34+ UI components | **STABLE** |
| `hooks/` | Custom Dioxus hooks | **STABLE** |
| `animations/` | Motion effects | **EMERGING** |
| `platform/` | WebView, notifications, devices | **STABLE** |
| `design_tokens.rs` | Theme system | **STABLE** |

---

## 6. MISSING BUT REQUIRED (Future Architecture)

| Component | Purpose | Required For |
|-----------|---------|--------------|
| **Policy Kernel** | Deterministic capability gate | Canvas, Agents |
| **Capability Registry** | Formal capability definitions | Canvas, Agents |
| **Audit Receipt System** | Tamper-proof execution receipts | Agent accountability |
| **Proposal Queue** | Agent action proposals awaiting approval | Safe agent collaboration |
| **Quarantine Layer** | Isolated execution environment | Untrusted agent code |
| **Canvas Client Adapter** | Canvas as UiServices consumer | Canvas UI |
| **Permission Delegation** | Fine-grained permission grants | Agent scopes |
| **Operation Schema Registry** | JSON Schema for all operations | Cross-client parity |

---

## 7. Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLIENTS                                  │
├─────────────────────────────────────────────────────────────────┤
│  Dioxus Desktop  │  MCP (AI Agents)  │  Headless  │  [Canvas]   │
└────────┬─────────┴────────┬──────────┴─────┬──────┴──────┬──────┘
         │                  │                │             │
         ▼                  ▼                ▼             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    communitas-ui-service                         │
│  ┌─────────┐ ┌─────────┐ ┌────────┐ ┌──────┐ ┌──────┐ ┌──────┐ │
│  │  Auth   │ │Messaging│ │ Kanban │ │Canvas│ │ Drive│ │ Call │ │
│  └────┬────┘ └────┬────┘ └───┬────┘ └──┬───┘ └──┬───┘ └──┬───┘ │
└───────┼──────────┼──────────┼─────────┼────────┼────────┼──────┘
        │          │          │         │        │        │
        ▼          ▼          ▼         ▼        ▼        ▼
┌─────────────────────────────────────────────────────────────────┐
│                      communitas-core                             │
│  ┌──────────────┐ ┌────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │ CoreContext  │ │   CRDT     │ │   Gossip    │ │  Security │ │
│  │ EntityService│ │ Documents  │ │   Overlay   │ │  Keystore │ │
│  │MessageService│ │ Operations │ │  Presence   │ │ Validation│ │
│  └──────────────┘ └────────────┘ └─────────────┘ └───────────┘ │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    EXTERNAL DEPENDENCIES                         │
│  saorsa-gossip-* │ saorsa-pqc │ yrs │ ant-quic │ tokio         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. Key Architectural Invariants

1. **All clients use UiServices** - Behavioral parity guaranteed
2. **CRDT for collaborative state** - Offline-first, conflict-free
3. **Gossip for networking** - No central servers
4. **PQC for identity** - ML-DSA-87 (NIST Level 5)
5. **Platform keyring for secrets** - No plaintext key storage

---

## 9. Integration Points

| Integration | Current | Future (Canvas/Agents) |
|-------------|---------|------------------------|
| UI → Services | UiServices singleton | Same (no change) |
| Services → Core | CommunitasApp | Same (no change) |
| Auth | AuthController | Policy Kernel gate |
| Operations | Direct execution | Capability + receipt |
| Audit | AuditService (optional) | Mandatory receipts |
