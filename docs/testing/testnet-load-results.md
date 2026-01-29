# Testnet Load Test Results

## Overview

Load testing results for MCP servers deployed on Saorsa Labs testnet infrastructure.

**Test Date**: 2026-01-29 (to be executed)
**Phase**: 10.8 - Testnet Deployment
**Nodes Tested**: saorsa-2 (NYC), saorsa-3 (SFO)
**Test Framework**: Playwright

## Test Configuration

### Load Levels

| Level | Concurrent Requests | Target Success Rate | Target P99 Latency |
|-------|-------------------|-------------------|------------------|
| Low | 10 | 100% | <1000ms |
| Medium | 50 | 100% | <2000ms |
| High | 100 | >95% | <5000ms |

### Test Nodes

- **saorsa-2**: 142.93.199.50:3040 (DigitalOcean NYC1)
- **saorsa-3**: 147.182.234.192:3040 (DigitalOcean SFO3)

Note: saorsa-7 (Hetzner Nuremberg) excluded due to firewall blocking port 3040.

## Test Results

### Results Summary

Results will be populated when tests are executed:

```bash
npm test tests/distributed/load-test.ts
```

### Expected Baseline

Based on initial deployment metrics:

| Node | Memory Initial | Memory Limit | CPU Initial | Startup Time |
|------|---------------|-------------|-------------|--------------|
| saorsa-2 | 3.4 MB | 512 MB | 21ms | <3s |
| saorsa-3 | 3.4 MB | 512 MB | 14ms | <3s |
| saorsa-7 | 3.5 MB | 512 MB | 22ms | <3s |

## Performance Metrics

### Low Load (10 Concurrent Requests)

| Node | Duration | Success | Failed | Avg (ms) | P95 (ms) | P99 (ms) |
|------|----------|---------|--------|----------|----------|----------|
| saorsa-2 | TBD | TBD | TBD | TBD | TBD | TBD |
| saorsa-3 | TBD | TBD | TBD | TBD | TBD | TBD |

### Medium Load (50 Concurrent Requests)

| Node | Duration | Success | Failed | Avg (ms) | P95 (ms) | P99 (ms) |
|------|----------|---------|--------|----------|----------|----------|
| saorsa-2 | TBD | TBD | TBD | TBD | TBD | TBD |
| saorsa-3 | TBD | TBD | TBD | TBD | TBD | TBD |

### High Load (100 Concurrent Requests)

| Node | Duration | Success | Failed | Avg (ms) | P95 (ms) | P99 (ms) |
|------|----------|---------|--------|----------|----------|----------|
| saorsa-2 | TBD | TBD | TBD | TBD | TBD | TBD |
| saorsa-3 | TBD | TBD | TBD | TBD | TBD | TBD |

## Resource Utilization

### Memory Usage During Load

```bash
# Check memory during tests
ssh root@<node-ip> 'systemctl show communitas-mcp-test --property=MemoryCurrent'
```

Results:

- **saorsa-2**: TBD
- **saorsa-3**: TBD

### CPU Usage During Load

Check system logs for CPU metrics:

```bash
journalctl -u communitas-mcp-test --since "5 minutes ago" | grep CPU
```

Results:

- **saorsa-2**: TBD
- **saorsa-3**: TBD

## Error Analysis

### Error Categories

| Error Type | saorsa-2 | saorsa-3 | Total |
|------------|----------|----------|-------|
| Timeouts | TBD | TBD | TBD |
| Connection Refused | TBD | TBD | TBD |
| HTTP 5xx | TBD | TBD | TBD |
| HTTP 4xx | TBD | TBD | TBD |

### Error Details

Detailed error logs will be captured during test execution.

## Acceptance Criteria

| Criterion | Target | Status |
|-----------|--------|--------|
| 100 concurrent requests without errors | >95% success | TBD |
| Memory usage under limit | <512MB | TBD |
| CPU usage reasonable | <100% | TBD |
| No request timeouts | 0 | TBD |
| P99 latency acceptable | <5000ms | TBD |

## Recommendations

Based on test results (to be filled after execution):

1. **Memory Optimization**: TBD
2. **Concurrency Tuning**: TBD
3. **Error Handling**: TBD
4. **Resource Limits**: TBD

## Test Execution

### Run All Load Tests

```bash
npm test tests/distributed/load-test.ts
```

### Run Specific Load Level

```bash
npm test tests/distributed/load-test.ts -g "10 Concurrent"
npm test tests/distributed/load-test.ts -g "50 Concurrent"
npm test tests/distributed/load-test.ts -g "100 Concurrent"
```

### Generate Report

```bash
npm test tests/distributed/load-test.ts -g "Comprehensive Load Report"
```

## Monitoring During Tests

### Real-time Service Monitoring

```bash
# Terminal 1: Monitor saorsa-2
ssh root@142.93.199.50 'journalctl -u communitas-mcp-test -f'

# Terminal 2: Monitor saorsa-3
ssh root@147.182.234.192 'journalctl -u communitas-mcp-test -f'

# Terminal 3: Run load tests
npm test tests/distributed/load-test.ts
```

### System Resource Monitoring

```bash
# Check memory usage periodically
watch -n 1 'ssh root@142.93.199.50 "systemctl show communitas-mcp-test --property=MemoryCurrent"'
```

## Next Steps

1. **Execute Load Tests**: Run tests and populate this document with results
2. **Analyze Performance**: Identify bottlenecks and optimization opportunities
3. **Document Findings**: Update this document with actual results
4. **Tune Configuration**: Adjust resource limits based on findings
5. **Retest**: Verify improvements with follow-up load tests

## References

- Load Test Script: `tests/distributed/load-test.ts`
- Deployment Documentation: `docs/testing/testnet-deployment.md`
- Testnet Status: `.planning/testnet-status.json`
- Phase 10.8 Plan: `.planning/PLAN-phase-10.8.md`
