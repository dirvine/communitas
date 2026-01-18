# Dioxus Desktop Prototype Plan

_Last updated: 2026-01-18_

## 1. Objective & Success Metrics

- **Goal**: Deliver a Dioxus-first desktop client that achieves full parity with the current Communitas feature set while elevating UX polish and keeping 100% of business logic inside `communitas-core`.  
- **Drivers**: Previous thin-GUI approaches relied on extra FFI glue and non-Rust layers (`docs/MCP_THIN_GUI_ARCHITECTURE.md`). The Dioxus prototype must remove those dependencies without regressing existing flows.  
- **Exit criteria (desktop)**:  
  1. Every feature listed in §2 is navigable and functionally equivalent.  
  2. Telemetry shows feature-complete parity and comparable or better latency (<5% variance on CRUD/messaging benchmarks).  
  3. UX reviews confirm target upgrades (modernized typography, transitions, and MCP affordances) with no accessibility regressions.  
  4. Build pipelines produce signed installers for macOS, Windows, Linux with WebView provisioning scripts validated on clean machines.

## 2. Feature Parity Inventory

| Feature cluster | Reference asset | Core/MCP owner | Dioxus parity notes |
| --- | --- | --- | --- |
| Auth & vault lifecycle | `features/auth/*`, `shared/vault`, entrypoints in `main_native.dart` | Vault + identity flows already exposed via `CommunitasApi` and MCP (`docs/MCP_THIN_GUI_ARCHITECTURE.md:30-41`, `docs/adr/ADR-017-legacy-thin-client-ffi-integration.md:75-115`) | Build Dioxus auth flows on shared Rust domain types; reuse four-word login + passphrase validation logic directly from `communitas-core`. |
| Entities & membership | `features/entities/*`, `features/navigation/*` | Core entity service (FFI + MCP) | Provide list/detail RSX components backed by async signals; ensure CRDT updates stream through watchers. |
| Messaging & reactions | `features/messaging/*` | Core messaging (FFI) + MCP | Implement thread list, composer, reactions, attachments; reuse CRDT-friendly view models to minimize diff churn. |
| Kanban | `features/kanban/*` | `communitas-kanban` crate | Render board/column cards using virtualized lists; map drag-drop gestures to CRDT ops. |
| Drive/files | `features/drive/*` | File service via MCP | Mirror CRUD flows, integrate OS pickers via Tauri plugins. |
| Contacts & favorites | `features/contacts/*` | Gossip + membership services | Provide search, favorites, and context actions; tie presence badges to gossip updates. |
| Calls/WebRTC | `features/calls/*` | `saorsa-webrtc-*` | Maintain UI parity for voice/video, but plan to validate Wry/WebView performance under load. |
| Network/presence | `features/network/*` | Gossip runtime | Port dashboards/stats; ensure instrumentation surfaces in-app diagnostics. |
| Canvas/collab surfaces | `src/canvas/*` | Canvas integration layer | Evaluate WebView Canvas vs. Blitz (WGPU) renderer; keep feature toggles for advanced visuals. |
| Demo mode & onboarding | `home`, `navigation`, `demo` flows | Core stub data | Reimplement guided onboarding, demo resets, and stub data flows using compile-time feature flags. |

Actions: create a parity checklist per cluster (owner, acceptance criteria, screenshots) and track in the prototype project board. Each checklist gates promotion to the next implementation phase.

## 3. Target Architecture

### 3.1 Workspace Layout

```
communitas-core/
communitas-mcp/
communitas-dioxus/        # new crate
  src/
    app.rs               # root router + window/bootstrap
    platform/            # wry/tauri bundling glue
    features/*           # mirrors production feature clusters
    ui/                  # design system + components
  build.rs               # asset pipelines (Tailwind, icons)
dx.dioxus.json           # CLI manifest
```

- `communitas-dioxus` depends directly on `communitas-core` APIs (no FFI). Shared DTOs move into a `communitas-ui-api` crate so MCP tools and the GUI consume identical structs.
- State management uses Dioxus signals/hooks; async services run through `tokio` inside the same binary, letting us reuse tracing, error handling, and domain services.

### 3.2 Tooling & Dev Loop

- Adopt the `dx` CLI from Dioxus 0.6+/0.7 for project scaffolding, Tailwind/asset bundling, and device deployment (`dx serve`, `dx bundle`, `dx device`).[^dioxus-cli]
- Enable Subsecond hot-patching (`dx serve --hotpatch`) so Rust UI code reloads without disrupting long-lived gossip sessions; note the limitation that only the app crate hotpatches while shared crates still rebuild.[^subsecond]
- Keep feature flags for Blitz (experimental WGPU renderer) vs. default Wry/WebView; production defaults to WebView until Blitz hardens.[^dioxus-07]

### 3.3 Packaging & Platform Targets

- Desktop builds ride on Tauri 2 runners (Linux, macOS, Windows). Package installers plus bootstrap scripts that install WebView2 (Windows) or verify system WebKit (macOS/Linux).[^tauri2-main]
- Mobile builds remain exploratory in this prototype but should be validated via `dx serve --platform android|ios` to catch blockers early; Tauri 2 introduces Android/iOS but still labels mobile “not yet first-class,” so plan for manual fixes.[^tauri-rc]

### 3.4 MCP Integration

- Evaluate `tauri-plugin-mcp`/`tauri-mcp` for embedding an MCP client/server so the Dioxus app exposes local UI state to agents (parity with today’s external MCP consumers).[^tauri-mcp]
- Provide optional build flavor where Communitas runs an MCP server alongside the UI for offline automation; ensure capability files align with MCP security policies.

## 4. Implementation Phases

| Phase | Scope | Key deliverables |
| --- | --- | --- |
| 0 — Discovery | Feature inventory, UX audits, infra spikes | Parity checklists, UX upgrade backlog, performance baselines |
| 1 — Foundations | Scaffold crate, DX toolchain, shared API crate, navigation shell | `communitas-dioxus` skeleton, Tailwind tokens, router + auth shell, CI jobs (lint/test/bundle) |
| 2 — Core flows | Auth, entities, messaging, contacts, network dashboards | Functional parity demos, instrumentation hooks, MCP data exposure |
| 3 — Advanced surfaces | Kanban, drive, calls/WebRTC, canvas | Drag/drop + virtualization, file pickers, WebRTC device management, canvas renderer evaluation |
| 4 — Experience polish | Visual system, accessibility, localization, offline/demo mode | Theming tokens, keyboard/accessibility checks, demo data toggles |
| 5 — Stabilization | Packaging, installers, telemetry, perf QA, MCP validation | Signed installers, runtime WebView provisioning, perf benchmarks vs. production baseline, MCP regression suite |

Phase gates require passing automated parity tests (component-level plus MCP-driven workflows) and UX sign-off.

## 5. Plugin & Capability Matrix (initial target)

| Capability | Candidate plugin / crate | Notes |
| --- | --- | --- |
| Notifications & push | `@tauri-apps/plugin-notification` / `tauri-plugin-notification` | Cross-platform notifications; mobile requires channel setup and permission prompts.[^notif] |
| Biometric unlock | `@tauri-apps/plugin-biometric` / `tauri-plugin-biometric` | FaceID/TouchID + Android biometrics for vault unlock + quick login.[^biometric] |
| Share **into** the app | `tauri-plugin-sharetarget` (Android) + community iOS share-intent plugin | Android plugin is stable; partner with community plugin author for iOS parity.[^sharetarget][^share-intents] |
| Share **from** the app | `tauri-plugin-mobile-share` | Supports iOS today; extend to Android or fall back to Web Share API capability.[^mobile-share] |
| Deep links & URL schemes | `@tauri-apps/plugin-deep-link` | Required for invite links and MCP callbacks; needs config per platform.[^deeplink] |
| App lifecycle hooks | `tauri-plugin-app-events` | Captures Android/iOS foreground/background, needed for gossip + call handling.[^appevents] |
| MCP integrations | `tauri-plugin-mcp`, `tauri-mcp` | Enables embedded MCP clients/servers for agent workflows.[^tauri-mcp] |
| Secure storage / keystore | Legacy mobile builds used platform secure storage — evaluate `tauri-plugin-stronghold`/custom plugin | If none exists, build plugin with Kotlin/Swift shims using guidance from the mobile plugin dev docs.[^mobile-plugin-guide] |
| Filesystem & pickers | Tauri shell/filesystem APIs, plus custom permission capability files | Track open issues around filesystem paths on mobile to avoid regressions.[^fs-issue] |
| Android package install (optional) | `tauri-plugin-android-package-install` | Useful for side-loaded updates in internal testing.[^android-install] |

Action: Maintain this matrix as a living document; each capability requires an owner, stability rating, and fallbacks when mobile coverage lags.

## 6. Risks & Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| **WebView dependencies**: Dioxus desktop rides on system WebViews (Edge WebView2, WebKitGTK). Missing runtimes break the UI. | Installers fail on fresh systems. | Ship runtime bootstrap scripts and runtime checks at startup; block UI until dependency satisfied. |
| **Mobile maturity gaps**: Tauri 2 acknowledges mobile isn’t yet first-class; plugin parity is incomplete. | Delays mobile GA; potential app-store rejections. | Keep prototype desktop-focused, run weekly Android/iOS smoke builds, and contribute fixes upstream (capability files, Gradle templates).[^tauri-rc][^android-bug] |
| **Tooling churn**: Subsecond hotpatch + Blitz are evolving; CLI updates may break workflows. | Developer productivity drops. | Pin CLI/toolchain versions per workspace, add smoke tests for `dx serve --hotpatch`, document fallback workflows. |
| **Plugin ecosystem variance**: Some must-have plugins are community-maintained (e.g., IAP, share intents). | Maintenance burden. | Budget time for in-house forks; follow mobile plugin dev guide to extend official plugins when necessary.[^mobile-plugin-guide] |
| **MCP security**: Embedding MCP inside the GUI increases attack surface. | Sensitive data exposure. | Restrict MCP capability files, require explicit opt-in, reuse existing MCP auth flows. |

## 7. Validation & QA Strategy

- **Automated parity tests**: Use Dioxus SSR/unit tests plus end-to-end suites that drive the UI via MCP tooling (e.g., `tauri-plugin-mcp` automation) and compare outputs to existing golden data captured from production workflows.  
- **Performance benchmarks**: Instrument core flows (auth login, entity list load, message send, kanban drag) and compare vs. the current production baseline; alert if Dioxus deviates >5%.  
- **UX validation**: Maintain screenshot baselines per feature, run contrast and accessibility checks, and review motion specs.  
- **Install/runtime validation**: CI builds signed installers, runs them on clean VMs to confirm WebView provisioning, plugin permissions, and secure storage behavior.  
- **Mobile smoke tests**: Nightly `dx bundle --platform android`/`ios` to catch SDK drift early, even if mobile isn’t GA in this prototype.

## 8. Next Steps

1. Approve this plan and spin up a tracker (issue or project board) keyed to the phases above.  
2. Assign owners for parity checklists and plugin investigations.  
3. Scaffold `communitas-dioxus` (`dx new communitas-dioxus --template desktop`), vendor Tailwind tokens, and wire up a hello-world screen that calls an existing `communitas-core` query.  
4. Stand up CI for linting, `dx check`, and packaging; ensure logs capture Subsecond hotpatch readiness.  
5. Begin Phase 0 discovery workshops with design/product to lock UX upgrade opportunities per feature cluster.

## 9. Feature Parity Checklists

Each item tracks: _Reference asset_, _Core dependencies_, _Dioxus deliverables_, and _Owner_ (to be assigned when work begins). Mark checkboxes in the tracker when each acceptance target is met.

| Feature cluster | Reference asset | Owner | Status | Notes |
| --- | --- | --- | --- | --- |
| Navigation shell & layout | `lib/src/core/router.dart`, `features/navigation/*` | _TBD_ | ⏳ In progress | Prototype sidebar/router implemented in `communitas-dioxus/src/main.rs:24-520` (routes for home/messages/projects/contacts/network/more). |
| Auth & vault lifecycle | `features/auth/*` | _TBD_ | ⏳ In progress | Login/create/recover flows wired to `CommunitasApi` in `communitas-dioxus/src/main.rs:70-364`. |
| Home/dashboard | `features/home/presentation/home_screen.dart` | _TBD_ | ☐ Not started | Cards, MCP shortcuts |
| Messaging & contact chat | `features/messaging/*`, `features/contacts/presentation/contact_chat_screen.dart` | _TBD_ | ☐ Not started | Threads, reactions, attachments |
| Projects & Kanban | `features/navigation/presentation/projects_list_screen.dart`, `features/kanban/*` | _TBD_ | ☐ Not started | Drag/drop, CRDT deltas |
| Drive/files | `features/drive/presentation/drive_browser_screen.dart` | _TBD_ | ☐ Not started | Upload/download, previews |
| Contacts & presence | `features/contacts/*` | _TBD_ | ☐ Not started | Search, favorites, presence badges |
| Network panel | `features/network/presentation/network_panel_screen.dart` | _TBD_ | ☐ Not started | Gossip stats + controls |
| Calls/WebRTC | `features/calls/*` | _TBD_ | ☐ Not started | Lobby, device management |
| Canvas/visual surfaces | `lib/src/canvas/*.dart` | _TBD_ | ☐ Not started | Canvas vs Blitz evaluation |
| Demo mode & onboarding | `features/home/*`, `navigation` flows | _TBD_ | ☐ Not started | Stub data, coach marks |

- **Navigation shell & layout**  
  - Reference asset: archived thin-client router definitions (GoRouter for login/home/messages/projects/contacts/more/entity detail/chat/drive/project board/contact chat/network).  
  - Dependencies: Auth state via `authNotifierProvider`, tab metadata (`features/navigation/*`).  
  - Dioxus deliverables: central router + layout with guarded routes, universal top/bottom navigation, deep-link support; screenshot parity for every route; instrumentation for navigation events.

- **Auth & vault lifecycle**  
  - Reference asset: archived thin-client auth flows.  
  - Dependencies: Vault creation/import/recovery APIs, four-word login.  
  - Deliverables: wizard flows, error messaging, clipboard interactions, biometric unlock hooks, demo-mode onboarding; parity acceptance test covering login+create+recover.

- **Home/dashboard**  
  - Reference asset: archived thin-client home screen implementation.  
  - Dependencies: aggregated stats (entities, unread messages, network alerts).  
  - Deliverables: Dioxus home layout with modular cards, MCP shortcuts, responsive grid behavior.

- **Messaging & contact chat**  
  - Reference asset: archived thin-client messaging + contact chat screens.  
  - Dependencies: Messaging service, reactions, attachments, presence.  
  - Deliverables: thread list, composer, emoji/reaction picker, attachments, per-entity contexts, keyboard shortcuts; load/perf tests for large histories.

- **Projects/Kanban**  
  - Reference asset: archived thin-client projects list + Kanban board screens.  
  - Dependencies: `communitas-kanban` CRDT services.  
  - Deliverables: project directory, Kanban board with drag/drop, swimlane filtering, CRDT conflict feedback, offline-first UX cues.

- **Drive/File browser**  
  - Reference asset: archived thin-client drive browser screen.  
  - Dependencies: File metadata service, upload/download, sharing.  
  - Deliverables: tree/list toggles, previews, upload/download progress, share links, quota indicators, OS picker integration.

- **Contacts & presence**  
  - Reference asset: archived thin-client contacts UI.  
  - Dependencies: Gossip presence, favorites, search.  
  - Deliverables: search-as-you-type, favorites pinning, contextual actions (invite, DM, call), connection word badges tied to gossip signals.

- **Network panel**  
  - Reference asset: archived thin-client network diagnostics panel.  
  - Dependencies: Gossip diagnostics, node metrics.  
  - Deliverables: charts/stats, MCP call-outs, controls (start/stop/connect), log viewer; instrumentation for performance counters.

- **Calls/WebRTC**  
  - Reference asset: archived thin-client calls surfaces.  
  - Dependencies: `saorsa-webrtc-*` crates.  
  - Deliverables: call lobby, device selector, in-call controls, screen share UI, multi-party layout, connection quality indicators.

- **Canvas/visual surfaces**  
  - Reference asset: archived thin-client canvas tooling.  
  - Dependencies: Canvas data sources, collaborative state (CRDT).  
  - Deliverables: evaluate HTML5 Canvas vs. Blitz; ensure tools (draw, annotate, embed) match historical behavior; performance tests for high-frequency updates.

- **Demo mode & onboarding**  
  - Reference asset: archived `home`/`navigation` modules (demo login, stub data).  
  - Dependencies: Demo storage toggles, onboarding hints.  
  - Deliverables: Dioxus demo feature flag with stub core bindings, onboarding coach marks, telemetry gating to prevent cross-contamination with real data.

## 10. Initial Engineering Tasks (Phase 1 backlog)

1. **Repo scaffolding**  
   - Create `communitas-dioxus` crate via `dx new ... --template desktop`.  
   - Add crate to `Cargo.toml` workspace members and configure shared `communitas-ui-api` crate for view models.
2. **Toolchain setup**  
   - Pin `dx` CLI + toolchain versions; `scripts/install_dx.sh` installs dx `0.7.3` consistently in CI and local environments.  
   - Configure Tailwind/Vite pipelines referenced by `build.rs`.
3. **Core integration**  
   - Wire `communitas-core` services into the Dioxus app using shared async traits; expose feature flags for demo mode.  
   - Implement auth + router shell with placeholder screens to unblock feature teams.
4. **CI & automation**  
   - Extend GitHub Actions (or existing CI) with `cargo fmt`, `cargo clippy` (workspace), `dx check`, and desktop bundling jobs. `rust.yml` now installs dx via the script above and runs `dx check --platform desktop` as a gating step.  
   - Provision artifact uploads for macOS dmg, Windows msi/exe, Linux AppImage/deb.
5. **Observability & QA hooks**  
   - Add tracing subscribers, panic hooks, and metrics exporters to `communitas-dioxus`.  
   - Stand up golden-test harness comparing archived reference snapshots vs. current Dioxus renders for shared data sets.

---

[^dioxus-cli]: Dioxus 0.6 overhauled the CLI (`dx serve`, device deploy, Tailwind bundling`) to pursue a dramatically improved developer experience. Source: Dioxus v0.6 release notes, Jun 2025 (`turn0search5`, `turn0search6`).
[^subsecond]: Dioxus 0.7 highlighted Subsecond hot-patching via `dx serve --hotpatch`, enabling runtime Rust code swaps without losing state (`turn0search0`, `turn0search1`, `turn0search8`).
[^dioxus-07]: Dioxus 0.7 announcements cite the experimental WGPU-based “Dioxus Native” renderer (“Blitz”) plus Axum integration (`turn0search7`).
[^tauri2-main]: Tauri 2.0 marketing outlines single-codebase support across desktop + mobile using system WebViews (`turn1search7`).
[^tauri-rc]: The Tauri 2.0 release-candidate post stresses mobile support exists but isn’t yet “first-class,” with ongoing plugin parity work (`turn1search8`).
[^notif]: Official Tauri notification plugin docs list support across desktop + mobile and setup requirements (`turn2search0`).
[^biometric]: Tauri biometric plugin docs describe Android/iOS biometric prompts and iOS plist requirements (`turn2search1`, `turn2search2`).
[^sharetarget]: `tauri-plugin-sharetarget` docs cover receiving Android share intents (`turn3search8`).
[^share-intents]: Community post announcing an Android/iOS share-intent plugin with queued ingestion (`turn3reddit12`).
[^mobile-share]: `tauri-plugin-mobile-share` docs describe iOS share sheet support and roadmap for Android parity (`turn4search7`).
[^deeplink]: Tauri deep-link plugin docs cover cross-platform scheme registration and limitations (`turn3search7`).
[^appevents]: `tauri-plugin-app-events` crate captures Android/iOS lifecycle hooks (`turn3search1`).
[^tauri-mcp]: `tauri-plugin-mcp` and related tools expose MCP capabilities inside Tauri apps (`turn4search0`, `turn4search3`, `turn4search5`).
[^mobile-plugin-guide]: Official Tauri mobile plugin development guide explains Kotlin/Swift shims and plugin structure (`turn1search0`, `turn3search0`).
[^fs-issue]: Open issue tracking filesystem path inconsistencies on iOS/Android (`turn3search5`).
[^android-install]: `tauri-plugin-android-package-install` adds Android package installation flows for testing (`turn3search10`).
[^android-bug]: Recent Android SDK compatibility bug (`turn2search3`) underscores need for proactive CI checks.
