# MCP CRDT-Aware Tool Checklist

Communitas is offline-first and CRDT-backed. Any MCP tool that creates, mutates, or reads CRDT-backed state must follow this checklist to avoid drift between UI and automation.

## When to Use This Checklist
- New MCP tool or modifying an existing tool.
- Any operation touching Kanban, messaging, drive, canvas, or other CRDT-backed state.
- Any tool that chains \"create\" then \"modify\" in a single flow (e.g., workspace init).

## Checklist (Required)
1. **Route CRDT operations through UiServices**
   - Create/update/delete for CRDT state must go through `communitas-ui-service` (`UiServices`), not direct `CommunitasApp` calls.
   - `CommunitasApp` is still used for entity creation, auth, or non-CRDT core operations.

2. **Do not mix app-only IDs with CRDT-only flows**
   - If an entity/board/column/card is created via `CommunitasApp`, you must load or create the same object in CRDT before CRDT operations.
   - Prefer creating CRDT-backed objects through `UiServices` so IDs are consistent immediately.

3. **Return CRDT IDs from tool responses**
   - Use `json_result` and include `id` for create operations.
   - Tests depend on `ToolResult.get_id()`-style extraction.

4. **Hydrate before listing or moving**
   - If a tool reads CRDT state that may not be loaded, ensure the service path loads or lists first (e.g., `list_kanban_boards`/`list_threads`).

5. **Allow eventual consistency in tests**
   - CRDT sync is asynchronous. Tests should allow retries or explicit waits where cross-node sync is expected.

6. **Keep UI parity first-class**
   - Any tool exposed to MCP must mirror the UI service behavior. If the UI uses a service, the tool should too.

7. **UI metadata stays honest**
   - If a tool triggers a UI, include `_meta.ui.resourceUri`.
   - Use `visibility: [\"model\", \"app\"]` for MCP Apps.
   - Only include `_meta.ui.context` when state is available (avoid empty payloads).

8. **Pre-auth safety**
   - Pre-auth tools must not leak secrets or perform CRDT mutations.
   - Pre-auth UI resources are allowed, but must avoid model-visible secrets.

9. **Update docs + parity tests**
   - Document the tool in `docs/api/mcp-api.md`.
   - Update parity scripts or add new ones when the surface changes.

## References
- ADR-019: Shared Rust UI Service
- ADR-022: MCP Apps Integration
- docs/api/mcp-api.md
