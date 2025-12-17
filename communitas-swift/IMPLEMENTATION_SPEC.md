# Communitas Swift Implementation Specification

**Target Platforms**: iOS 17+, macOS 14+ (Sonoma)
**Language**: Swift 5.9+
**UI Framework**: SwiftUI
**Core Binding**: Rust via UniFFI

## 1. Executive Summary

This document outlines the technical specification for `communitas-swift`, a native iOS and macOS client for the Communitas collaboration platform. This client will achieve feature parity with the existing Tauri-based desktop application by leveraging the shared `communitas-core` Rust library.

The implementation prioritizes:
- **Code Sharing**: 90%+ of business logic resides in `communitas-core`.
- **Performance**: Native SwiftUI rendering with minimal bridging overhead.
- **Security**: Direct usage of ML-DSA-87/ML-KEM-768 from the Rust core.
- **Resilience**: Full support for local-first, offline-capable P2P networking.

## 2. Architecture

### 2.1 High-Level Stack

```mermaid
graph TD
    UI[SwiftUI App\niOS / macOS] --> ViewModel[Swift ViewModels]
    ViewModel --> Repository[Swift Repositories]
    Repository --> FFI[Swift/Rust FFI Layer\n(UniFFI Generated)]
    FFI --> RustLib[communitas-core\n(Rust)]
    RustLib --> Network[QUIC / UDP / TCP]
    RustLib --> Disk[Encrypted Storage]
```

### 2.2 The Bridge: UniFFI

We will use **UniFFI** to generate the bindings. This requires a new crate, `communitas-bindings`, or a feature flag in `communitas-core` that exposes a UniFFI-compatible interface.

**Why UniFFI?**
- Automatic generation of Swift code.
- Type safety across the boundary.
- Async/Await support (mapping Rust `async` to Swift `async`).

## 3. Core Interface Definition

The Swift client requires access to the following core services. We will define a `CommunitasClient` object in Rust that holds the `CoreContext` and exposes these capabilities.

### 3.1 Initialization & Identity

**Rust Source**: `core_context.rs`, `identity.rs`

**Swift Interface**:
```swift
actor CommunitasCore {
    // Initialize the core, load keys, setup storage
    static func initialize(
        fourWords: String, 
        displayName: String, 
        deviceName: String, 
        storagePath: String
    ) async throws -> CommunitasCore

    // Get current identity
    func getIdentity() async -> UserProfile
    
    // Sign data (for verification)
    func sign(data: Data) async throws -> Data
}
```

### 3.2 Networking & Presence

**Rust Source**: `gossip.rs`, `presence_service.rs`

**Swift Interface**:
```swift
extension CommunitasCore {
    // Start the P2P listener
    func startNetworking(port: UInt16) async throws
    
    // Advertise presence to the mesh
    func advertisePresence() async throws
    
    // Connect to specific peer or bootstrap node
    func connectToPeer(address: String) async throws
    
    // Stream of presence updates (Combine Publisher / AsyncStream)
    func presenceUpdates() -> AsyncStream<PresenceEvent>
}
```

### 3.3 Entities (Groups, Channels, Orgs)

**Rust Source**: `entity_service.rs`

**Swift Interface**:
```swift
extension CommunitasCore {
    // List all entities
    func listEntities() async throws -> [Entity]
    
    // Get specific entity details
    func getEntity(id: String) async throws -> Entity
    
    // Create new entity
    func createEntity(name: String, type: EntityType) async throws -> Entity
    
    // Manage members
    func addMember(entityId: String, memberId: String) async throws
}
```

### 3.4 Messaging

**Rust Source**: `message_service.rs`

**Swift Interface**:
```swift
extension CommunitasCore {
    // Send message
    func sendMessage(entityId: String, content: String, replyTo: String?) async throws -> Message
    
    // Get history
    func getMessages(entityId: String, limit: Int, before: Date?) async throws -> [Message]
    
    // Real-time message stream
    func messageStream(entityId: String) -> AsyncStream<Message>
}
```

## 4. Implementation Plan

### Phase 1: Scaffolding & Bindings ✅ COMPLETE
- [x] Create `communitas-bindings` Rust crate.
- [x] Define UniFFI interface with procedural macros.
- [x] Set up Swift Package Manager project (`CommunitasApp`).
- [x] Configure build script to compile Rust to `libcommunitas_bindings.a` and generate Swift glue.
- [x] Resolve async FFI issues (Tokio runtime with `block_on` wrappers).
- [x] **Milestone**: Swift app successfully calls Rust functions.

### Phase 2: Identity & Lifecycle ✅ COMPLETE
- [x] Implement `CoreContext` initialization in bindings.
- [x] Map `UserProfile` struct to Swift.
- [x] Create "Welcome / Login" screens in SwiftUI (AuthenticationView.swift).
- [x] Input "Four Words" or Generate New Identity.
- [x] Persist identity in macOS Keychain via Swift Security framework.
- [x] Encrypted vault storage with password protection.
- [x] Passkey/biometric authentication support (Touch ID).
- [x] **Milestone**: App launches, generates/loads ID, and shows dashboard with identity.

### Phase 3: Networking & Presence ✅ COMPLETE
- [x] Expose `start_networking` and `gossip` controls.
- [x] Integrate `saorsa-gossip` events.
- [x] Build "Network Status" UI (Peers count, Connection status).
- [x] P2P connectivity via QUIC.
- [x] **Milestone**: P2P networking functional.

### Phase 4: Entities & Data - IN PROGRESS
- [ ] Bind `EntityService` (partial).
- [ ] Implement "Sidebar" UI: List of Orgs, Projects, Channels.
- [ ] Implement "Create Channel" flows.
- [ ] **Milestone**: User can create a channel on iPhone, see it appear on Mac.

### Phase 5: Messaging - PENDING
- [ ] Bind `MessageService`.
- [ ] Implement Chat UI (Bubble view, Input bar).
- [ ] Handle attachments (basic file picking).
- [ ] **Milestone**: Full chat capability between iOS and Desktop.

### Phase 6: Virtual Disks & Storage - PENDING
- [ ] Implement virtual disk UI (Private/Public/Shared).
- [ ] File browsing and management.
- [ ] CRDT synchronization display.
- [ ] **Milestone**: Users can browse and manage files in virtual disks.

### Phase 7: Polish & iOS Support - PENDING
- [ ] iOS-specific optimizations.
- [ ] Background sync with BackgroundTasks framework.
- [ ] Push notification integration.
- [ ] App Store preparation.
- [ ] **Milestone**: iOS app ready for TestFlight.

## 5. Platform Specifics

### 5.1 Storage
- **iOS**: Use `FileManager.default.urls(for: .documentDirectory, ...)` to pass a valid writable path to `communitas-core`.
- **Background**: iOS strictly limits background execution. We must use `BackgroundTasks` framework to periodically sync if the app is backgrounded, or rely on "Background Fetch". *Note: P2P on iOS background is challenging; initially, sync occurs only when foregrounded.*

### 5.2 Keychain
- The Rust `keyring` crate may not work out-of-the-box on iOS due to entitlements.
- **Strategy**: We might need to expose `load_keys` / `save_keys` in the Rust trait and implement the storage provider in Swift, passing it back to Rust, OR configure `keyring` to use the iOS keychain correctly with a specific access group.

### 5.3 Concurrency
- Rust `tokio` runtime must be managed carefully. It should live on a dedicated thread spawned by the Rust side, not blocking the main Swift thread. UniFFI handles this well by making async functions return Swift `Task`s.

## 6. Testing Strategy

- **Unit Tests (Rust)**: Continue running `cargo test` in `communitas-core`.
- **Integration Tests (Swift)**: XCTest cases that instantiate the `CommunitasCore` mock or real instance and assert state changes.
- **E2E**: Manual testing using the "Bridge" or Desktop app as a peer.

## 7. Next Steps

1.  Initialize the `communitas-bindings` crate inside the repo.
2.  Configure the Cargo workspace.
3.  Generate the Xcode project.
