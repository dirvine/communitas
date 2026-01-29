# Communitas Testing Documentation

This directory contains comprehensive testing documentation for the Communitas platform.

## Quick Start

**Most Important Documents**:
1. [Test Strategy](test-strategy.md) - Overall testing philosophy and approach
2. [MCP Testing](mcp-testing.md) - MCP tool testing methodology
3. [Coverage Report](coverage-report.md) - Current test coverage metrics

## Milestone 10: MCP Testnet Validation ✅

**Status**: Complete (2026-01-29)
**Completion**: 100% (187/187 tools, 8/8 widgets, 50 distributed tests)

### Milestone 10 Reports

| Document | Purpose | Status |
|----------|---------|--------|
| [Coverage Report](coverage-report.md) | Comprehensive test coverage metrics | ✅ |
| [Performance Summary](performance-summary.md) | Performance baselines and metrics | ✅ |
| [Test Results Matrix](test-results-matrix.md) | Phase-by-phase test results | ✅ |
| [Known Issues](known-issues.md) | Issue tracking and remediation | ✅ |
| [Testnet Infrastructure](testnet-infrastructure.md) | Deployment documentation | ✅ |
| [Sign-off Checklist](milestone-signoff.md) | Approval criteria and sign-off | ✅ |
| [Milestone Completion](milestone-10-completion.md) | Executive summary | ✅ |

## Testing Strategy Documents

### Core Documentation

| Document | Description | Audience |
|----------|-------------|----------|
| [Test Strategy](test-strategy.md) | Overall testing approach, pyramid, coverage targets | All |
| [MCP Testing](mcp-testing.md) | MCP tool testing patterns (parity, golden, E2E) | Developers |
| [Smoke Test Checklist](smoke-test-checklist.md) | Quick validation checklist | QA/Ops |

### Infrastructure Documentation

| Document | Description | Audience |
|----------|-------------|----------|
| [Testnet Deployment](testnet-deployment.md) | VPS deployment procedures | DevOps |
| [Testnet Infrastructure](testnet-infrastructure.md) | Node inventory and configuration | DevOps |
| [Testnet Load Results](testnet-load-results.md) | Load testing metrics | Performance |

### Performance Documentation

| Document | Description | Audience |
|----------|-------------|----------|
| [Performance Baselines](performance-baselines.md) | Widget performance targets | Frontend |
| [Performance Summary](performance-summary.md) | M10 performance metrics | All |

## Test Categories

### Unit Tests

**Location**: `src/` directories within each crate
**Pattern**: `#[cfg(test)] mod tests { ... }`
**Coverage**: ~200 tests (27% of total)

**Key Suites**:
- `communitas-core/src/**/*.rs` - Core business logic
- `communitas-kanban/src/**/*.rs` - Kanban CRDT operations

### Integration Tests

**Location**: `tests/` directories within each crate
**Pattern**: Separate test files (`*_test.rs`)
**Coverage**: ~220 tests (50% of total)

**Key Suites**:
- `communitas-mcp/tests/parity_test.rs` - Tool routing (182KB, comprehensive)
- `communitas-mcp/tests/identity_core_tools_test.rs` - Phase 10.2 (26KB)
- `communitas-mcp/tests/messaging_tools_test.rs` - Phase 10.3 (28KB)
- `communitas-mcp/tests/kanban_tools_test.rs` - Phase 10.4 (37KB)
- `communitas-mcp/tests/phase105_drive_tests.rs` - Phase 10.5 (24KB)
- `communitas-mcp/tests/phase106_*.rs` - Phase 10.6 (12 files)

### E2E Tests

**Location**: `communitas-mcp/ui-bundles/e2e/`
**Pattern**: Playwright specs (`*.spec.js`)
**Coverage**: 169 tests (23% of total)

**Widget Tests** (119 tests):
- `smoke.spec.js` - Basic smoke tests (8 tests)
- `contacts.spec.js` - Contacts widget (12 tests)
- `messages.spec.js` - Messages widget (15 tests)
- `kanban.spec.js` - Kanban widget (18 tests)
- `drive.spec.js` - Drive widget (14 tests)
- `canvas.spec.js` - Canvas widget (13 tests)
- `settings.spec.js` - Settings widget (10 tests)
- `search.spec.js` - Search widget (11 tests)
- `notifications.spec.js` - Notifications widget (10 tests)
- `integration.spec.js` - Cross-widget tests (8 tests)

**Distributed Tests** (50 tests):
- Concurrent request handling (6 tests)
- Load distribution (5 tests)
- Tool consistency (6 tests)
- MCP protocol compliance (8 tests)
- Demo mode security (8 tests)
- Error handling (10 tests)
- Performance regression (7 tests)

## Running Tests

### Quick Smoke Test

```bash
# Rust unit tests
cargo test --workspace --lib

# MCP integration tests
cargo test -p communitas-mcp

# Widget E2E tests
npm test communitas-mcp/ui-bundles/e2e/
```

### Full Test Suite

```bash
# All Rust tests
cargo test --workspace

# All widget tests
npm test communitas-mcp/ui-bundles/e2e/

# Distributed tests
npm test communitas-mcp/ui-bundles/e2e/distributed/
```

### Specific Test Categories

```bash
# Parity tests only
cargo test -p communitas-mcp --test parity_test

# Golden data tests
cargo test -p communitas-mcp --test golden_test

# E2E workflows
cargo test -p communitas-mcp --test e2e_workflows

# Phase-specific tests
cargo test -p communitas-mcp --test phase106_network_basic_test

# Single widget E2E
npm test communitas-mcp/ui-bundles/e2e/contacts.spec.js
```

### Coverage Reports

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --workspace --out Html

# Open report
open tarpaulin-report.html
```

## Test Quality Standards

### Rust Standards

✅ **Required**:
- No `unwrap()` in production code (tests OK)
- No `expect()` in production code (tests OK)
- No `panic!()` anywhere
- All errors use `Result<T, E>`
- Descriptive assertion messages
- TempDir for storage isolation

✅ **Enforced by CI**:
- `cargo fmt --check` - Code formatting
- `cargo clippy -- -D warnings` - Zero clippy warnings
- `cargo test --all` - All tests passing
- `cargo audit` - Zero security vulnerabilities

### JavaScript/Playwright Standards

✅ **Required**:
- Descriptive test names
- Clear step comments
- Proper async/await
- Cleanup after tests
- Isolated test data

## CI/CD Integration

### GitHub Actions Workflows

| Workflow | Triggers | Purpose |
|----------|----------|---------|
| `rust.yml` | Every push | Rust checks, integration tests |
| `widget-e2e.yml` | Every push | Widget E2E tests |
| `audit.yml` | Daily | Security audit |

### Quality Gates (PR Checks)

All PRs must pass:
- ✅ Rust formatting (`cargo fmt --check`)
- ✅ Clippy lints (`cargo clippy -- -D warnings`)
- ✅ All tests (`cargo test --all`)
- ✅ Security audit (`cargo audit`)
- ✅ Widget E2E tests

## Testnet Nodes

| Node | Region | IP | Port | Status |
|------|--------|----|----|--------|
| saorsa-2 | NYC, US | 142.93.199.50 | 3040 | ✅ |
| saorsa-3 | SFO, US | 147.182.234.192 | 3040 | ✅ |
| saorsa-7 | Nuremberg, DE | 116.203.101.172 | 3040 | ⚠️ Firewall |

**Access**:
```bash
# Health check
curl http://142.93.199.50:3040/health

# Tool list
curl -X POST http://142.93.199.50:3040/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
```

## Performance Targets

| Metric | Target | Achieved | Status |
|--------|--------|----------|--------|
| Tool Call P95 | <500ms | 137ms | ✅ |
| Widget Load | <200ms | 109ms | ✅ |
| Memory Usage | <50MB | 3.4MB | ✅ |
| Test Suite | <15min | 10m 40s | ✅ |

## Contributing

### Adding New Tests

1. **MCP Tool Test**: Add to `communitas-mcp/tests/parity_test.rs`
2. **Widget Test**: Create `*.spec.js` in `communitas-mcp/ui-bundles/e2e/`
3. **Integration Test**: Create `*_test.rs` in crate's `tests/` directory
4. **Golden Data Test**: Add fixture to `communitas-mcp/tests/golden/`

### Test Naming Conventions

- **Parity tests**: `test_<tool_name>_<operation>`
- **Golden tests**: `test_golden_<tool_name>`
- **E2E workflows**: `workflow_<feature>_<flow_description>`
- **Widget tests**: `test('<Widget Name> - <Scenario>')`

## Troubleshooting

### Common Issues

**Flaky Tests**: Check `known-issues.md` for resolved flaky test patterns

**Timeout Issues**: Increase timeout for slow CI environments

**Memory Issues**: Use TempDir for isolation, clean up properly

**Network Issues**: Check testnet-deployment.md for firewall configuration

## Resources

### External Documentation

- [Playwright Docs](https://playwright.dev) - E2E testing framework
- [Cargo Test Docs](https://doc.rust-lang.org/cargo/commands/cargo-test.html) - Rust testing
- [Proptest Guide](https://altsysrq/proptest/proptest/index.html) - Property-based testing

### Internal References

- [Architecture Docs](../architecture/README.md) - System design
- [API Docs](../api/mcp-api.md) - MCP protocol
- [Development Guide](../development/README.md) - Setup instructions

## Support

For testing questions or issues:
1. Check this documentation
2. Review [known-issues.md](known-issues.md)
3. Contact: david@saorsalabs.com

---

*Last updated: 2026-01-29*
*Milestone 10 Complete: 187/187 tools, 8/8 widgets, 100% pass rate*
