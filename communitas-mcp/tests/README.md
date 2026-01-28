# MCP Test Architecture

This document describes the testing infrastructure for `communitas-mcp`.

## Overview

The MCP test suite provides comprehensive coverage for all 197 MCP tools through:

- **Unit tests**: In-process testing using `McpTestClient`
- **Integration tests**: HTTP transport testing using `McpTestNode`
- **E2E tests**: Multi-node distributed testing
- **Coverage tracking**: Automated tool coverage reporting

## Directory Structure

```
tests/
├── README.md                  # This file
├── harness/                   # Test harness library
│   ├── mod.rs                 # Module exports
│   ├── client.rs              # McpTestClient and McpTestNode
│   └── results.rs             # Test result aggregator
├── coverage/                  # Coverage tracking
│   ├── mod.rs                 # Module exports
│   ├── tracker.rs             # Coverage tracker implementation
│   ├── report.json            # Generated coverage data
│   └── REPORT.md              # Generated coverage report
├── generator/                 # Test stub generator
│   ├── mod.rs                 # Module exports
│   ├── generator.rs           # Test stub generator
│   └── REPORT.md              # Generator report
├── fixtures/                  # Reusable test data
│   ├── mod.rs                 # Module exports
│   ├── contacts.rs            # Contact fixtures
│   ├── entities.rs            # Entity fixtures
│   ├── kanban.rs              # Kanban fixtures
│   ├── messaging.rs           # Messaging fixtures
│   └── drive.rs               # Drive fixtures
├── inventory/                 # Tool inventory
│   └── tools.json             # Complete tool inventory
├── golden/                    # Golden files for snapshot testing
│   └── list_tools.json        # Expected tool list
├── coverage_check.rs          # Coverage check test
├── generate_stubs.rs          # Test generator runner
├── fixtures_test.rs           # Fixture validation tests
├── harness_test.rs            # Harness library tests
├── parity_test.rs             # Widget parity tests
├── comprehensive_e2e.rs       # Comprehensive E2E tests
├── mcp_e2e.rs                 # MCP E2E workflow tests
├── mcp_apps_test.rs           # MCP Apps UI tests
├── golden_test.rs             # Golden file tests
├── auth_tests.rs              # Authentication tests
├── csp_test.rs                # CSP configuration tests
├── e2e_workflows.rs           # E2E workflow tests
└── generated_stubs.rs         # Auto-generated test stubs
```

## Test Harness

### McpTestClient (In-Process)

For unit testing without HTTP overhead:

```rust
use harness::{McpTestClient, ToolAssert};
use serde_json::json;

#[tokio::test]
async fn test_create_contact() {
    let client = McpTestClient::new().await;

    let result = client.call_tool("create_contact", json!({
        "name": "Alice Anderson"
    })).await;

    result.assert_success()
          .assert_has("id")
          .assert_str_eq("name", "Alice Anderson");
}
```

### McpTestNode (HTTP Transport)

For integration testing with HTTP transport:

```rust
use harness::McpTestNode;

#[tokio::test]
async fn test_with_http() {
    let node = McpTestNode::start("test").await;

    let result = node.call_tool("list_contacts", json!({
        "limit": 10
    })).await;

    result.assert_success();
}
```

### ToolResult Assertions

| Method | Description |
|--------|-------------|
| `assert_success()` | Verify tool call succeeded |
| `assert_error()` | Verify tool call failed |
| `assert_has(key)` | Verify field exists |
| `assert_str_eq(key, value)` | Verify string field equals |
| `assert_array_min(key, min)` | Verify array has minimum length |
| `assert_contains(pattern)` | Verify response contains pattern |

## Coverage Tracking

### Running Coverage Check

```bash
cargo test -p communitas-mcp --test coverage_check -- --nocapture
```

This generates:
- `tests/coverage/report.json` - Machine-readable coverage data
- `tests/coverage/REPORT.md` - Human-readable coverage report

### Coverage Metrics

The tracker identifies:
- Total tools (from inventory)
- Tested tools (found in test files)
- Untested tools (gap analysis)
- Coverage percentage

## Test Generator

Generate test stubs for untested tools:

```bash
cargo test -p communitas-mcp --test generate_stubs -- --nocapture
```

Generated stubs are written to `tests/generated_stubs.rs`.

## Test Fixtures

Reusable test data for consistent testing:

```rust
use fixtures::*;

// Contact fixtures
let alice = alice_contact();
let bob = bob_contact();

// Kanban fixtures
let board = board_fixture("My Board");
let card = card_fixture("column-123", "Test Task");

// Messaging fixtures
let msg = message_fixture("Hello!");
let thread = thread_with_participants("Team Chat", &["user1", "user2"]);

// Drive fixtures
let file = file_fixture("test.txt", "content");
let content = sample_binary_content();
```

## CI Integration

The `.github/workflows/mcp-coverage.yml` workflow:

1. Runs coverage check on PRs
2. Generates coverage report
3. Uploads artifacts
4. Posts summary to GitHub Actions

### Coverage Threshold

Current threshold: 60%

If coverage drops below threshold, a warning is shown but the build continues.

## Running Tests

### All MCP Tests

```bash
cargo test -p communitas-mcp
```

### Specific Test File

```bash
cargo test -p communitas-mcp --test parity_test
```

### With Output

```bash
cargo test -p communitas-mcp -- --nocapture
```

## Testnet Deployment

For distributed E2E testing, use the deployment script:

```bash
# Deploy to all testnet nodes
./scripts/deploy-mcp-testnet.sh -b

# Check status
./scripts/deploy-mcp-testnet.sh -s

# Tear down
./scripts/deploy-mcp-testnet.sh -t
```

### Testnet Nodes

| Node | IP | Role |
|------|-----|------|
| saorsa-2 | 142.93.199.50 | Bootstrap (NYC) |
| saorsa-3 | 147.182.234.192 | Bootstrap (SFO) |
| saorsa-7 | 116.203.101.172 | General (DE) |

## Adding New Tests

1. **Unit Test**: Add test to appropriate `*_test.rs` file
2. **Update Inventory**: If adding new tool, update `inventory/tools.json`
3. **Add Fixtures**: Create reusable test data in `fixtures/`
4. **Run Coverage**: Verify coverage improves

## Tool Categories

| Category | Description | Tools |
|----------|-------------|-------|
| identity | Auth and profiles | 9 |
| entities | Groups, channels, projects | 11 |
| members | Membership management | 8 |
| contacts | Contact management | 13 |
| messaging | Messages and threads | 20 |
| kanban | Boards, columns, cards | 22 |
| drive | Files and directories | 31 |
| canvas | Collaborative whiteboard | 20 |
| call | Voice/video calls | 19 |
| network | P2P connectivity | 5 |
| presence | User presence | 10 |
| ... | ... | ... |

Total: **197 tools** across **23 categories**

## Best Practices

1. **Use fixtures**: Don't hardcode test data
2. **Test isolation**: Each test should be independent
3. **Assert specifics**: Use targeted assertions
4. **Clean up**: Tests should not leave artifacts
5. **Document**: Add doc comments to test functions
