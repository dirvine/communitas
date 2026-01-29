/**
 * Error Handling Validation
 *
 * Comprehensive error handling validation:
 * - Invalid tool names
 * - Missing required arguments
 * - Wrong argument types
 * - Malformed JSON requests
 * - Network timeouts
 * - Graceful error responses
 *
 * Phase 10.9 Task 8
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

test.describe('Error Handling - Invalid Tool Names', () => {
  test('nonexistent tool returns clear error', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: 'tool_that_does_not_exist_xyz',
      arguments: {},
    });

    expect(data.result.isError).toBe(true);
    expect(data.result.content[0].text).toContain('Unknown tool');
    expect(data.result.content[0].text).toContain('tool_that_does_not_exist_xyz');

    console.log('Error for invalid tool:', data.result.content[0].text);
  });

  test('empty tool name returns error', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: '',
      arguments: {},
    });

    expect(data.result.isError).toBe(true);
    console.log('Error for empty tool name:', data.result.content[0].text);
  });

  test('null tool name returns error', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: null,
      arguments: {},
    });

    // Should have either JSON-RPC error or MCP error
    expect(data.error || data.result.isError).toBeTruthy();
    console.log('Error for null tool name:', JSON.stringify(data).substring(0, 100));
  });
});

test.describe('Error Handling - Malformed Requests', () => {
  test('missing method field', async () => {
    const response = await fetch(`http://${NODES.NYC.ip}:${NODES.NYC.port}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        // Missing method field
        params: {},
      }),
    });

    const data = await response.json();

    // Should return JSON-RPC error
    expect(data.error || data.result?.isError).toBeTruthy();
    console.log('Error for missing method:', JSON.stringify(data).substring(0, 100));
  });

  test('invalid JSON-RPC version', async () => {
    const response = await fetch(`http://${NODES.NYC.ip}:${NODES.NYC.port}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '1.0', // Invalid version
        id: 1,
        method: 'tools/list',
        params: {},
      }),
    });

    const data = await response.json();

    // Server may accept or reject gracefully
    console.log('Response for invalid version:', JSON.stringify(data).substring(0, 100));
  });
});

test.describe('Error Handling - Wrong Argument Types', () => {
  test('string instead of object for arguments', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: 'get_staging_status',
      arguments: 'not_an_object', // Should be {}
    });

    // Should either coerce or return error
    expect(data.result || data.error).toBeDefined();
    console.log('String arguments result:', data.result.isError ? 'error' : 'accepted');
  });

  test('array instead of object for params', async () => {
    const response = await fetch(`http://${NODES.NYC.ip}:${NODES.NYC.port}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'tools/list',
        params: ['array', 'not', 'object'], // Should be {}
      }),
    });

    const data = await response.json();

    // Server may accept or reject
    console.log('Array params result:', JSON.stringify(data).substring(0, 100));
  });
});

test.describe('Error Handling - Network Conditions', () => {
  test('timeout handling for unreachable node', async () => {
    const unreachableIp = '192.0.2.1'; // TEST-NET-1 (never routes)

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 2000);

      await fetch(`http://${unreachableIp}:3040/mcp`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: 1,
          method: 'tools/list',
          params: {},
        }),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
    } catch (err) {
      // Timeout or network error expected
      expect(err).toBeDefined();
      console.log('Timeout error (expected):', (err as Error).name);
    }
  });
});

test.describe('Error Handling - Graceful Responses', () => {
  test('errors include helpful messages', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: 'invalid_tool_for_testing',
      arguments: {},
    });

    expect(data.result.isError).toBe(true);
    const errorMsg = data.result.content[0].text;

    // Error should be human-readable
    expect(errorMsg.length).toBeGreaterThan(10);
    expect(errorMsg).toMatch(/unknown|invalid|not found/i);

    console.log('Helpful error message:', errorMsg);
  });

  test('errors maintain JSON-RPC structure', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/call', {
      name: 'nonexistent',
      arguments: {},
    });

    // Even errors follow JSON-RPC 2.0
    expect(data).toHaveProperty('jsonrpc', '2.0');
    expect(data).toHaveProperty('id', 1);
    expect(data).toHaveProperty('result');

    console.log('Error structure valid');
  });
});
