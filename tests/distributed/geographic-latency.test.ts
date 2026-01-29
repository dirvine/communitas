/**
 * Geographic Latency Baseline Tests
 *
 * Measures and establishes baseline latency expectations for MCP operations
 * across testnet nodes in different geographic regions.
 *
 * Phase 10.9 Task 2
 */

import { test, expect } from '@playwright/test';

// Testnet node configuration
const NODES = {
  'saorsa-2': { ip: '142.93.199.50', port: 3040, region: 'NYC' },
  'saorsa-3': { ip: '147.182.234.192', port: 3040, region: 'SFO' },
};

/**
 * Measure latency of an HTTP request
 */
async function measureLatency(
  url: string,
  options?: RequestInit
): Promise<number> {
  const start = performance.now();
  await fetch(url, options);
  return performance.now() - start;
}

/**
 * Calculate percentiles from an array of measurements
 */
function calculatePercentiles(values: number[]): {
  p50: number;
  p95: number;
  p99: number;
  min: number;
  max: number;
  avg: number;
} {
  const sorted = values.slice().sort((a, b) => a - b);
  const p50 = sorted[Math.floor(sorted.length * 0.5)];
  const p95 = sorted[Math.floor(sorted.length * 0.95)];
  const p99 = sorted[Math.floor(sorted.length * 0.99)];
  const min = sorted[0];
  const max = sorted[sorted.length - 1];
  const avg = values.reduce((a, b) => a + b, 0) / values.length;
  return { p50, p95, p99, min, max, avg };
}

test.describe('Health Endpoint Latency', () => {
  test('measure health endpoint response time for all nodes', async () => {
    const results: Record<string, number[]> = {};

    for (const [name, node] of Object.entries(NODES)) {
      results[name] = [];

      // Take 10 measurements
      for (let i = 0; i < 10; i++) {
        const latency = await measureLatency(`http://${node.ip}:${node.port}/health`);
        results[name].push(latency);
        await new Promise(resolve => setTimeout(resolve, 100)); // Small delay between requests
      }

      const stats = calculatePercentiles(results[name]);
      console.log(`${name} (${node.region}) health endpoint latency:`, {
        p50: `${stats.p50.toFixed(2)}ms`,
        p95: `${stats.p95.toFixed(2)}ms`,
        avg: `${stats.avg.toFixed(2)}ms`,
      });

      // Basic sanity check - should respond within 1 second
      expect(stats.p95).toBeLessThan(1000);
    }
  });
});

test.describe('MCP Tools List Latency', () => {
  test('measure tools/list call latency', async () => {
    const results: Record<string, number[]> = {};

    for (const [name, node] of Object.entries(NODES)) {
      results[name] = [];

      // Take 5 measurements
      for (let i = 0; i < 5; i++) {
        const latency = await measureLatency(`http://${node.ip}:${node.port}/mcp`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'tools/list',
            id: i + 1,
          }),
        });
        results[name].push(latency);
        await new Promise(resolve => setTimeout(resolve, 200));
      }

      const stats = calculatePercentiles(results[name]);
      console.log(`${name} (${node.region}) tools/list latency:`, {
        p50: `${stats.p50.toFixed(2)}ms`,
        p95: `${stats.p95.toFixed(2)}ms`,
        avg: `${stats.avg.toFixed(2)}ms`,
      });

      // tools/list should be reasonably fast (within 2 seconds)
      expect(stats.p95).toBeLessThan(2000);
    }
  });
});

test.describe('Simple Tool Call Latency', () => {
  test('measure create_identity latency', async () => {
    const results: Record<string, number[]> = {};

    for (const [name, node] of Object.entries(NODES)) {
      results[name] = [];

      // Take 3 measurements
      for (let i = 0; i < 3; i++) {
        const latency = await measureLatency(`http://${node.ip}:${node.port}/mcp`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'tools/call',
            params: {
              name: 'create_identity',
              arguments: {
                display_name: `latency-test-${Date.now()}`,
              },
            },
            id: i + 1,
          }),
        });
        results[name].push(latency);
        await new Promise(resolve => setTimeout(resolve, 500));
      }

      const stats = calculatePercentiles(results[name]);
      console.log(`${name} (${node.region}) create_identity latency:`, {
        p50: `${stats.p50.toFixed(2)}ms`,
        p95: `${stats.p95.toFixed(2)}ms`,
        avg: `${stats.avg.toFixed(2)}ms`,
      });

      // Identity creation is I/O intensive, allow up to 5 seconds
      expect(stats.p95).toBeLessThan(5000);
    }
  });
});

test.describe('Complex Tool Call Latency', () => {
  test('measure create_kanban_board latency', async () => {
    const results: Record<string, number[]> = {};

    for (const [name, node] of Object.entries(NODES)) {
      results[name] = [];

      // Take 3 measurements
      for (let i = 0; i < 3; i++) {
        const latency = await measureLatency(`http://${node.ip}:${node.port}/mcp`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'tools/call',
            params: {
              name: 'create_kanban_board',
              arguments: {
                entity_id: `latency-test-${Date.now()}`,
                name: `Latency Test Board ${i}`,
                description: 'Latency measurement test',
              },
            },
            id: i + 1,
          }),
        });
        results[name].push(latency);
        await new Promise(resolve => setTimeout(resolve, 500));
      }

      const stats = calculatePercentiles(results[name]);
      console.log(`${name} (${node.region}) create_kanban_board latency:`, {
        p50: `${stats.p50.toFixed(2)}ms`,
        p95: `${stats.p95.toFixed(2)}ms`,
        avg: `${stats.avg.toFixed(2)}ms`,
      });

      // Complex operations, allow up to 5 seconds
      expect(stats.p95).toBeLessThan(5000);
    }
  });
});

test.describe('Cross-Region Latency Comparison', () => {
  test('compare latency between NYC and SFO', async () => {
    const allLatencies: Record<string, number[]> = {};

    for (const [name, node] of Object.entries(NODES)) {
      allLatencies[name] = [];

      // Collect 10 samples for better comparison
      for (let i = 0; i < 10; i++) {
        const latency = await measureLatency(`http://${node.ip}:${node.port}/health`);
        allLatencies[name].push(latency);
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }

    // Calculate and compare statistics
    const nycStats = calculatePercentiles(allLatencies['saorsa-2']);
    const sfoStats = calculatePercentiles(allLatencies['saorsa-3']);

    console.log('\nCross-Region Latency Comparison:');
    console.log('NYC (saorsa-2):', {
      p50: `${nycStats.p50.toFixed(2)}ms`,
      p95: `${nycStats.p95.toFixed(2)}ms`,
      avg: `${nycStats.avg.toFixed(2)}ms`,
    });
    console.log('SFO (saorsa-3):', {
      p50: `${sfoStats.p50.toFixed(2)}ms`,
      p95: `${sfoStats.p95.toFixed(2)}ms`,
      avg: `${sfoStats.avg.toFixed(2)}ms`,
    });

    // Both regions should have reasonable latency (US-based)
    expect(nycStats.p95).toBeLessThan(500);
    expect(sfoStats.p95).toBeLessThan(500);

    // Log baseline for regression testing
    console.log('\nBaseline established for regression testing');
  });
});
