# Dioxus Desktop Prototype — Milestone 1 Plan
_Status: Near Complete (7/8 validation gates passing) • Target complete: February 7, 2026_

## 1. Goal & Exit Criteria

Deliver a production-ready navigation shell plus end-to-end auth/vault flows in `communitas-dioxus`, powered entirely by the shared Rust UI service (ADR-019) and packaged through Tauri 2 installers. This unlocks the rest of the parity program by proving we can render the full layout, guard routes, and authenticate without Dart/FFI.

Exit when:

1. Dioxus router covers every path defined in the product navigation spec (login, create, recover, dashboard, messages, projects, contacts, network, entity detail/chat/drive, project board, contact chat, “more”).  
2. Auth creation/login/recovery UX matches the archived production behavior, including four-word login, passphrase validation, clipboard interactions, and error handling.  
3. `UiServices` (auth + navigation + directory) is the single state source for both Dioxus and MCP tooling; no Dioxus-only service code remains.  
4. Desktop bundles (macOS dmg, Windows NSIS/msi, Linux AppImage) launch into the nav shell and can perform a complete login on fresh machines with required WebView dependencies installed or bootstrapped.  
5. Automated tests verify nav guarding/auth flows and MCP-driven smoke tests can log in and enumerate entities end to end.

## 2. Scope

### In-scope

- Navigation shell (sidebar, top app bar, route guards, deep links, skeleton screens).  
- Auth/vault lifecycle (login, create, recover, demo mode stub).  
- Shared Rust UI service wiring (auth controller, navigation service, directory snapshot).  
- Telemetry + logging for the above flows.  
- Installer prerequisites (WebView2 bootstrap on Windows, WebKitGTK check on Linux, notarization hooks on macOS).[^tauri-prereqs]

### Out-of-scope (later milestones)

- Messaging UI, Kanban, Drive, Calls/WebRTC, Canvas, demo-mode guided tours.  
- Mobile polish (Android/iOS bundles beyond smoke builds).  
- Performance benchmark suite (only basic metrics for auth/nav in this milestone).

## 3. Architecture Overview

| Layer | Milestone 1 responsibilities | Notes |
| --- | --- | --- |
| `communitas-ui-service` | Provide `UiServices::bootstrap()`, `AuthService`, `NavigationService`, `DirectoryService` shared with MCP. | Must expose async signals/event streams consumed by Dioxus hooks. |
| `communitas-dioxus` | Compose router + layout, manage auth state, render screens. | Uses Dioxus signals/hooks + router; leverages Subsecond hot-patching for faster iteration.[^dioxus-07] |
| Tauri 2 runner | Bundle desktop app, enforce permissions, install WebView dependencies. | Plugin manifest reserved for future MCP embedding; nav/auth milestone only requires fs/notification defaults. |
| MCP | Reuse same auth + navigation contexts for automation; milestone ensures capability manifest exposes nav/auth actions even when UI drives them. |

## 4. Workstreams & Tasks

### 4.1 Shared Rust Service Hardening

1. Finalize `communitas-ui-service` crate API for auth/navigation (structs, error enums, events).  
2. Move any remaining Dioxus-specific auth helpers into the shared crate (passphrase validation, clipboard helpers behind trait).  
3. Expose navigation snapshots + directory snapshots via `tokio::sync::watch` with debounced updates to avoid UI thrash.  
4. Add tracing spans + metrics (auth attempt latency, nav transition counts).

### 4.2 Navigation Shell Implementation

1. Mirror the production route table inside `Route` enum (`#[derive(Routable)]`).  
2. Implement `AppShell` component with:  
   - Responsive sidebar (collapsed/expanded) derived from window width.  
   - Top bar showing entity filter + presence indicator.  
   - Guarded routes based on `AuthState`.  
3. Integrate deep-link parsing (`use_route`, `use_navigator`) and add fallback to `/login` when unauthenticated.  
4. Provide skeleton loaders (signals + suspense) for directory and entity lists.  
5. Emit navigation analytics events to tracing (`nav.route_change`).

### 4.3 Auth & Vault Flows

1. Login: four-word entry + passphrase, FFI parity on validation errors, busy states, retry/backoff.  
2. Create identity: multi-step wizard, passphrase strength meter, identity word display with copy + confirmation.  
3. Recover identity: mnemonic paste/import, file-drop for vault backups (desktop).  
4. Demo mode toggle: compile-time flag hooking into shared service to spin up stub data.  
5. Biometric unlock scaffolding (desktop placeholder) aligning with future Tauri biometric plugin adoption.[^tauri-plugins]

### 4.4 Packaging & Tooling

1. Pin `dx` CLI version (0.7.x) and document installation script.  
2. Configure `tauri.conf.json` for desktop bundle targets, enabling `minimumWebview2Version` to auto-install/upg WebView2.[^tauri-bundler]  
3. Update CI to run `cargo fmt`, lint (`cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`), `dx check`, and `tauri build --target` per platform.  
4. Add smoke workflow that launches bundle on clean VM image, asserts nav/auth flows succeed with mocked credentials (`scripts/ci_dx_bundle.sh`).  
5. Publish onboarding docs for dev environment prerequisites (Rust toolchain, dx CLI, WebView runtimes, C++ build tools on Windows).[^tauri-prereqs]

### 4.5 Observability & QA Hooks

1. Panic/Crash reporting wired to `color-eyre` + `tracing`.  
2. Feature flag to force auth errors for QA.  
   - Set `COMMUNITAS_UI_FORCE_AUTH_ERROR=1` before running the UI to verify failure handling paths wired into `communitas-ui-service`.  
3. Snapshot testing of nav shell (SSR/hydration) + Auth flows (component tests).  
4. MCP-driven smoke: script uses MCP auth commands to provision a vault, then asserts Dioxus UI renders same entity list.  
5. Accessibility pass (keyboard nav, focus order) on login + nav shell screens.

## 5. Validation Strategy

| Layer | Test | Notes |
| --- | --- | --- |
| Rust unit tests | `communitas-ui-service` auth/nav modules | Cover success/error states, ensure no `unwrap/expect/panic` in non-test code. |
| Component tests | Dioxus SSR tests for Login, Create, Recover, AppShell | Validate props/state, route guards, loading states. |
| Integration | Headless Tauri tests (`tauri-driver`) to click through login & nav transitions. |
| MCP parity | CLI harness logs into Communitas via MCP, then triggers UI watchers to confirm identical directory snapshot. |
| Installer smoke | Run generated installers on clean macOS 14, Windows 11, Ubuntu 24.04 images; confirm WebView bootstrap. |

## 6. Risks & Mitigations

| Risk | Mitigation |
| --- | --- |
| WebView provisioning failures on end-user machines | Use `minimumWebview2Version` + bootstrap prompts; block UI until runtime present.[^tauri-bundler] |
| Tauri mobile plugin gaps delay biometric unlock | Abstract biometrics behind trait now; add desktop placeholder until `tauri-plugin-biometric` stabilizes.[^tauri-plugins] |
| Hot-patching instability during dev | Pin dx CLI 0.7 release and provide fallback `cargo run --features desktop` path.[^dioxus-07] |
| MCP capability drift | Include nav/auth commands in MCP regression suite; treat MCP results as source of truth for UI watchers. |

## 7. Timeline & Deliverables

| Week | Deliverable |
| --- | --- |
| Week 1 (Jan 20–24) | Finalize `UiServices` APIs, move remaining auth helpers, add tracing, create nav/auth component scaffolds. |
| Week 2 (Jan 27–31) | Complete router + AppShell layout, implement login/create/recover UX, connect to services, add tests. |
| Week 3 (Feb 3–7) | Harden packaging/CI, run installer smokes, integrate MCP parity harness, capture doc screenshots & ADR updates. |

## 8. Documentation & Follow-ups

- Update `docs/architecture/dioxus_desktop_prototype_plan.md` status table when exit criteria are met.
- Attach nav/auth screenshots + test logs to the milestone tracking issue.
- Prep Milestone 2 (Messaging + Entities) backlog once nav/auth is signed off.

## 8.1 Recent Progress (January 2026)

### Completed Work

**Phase 1: UI Polish**
- Added skeleton loading states (`SkeletonPulse`, `SkeletonWelcomeCard`, `SkeletonStatsGrid`, `SkeletonSpacesSection`) for improved UX during directory loading
- Implemented empty field validation on login form with inline error feedback
- Commit: `f7ef628 feat(ui): Add skeleton loading states and improved validation`

**Phase 2: Test Coverage**
- Extended UiServices unit tests covering auth, navigation, and directory services (53 tests total)
- Added directory watcher tests verifying auth integration and snapshot updates
- Extended WebDriver tests with sidebar toggle, route guard verification, and error flash tests
- Enhanced MCP parity script with contacts comparison and JSON artifact archiving
- Commits: `52915ed test(mcp-parity)`, `b6fffef test(webdriver)`

**Phase 3: CI Infrastructure**
- Added artifact upload steps for MCP parity JSON diffs
- Added WebDriver test artifact capture with screenshot-on-failure
- Configured JSON test reporter for structured CI output
- Commit: `2a9de65 ci: Improve test artifacts and failure debugging`

**Phase 4: Telemetry & Accessibility**
- Added tracing instrumentation (`#[instrument]`) to navigation and directory services
- Spans: `ui.nav.record_entity`, `ui.nav.record_contact`, `ui.nav.toggle_star_entity`, `ui.nav.toggle_star_contact`, `ui.nav.clear`, `ui.directory.refresh_all`
- Created accessibility smoke tests (12 tests covering heading hierarchy, form labels, keyboard navigation)
- Commit: `1d284ab feat: Add telemetry spans and accessibility tests`

### Test Files Added/Updated
- `communitas-ui-service/src/auth.rs` - Unit tests for auth lifecycle
- `communitas-ui-service/src/navigation.rs` - Unit tests + tracing spans
- `communitas-ui-service/src/directory.rs` - Unit tests + tracing spans
- `tests/webdriverio/specs/nav-auth.smoke.js` - Extended WebDriver tests
- `tests/webdriverio/specs/accessibility.smoke.js` - New accessibility tests
- `scripts/tests/mcp_nav_auth.sh` - Enhanced MCP parity harness

## 9. Validation Checklist & Evidence Capture

| Area | Test / Evidence | Command / Tooling | Artifact | Status |
| --- | --- | --- | --- | --- |
| Shared services | `communitas-ui-service` auth + navigation unit tests covering happy/error paths, forced-failure flag, persistence round-trips. | `cargo test -p communitas-ui-service` | Test report uploaded to CI (retain for milestone closeout). | ✅ 53 tests passing |
| Directory snapshot wiring | Snapshot watcher test verifying auth triggers directory refresh and nav recents update. | `cargo test -p communitas-ui-service directory::tests` | Attach log excerpt proving watchers fire post-auth. | ✅ Tests added |
| Dioxus components | SSR/component tests for `LoginRoute`, `CreateIdentityRoute`, `RecoverIdentityRoute`, and `AppShell` route guards. | `cargo test -p communitas-dioxus` | Screenshot diffs for golden snapshots + test output. | ✅ 7 tests passing |
| Router guarding | `tauri-driver` or `dx test --headless` script walks through login, route transitions, and logout to ensure guards + analytics events fire. | `scripts/tests/m1_nav_auth.tauri.sh` + WebDriverIO | CI artifact: video or log trace w/ timestamps + route list. | ✅ WebDriver tests in CI |
| MCP parity | CLI harness provisions vault via MCP, then opens Dioxus app and asserts JSON snapshots of `UiServices::directory()` match. | `scripts/tests/mcp_nav_auth.sh` | Stored JSON diff + pass/fail summary. | ✅ Contacts + entities |
| Installer smoke | `scripts/ci_dx_bundle.sh` builds desktop bundles; GitHub Actions workflow spins clean macOS/Windows/Linux VMs, installs WebView runtimes, performs scripted login, and captures screenshots. | `scripts/ci_dx_bundle.sh` + `scripts/tests/m1_installer_smoke.ps1/.sh` | Attach zipped screenshots & logs to milestone issue. | ⏳ Linux CI only |
| Accessibility | Keyboard-only traversal of login + AppShell using `axe-core`/`playwright` audit plus manual screen reader spot-check. | `tests/webdriverio/specs/accessibility.smoke.js` | Store audit report + manual checklist in `/docs/testing/m1_nav_auth_accessibility.md`. | ✅ 12 tests added |
| Telemetry | Trace log sample showing `ui.auth`, `ui.nav`, and `ui.directory` spans during login + navigation. | `RUST_LOG=info dx serve --platform desktop --features telemetry` | Include log snippet + Jaeger trace screenshot. | ✅ Spans instrumented |

**Approval flow**: Milestone 1 cannot close until every row above has (a) passing automation in CI and (b) linked artifacts/screenshots in the milestone tracking issue. QA signs off on accessibility + installer smoke, while the MCP team signs off on parity scripts.

- `scripts/tests/mcp_nav_auth.sh` + `docs/testing/mcp_nav_auth_parity.md` document the MCP parity harness and its JSON diff expectations.

---

 [^dioxus-07]: The Dioxus 0.7 hot-reload guide describes Subsecond hot-patching via `dx serve --hotpatch`, its limitations (tip crate only), and Tailwind/watch integrations—key dev-loop assumptions for this milestone.【turn1search6】  
 [^tauri-plugins]: The Tauri 2.0 RC blog details the mobile plugin push (biometric, NFC, barcode, haptics, geolocation) while noting mobile parity gaps, and the Tauri plugin catalog shows current platform coverage, so we plan around these constraints.【turn2search0】【turn2search4】  
 [^tauri-bundler]: `tauri-bundler@2.0.1-rc.6` introduces the `minimumWebview2Version` installer option so desktop bundles can trigger WebView2 upgrades when required.【turn1search1】  
 [^tauri-prereqs]: Tauri’s prerequisites guide requires Microsoft C++ Build Tools and the Edge WebView2 runtime on Windows, so our onboarding scripts and installer checks must enforce those dependencies.【turn1search0】 
