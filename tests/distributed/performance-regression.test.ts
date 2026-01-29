/**
 * Performance Regression Suite
 *
 * Establishes performance baselines for regression testing:
 * - Tool execution time baselines
 * - Request throughput limits
 * - Response time percentiles
 * - Latency under load
 *
 * Phase 10.9 Task 9
 */

import { test, expect } from '@playwright/test';

const NODES = {
  NYC: { ip: '142.93.199.50', port: 3040 },
  SFO: { ip: '147.182.234.192', port: 3040 },
};

interface PerformanceMetrics {
  p50: number;
  p95: number;
  p99: number;
  min: number;
  max: number;
  avg: number;
  count: number;
}

async function measureLatency(
  nodeIp: string,
  port: number,
  tool: string,
  args: Record<string, unknown> = {}
): Promise<number> {
  const start = performance.now();
  await fetch(`http://${nodeIp}:${port}/mcp`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: Date.now(),
      method: 'tools/call',
      params: { name: tool, arguments: args },
    }),
  });
  return performance.now() - start;
}

function calculateMetrics(values: number[]): PerformanceMetrics {
  const sorted = values.slice().sort((a, b) => a - b);
  return {
    p50: sorted[Math.floor(sorted.length * 0.5)],
    p95: sorted[Math.floor(sorted.length * 0.95)],
    p99: sorted[Math.floor(sorted.length * 0.99)],
    min: sorted[0],
    max: sorted[sorted.length - 1],
    avg: values.reduce((a, b) => a + b, 0) / values.length,
    count: values.length,
  };
}

test.describe('Performance Baselines - Single Request', () => {
  test('get_staging_status single request baseline', async () => {
    const samples: number[] = [];

    for (let i = 0; i < 20; i++) {
      const latency = await measureLatency(
        NODES.NYC.ip,
        NODES.NYC.port,
        'get_staging_status'
      );
      samples.push(latency);
    }

    const metrics = calculateMetrics(samples);

    console.log(
      `get_staging_status: P50=${metrics.p50.toFixed(0)}ms, P95=${metrics.p95.toFixed(0)}ms, P99=${metrics.p99.toFixed(0)}ms`
    );

    // Baselines (adjust based on actual performance)
    expect(metrics.p50).toBeLessThan(1000);
    expect(metrics.p95).toBeLessThan(2000);
    expect(metrics.p99).toBeLessThan(3000);
  });

  test('create_identity single request baseline', async () => {
    const samples: number[] = [];

    for (let i = 0; i < 10; i++) {
      const latency = await measureLatency(
        NODES.NYC.ip,
        NODES.NYC.port,
        'create_identity',
        { display_name: `perf-test-${Date.now()}` }
      );
      samples.push(latency);
    }

    const metrics = calculateMetrics(samples);

    console.log(
      `create_identity: P50=${metrics.p50.toFixed(0)}ms, P95=${metrics.p95.toFixed(0)}ms, P99=${metrics.p99.toFixed(0)}ms`
    );

    // Identity creation may be slower (crypto operations)
    expect(metrics.p50).toBeLessThan(2000);
    expect(metrics.p95).toBeLessThan(5000);
  });
});

test.describe('Performance Baselines - Concurrent Load', () => {
  test('10 concurrent requests performance', async () => {
    const promises = Array.from({ length: 10 }, () =>
      measureLatency(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
    );

    const latencies = await Promise.all(promises);
    const metrics = calculateMetrics(latencies);

    console.log(
      `10 concurrent: P50=${metrics.p50.toFixed(0)}ms, P95=${metrics.p95.toFixed(0)}ms, Max=${metrics.max.toFixed(0)}ms`
    );

    // Under light concurrent load
    expect(metrics.p50).toBeLessThan(1500);
    expect(metrics.p95).toBeLessThan(3000);
  });

  test('50 concurrent requests performance', async () => {
    const promises = Array.from({ length: 50 }, () =>
      measureLatency(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
    );

    const latencies = await Promise.all(promises);
    const metrics = calculateMetrics(latencies);

    console.log(
      `50 concurrent: P50=${metrics.p50.toFixed(0)}ms, P95=${metrics.p95.toFixed(0)}ms, Max=${metrics.max.toFixed(0)}ms`
    );

    // Under moderate concurrent load
    expect(metrics.p50).toBeLessThan(2000);
    expect(metrics.p95).toBeLessThan(5000);
  });
});

test.describe('Performance Baselines - Throughput', () => {
  test('measure requests per second capacity', async () => {
    const duration = 5000; // 5 seconds
    const start = Date.now();
    let requestCount = 0;

    while (Date.now() - start < duration) {
      await measureLatency(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status');
      requestCount++;
    }

    const elapsed = Date.now() - start;
    const rps = (requestCount / elapsed) * 1000;

    console.log(`Throughput: ${rps.toFixed(1)} requests/second (${requestCount} requests in ${elapsed}ms)`);

    // Expect at least 1 request/second
    expect(rps).toBeGreaterThan(1);
  });
});

test.describe('Performance Baselines - Geographic Comparison', () => {
  test('compare NYC vs SFO latency', async () => {
    const nycSamples: number[] = [];
    const sfoSamples: number[] = [];

    for (let i = 0; i < 10; i++) {
      nycSamples.push(
        await measureLatency(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
      );
      sfoSamples.push(
        await measureLatency(NODES.SFO.ip, NODES.SFO.port, 'get_staging_status')
      );
    }

    const nycMetrics = calculateMetrics(nycSamples);
    const sfoMetrics = calculateMetrics(sfoSamples);

    console.log(
      `NYC: P50=${nycMetrics.p50.toFixed(0)}ms, SFO: P50=${sfoMetrics.p50.toFixed(0)}ms`
    );

    // Both should be reasonably fast
    expect(nycMetrics.p50).toBeLessThan(1000);
    expect(sfoMetrics.p50).toBeLessThan(1000);

    // Difference shouldn't be huge (both US)
    const diff = Math.abs(nycMetrics.p50 - sfoMetrics.p50);
    expect(diff).toBeLessThan(500);
  });
});

test.describe('Performance Baselines - Regression Detection', () => {
  test('establish baseline for CI comparison', async () => {
    const samples: number[] = [];

    // Collect 30 samples for statistical significance
    for (let i = 0; i < 30; i++) {
      samples.push(
        await measureLatency(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
      );
    }

    const metrics = calculateMetrics(samples);

    // Log baselines for CI to compare against
    console.log('=== BASELINE METRICS ===');
    console.log(JSON.stringify(metrics, null, 2));
    console.log('========================');

    // Sanity checks
    expect(metrics.p50).toBeGreaterThan(0);
    expect(metrics.p95).toBeGreaterThan(metrics.p50);
    expect(metrics.max).toBeGreaterThan(metrics.p95);
  });
});
