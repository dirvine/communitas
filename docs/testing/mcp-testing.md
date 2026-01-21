# MCP Testing Strategy

This document describes the testing strategy for the `communitas-mcp` crate, which provides Model Context Protocol (MCP) tools for AI agents to interact with Communitas.

## Overview

### Purpose

The MCP test suite validates that all 138+ MCP tools work correctly, route through UiServices properly, and maintain API contract stability. The testing philosophy ensures that any operation performed via MCP produces the same observable state changes as the equivalent operation via UiServices (the "parity principle").

### Test Coverage

| Test Type | Count | Purpose |
|-----------|-------|---------|
| Parity Tests | 211 | Verify MCP tools route to UiServices correctly |
| Golden Tests | 11 | Validate response structure (keys, types) |
| E2E Workflows | 4 | Test multi-step real-world operations |

### Test Files

- [`communitas-mcp/tests/parity_test.rs`](../../communitas-mcp/tests/parity_test.rs) - Parity integration tests
- [`communitas-mcp/tests/golden_test.rs`](../../communitas-mcp/tests/golden_test.rs) - Golden data comparison tests
- [`communitas-mcp/tests/e2e_workflows.rs`](../../communitas-mcp/tests/e2e_workflows.rs) - End-to-end workflow tests
- [`communitas-mcp/tests/golden/`](../../communitas-mcp/tests/golden/) - Golden data fixtures

## Parity Testing

### Purpose

Parity tests ensure that MCP tools correctly route to UiServices, guaranteeing that the Dioxus UI and MCP AI agents see the same data and behavior. This is critical because both interfaces share the same underlying services.

### Core Pattern

Every parity test follows this pattern:

```rust
#[test]
fn test_tool_name_operation() {
    run_async_test!(async {
        // 1. Create isolated test environment
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // 2. Call the MCP tool
        let result = call_tool(&app, &services, "tool_name", Some(json!({
            "param1": "value1",
            "param2": "value2"
        }))).await;

        // 3. Verify response
        assert!(!result.is_error, "Tool should succeed");
        let response = parse_tool_response(&result);
        
        // 4. Verify specific fields or behaviors
        assert!(response.get("expected_field").is_some());
    });
}
```

### Key Components

#### `make_test_services()`

Creates an isolated test environment with:
- Temporary storage directory
- Fresh `CommunitasApp` instance
- Configured `UiServices` layer

```rust
async fn make_test_services(temp: &TempDir) -> (Arc<CommunitasApp>, UiServices) {
    let storage = UiStorage::from_path(temp.path()).expect("failed to create storage");
    let app = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            temp.path().join("app_storage").to_string_lossy().to_string(),
        )
        .await
        .expect("failed to create app"),
    );
    let services = UiServices::new(storage, app.clone()).expect("failed to create services");
    (app, services)
}
```

#### `run_async_test!` Macro

Handles async test execution with adequate stack size (8MB) to prevent stack overflows in complex async operations:

```rust
macro_rules! run_async_test {
    ($test_fn:expr) => {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on($test_fn);
            })
            .unwrap()
            .join()
            .unwrap();
    };
}
```

#### `call_tool()`

Invokes an MCP tool by name with optional JSON arguments:

```rust
let result = call_tool(&app, &services, "create_kanban_board", Some(json!({
    "title": "Test Board",
    "entity_id": "test-entity"
}))).await;
```

#### `parse_tool_response()`

Extracts and parses JSON from tool responses:

```rust
fn parse_tool_response(result: &ToolCallResult) -> Value {
    if result.is_error {
        return json!({ "error": true });
    }
    result.content.first()
        .and_then(extract_text)
        .and_then(|text| serde_json::from_str(text).ok())
        .unwrap_or(json!({}))
}
```

### Test Categories

Parity tests are organized by service domain:

1. **Tool Registration Tests** - Verify tools are registered correctly
2. **Navigation Service Tests** - list_disks, list_entities, etc.
3. **Auth Service Tests** - authenticate, create_vault, etc.
4. **Messaging Service Tests** - send_message, list_threads, etc.
5. **Kanban Service Tests** - Board, column, card, and tag operations
6. **Canvas Service Tests** - Snapshot and element operations
7. **Network Service Tests** - network_status, peer operations

## Golden Data Testing

### Purpose

Golden data tests validate that MCP tool responses conform to expected JSON structures without requiring exact value matches. This enables CI detection of API contract changes.

### Fixture Format

Golden fixtures are JSON files in `communitas-mcp/tests/golden/` with this structure:

```json
{
  "_description": "Human-readable description of what this tests",
  "structure": {
    "field_name": "type_spec",
    "nested_object": {
      "inner_field": "type_spec"
    },
    "array_field": [
      {
        "item_field": "type_spec"
      }
    ]
  },
  "required_tools": ["tool1", "tool2"],
  "minimum_tool_count": 100
}
```

### Type Specifications

| Type Spec | Matches |
|-----------|---------|
| `"string"` | JSON string values |
| `"number"` | JSON number values |
| `"boolean"` | JSON boolean values |
| `"array"` | JSON array values |
| `"object"` | JSON object values |
| `"any"` | Any JSON value |
| `"type\|optional"` | Field may be missing; if present, must match type |

### Example Golden Fixture

`tests/golden/list_entities.json`:
```json
{
  "_description": "Golden data for list_entities - verifies entity list structure",
  "response_structure": {
    "entities": "array",
    "count": "number|optional"
  },
  "item_structure": {
    "entity_id": "string",
    "name": "string",
    "entity_type": "string"
  }
}
```

### Verification Functions

```rust
/// Verify that a JSON object contains expected keys with matching types
fn verify_object_structure(actual: &Value, expected_structure: &Value) -> Result<(), String>;

/// Verify tool response structure matches golden data expectations
fn verify_tool_response_structure(result: &ToolCallResult, golden: &Value) -> Result<(), String>;

/// Verify array items match expected structure
fn verify_array_items_structure(response: &Value, array_key: &str, golden: &Value, item_structure_key: &str) -> Result<(), String>;
```

## E2E Workflow Testing

### Purpose

E2E workflow tests verify multi-step operations that demonstrate real-world usage patterns and verify state consistency across operations. These tests ensure that sequences of tool calls work correctly together.

### Workflow Patterns

#### Workflow 1: Entity Management Lifecycle

Tests the complete entity lifecycle: create, list, update, delete, verify deletion.

```rust
#[test]
fn workflow_entity_management_lifecycle() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Step 1: List entities initially
        let initial_list = call_tool(&app, &services, "list_entities", None).await;

        // Step 2: Create a new entity
        let create_result = call_tool(&app, &services, "create_entity", Some(json!({
            "name": "E2E Test Entity",
            "entity_type": "group"
        }))).await;

        // Step 3: List entities and verify new entity exists
        let list_after_create = call_tool(&app, &services, "list_entities", None).await;

        // Step 4: Update the entity
        let update_result = call_tool(&app, &services, "update_entity", Some(json!({
            "entity_type": "group",
            "entity_id": entity_id,
            "name": "E2E Test Entity - Updated"
        }))).await;

        // Step 5: Delete the entity
        let delete_result = call_tool(&app, &services, "delete_entity", Some(json!({
            "entity_type": "group",
            "entity_id": entity_id
        }))).await;

        // Step 6: List entities and verify deletion
        let final_list = call_tool(&app, &services, "list_entities", None).await;
    });
}
```

#### Workflow 2: Kanban Board Workflow

Tests complete Kanban workflow: create board, add columns, create cards, move cards.

#### Workflow 3: Messaging Workflow

Tests messaging flow: create thread, send messages, list messages.

#### Workflow 4: Canvas Workflow

Tests canvas operations: create canvas, add elements, get snapshot.

### State Verification

E2E tests verify state consistency by:
1. Checking entity counts before and after operations
2. Verifying created items appear in list operations
3. Confirming deleted items no longer appear
4. Validating updates persist correctly

## Running Tests

### Run All MCP Tests

```bash
cargo test -p communitas-mcp
```

### Run Specific Test Types

```bash
# Parity tests only
cargo test -p communitas-mcp --test parity_test

# Golden data tests only
cargo test -p communitas-mcp --test golden_test

# E2E workflow tests only (single-threaded for state consistency)
cargo test -p communitas-mcp --test e2e_workflows -- --test-threads=1
```

### Run Specific Test

```bash
# Run a specific test by name
cargo test -p communitas-mcp --test parity_test test_create_kanban_board

# Run with output
cargo test -p communitas-mcp --test parity_test -- --nocapture
```

### Debug Mode

```bash
RUST_LOG=debug cargo test -p communitas-mcp --test parity_test -- --nocapture
```

## CI Integration

### GitHub Actions Workflow

Tests run automatically in `.github/workflows/rust.yml`:

```yaml
- name: Run tests
  run: cargo test --all-features

- name: MCP nav/auth parity check
  run: ./scripts/tests/mcp_nav_auth.sh

- name: MCP messaging parity check
  run: ./scripts/tests/mcp_messaging.sh
```

### When Tests Run

- On every push to any branch
- On every pull request
- Can be triggered manually via workflow dispatch

### Test Failure Handling

When tests fail in CI:
1. Check the GitHub Actions log for the failing test
2. Run the test locally to reproduce
3. Fix the issue and push again
4. CI will re-run automatically

## Adding New Tests

### Adding a New Parity Test

1. **Identify the tool to test** - Check `communitas-mcp/src/tools/` for available tools

2. **Add the test function** to `parity_test.rs`:

```rust
#[test]
fn test_new_tool_name_operation() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Call the tool
        let result = call_tool(&app, &services, "new_tool_name", Some(json!({
            "required_param": "value"
        }))).await;

        // Verify routing worked (no "Unknown tool" error)
        let text = extract_text(result.content.first().unwrap()).unwrap_or("");
        assert!(!text.contains("Unknown tool"), "Tool should route correctly");

        // Verify expected behavior
        if !result.is_error {
            let response = parse_tool_response(&result);
            assert!(response.get("expected_field").is_some());
        }
    });
}
```

3. **Run the test locally**:
```bash
cargo test -p communitas-mcp --test parity_test test_new_tool_name
```

### Adding a New Golden Test

1. **Create the golden fixture** in `tests/golden/`:

```json
{
  "_description": "Golden data for new_tool - description",
  "response_structure": {
    "field1": "string",
    "field2": "number",
    "optional_field": "string|optional"
  },
  "item_structure": {
    "item_field": "string"
  }
}
```

2. **Add the test function** to `golden_test.rs`:

```rust
#[test]
fn test_golden_new_tool() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Load golden data
        let golden = load_golden("new_tool");

        // Call the tool
        let result = call_tool(&app, &services, "new_tool", Some(json!({
            "param": "value"
        }))).await;

        // Verify structure matches golden data
        verify_tool_response_structure(&result, &golden)
            .expect("Response should match golden structure");
    });
}
```

3. **Run the test**:
```bash
cargo test -p communitas-mcp --test golden_test test_golden_new_tool
```

### Adding a New E2E Workflow

1. **Design the workflow** - Identify the sequence of operations to test

2. **Add the test function** to `e2e_workflows.rs`:

```rust
#[test]
fn workflow_new_feature_flow() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Step 1: Setup
        let setup_result = call_tool(&app, &services, "setup_tool", None).await;
        assert!(!setup_result.is_error, "Step 1: Setup should succeed");

        // Step 2: Perform action
        let action_result = call_tool(&app, &services, "action_tool", Some(json!({
            "param": "value"
        }))).await;
        assert!(!action_result.is_error, "Step 2: Action should succeed");

        // Step 3: Verify state
        let verify_result = call_tool(&app, &services, "verify_tool", None).await;
        let response = parse_tool_response(&verify_result);
        assert_eq!(response["expected"], json!("value"), "Step 3: State should be correct");

        // Step 4: Cleanup
        let cleanup_result = call_tool(&app, &services, "cleanup_tool", None).await;
        assert!(!cleanup_result.is_error, "Step 4: Cleanup should succeed");
    });
}
```

3. **Run the test** (single-threaded to avoid state conflicts):
```bash
cargo test -p communitas-mcp --test e2e_workflows workflow_new_feature -- --test-threads=1
```

### Updating Golden Data

When APIs change legitimately:

1. **Verify the change is intentional** - Review the API change that caused the failure

2. **Update the fixture** - Modify the JSON in `tests/golden/`

3. **Run the test to verify**:
```bash
cargo test -p communitas-mcp --test golden_test test_golden_affected_tool
```

4. **Document the change** in your commit message

## Best Practices

### Test Isolation

- Always use `TempDir` for storage to ensure test isolation
- Tests should not depend on external state
- Each test should clean up after itself

### Error Handling in Tests

- Use `unwrap()`/`expect()` freely in tests for clarity
- Include descriptive messages: `assert!(!result.is_error, "Step 2: Action should succeed")`
- Check for "Unknown tool" errors to verify routing

### Naming Conventions

- Parity tests: `test_<tool_name>_<operation>`
- Golden tests: `test_golden_<tool_name>`
- E2E workflows: `workflow_<feature>_<flow_description>`

### Documentation

- Add doc comments to complex test helpers
- Update this document when adding new test patterns
- Include step comments in E2E workflows
