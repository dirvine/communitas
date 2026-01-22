# Communitas Roadmap

_Last updated: 2026-01-22_

## Current Milestone: M6 Beta-Ready (Apple Desktop)

**Scope**: Bring Communitas to internal beta quality for macOS desktop.

**Key decisions** (from GSD interview):
- Foundation-first priority order
- Password-only auth (defer passkeys to post-beta)
- Full calls feature set (voice, video, screen share, recording)
- Comprehensive testing coverage
- Opt-in telemetry with local preview
- Internal-only release first

### Phases

| Phase | Description | Status |
|-------|-------------|--------|
| **6.1** | Auth Hardening | Complete |
| **6.2** | Messaging & Contacts | Complete |
| **6.3** | Drive & Attachments | Complete |
| **6.4** | Calls & Presence | Complete |
| **6.5** | Canvas Integration | Complete |
| **6.6** | Kanban Polish | In Progress |
| **6.7** | UX & Accessibility | Pending |
| **6.8** | Testing & Tooling | Pending |
| **6.9** | Apple Beta Packaging | Pending |

---

## Phase 6.1: Auth Hardening

**Goal**: Solidify authentication for beta users.

**Tasks** (to be detailed in PLAN-36):
- Multi-identity UX (quick switch between identities)
- Biometric unlock flow (macOS TouchID integration)
- Recovery flow documentation and testing
- Audit logging for device changes
- Session timeout and refresh handling
- Migration path documentation for existing vaults

---

## Phase 6.2: Messaging & Contacts

**Goal**: Complete messaging experience with DMs, presence, and parity.

**Tasks**:
- DM threads implementation
- Contact presence indicators
- Typing indicators
- Unread count persistence
- Offline send queue with CRDT reconciliation
- Parity harness: golden fixtures for CI
- Parity harness: live data export for QA
- Search functionality
- Pinned chats
- MCP automation hooks for Slack-like workflows

---

## Phase 6.3: Drive & Attachments

**Goal**: Production-quality file handling.

**Tasks**:
- Streaming transfers via core storage
- Checksum verification
- Resume support for interrupted transfers
- macOS file picker integration via Tauri
- Share links generation
- Permission dialogs
- Image/PDF previews
- Offline staging area with sync-on-connect

---

## Phase 6.4: Calls & Presence

**Goal**: Full-featured audio/video calls.

**Tasks**:
- saorsa-webrtc-* integration for audio/video
- Device enumeration via platform layer
- Presence indicators (active call, muted, screen sharing)
- Screen share implementation
- Call recording toggles
- Quality metrics (latency, jitter, packet loss)
- Group call support
- Call history and missed call notifications

---

## Phase 6.5: Canvas Integration

**Goal**: Real-time collaborative editing.

**Tasks**:
- Wire CanvasService to saorsa-canvas CRDT scene
- Bidirectional sync implementation
- Shared cursors with user identification
- History scrubbing (undo/redo timeline)
- Offline queue replay via core commands
- MCP tools for canvas automation
- Canvas element selection and manipulation

---

## Phase 6.6: Kanban Polish

**Goal**: Competitive task management.

**Tasks**:
- Subscribe to CRDT events for live updates
- Swimlanes implementation
- Priority traits
- Due date handling
- Messaging integration (link discussions to cards)
- Keyboard-accessible drag/drop
- Conflict banners for concurrent edits
- Analytics dashboard (velocity, burndown)

---

## Phase 6.7: UX & Accessibility

**Goal**: Professional, accessible user experience.

**Tasks**:
- Keyboard navigation audit (all Dioxus components)
- Screen reader labels (ARIA)
- Motion preferences (reduce motion)
- Contrast compliance (WCAG AA)
- Skeleton loaders for async content
- Design token system
- Responsive layouts
- Micro-interactions for offline state transitions
- Loading states and error boundaries

---

## Phase 6.8: Testing & Tooling

**Goal**: Comprehensive test coverage for beta confidence.

**Tasks**:
- WebDriver coverage: login flows
- WebDriver coverage: messaging workflows
- WebDriver coverage: drive operations
- WebDriver coverage: calls
- WebDriver coverage: canvas
- WebDriver coverage: kanban
- Parity scripts for Drive
- Parity scripts for Kanban
- Parity scripts for Canvas
- Parity scripts for Calls
- CI artifacts with real data (not placeholders)
- Edge case test suite
- Offline scenario tests
- Concurrent operation tests
- Stress tests
- Documentation updates

---

## Phase 6.9: Apple Beta Packaging

**Goal**: Ship internal beta.

**Tasks**:
- Crash reporting integration
- Auto-update mechanism
- Onboarding tour implementation
- Opt-in telemetry with local preview
- Sync latency tracking
- Offline duration metrics
- CRDT conflict frequency tracking
- Internal distribution workflow
- Beta feedback collection mechanism

---

## Completed Milestones

| Milestone | Description | Status |
|-----------|-------------|--------|
| **M0** | Discovery & Inventory | Complete |
| **M1** | Bootstrap Node Enhancement | Complete |
| **M2** | Messaging & Entities | Complete |
| **M3** | Advanced Surfaces (UI) | Complete |
| **M4** | Polish & Performance (UI) | Complete |
| **M3.1** | Remediation - Wire Services | Complete (47 tasks) |
| **M5** | Stabilization | Complete |

---

## Reference

- Main plan: `docs/architecture/dioxus_desktop_prototype_plan.md`
- M5 archive: `docs/gsd/archive/m5/`
- M3.1 archive: `docs/gsd/archive/m3.1/`
- Legacy archive: `.planning/archive/`
