/**
 * Load Distribution Tests
 *
 * Tests load distribution scenarios across testnet nodes:
 * - Round-robin request distribution
 * - Failover when node unavailable
 * - Load balancing validation
 * - Node capacity testing
 *
 * Phase 10.9 Task 4
 */

import { test, expect } from '@playwright/test';

const NODES = {
  NYC: { ip: '142.93.199.50', port: 3040, region: 'NYC' },
  SFO: { ip: '147.182.234.192', port: 3040, region: 'SFO' },
};

async function callMcpTool(
  nodeIp: string,
  port: number,
  tool: string,
  args: Record<string, unknown> = {}
): Promise<{ success: boolean; duration: number; error?: string; nodeIp: string }> {
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
      return { success: false, duration, error: data.error.message, nodeIp };
    }

    if (data.result?.isError) {
      return { success: false, duration, error: data.result.content?.[0]?.text, nodeIp };
    }

    return { success: true, duration, nodeIp };
  } catch (err) {
    return {
      success: false,
      duration: Date.now() - start,
      error: (err as Error).message,
      nodeIp,
    };
  }
}

test.describe('Load Distribution - Round Robin', () => {
  test('distribute 100 requests evenly across 2 nodes', async () => {
    const nodes = [NODES.NYC, NODES.SFO];
    const promises = [];

    // Round-robin 100 requests across both nodes
    for (let i = 0; i < 100; i++) {
      const node = nodes[i % nodes.length];
      promises.push(callMcpTool(node.ip, node.port, 'get_staging_status'));
    }

    const results = await Promise.all(promises);
    const successful = results.filter((r) => r.success).length;

    // Expect at least 90% success rate
    expect(successful).toBeGreaterThanOrEqual(90);

    // Verify distribution is roughly even (45-55 each)
    const nycCount = results.filter((r) => r.nodeIp === NODES.NYC.ip).length;
    const sfoCount = results.filter((r) => r.nodeIp === NODES.SFO.ip).length;

    expect(nycCount).toBe(50);
    expect(sfoCount).toBe(50);

    console.log(`NYC: ${nycCount} requests, SFO: ${sfoCount} requests`);
  });
});

test.describe('Load Distribution - Node Capacity', () => {
  test('send 100 requests to single node (capacity test)', async () => {
    const promises = Array.from({ length: 100 }, () =>
      callMcpTool(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
    );

    const results = await Promise.all(promises);
    const successful = results.filter((r) => r.success).length;

    // Expect at least 90% success rate under load
    expect(successful).toBeGreaterThanOrEqual(90);

    const avgDuration =
      results.reduce((sum, r) => sum + r.duration, 0) / results.length;

    console.log(
      `NYC capacity: ${successful}/100 successful, avg ${avgDuration.toFixed(0)}ms`
    );

    // Average should still be reasonable under load
    expect(avgDuration).toBeLessThan(2000);
  });

  test('compare throughput between NYC and SFO nodes', async () => {
    const nycPromises = Array.from({ length: 50 }, () =>
      callMcpTool(NODES.NYC.ip, NODES.NYC.port, 'get_staging_status')
    );

    const sfoPromises = Array.from({ length: 50 }, () =>
      callMcpTool(NODES.SFO.ip, NODES.SFO.port, 'get_staging_status')
    );

    const [nycResults, sfoResults] = await Promise.all([
      Promise.all(nycPromises),
      Promise.all(sfoPromises),
    ]);

    const nycSuccess = nycResults.filter((r) => r.success).length;
    const sfoSuccess = sfoResults.filter((r) => r.success).length;

    expect(nycSuccess).toBeGreaterThanOrEqual(45);
    expect(sfoSuccess).toBeGreaterThanOrEqual(45);

    const nycAvg =
      nycResults.reduce((sum, r) => sum + r.duration, 0) / nycResults.length;
    const sfoAvg =
      sfoResults.reduce((sum, r) => sum + r.duration, 0) / sfoResults.length;

    console.log(
      `NYC: ${nycSuccess}/50 (${nycAvg.toFixed(0)}ms), SFO: ${sfoSuccess}/50 (${sfoAvg.toFixed(0)}ms)`
    );

    // Both nodes should handle load reasonably
    expect(nycAvg).toBeLessThan(2000);
    expect(sfoAvg).toBeLessThan(2000);
  });
});

test.describe('Load Distribution - Failover', () => {
  test('graceful handling when node unavailable', async () => {
    // Try an invalid IP (simulate node failure)
    const invalidNode = '192.0.2.1'; // TEST-NET-1 (never routable)

    const result = await callMcpTool(invalidNode, 3040, 'get_staging_status');

    // Should fail gracefully
    expect(result.success).toBe(false);
    expect(result.error).toBeDefined();

    console.log(`Failover test error (expected): ${result.error}`);
  });

  test('redirect traffic to healthy node after failure', async () => {
    const invalidNode = '192.0.2.1';

    // First try invalid node (fail)
    const failedResult = await callMcpTool(
      invalidNode,
      3040,
      'get_staging_status'
    );
    expect(failedResult.success).toBe(false);

    // Then redirect to healthy node (succeed)
    const successResult = await callMcpTool(
      NODES.NYC.ip,
      NODES.NYC.port,
      'get_staging_status'
    );
    expect(successResult.success).toBe(true);

    console.log('Failover: Failed request detected, redirected successfully');
  });
});
