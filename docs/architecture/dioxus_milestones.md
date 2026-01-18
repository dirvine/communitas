# Dioxus Desktop Prototype — Milestone Tracker

_Last updated: 2026-01-18_

This tracker collapses the prototype plan into milestone-sized chunks so we can see scope, status, and the evidence required for sign-off. Each milestone links back to the deeper planning doc and its validation checklist.

| Milestone | Scope Highlights | Status | Validation Gates |
| --- | --- | --- | --- |
| **0 — Discovery & Inventory** | Feature inventory, UX audit, plugin spike, performance baselines. | ✅ Complete (Jan 18, 2026) | Inventory spreadsheet + UX backlog attached to planning issue #421. |
| **1 — Foundations (Nav shell + Auth)** | Scaffold `communitas-dioxus`, wire `communitas-ui-service`, deliver router/nav shell, login/create/recover, desktop bundles + installer smoke. | ⏳ In progress (target Feb 7, 2026) | All rows in `docs/architecture/dioxus_milestone1_nav_auth.md#9-validation-checklist--evidence-capture` have passing CI artifacts + screenshots. |
| **2 — Core Flows (Messaging/Entities/Contacts)** | Thread list, composer, entity directory, presence badges, MCP exposure, perf instrumentation. | ☐ Not started | Component + MCP parity tests for messaging/entity flows, perf delta <5%. |
| **3 — Advanced Surfaces (Kanban/Drive/Calls/Canvas)** | Drag & drop Kanban, drive uploads, WebRTC lobby + controls, canvas renderer decision (Wry vs Blitz). | ☐ Not started | Feature demos recorded, WebRTC soak tests (4 peers), file picker smoke on all OS runners. |
| **4 — Experience Polish (Design system, accessibility, localization, demo mode)** | Design token system, keyboard/reader support, localization scaffolding, onboarding/demo reset. | ☐ Not started | Axe-core reports, localization coverage checklist, demo-mode telemetry guardrails. |
| **5 — Stabilization (Packaging + MCP validation)** | Signed installers, WebView bootstrap scripts, telemetry dashboards, MCP regression suite + capability review. | ☐ Not started | Installer smoke logs (macOS/Windows/Linux), MCP automation transcripts, release checklist signed. |

## How to use this tracker

1. **Update Statuses Weekly** — Edit the table as milestones move from ☐ → ⏳ → ✅. Record target completion dates inline.  
2. **Link Evidence** — Every validation gate refers to artifacts stored in CI (logs, screenshots, JSON diffs). Paste URLs or issue numbers next to the relevant checkpoint once available.  
3. **Escalate Risks Early** — If a milestone risks slipping, capture blockers directly in the scope row so adjacent teams (MCP, Core, QA) can assist.  
4. **Keep Milestone Docs Canonical** — This tracker summarizes; detailed acceptance criteria remain in the milestone-specific docs (e.g., `docs/architecture/dioxus_milestone1_nav_auth.md`).  
5. **Tie CI to Milestones** — Ensure workflows (`cargo fmt`, `cargo clippy`, `dx check`, `scripts/ci_dx_bundle.sh`, MCP parity harness) reference the target milestone in their artifact names so evidence is easy to discover.
