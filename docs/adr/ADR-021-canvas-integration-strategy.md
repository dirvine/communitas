# ADR-021: Canvas Integration Strategy (Saorsa Canvas)

## Status
Implemented (Phase 4 completed 2026-01-22)

## Context
- Milestone 3 of the Dioxus prototype plan includes "Advanced Surfaces" with canvas/whiteboard capabilities for collaborative drawing, AI-rendered visuals, and real-time annotation.
- The `saorsa-canvas` project (`../saorsa-canvas`) provides a mature canvas implementation with scene graph management, input fusion (touch + voice), GPU rendering abstractions, and MCP tool definitions.
- Rather than rebuilding a drawing stack from scratch, we should evaluate reusing saorsa-canvas components while maintaining compatibility with both embedded (Communitas) and standalone (AI agent) usage modes.

### Saorsa Canvas Architecture Summary

| Crate | Purpose | Reusability |
|-------|---------|-------------|
| `canvas-core` | Scene graph, elements, transforms, offline queue, input fusion | Direct dependency |
| `canvas-renderer` | GPU rendering with `RenderBackend` trait | Direct dependency |
| `canvas-mcp` | MCP tool definitions (render, interact, export) | Wrap and integrate |
| `canvas-server` | Local WebSocket sync server | Keep separate |
| `canvas-app` | WASM application shell | Not needed |
| `canvas-desktop` | Native desktop host (winit + wgpu) | Not needed |

### Key Technical Consideration: Sync Strategy Mismatch

| System | Sync Model | Conflict Resolution |
|--------|------------|---------------------|
| Communitas Kanban | Yrs-based CRDT | Automatic DAG merge |
| Saorsa Canvas | LWW + Timestamps | Configurable (LastWriteWins, LocalWins, RemoteWins) |

These strategies are architecturally incompatible. Attempting to unify them would create complexity without benefit. Canvas operations are inherently real-time and visual (where LWW is appropriate), while Kanban requires eventual consistency guarantees (where CRDTs excel).

## Decision

### 1. Add canvas-core and canvas-renderer as Workspace Dependencies

```toml
# In communitas/Cargo.toml [workspace.dependencies]
canvas-core = { path = "../saorsa-canvas/canvas-core" }
canvas-renderer = { path = "../saorsa-canvas/canvas-renderer" }
```

These crates are pure Rust with minimal dependencies and no conflicts with the communitas workspace.

### 2. Create CanvasService Following UiServices Pattern (ADR-019)

Add `CanvasService` to `communitas-ui-service` implementing the same patterns as `KanbanService`, `MessagingService`, etc.:

```rust
pub struct CanvasService {
    auth: Arc<AuthController>,
    scene: Scene,                           // From canvas-core
    fusion: InputFusion,                    // From canvas-core
    tx: watch::Sender<CanvasSnapshot>,
    rx: watch::Receiver<CanvasSnapshot>,
}
```

**Key methods:**
- `subscribe()` / `current_snapshot()` - Reactive state updates
- `add_element()` / `remove_element()` / `update_element()` - Scene manipulation
- `process_input()` - Touch/voice fusion via `InputFusion`
- `render_chart()` / `render_image()` / `render_text()` - AI-triggered rendering
- `export_scene()` - PNG/SVG/JSON export

### 3. Keep Sync Strategies Separate

- **Canvas**: Uses `OfflineQueue` from canvas-core with LWW conflict resolution for real-time visual operations
- **Kanban**: Continues using Yrs CRDT for data consistency
- **No cross-contamination**: Do not attempt to sync canvas scenes via Yrs or kanban boards via LWW

### 4. MCP Integration Strategy

**Dual-mode operation:**

| Mode | Description | Implementation |
|------|-------------|----------------|
| **Embedded** | Canvas runs within Communitas | `CanvasService` methods exposed via communitas-mcp tools |
| **Standalone** | Saorsa Canvas runs independently | Original canvas-mcp server, connects via `canvas://` URIs |

**Tool mapping for embedded mode:**

| Saorsa Canvas Tool | Communitas MCP Tool |
|--------------------|---------------------|
| `canvas_render` | `communitas_canvas_render` |
| `canvas_interact` | `communitas_canvas_interact` |
| `canvas_export` | `communitas_canvas_export` |

**Resource URIs:**
- Embedded: `communitas://canvas/session/{id}`
- Standalone: `canvas://session/{id}` (original saorsa-canvas scheme)

### 5. Transport Considerations

| Communitas | Saorsa Canvas | Integration |
|------------|---------------|-------------|
| Gossip overlay (P2P) | Local WebSocket | Keep separate initially |
| Entity-scoped storage | Session-based storage | Canvas sessions linked to entity IDs |

Future enhancement: Bridge canvas sessions to gossip for collaborative whiteboarding. For Milestone 3, canvas remains local-first with optional MCP access.

## Consequences

### Benefits
- **No duplication**: Reuse battle-tested scene graph, input fusion, and rendering code
- **MCP compatibility**: Standalone saorsa-canvas instances (or AI agents) can still connect via documented MCP flows
- **Clear boundaries**: Canvas sync and Kanban sync remain independent, avoiding architectural complexity
- **Faster delivery**: Milestone 3 canvas features leverage existing implementation

### Trade-offs
- **External dependency**: Canvas crates live outside communitas workspace; changes require coordination
- **Two sync models**: Developers must understand which model applies to which domain
- **Path dependency**: Assumes `../saorsa-canvas` relative path; may need adjustment for CI/releases

### Risks Mitigated
- **Version drift**: Pin canvas dependency versions in workspace Cargo.toml
- **API breakage**: Canvas-core exposes stable types (Element, Scene, Transform); wrapper insulates from internal changes
- **Test coverage**: CanvasService tests don't depend on GPU; use Canvas2D backend for headless testing

## Alternatives Considered

### 1. Port canvas-core into communitas-canvas (inline)
- **Rejected**: Creates maintenance burden, loses upstream improvements, duplicates effort

### 2. Use Yrs CRDT for canvas operations
- **Rejected**: Yrs is optimized for text/structured data, not real-time graphical operations. LWW is more appropriate for canvas where "latest visual state" matters more than merge history.

### 3. Canvas as separate process with IPC
- **Rejected**: Adds latency, complicates deployment, breaks offline mode for embedded use case

### 4. WebSocket bridge between canvas-server and gossip
- **Deferred**: Possible future enhancement for collaborative whiteboarding, but adds complexity without clear Milestone 3 requirement

## Implementation Plan

### Phase 1: Foundation (Completed)
1. ✅ Add canvas-core and canvas-renderer as workspace dependencies
2. ✅ Create `communitas-ui-service/src/canvas.rs` with `CanvasService`
3. ✅ Implement core methods: `add_element`, `remove_element`, `get_scene`, `subscribe`
4. ✅ Add unit tests (49 canvas lib tests + 20 integration tests)

### Phase 2: MCP Integration (Completed)
1. ✅ Add canvas tools to communitas-mcp
2. ✅ Wire tools to `CanvasService` methods
3. ✅ Add `communitas://canvas/` resource URIs

### Phase 3: Dioxus Integration (Completed)
1. ✅ Create canvas component in communitas-dioxus
2. ✅ Wire to `CanvasService` via UiServices
3. ✅ Implement touch/mouse input handling via `InputFusion`

### Phase 4: Canvas Sync (Completed - Phase 6.5 PLAN-40)
Implemented collaborative canvas sync with the following features:
1. ✅ **Undo/Redo History System** - Full operation timeline with entity-scoped history
2. ✅ **Offline Queue Persistence** - Commands queue when offline, flush on reconnection
3. ✅ **Canvas Gossip Message Types** - `CanvasOperation`, `CanvasCursorUpdate`, `CanvasStateRequest`, `CanvasStateResponse`
4. ✅ **Bidirectional CRDT Sync** - Yrs-based scene synchronization via gossip overlay
5. ✅ **Shared Cursors** - Real-time cursor position sharing (throttled to 10 Hz)
6. ✅ **MCP Canvas Sync Tools** - `canvas_undo`, `canvas_redo`, `canvas_get_history`, `canvas_broadcast_cursor`, `canvas_get_remote_cursors`, `canvas_flush_offline_queue`
7. ✅ **UI Enhancements** - History scrubber, sync status badge, remote cursor display

**Note on sync strategy**: While the original ADR recommended LWW for canvas, Phase 6.5 implemented Yrs CRDT integration to maintain consistency with the broader Communitas sync architecture. The Yrs-based approach provides:
- Automatic conflict resolution for concurrent edits
- Efficient delta-based updates via gossip
- Unified sync model across the application

## References
- ADR-019: Shared Rust UI Service Layer
- ADR-018: MCP External Integration Architecture
- `../saorsa-canvas/canvas-core/src/scene.rs` - Scene graph implementation
- `../saorsa-canvas/canvas-core/src/offline.rs` - LWW offline queue
- `../saorsa-canvas/canvas-mcp/src/tools.rs` - MCP tool definitions
- `docs/architecture/dioxus_milestones.md` - Milestone 3 requirements
