# Changelog

All notable changes to the Communitas app will be documented in this file.

> Note: entries prior to the all-Rust pivot refer to the retired thin-client FFI integrations retained in the archive.

## [0.11.7] - 2026-04-05

### Fixed
- Aligned `communitas-x0x-client` parity tests with the Rust 1.88 formatter used in CI so release, publish, and security workflows stay green

---

## [0.11.6] - 2026-04-05

### Added
- API coverage guardrails and Swift parity tests for the `communitas-x0x-client` surface

### Fixed
- Restored green release validation by correcting `communitas-core` doctest imports to use the published library crate name
- Resolved cargo-deny/license compliance issues in CI

### Documentation
- Comprehensive documentation audit and ADR cleanup across the workspace

---

## [0.11.5] - 2026-04-05

### Documentation
- Comprehensive documentation audit: fixed version references, broken links, stale ADR statuses
- Updated ADR index with ADR-023 through ADR-028
- Marked networking ADRs (007, 013, 014, 015) as superseded per ADR-028

---

## [0.11.0] - 2026-03

### Changed
- **x0x Daemon Delegation (ADR-028)**: All P2P networking delegated to x0xd daemon
- Removed `communitas-mcp`, `communitas-headless`, `communitas-p2p-test` crates
- Added `communitas-x0x-client` for x0xd REST + WebSocket integration
- Networking ADRs (007, 013, 014, 015) superseded

---

## [0.10.0] - 2026-02

### Added
- ADR-023 through ADR-027: Security capability architecture (unlock grants, policy kernel, capability registry, principal hierarchy, canvas client strategy)
- `communitas-workspace-hack` crate for dependency unification

---

## [0.9.0] - 2026-02

### Changed
- Workspace restructuring and crate consolidation
- Version bumps for crates.io publishing alignment

---

## [0.8.2] - 2026-01-25

### M8 Production Readiness & UX Polish

This release completes Milestone 8 with 50 tasks across 6 phases, adding production-grade observability, improved offline experience, and comprehensive test coverage.

### Added
- **OpenTelemetry Integration** (Phase 8.6): OTLP metrics export with sync latency histograms, error counters, queue depth gauges, and CRDT conflict tracking
- **Grafana Dashboard Template**: Pre-built monitoring dashboard for production deployments
- **WebDriverIO Test Suite** (Phase 8.5): 13 test specs covering smoke tests, accessibility, and offline scenarios
- **Passkey/Biometric Authentication** (Phase 8.2): Platform-native biometric auth support
- **Offline UX Indicators** (Phase 8.3): Visual feedback for offline state with automatic reconnection

### Changed
- **Call Service Integration** (Phase 8.1): Improved WebRTC call management
- **Drive Resume API** (Phase 8.4): Resumable uploads that persist across app restarts

### Technical
- All phases reviewed with 11-agent code review (zero critical issues)
- 7 new metrics unit tests
- Feature-gated telemetry (`metrics` feature flag)
- Zero compilation errors/warnings

---

## [0.8.1] - 2026-01-24

### Patch Release

Minor release addressing CI/CD compliance and documentation quality.

### Fixed
- Removed `expect()` from `InputValidator::default()` in production code
- Added `#[allow(clippy::panic)]` for compile-time const assertion in Kanban service
- Fixed all documentation warnings (broken intra-doc links, code block formats)

### Changed
- Updated workspace dependency versions for crates.io publishing consistency

---

## [0.8.0] - 2026-01-24

### 🎉 Production Release

Communitas v0.8.0 marks the production release of our local-first, post-quantum ready collaboration platform. This release has been thoroughly tested through the beta program and is ready for general use.

### Highlights
- **Complete Feature Set**: Messaging, Drive, Canvas, Kanban, and Calls all fully functional
- **Post-Quantum Security**: ML-DSA/ML-KEM cryptography protects your data from future threats
- **Auto-Updates**: Seamless background updates via GitHub Releases
- **Accessibility**: Full keyboard navigation and screen reader support
- **Performance**: Optimized for smooth 60fps operation under 200MB memory

### Changes from Beta
- Removed beta language from onboarding and UI
- Production release workflow for stable version tags
- Updated documentation for general availability
- Full regression test suite

For detailed feature list, see the v1.0.0-beta.1 (now superseded) release notes below.

---

## [1.0.0-beta.1] - 2026-01-23

### 🎉 First Beta Release - M6 Beta-Ready (Apple Desktop)

This is the first public beta release of Communitas, featuring a complete local-first collaboration platform with end-to-end encryption.

### Added
- **Authentication & Security** (Phase 6.1)
  - Post-quantum cryptography (ML-DSA/ML-KEM)
  - Secure session management with auto-refresh
  - Rate limiting and input validation

- **Messaging & Contacts** (Phase 6.2)
  - End-to-end encrypted messaging
  - Thread-based conversations
  - Contact management with four-word addresses
  - Message reactions and editing

- **Drive & Attachments** (Phase 6.3)
  - Virtual disk system (Private/Public/Shared)
  - File upload/download with progress
  - Attachment support in messages
  - CRDT-based file synchronization

- **Calls & Presence** (Phase 6.4)
  - WebRTC-based voice/video calls
  - Online/offline presence indicators
  - Device discovery and selection

- **Canvas Integration** (Phase 6.5)
  - Collaborative whiteboard with CRDT sync
  - Drawing tools and shapes
  - Real-time multi-user editing

- **Kanban Project Management** (Phase 6.6)
  - CRDT-based boards and cards
  - Swimlane organization
  - Due dates and labels
  - Drag-and-drop interface

- **UX & Accessibility** (Phase 6.7)
  - Full keyboard navigation
  - Screen reader support (ARIA)
  - High contrast mode
  - Focus management

- **Onboarding Tour** (Phase 6.10)
  - 8-step interactive tour for new users
  - Skip/previous/next navigation
  - Keyboard shortcuts (Escape to skip)

- **Auto-Update System** (Phase 6.10)
  - Automatic update checking
  - Background download and install
  - Version display in Settings

### Performance (Phase 6.8)
- Lazy device enumeration
- LRU cache for board documents
- Debounced auto-login
- Optimized CRDT synchronization

### Testing (Phase 6.9)
- Comprehensive E2E test suite
- Property-based tests (proptest)
- Stress tests for concurrent operations
- 80%+ code coverage

### Platform Support
- macOS (Universal Binary - Intel & Apple Silicon)
- Signed and notarized by Apple
- Auto-updater enabled

## [0.2.8] - 2025-08-13

### Added
- **Four-Word Identity Packet Architecture**: Complete implementation of comprehensive identity system
  - Identity packets with public keys, storage addresses, and network forwards  
  - DHT validation rules preventing spam with dictionary word constraints
  - Signature-based ownership verification for all identity claims
  - Universal entity system supporting individuals, organizations, projects, groups, and channels
  - Foundation for decentralized markdown web with human-readable addressing
  
- **Enhanced Identity Commands**: 10 new legacy UI commands for identity management
  - `generate_four_word_identity` - Generate new four-word identities using four-word-networking crate
  - `validate_four_word_identity` - Validate format and dictionary membership
  - `check_identity_availability` - Check if identity is claimed
  - `claim_four_word_identity` - Claim and register new identity
  - `calculate_dht_id` - Generate BLAKE3 hash for DHT key
  - `get_identity_info` - Retrieve complete identity information with visual elements
  - Additional batch operations and statistics commands

- **Four-Word-Networking Integration**: Full integration with four-word-networking v2.3
  - Curated dictionary ensures controlled vocabulary
  - Word validation using `FourWordAdaptiveEncoder`
  - Consistent with existing four-word addressing in the ecosystem

## [0.2.7] - 2025-01-11

### Added
- **Network Auto-Connection**: App automatically connects to P2P network on startup
  - Sequential connection attempts to Digital Ocean bootstrap nodes
  - DHT initialization with fallback creation
  - Comprehensive error handling and logging
  
- **Projects Entity Support**: Full implementation of Projects within Organizations
  - Project creation dialog with priority levels (Low/Medium/High/Critical)
  - Deadline tracking and storage allocation (1-100GB)
  - Project member management with four-word addresses
  - Voice/video conferencing and file sharing per project
  
- **Enhanced Organization Dashboard**: Three-tab navigation system
  - Projects tab with project cards and management
  - Groups tab for team collaboration
  - Individuals tab for member directory
  - Unified search across all entity types
  
- **Backend Integration**: Improved service layer with legacy UI backend
  - `create_organization_dht` command for organization creation
  - `create_group_dht` command for group creation
  - `create_project_dht` command for project creation
  - Automatic fallback to mock data when backend unavailable

- **Context Switching Navigation**: Hierarchical navigation system
  - NavigationContext provider for centralized navigation state
  - Breadcrumb navigation with visual back button
  - Context-aware sidebar showing Personal/Organization hierarchy
  - Expandable organization structure with groups and projects
  - Real-time context indicators and navigation history

### Changed
- OrganizationService now attempts backend calls before using mock data
- Organization dashboard displays real-time member counts and storage usage
- Entity cards show communication options (voice/video/messaging/files)

### Fixed
- Network connection timeout handling
- DHT initialization when not available from P2P node
- Port conflicts during development (port 1420)

## [0.2.6] - Previous Release

- Initial Communitas implementation
- Basic organization structure
- P2P networking foundation
