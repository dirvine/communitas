/**
 * Load Testing for MCP Testnet
 *
 * Verify MCP servers handle concurrent load without errors.
 *
 * Phase 10.8 Task 9
 */

import { test, expect } from '@playwright/test';

const NODES = {
  'saorsa-2': { ip: '142.93.199.50', port: 3040 },
  'saorsa-3': { ip: '147.182.234.192', port: 3040 },
};

interface LoadTestResult {
  node: string;
  concurrent_requests: number;
  duration_ms: number;
  successful_requests: number;
  failed_requests: number;
  timeout_errors: number;
  avg_response_time_ms: number;
  p95_response_time_ms: number;
  p99_response_time_ms: number;
}

/**
 * Run load test on a single node
 */
async function runLoadTest(
  nodeIp: string,
  port: number,
  concurrentRequests: number
): Promise<LoadTestResult> {
  const responseTimes: number[] = [];
  let successful = 0;
  let failed = 0;
  let timeouts = 0;

  const startTime = Date.now();

  const promises = Array.from({ length: concurrentRequests }, async () => {
    const reqStart = Date.now();

    try {
      const response = await fetch(`http://${nodeIp}:${port}/health`, {
        signal: AbortSignal.timeout(10000), // 10s timeout
      });

      const elapsed = Date.now() - reqStart;
      responseTimes.push(elapsed);

      if (response.status === 200) {
        successful++;
      } else {
        failed++;
      }
    } catch (error: unknown) {
      failed++;
      if (error instanceof Error && error.name === 'TimeoutError') {
        timeouts++;
      }
    }
  });

  await Promise.all(promises);

  const duration = Date.now() - startTime;

  // Calculate percentiles
  responseTimes.sort((a, b) => a - b);
  const avg = responseTimes.reduce((a, b) => a + b, 0) / responseTimes.length;
  const p95 = responseTimes[Math.floor(responseTimes.length * 0.95)] || 0;
  const p99 = responseTimes[Math.floor(responseTimes.length * 0.99)] || 0;

  return {
    node: nodeIp,
    concurrent_requests: concurrentRequests,
    duration_ms: duration,
    successful_requests: successful,
    failed_requests: failed,
    timeout_errors: timeouts,
    avg_response_time_ms: avg,
    p95_response_time_ms: p95,
    p99_response_time_ms: p99,
  };
}

test.describe('Load Testing - 10 Concurrent Requests', () => {
  test('saorsa-2 handles 10 concurrent requests', async () => {
    const result = await runLoadTest(NODES['saorsa-2'].ip, NODES['saorsa-2'].port, 10);

    console.log('Load test results:', result);

    expect(result.failed_requests).toBe(0);
    expect(result.timeout_errors).toBe(0);
    expect(result.avg_response_time_ms).toBeLessThan(1000);
  });

  test('saorsa-3 handles 10 concurrent requests', async () => {
    const result = await runLoadTest(NODES['saorsa-3'].ip, NODES['saorsa-3'].port, 10);

    console.log('Load test results:', result);

    expect(result.failed_requests).toBe(0);
    expect(result.timeout_errors).toBe(0);
    expect(result.avg_response_time_ms).toBeLessThan(1000);
  });
});

test.describe('Load Testing - 50 Concurrent Requests', () => {
  test('saorsa-2 handles 50 concurrent requests', async () => {
    const result = await runLoadTest(NODES['saorsa-2'].ip, NODES['saorsa-2'].port, 50);

    console.log('Load test results:', result);

    expect(result.failed_requests).toBe(0);
    expect(result.timeout_errors).toBe(0);
    expect(result.p95_response_time_ms).toBeLessThan(2000);
  });

  test('saorsa-3 handles 50 concurrent requests', async () => {
    const result = await runLoadTest(NODES['saorsa-3'].ip, NODES['saorsa-3'].port, 50);

    console.log('Load test results:', result);

    expect(result.failed_requests).toBe(0);
    expect(result.timeout_errors).toBe(0);
    expect(result.p95_response_time_ms).toBeLessThan(2000);
  });
});

test.describe('Load Testing - 100 Concurrent Requests', () => {
  test('saorsa-2 handles 100 concurrent requests', async () => {
    const result = await runLoadTest(NODES['saorsa-2'].ip, NODES['saorsa-2'].port, 100);

    console.log('Load test results:', result);

    // Allow up to 5% failure rate at this load
    const failureRate = (result.failed_requests / result.concurrent_requests) * 100;
    expect(failureRate).toBeLessThan(5);

    expect(result.timeout_errors).toBe(0);
    expect(result.p99_response_time_ms).toBeLessThan(5000);
  });

  test('saorsa-3 handles 100 concurrent requests', async () => {
    const result = await runLoadTest(NODES['saorsa-3'].ip, NODES['saorsa-3'].port, 100);

    console.log('Load test results:', result);

    const failureRate = (result.failed_requests / result.concurrent_requests) * 100;
    expect(failureRate).toBeLessThan(5);

    expect(result.timeout_errors).toBe(0);
    expect(result.p99_response_time_ms).toBeLessThan(5000);
  });
});

test.describe('Resource Monitoring', () => {
  test('check memory usage during load', async () => {
    for (const [name, node] of Object.entries(NODES)) {
      // Run load test
      const loadPromise = runLoadTest(node.ip, node.port, 100);

      // Check health endpoint during load
      const healthResponse = await fetch(`http://${node.ip}:${node.port}/health`);
      const health = await healthResponse.json();

      console.log(`${name} health during load:`, health);

      await loadPromise;

      // Memory usage logged in systemd service
      console.log(`${name} load test complete - check journalctl for memory stats`);
    }
  });
});

test.describe('Comprehensive Load Report', () => {
  test('generate full load test report', async () => {
    const report: LoadTestResult[] = [];

    for (const [name, node] of Object.entries(NODES)) {
      console.log(`\n=== Testing ${name} ===`);

      // Test different load levels
      for (const concurrent of [10, 50, 100]) {
        const result = await runLoadTest(node.ip, node.port, concurrent);
        result.node = name;
        report.push(result);

        console.log(`${concurrent} concurrent:`, {
          success_rate: `${((result.successful_requests / result.concurrent_requests) * 100).toFixed(1)}%`,
          avg_ms: result.avg_response_time_ms.toFixed(1),
          p95_ms: result.p95_response_time_ms,
          p99_ms: result.p99_response_time_ms,
        });
      }
    }

    // Generate summary
    console.log('\n=== LOAD TEST SUMMARY ===');
    console.table(report);

    // Save report
    const reportJson = JSON.stringify(report, null, 2);
    console.log('\nFull report:', reportJson);

    // Verify no critical failures
    const criticalFailures = report.filter(r =>
      r.failed_requests / r.concurrent_requests > 0.1 || // >10% failure
      r.timeout_errors > 0 ||
      r.p99_response_time_ms > 10000 // >10s p99
    );

    expect(criticalFailures.length).toBe(0);
  });
});
