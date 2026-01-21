# Project State: Communitas

## Current Session
- **Date**: 2026-01-21
- **Milestone**: M6 Beta-Ready (Apple Desktop)
- **Phase**: Not started
- **Status**: INITIALIZED

## Milestone Scope

Bring Communitas to internal beta quality for macOS desktop. Nine major areas:

1. **Auth & Passkeys** - Password-only for beta (defer passkeys)
2. **Messaging & Contacts** - DMs, presence, typing, parity harness
3. **Drive & Attachments** - Streaming, share links, previews
4. **Calls & Presence** - Full feature set (voice, video, screen share, recording)
5. **Canvas & Collaborative Editing** - CRDT sync, cursors, history
6. **Kanban & Tasking** - Live updates, swimlanes, analytics
7. **UX & Accessibility** - Keyboard nav, screen readers, design tokens
8. **Testing & Tooling** - WebDriver, parity scripts (golden + live), comprehensive coverage
9. **Apple Beta Packaging** - Crash reporting, auto-update, opt-in telemetry

## Interview Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Priority order | Foundation first | Auth → Messaging → Drive → Calls → Canvas → Kanban → UX → Testing → Packaging |
| Passkey approach | Password-only for beta | Ship faster with existing auth; defer WebAuthn to post-beta |
| Calls scope | Full feature set | Voice, video, screen share, recording toggles, quality metrics |
| Autonomy level | Plan approval per phase | User approves phase plan, then autonomous execution |
| Parity harness | Both approaches | Golden fixtures for CI + live export for manual QA |
| Testing scope | Comprehensive coverage | Edge cases, offline scenarios, concurrent ops, stress tests |
| Telemetry | Opt-in with local preview | Users see exactly what would be sent before opting in |
| Release strategy | Internal only first | Stable internally before any external users |

## Phases

| Phase | Description | Status |
|-------|-------------|--------|
| 6.1 | Auth Hardening | Pending |
| 6.2 | Messaging & Contacts | Pending |
| 6.3 | Drive & Attachments | Pending |
| 6.4 | Calls & Presence | Pending |
| 6.5 | Canvas Integration | Pending |
| 6.6 | Kanban Polish | Pending |
| 6.7 | UX & Accessibility | Pending |
| 6.8 | Testing & Tooling | Pending |
| 6.9 | Apple Beta Packaging | Pending |

## Position
- Current phase: None
- Completed tasks: 0
- Pending tasks: ~80 (estimated across 9 phases)

## Implementation Status (Pre-M6)

| Area | Status | Notes |
|------|--------|-------|
| Auth | Production | Password + recovery working |
| Messaging | Production | CRDT sync, threads, reactions |
| Drive | Production | Virtual disks, basic transfers |
| Calls | Research | Signaling ready, no media |
| Canvas | Production | LWW CRDT working |
| Kanban | Production | Full CRDT boards |
| UI | Beta | Components exist, polish needed |
| macOS Bundle | Production | Phase 5.1 complete |
| MCP | Production | 100+ tools |

## Blockers
- None currently

## Context for Next Session
M6 initialized with foundation-first priority. Ready to plan Phase 6.1 (Auth Hardening).

Run `/gsd:plan-phase` to create detailed task plan for Phase 6.1.
