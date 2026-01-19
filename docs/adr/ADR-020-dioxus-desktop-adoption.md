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

---

## M3 Addendum: Advanced Surface Patterns (2026-01-19)

Milestone 3 delivered four advanced surfaces (Kanban, Drive, Calls, Canvas). This addendum documents the Dioxus patterns established.

### Drag-and-Drop Patterns

**Event handling** uses Dioxus's native drag events:

```rust
rsx! {
    div {
        draggable: true,
        ondragstart: move |e| dragging.set(Some(card_id)),
        ondragover: move |e| e.prevent_default(),
        ondrop: move |e| {
            if let Some(card) = dragging.take() {
                kanban_service.move_card(card, target_column);
            }
        },
        // ...
    }
}
```

**Optimistic UI**: Update local state immediately, reconcile on service confirmation. Show CRDT conflict banners when remote changes arrive mid-drag.

**Keyboard fallback**: Arrow keys + Enter/Space for users who cannot drag.

### Virtualization Patterns

Column virtualization for Kanban boards with many columns:

```rust
fn use_virtualized_columns(
    columns: &[Column],
    viewport_width: f32,
    column_width: f32,
) -> Vec<&Column> {
    let visible_count = (viewport_width / column_width).ceil() as usize + 2; // buffer
    let scroll_offset = use_scroll_position();
    let start = (scroll_offset / column_width) as usize;
    columns.iter().skip(start).take(visible_count).collect()
}
```

**Buffer zones** (2 columns on each side) ensure smooth scrolling without visual pops.

### Real-Time Collaboration Patterns

**Watch channels** from `UiServices` provide reactive updates:

```rust
let snapshot = use_signal(|| service.snapshot());
use_effect(move || {
    let mut rx = service.subscribe();
    spawn(async move {
        while let Ok(update) = rx.recv().await {
            snapshot.set(update);
        }
    });
});
```

**Remote cursors** (Canvas): Render colored cursors for each participant with debounced position updates (50ms).

**CRDT conflict UI**: Flash banner when local and remote edits conflict, with "Keep Mine" / "Accept Theirs" / "Merge" options.

### Media Handling Patterns (Calls)

**Device enumeration** happens once at component mount:

```rust
let devices = use_resource(|| async {
    call_service.enumerate_devices().await
});
```

**Graceful degradation**: If camera fails, offer listen-only mode. If microphone fails, show clear error with retry button.

**Permission flow**: Request permissions progressively—start with audio, add video only when user clicks camera button.

### Canvas Rendering Patterns

**Wry WebView** (default): Canvas uses HTML5 `<canvas>` element via Dioxus's WebView backend. This provides broad compatibility.

**Blitz feature flag**: For future GPU-accelerated rendering:

```toml
[features]
blitz = ["dioxus/blitz"]
```

**Layer management**: Z-index stack with explicit ordering. Layers expose `bring_to_front()` / `send_to_back()` via context menu.

**History scrubber**: Store snapshots at key moments (every 10 operations or manual checkpoint). Scrubber component renders thumbnails.

### Performance Patterns

**Lazy loading**: Directory/file lists load first 50 items, fetch more on scroll.

**Debounced inputs**: Search boxes debounce 300ms before triggering queries.

**Memoization**: Use `use_memo` for expensive computations:

```rust
let filtered_cards = use_memo(move || {
    cards.iter()
        .filter(|c| c.title.contains(&search_term()))
        .collect::<Vec<_>>()
});
```

**Background tasks**: Long operations (file upload, video encoding) run in Tokio tasks with progress channels feeding the UI.

### Component Structure

Each surface follows the same module pattern:

```
communitas-dioxus/src/components/{surface}/
├── mod.rs           # Re-exports
├── {main_view}.rs   # Primary container component
├── {list}.rs        # List/grid rendering
├── {item}.rs        # Individual item component
├── {modal}.rs       # Detail/edit modals
└── {toolbar}.rs     # Surface-specific toolbar
```

### Service Integration

All surfaces use the shared `UiServices` pattern (ADR-019):

```rust
fn SomeComponent() -> Element {
    let services = use_context::<UiServices>();
    let kanban = services.kanban();

    // Read from snapshot signal
    let boards = kanban.boards();

    // Write via service methods
    let create_board = move |name: String| {
        spawn(async move {
            kanban.create_board(name).await;
        });
    };

    rsx! { /* ... */ }
}
```

### MCP Parity

Every UI action has a corresponding MCP tool. The parity test harness (`scripts/tests/mcp_parity.sh`) validates round-trip consistency:

1. Create via MCP → Read via MCP → Verify match
2. Create via UI → Read via MCP → Verify match
3. Create via MCP → Read via UI → Verify match

This ensures AI agents and human users have identical capabilities.
