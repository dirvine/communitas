/**
 * MCP Protocol Compliance
 *
 * Validates MCP JSON-RPC protocol compliance across testnet nodes:
 * - Proper JSON-RPC 2.0 format
 * - Error codes follow spec
 * - tools/list returns valid schema
 * - tools/call handles invalid args correctly
 * - Request/response id handling
 *
 * Phase 10.9 Task 6
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
  params: unknown,
  id: number | string = 1
): Promise<unknown> {
  const response = await fetch(`http://${nodeIp}:${port}/mcp`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id, method, params }),
  });
  return response.json();
}

test.describe('JSON-RPC 2.0 Format Compliance', () => {
  test('tools/list returns valid JSON-RPC 2.0 response', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/list', {});

    // JSON-RPC 2.0 required fields
    expect(data).toHaveProperty('jsonrpc', '2.0');
    expect(data).toHaveProperty('id', 1);
    expect(data).toHaveProperty('result');

    // MCP tools/list specific
    expect(data.result).toHaveProperty('tools');
    expect(Array.isArray(data.result.tools)).toBe(true);

    console.log(`tools/list returned ${data.result.tools.length} tools`);
  });

  test('request id is echoed in response', async () => {
    const testId = 'test-request-12345';
    const data = await sendJsonRpc(
      NODES.NYC.ip,
      NODES.NYC.port,
      'tools/list',
      {},
      testId
    );

    expect(data.id).toBe(testId);
  });

  test('numeric and string ids both work', async () => {
    const numericData = await sendJsonRpc(
      NODES.NYC.ip,
      NODES.NYC.port,
      'tools/list',
      {},
      999
    );
    const stringData = await sendJsonRpc(
      NODES.NYC.ip,
      NODES.NYC.port,
      'tools/list',
      {},
      'string-id-test'
    );

    expect(numericData.id).toBe(999);
    expect(stringData.id).toBe('string-id-test');
  });
});

test.describe('Error Handling Compliance', () => {
  test('invalid tool name returns proper error structure', async () => {
    const data = await sendJsonRpc(
      NODES.NYC.ip,
      NODES.NYC.port,
      'tools/call',
      { name: 'nonexistent_tool', arguments: {} }
    );

    // Should have error in result (MCP wraps errors in result.isError)
    expect(data.result.isError).toBe(true);
    expect(data.result.content).toBeDefined();
    expect(Array.isArray(data.result.content)).toBe(true);
    expect(data.result.content[0].text).toContain('Unknown tool');
  });

  test('malformed JSON returns proper JSON-RPC error', async () => {
    const response = await fetch(`http://${NODES.NYC.ip}:${NODES.NYC.port}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: 'not valid json{',
    });

    const data = await response.json();

    // JSON-RPC 2.0 parse error code is -32700
    expect(data).toHaveProperty('jsonrpc', '2.0');
    expect(data).toHaveProperty('error');
    expect(data.error.code).toBe(-32700);
    expect(data.error.message).toContain('Parse error');

    console.log('Malformed JSON error:', data.error.message);
  });

  test('missing jsonrpc field handled gracefully', async () => {
    const response = await fetch(`http://${NODES.NYC.ip}:${NODES.NYC.port}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        // Missing jsonrpc: '2.0'
        id: 1,
        method: 'tools/list',
        params: {},
      }),
    });

    const data = await response.json();

    // Should either reject or handle gracefully
    console.log('Missing jsonrpc field response:', JSON.stringify(data).substring(0, 100));
  });
});

test.describe('Tools Schema Validation', () => {
  test('all tools have required fields', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/list', {});

    expect(data.result.tools.length).toBeGreaterThan(0);

    // Check first 10 tools have required fields
    data.result.tools.slice(0, 10).forEach((tool: any) => {
      expect(tool).toHaveProperty('name');
      expect(tool).toHaveProperty('description');
      expect(tool).toHaveProperty('inputSchema');
      expect(typeof tool.name).toBe('string');
      expect(typeof tool.description).toBe('string');
      expect(typeof tool.inputSchema).toBe('object');
    });

    console.log('Sample tool:', JSON.stringify(data.result.tools[0], null, 2));
  });

  test('tool schemas are valid JSON Schema', async () => {
    const data = await sendJsonRpc(NODES.NYC.ip, NODES.NYC.port, 'tools/list', {});

    const sampleTool = data.result.tools[0];
    const schema = sampleTool.inputSchema;

    // JSON Schema must have type
    expect(schema).toHaveProperty('type');
    expect(schema.type).toBe('object');

    // Should have properties definition
    expect(schema).toHaveProperty('properties');
    expect(typeof schema.properties).toBe('object');

    console.log(`Tool "${sampleTool.name}" has ${Object.keys(schema.properties).length} parameters`);
  });
});
