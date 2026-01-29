# Distributed E2E Test Suite

Comprehensive end-to-end testing framework for Communitas MCP servers deployed across geographic regions.

## Overview

The distributed E2E test suite validates MCP functionality across deployed testnet nodes (NYC, SFO, Nuremberg). These tests ensure:

- **Multi-node consistency**: Same tool behavior across all nodes
- **Geographic performance**: Latency baselines for cross-US deployments
- **Protocol compliance**: JSON-RPC 2.0 and MCP specification adherence
- **Security validation**: Demo mode constraints and input validation
- **Load handling**: Concurrent requests and failover scenarios

**Note**: Current deployment uses isolated MCP servers (not P2P-connected). Tests focus on HTTP MCP endpoints, geographic latency, and multi-node validation.

## Test Suites

### 1. Multi-Node Tool Validation (`multi-node-tool-validation.test.ts`)

Validates all MCP tool categories across nodes:

- **Coverage**: Identity, Member, Messaging, Kanban, Canvas, Drive tools
- **Purpose**: Ensure tool consistency and error handling
- **Nodes tested**: NYC, SFO

### 2. Geographic Latency Baseline (`geographic-latency.test.ts`)

Measures response times across regions:

- **Metrics**: P50, P95, P99 percentiles
- **Baselines**: <200ms US-US, <500ms for complex operations
- **Tools tested**: Health endpoints, tools/list, simple/complex tool calls

### 3. Concurrent Request Handling (`concurrent-requests.test.ts`)

Tests concurrent load handling:

- **Scenarios**: 10/50/100 concurrent requests per node
- **Validation**: 100% success rate, reasonable response times
- **Metrics**: Average duration, P50/P95/P99 under load

### 4. Load Distribution Tests (`load-distribution.test.ts`)

Tests load balancing and failover:

- **Round-robin**: 100 requests distributed evenly across 2 nodes
- **Capacity**: 100 requests to single node
- **Failover**: Graceful handling of node failures

### 5. Tool Consistency Validation (`tool-consistency.test.ts`)

Ensures identical behavior across nodes:

- **Structure**: Same tool, same args → same result format
- **Errors**: Consistent error messages and codes
- **Schema**: 187 tools with matching schemas

### 6. MCP Protocol Compliance (`protocol-compliance.test.ts`)

Validates JSON-RPC 2.0 compliance:

- **Format**: jsonrpc, id, result/error fields
- **Error codes**: -32700 (parse error), -32600 (invalid request)
- **Schema**: Valid JSON Schema for all tool definitions

### 7. Demo Mode Security Tests (`demo-security.test.ts`)

Tests security constraints:

- **Authentication**: No auth required (demo mode)
- **Injection**: SQL/command injection protection
- **Data exposure**: No system paths or secrets in errors
- **Input validation**: Oversized requests handled gracefully

### 8. Error Handling Validation (`error-handling.test.ts`)

Comprehensive error scenarios:

- **Invalid tools**: Clear error messages
- **Malformed requests**: Proper JSON-RPC errors
- **Wrong types**: Graceful handling or coercion
- **Network errors**: Timeout handling

### 9. Performance Regression Suite (`performance-regression.test.ts`)

Establishes baselines for CI:

- **Single request**: P50=121ms, P95=137ms, P99=161ms
- **Concurrent load**: 10/50 requests with metrics
- **Throughput**: ~7.7 req/sec sequential
- **Geographic**: NYC vs SFO comparison

## Running Tests

### Prerequisites

```bash
cd tests/distributed
npm install
```

Dependencies:
- Node.js 18+
- Playwright 1.58+
- Network access to testnet nodes

### Run All Tests

```bash
npm test
```

### Run Specific Suite

```bash
npx playwright test concurrent-requests.test.ts
npx playwright test protocol-compliance.test.ts --reporter=list
```

### Run with Different Reporters

```bash
# Line reporter (minimal output)
npx playwright test --reporter=line

# HTML report (detailed results)
npx playwright test --reporter=html
npx playwright show-report

# JSON reporter (for CI)
npx playwright test --reporter=json
```

## Test Results Interpretation

### Success Criteria

- **Pass rate**: 100% (all tests must pass)
- **Latency**: <1s for simple operations, <2s for complex
- **Throughput**: >1 req/sec sustained
- **Consistency**: Identical behavior across nodes

### Common Issues

#### Test Timeouts

**Symptom**: Tests fail with "Timeout exceeded"

**Causes**:
- Network congestion
- Node overload
- Firewall blocking (saorsa-7 Nuremberg)

**Solution**:
```bash
# Increase timeout
npx playwright test --timeout=60000
```

#### Connection Failures

**Symptom**: "fetch failed" errors

**Causes**:
- Node offline
- IP address changed
- Network routing issues

**Solution**:
1. Verify node health: `curl http://142.93.199.50:3040/health`
2. Check IP addresses in test files match current deployment
3. Confirm network access (VPN, firewall)

#### Inconsistent Results

**Symptom**: Tool behavior differs between nodes

**Causes**:
- Version mismatch (nodes running different binary versions)
- Configuration drift
- Demo identity state divergence

**Solution**:
1. Check versions: Compare `version` field in `/health` responses
2. Redeploy nodes if versions differ
3. Use fresh demo identities for consistency

## Testnet Configuration

### Active Nodes

| Node | IP | Region | Port | Status |
|------|-----|--------|------|--------|
| saorsa-2 | 142.93.199.50 | NYC1 (DigitalOcean) | 3040 | ✅ Active |
| saorsa-3 | 147.182.234.192 | SFO3 (DigitalOcean) | 3040 | ✅ Active |
| saorsa-7 | 77.42.39.239 | Nuremberg (Hetzner) | 3040 | ⚠️ Firewall blocks external |

### Endpoints

- **Health**: `http://<ip>:3040/health`
- **MCP**: `http://<ip>:3040/mcp`
- **Tools List**: POST to `/mcp` with `{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}`

## Extending Tests

### Adding New Test Suite

1. Create test file: `tests/distributed/my-new-test.test.ts`
2. Import Playwright: `import { test, expect } from '@playwright/test';`
3. Define test groups: `test.describe('My Feature', () => { ... });`
4. Add tests: `test('does something', async () => { ... });`

Example:

```typescript
import { test, expect } from '@playwright/test';

const NODES = {
  NYC: { ip: '142.93.199.50', port: 3040 },
  SFO: { ip: '147.182.234.192', port: 3040 },
};

test.describe('My Feature Tests', () => {
  test('validates something', async () => {
    const response = await fetch(`http://${NODES.NYC.ip}:${NODES.NYC.port}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'tools/call',
        params: { name: 'my_tool', arguments: {} },
      }),
    });

    const data = await response.json();
    expect(data.result).toBeDefined();
  });
});
```

### Adding Test to CI

Update `.github/workflows/distributed-e2e.yml`:

```yaml
- name: Run distributed E2E tests
  run: |
    cd tests/distributed
    npx playwright test --reporter=list
```

## Known Limitations

### P2P Testing Not Available

Current deployment = standalone MCP servers (HTTP only). For P2P testing:

- **Network configuration**: Requires bootstrap nodes, peer discovery
- **CRDT synchronization**: Multi-node state sync not tested
- **NAT traversal**: Hole punching scenarios unavailable
- **Partition recovery**: Split-brain scenarios need P2P network

**Future work**: Phase 10.10+ will add P2P network testing when infrastructure ready.

### Geographic Coverage

- **US-only**: NYC and SFO (both DigitalOcean US)
- **Europe**: Nuremberg node blocked by firewall (not in tests)
- **Asia**: No nodes deployed

### Demo Mode Only

Tests run against `--demo` mode MCP servers:

- No authentication
- Fresh identities each session
- Limited state persistence
- Not representative of production security

## Troubleshooting

### "Unknown tool" Errors

**Cause**: Tool name changed or doesn't exist on node version.

**Fix**:
```bash
# Check available tools
curl -s http://142.93.199.50:3040/mcp -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | jq '.result.tools[] | .name'
```

### Firewall Issues (saorsa-7)

**Symptom**: saorsa-7 (Nuremberg) tests timeout.

**Cause**: Hetzner firewall blocks external traffic to port 3040.

**Fix**: Use NYC or SFO nodes only, or configure Hetzner firewall rules.

### Performance Degradation

**Symptom**: Tests slower than baselines.

**Causes**:
- Node under load from other tests
- Network congestion
- Geographic routing changes

**Fix**:
```bash
# Run performance test in isolation
npx playwright test performance-regression.test.ts --workers=1
```

## Maintenance

### Updating Node IPs

When testnet node IPs change:

1. Update IP addresses in test files (search for old IP)
2. Update this documentation
3. Verify with health check: `curl http://<new-ip>:3040/health`

### Version Updates

When MCP protocol or tools change:

1. Review tool list: Check for added/removed tools
2. Update test expectations
3. Re-baseline performance metrics
4. Update documentation

## Support

For issues or questions:

- **Repository**: https://github.com/saorsa-labs/communitas
- **Issues**: File GitHub issue with `[distributed-e2e]` tag
- **Contact**: david@saorsalabs.com
