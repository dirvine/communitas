# ADR-020: Dioxus Desktop Adoption

## Status

Accepted (2026-01-18)

## Context

The legacy multi-language thin-client (ADR-017) added a Dart runtime, method-channel
wrappers, and FRB (Rust ↔ Dart bridge) bindings on top of the Rust core. Maintaining
that parallel stack slowed delivery, fragmented UX state, and complicated MCP
automation. On January 18, 2026 we archived all related code and its auxiliary
layers under the repository’s `archive/` tree, leaving Dioxus as the sole GUI path.
Our goals are:

1. **All business logic lives in `communitas-core`** with zero Dart/TypeScript
   mirrors or HTTP/FFI interop layers.
2. **MCP parity stays automatic**—identical commands, DTOs, and lifecycle hooks
   flow through the shared `communitas-ui-service` crate (ADR-019).
3. **Advanced desktop + mobile UX** leverage Rust-native tooling (Subsecond
   hot-patching, tracing, WGPU renderers) without JNI/Obj-C bridges.
4. **Plugin risk drops** because we reuse the Rust + Tauri ecosystem instead of
   bespoke legacy plugins or host language shims.

## Decision

Adopt **Dioxus + Tauri 2** as the primary desktop client, packaged as the
`communitas-dioxus` crate that links directly against `communitas-core` and the
`communitas-ui-service` layer.

- **State & services**: The shared Rust UI service (ADR-019) is the canonical
  abstraction for auth, navigation, directory snapshots, presence, and feature
  toggles. Every surface—Dioxus desktop, future mobile builds, and MCP—consumes
  these services directly; no Dart-facing API remains.
- **Rendering**: Use Dioxus WebView (Wry) for GA builds, with an opt-in Blitz
  renderer flag when the WGPU backend matures. The router enumerates every app
  route (`/login`, `/messages`, `/projects`, etc.) so Communitas stays
  fully-controllable via MCP deep links.
- **Packaging**: Bundle via Tauri 2 (desktop-first, exploratory mobile). Install
  scripts ensure WebView2/WebKitGTK availability and seed the MCP capability
  manifest.
- **Automation**: Embed an optional MCP server (tauri-plugin-mcp) so AI models
  can drive the UI and reach every feature exposed through the MCP contract.
- **Phasing**: Execute the prototype plan in docs/architecture/
  dioxus_desktop_prototype_plan.md—Phase 1 (Milestone 1) delivers the nav shell
  and auth flows; later milestones bring feature parity for messaging, kanban,
  drive, calls, canvas, and demo mode.

## Consequences

### Benefits

1. **Pure Rust UX stack**: No Dart runtime or method channels—just Rust crates
   shared across desktop/mobile/MCP, which simplifies profiling and debugging.
2. **Single DTO surface**: `communitas-ui-api` feeds Dioxus and MCP alike,
   eliminating duplicate adapters and keeping UX/machine semantics aligned.
3. **AI automation parity**: MCP and the GUI hit the same Rust handlers, so
   every screen is scriptable via MCP with zero impedance mismatch.
4. **Faster iteration**: Dioxus Subsecond hot-patching and Tauri’s tooling
   shorten dev loops versus the retired thin-client build/restart cycle.

### Trade-offs

1. **WebView dependencies**: Desktop users must have WebView2/WebKitGTK. We
   need installer checks and fallback messaging.
2. **Plugin maturity**: Some Tauri mobile plugins (biometrics, share sheets)
   still trail other ecosystems; we must budget engineering time for upstream
   contributions or temporary native shims.
3. **New tooling**: Engineers must learn `dx`, Subsecond hot-reload, and Tauri’s
   security/permissions model.

### Mitigations

- Gate each milestone with automated MCP-driven regression tests and UX review
  to prevent drift from core behavior.
- Upstream or fork critical Tauri plugins early (notifications, biometrics,
  share target, MCP embedding) to avoid blocking later milestones.

## Alternatives Considered

1. **Retain multi-language UI stacks** (Dart, React Native, etc.). Rejected:
   duplicating business logic outside Rust reintroduces drift and FFI layers.
2. **Adopt Tauri + Yew/Leptos**. Rejected: the team already invests in Dioxus,
   whose component model and CLI align with existing skill sets.
3. **Rewrite MCP GUI in Electron/TypeScript**. Rejected for performance, larger
   footprint, and duplicating business logic outside Rust.

## References

- docs/architecture/dioxus_desktop_prototype_plan.md
- docs/MCP_THIN_GUI_ARCHITECTURE.md
- ADR-017, ADR-018, ADR-019
