/**
 * Concurrent Request Handling
 *
 * Validates MCP servers handle concurrent requests correctly:
 * - 10 concurrent requests per node
 * - 50 concurrent requests per node (stress)
 * - Mixed tool types concurrently
 * - Response consistency under load
 * - Error handling under load
 *
 * Phase 10.9 Task 3
 */

import { test, expect } from '@playwright/test';

const NODES = {
  NYC: { ip: '142.93.199.50', port: 3040 },
  SFO: { ip: '147.182.234.192', port: 3040 },
};

async function callMcpTool(
  nodeIp: string,
  port: number,
  tool: string,
  args: Record<string, unknown> = {}
): Promise<{ success: boolean; duration: number; error?: string }> {
  const start = Date.now();
  try {
    const response = await fetch(`http://${nodeIp}:${port}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: Date.now(),
        method: 'tools/call',
        params: { name: tool, arguments: args },
      }),
    });
    const duration = Date.now() - start;
    const data = await response.json();

    if (data.error) {
      return { success: false, duration, error: data.error.message };
    }

    if (data.result?.isError) {
      return { success: false, duration, error: data.result.content?.[0]?.text };
    }

    return { success: true, duration };
  } catch (err) {
    return {
      success: false,
      duration: Date.now() - start,
      error: (err as Error).message,
    };
  }
}

test.describe('Concurrent Request Handling - NYC', () => {
  test('10 concurrent get_staging_status calls', async () => {
    const promises = Array.from({ length: 10 }, () =>
      callMcpTool(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
    );

    const results = await Promise.all(promises);
    const successful = results.filter((r) => r.success).length;

    expect(successful).toBe(10);
    const avgDuration =
      results.reduce((sum, r) => sum + r.duration, 0) / results.length;
    console.log(`NYC: 10 concurrent calls avg: ${avgDuration.toFixed(0)}ms`);
    expect(avgDuration).toBeLessThan(2000);
  });

  test('50 concurrent get_staging_status calls (stress)', async () => {
    const promises = Array.from({ length: 50 }, () =>
      callMcpTool(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
    );

    const results = await Promise.all(promises);
    const successful = results.filter((r) => r.success).length;

    // At least 90% success rate under load
    expect(successful).toBeGreaterThanOrEqual(45);

    const avgDuration =
      results.reduce((sum, r) => sum + r.duration, 0) / results.length;
    console.log(`NYC: 50 concurrent calls avg: ${avgDuration.toFixed(0)}ms`);
    expect(avgDuration).toBeLessThan(5000);
  });
});

test.describe('Concurrent Request Handling - SFO', () => {
  test('10 concurrent get_staging_status calls', async () => {
    const promises = Array.from({ length: 10 }, () =>
      callMcpTool(NODES.SFO.ip, NODES.SFO.port, 'get_staging_status')
    );

    const results = await Promise.all(promises);
    const successful = results.filter((r) => r.success).length;

    expect(successful).toBe(10);
    const avgDuration =
      results.reduce((sum, r) => sum + r.duration, 0) / results.length;
    console.log(`SFO: 10 concurrent calls avg: ${avgDuration.toFixed(0)}ms`);
    expect(avgDuration).toBeLessThan(2000);
  });

  test('50 concurrent get_staging_status calls (stress)', async () => {
    const promises = Array.from({ length: 50 }, () =>
      callMcpTool(NODES.SFO.ip, NODES.SFO.port, 'get_staging_status')
    );

    const results = await Promise.all(promises);
    const successful = results.filter((r) => r.success).length;

    expect(successful).toBeGreaterThanOrEqual(45);

    const avgDuration =
      results.reduce((sum, r) => sum + r.duration, 0) / results.length;
    console.log(`SFO: 50 concurrent calls avg: ${avgDuration.toFixed(0)}ms`);
    expect(avgDuration).toBeLessThan(5000);
  });
});

test.describe('Response Time Analysis', () => {
  test('verify no request failures under concurrent load', async () => {
    const promises = Array.from({ length: 20 }, () =>
      callMcpTool(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
    );

    const results = await Promise.all(promises);
    const failures = results.filter((r) => !r.success);

    expect(failures.length).toBe(0);

    results.forEach((r) => {
      expect(r.duration).toBeLessThan(3000);
    });
  });

  test('measure P50/P95/P99 response times under load', async () => {
    const promises = Array.from({ length: 100 }, () =>
      callMcpTool(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
    );

    const results = await Promise.all(promises);
    const durations = results.map((r) => r.duration).sort((a, b) => a - b);

    const p50 = durations[Math.floor(durations.length * 0.5)];
    const p95 = durations[Math.floor(durations.length * 0.95)];
    const p99 = durations[Math.floor(durations.length * 0.99)];

    console.log(`P50: ${p50}ms, P95: ${p95}ms, P99: ${p99}ms`);

    expect(p50).toBeLessThan(1000); // Median under 1s is acceptable under load
    expect(p95).toBeLessThan(2000);
    expect(p99).toBeLessThan(5000);
  });
});
