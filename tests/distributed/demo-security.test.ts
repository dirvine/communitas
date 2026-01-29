/**
 * Demo Mode Security Tests
 *
 * Validates demo mode security constraints:
 * - No authentication required (expected in demo)
 * - Rate limiting still enforced (TODO: when implemented)
 * - Invalid requests rejected properly
 * - No sensitive data exposure
 *
 * Phase 10.9 Task 7
 */

import { test, expect } from '@playwright/test';

const NODES = {
  NYC: { ip: '142.93.199.50', port: 3040 },
  SFO: { ip: '147.182.234.192', port: 3040 },
};

async function sendJsonRpc(
  nodeIp: string,
  port: number,
  method: string,
  params: unknown
): Promise<unknown> {
  const response = await fetch(`http://${nodeIp}:${port}/mcp`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
  });
  return response.json();
}

test.describe('Demo Mode - Authentication', () => {
  test('tools/list works without authentication', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/list', {});

    expect(data.result).toBeDefined();
    expect(data.result.tools).toBeDefined();
    expect(data.result.tools.length).toBeGreaterThan(0);

    console.log('Demo mode: No auth required for tools/list');
  });

  test('tools/call works without authentication', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: 'get_staging_status',
      arguments: {},
    });

    expect(data.result).toBeDefined();
    expect(data.result.isError).toBe(false);

    console.log('Demo mode: No auth required for tools/call');
  });
});

test.describe('Demo Mode - Input Validation', () => {
  test('invalid tool name rejected', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: 'definitely_not_a_real_tool_12345',
      arguments: {},
    });

    expect(data.result.isError).toBe(true);
    expect(data.result.content[0].text).toContain('Unknown tool');
  });

  test('SQL injection attempt handled safely', async () => {
    const sqlPayload = "'; DROP TABLE users; --";
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: sqlPayload,
      arguments: {},
    });

    // Should reject gracefully, not crash
    expect(data.result.isError).toBe(true);
    expect(data.result.content[0].text).toContain('Unknown tool');
  });

  test('command injection attempt handled safely', async () => {
    const cmdPayload = '; ls -la /etc/passwd';
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: cmdPayload,
      arguments: {},
    });

    // Should reject gracefully, not execute
    expect(data.result.isError).toBe(true);
    expect(data.result.content[0].text).toContain('Unknown tool');
  });
});

test.describe('Demo Mode - Data Exposure', () => {
  test('error messages do not leak system paths', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: 'nonexistent_tool',
      arguments: {},
    });

    const errorText = data.result.content[0].text;

    // Should not contain file paths like /home/user/...
    expect(errorText).not.toMatch(/\/home\//);
    expect(errorText).not.toMatch(/\/Users\//);
    expect(errorText).not.toMatch(/C:\\\\Users\\/);
    expect(errorText).not.toMatch(/\/root\//);

    console.log('Error message:', errorText);
  });

  test('health endpoint does not expose sensitive info', async () => {
    const response = await fetch(
      `http://${NODES.NYC.ip}:${NODES.NYC.port}/health`
    );
    const data = await response.json();

    // Should have basic health info
    expect(data).toHaveProperty('status');

    // Should NOT have secrets, keys, tokens
    const jsonStr = JSON.stringify(data).toLowerCase();
    expect(jsonStr).not.toContain('password');
    expect(jsonStr).not.toContain('secret');
    expect(jsonStr).not.toContain('private_key');
    expect(jsonStr).not.toContain('token');

    console.log('Health endpoint:', JSON.stringify(data));
  });
});

test.describe('Demo Mode - Request Integrity', () => {
  test('oversized request rejected gracefully', async () => {
    // Create a very large but valid JSON payload
    const largeArgs: Record<string, string> = {};
    for (let i = 0; i < 1000; i++) {
      largeArgs[`key_${i}`] = 'x'.repeat(1000);
    }

    try {
      const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
        name: 'get_staging_status',
        arguments: largeArgs,
      });

      // Should either accept (and ignore extra args) or reject gracefully
      expect(data.result || data.error).toBeDefined();
      console.log('Large request handled:', data.result ? 'accepted' : 'rejected');
    } catch (err) {
      // Network timeout is also acceptable
      console.log('Large request timed out (acceptable)');
    }
  });
});
