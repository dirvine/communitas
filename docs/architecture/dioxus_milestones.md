# Dioxus Desktop Prototype — Milestone Tracker

_Last updated: 2026-01-19_

This tracker collapses the prototype plan into milestone-sized chunks so we can see scope, status, and the evidence required for sign-off. Each milestone links back to the deeper planning doc and its validation checklist.

| Milestone | Scope Highlights | Status | Validation Gates |
| --- | --- | --- | --- |
| **0 — Discovery & Inventory** | Feature inventory, UX audit, plugin spike, performance baselines. | ✅ Complete (Jan 18, 2026) | Inventory spreadsheet + UX backlog attached to planning issue #421. |
| **1 — Foundations (Nav shell + Auth)** | Scaffold `communitas-dioxus`, wire `communitas-ui-service`, deliver router/nav shell, login/create/recover, desktop bundles + installer smoke. | ✅ Complete (8/8 gates passing, Jan 18, 2026) | All validation gates passing: shared services ✅, directory wiring ✅, Dioxus components ✅, router guarding ✅, MCP parity ✅, installer smoke (Linux + macOS) ✅, accessibility ✅, telemetry ✅. |
| **2 — Core Flows (Messaging/Entities/Contacts)** | Thread list, composer, entity directory, presence badges, MCP exposure, perf instrumentation. | ✅ Complete (Jan 19, 2026) | All validation gates passing: MessagingService ✅, PresenceService ✅, Dioxus components ✅, MCP parity ✅, WebDriver tests ✅, accessibility ✅, telemetry ✅. See `dioxus_milestone2_messaging_entities.md`. |
| **3 — Advanced Surfaces (Kanban/Drive/Calls/Canvas)** | Drag & drop Kanban with column virtualization and CRDT conflict banners, drive browser with tree/list views and checksum validation, WebRTC lobby with device selectors and graceful fallbacks, canvas with toolbar/layers/shared cursors/history scrubber. | ✅ Complete (Jan 19, 2026) | All validation gates passing: UiServices (4 surfaces) ✅, Dioxus components ✅, MCP tools ✅, MCP parity tests (9/9) ✅, Call networking deferred to integration. See `dioxus_milestone3_advanced_surfaces.md`. |
| **4 — Experience Polish (Design system, accessibility, localization, demo mode)** | Design token system, keyboard/reader support, localization scaffolding, onboarding/demo reset. | ☐ Not started | Axe-core reports, localization coverage checklist, demo-mode telemetry guardrails. |
| **5 — Stabilization (Packaging + MCP validation)** | Signed installers, WebView bootstrap scripts, telemetry dashboards, MCP regression suite + capability review. | ☐ Not started | Installer smoke logs (macOS/Windows/Linux), MCP automation transcripts, release checklist signed. |

## How to use this tracker

1. **Update Statuses Weekly** — Edit the table as milestones move from ☐ → ⏳ → ✅. Record target completion dates inline.  
2. **Link Evidence** — Every validation gate refers to artifacts stored in CI (logs, screenshots, JSON diffs). Paste URLs or issue numbers next to the relevant checkpoint once available.  
3. **Escalate Risks Early** — If a milestone risks slipping, capture blockers directly in the scope row so adjacent teams (MCP, Core, QA) can assist.  
4. **Keep Milestone Docs Canonical** — This tracker summarizes; detailed acceptance criteria remain in the milestone-specific docs (e.g., `docs/architecture/dioxus_milestone1_nav_auth.md`).  
5. **Tie CI to Milestones** — Ensure workflows (`cargo fmt`, `cargo clippy`, `dx check`, `scripts/ci_dx_bundle.sh`, MCP parity harness) reference the target milestone in their artifact names so evidence is easy to discover.
