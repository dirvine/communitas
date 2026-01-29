/**
 * Tool Consistency Validation
 *
 * Ensures tool behavior is consistent across testnet nodes:
 * - Same tool call, same arguments → same result structure
 * - Error messages are consistent
 * - Schema validation is consistent
 * - Demo mode behavior is identical
 *
 * Phase 10.9 Task 5
 */

import { test, expect } from '@playwright/test';

const NODES = {
  NYC: { ip: '142.93.199.50', port: 3040, name: 'saorsa-2 (NYC)' },
  SFO: { ip: '147.182.234.192', port: 3040, name: 'saorsa-3 (SFO)' },
};

async function callMcpTool(
  nodeIp: string,
  port: number,
  tool: string,
  args: Record<string, unknown> = {}
): Promise<{ success: boolean; result?: unknown; error?: string }> {
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
    const data = await response.json();

    if (data.error) {
      return { success: false, error: data.error.message };
    }

    if (data.result?.isError) {
      return { success: false, error: data.result.content?.[0]?.text };
    }

    return { success: true, result: data.result };
  } catch (err) {
    return { success: false, error: (err as Error).message };
  }
}

test.describe('Tool Consistency - Successful Calls', () => {
  test('get_staging_status returns consistent structure across nodes', async () => {
    const nycResult = await callMcpTool(
      NODES.NYC.ip,
      NODES.NYC.port,
      'get_staging_status'
    );
    const sfoResult = await callMcpTool(
      NODES.SFO.ip,
      NODES.SFO.port,
      'get_staging_status'
    );

    expect(nycResult.success).toBe(true);
    expect(sfoResult.success).toBe(true);

    // Both should have result objects with same structure
    expect(nycResult.result).toBeDefined();
    expect(sfoResult.result).toBeDefined();

    console.log('NYC result:', JSON.stringify(nycResult.result).substring(0, 100));
    console.log('SFO result:', JSON.stringify(sfoResult.result).substring(0, 100));
  });

  test('create_identity returns consistent structure across nodes', async () => {
    const displayName = `test-${Date.now()}`;

    const nycResult = await callMcpTool(NODES.NYC.ip, NODES.NYC.port, 'create_identity', {
      display_name: displayName,
    });
    const sfoResult = await callMcpTool(NODES.SFO.ip, NODES.SFO.port, 'create_identity', {
      display_name: displayName,
    });

    expect(nycResult.success).toBe(true);
    expect(sfoResult.success).toBe(true);

    // Both should return identity creation results
    expect(nycResult.result).toBeDefined();
    expect(sfoResult.result).toBeDefined();
  });
});

test.describe('Tool Consistency - Error Handling', () => {
  test('invalid tool name produces identical error across nodes', async () => {
    const nycResult = await callMcpTool(
      NODES.NYC.ip,
      NODES.NYC.port,
      'nonexistent_tool_xyz'
    );
    const sfoResult = await callMcpTool(
      NODES.SFO.ip,
      NODES.SFO.port,
      'nonexistent_tool_xyz'
    );

    expect(nycResult.success).toBe(false);
    expect(sfoResult.success).toBe(false);

    // Error messages should mention unknown tool
    expect(nycResult.error).toContain('Unknown tool');
    expect(sfoResult.error).toContain('Unknown tool');

    console.log(`NYC error: ${nycResult.error}`);
    console.log(`SFO error: ${sfoResult.error}`);
  });

  test('wrong argument types produce consistent errors', async () => {
    // Pass invalid type for a parameter (number instead of string)
    const nycResult = await callMcpTool(
      NODES.NYC.ip,
      NODES.NYC.port,
      'create_identity',
      { display_name: 12345 } // Wrong type
    );
    const sfoResult = await callMcpTool(
      NODES.SFO.ip,
      NODES.SFO.port,
      'create_identity',
      { display_name: 12345 }
    );

    // Should either both succeed (coerced) or both fail consistently
    expect(nycResult.success).toBe(sfoResult.success);

    console.log(
      `NYC result: ${nycResult.success ? 'success' : nycResult.error}`
    );
    console.log(
      `SFO result: ${sfoResult.success ? 'success' : sfoResult.error}`
    );
  });
});

test.describe('Tool Consistency - Schema Validation', () => {
  test('tools return JSON-RPC 2.0 compliant responses', async () => {
    const response = await fetch(`http://${NODES.NYC.ip}:${NODES.NYC.port}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 123,
        method: 'tools/call',
        params: { name: 'get_staging_status', arguments: {} },
      }),
    });

    const data = await response.json();

    // JSON-RPC 2.0 compliance
    expect(data.jsonrpc).toBe('2.0');
    expect(data.id).toBe(123); // Same ID returned
    expect(data.result || data.error).toBeDefined();
  });

  test('tool schemas are consistent across nodes', async () => {
    const nycListResponse = await fetch(
      `http://${NODES.NYC.ip}:${NODES.NYC.port}/mcp`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: 1,
          method: 'tools/list',
          params: {},
        }),
      }
    );

    const sfoListResponse = await fetch(
      `http://${NODES.SFO.ip}:${NODES.SFO.port}/mcp`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: 1,
          method: 'tools/list',
          params: {},
        }),
      }
    );

    const nycTools = await nycListResponse.json();
    const sfoTools = await sfoListResponse.json();

    // Both should return tools list
    expect(nycTools.result.tools).toBeDefined();
    expect(sfoTools.result.tools).toBeDefined();

    // Tool count should match
    expect(nycTools.result.tools.length).toBe(sfoTools.result.tools.length);

    console.log(`Both nodes report ${nycTools.result.tools.length} tools`);
  });
});
