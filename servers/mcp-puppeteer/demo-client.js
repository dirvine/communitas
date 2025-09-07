#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const CMD = 'node';
const ARGS = ['servers/mcp-puppeteer/server.js'];

const env = {
  ...process.env,
  MCP_BROWSER_URL: process.env.MCP_BROWSER_URL || 'http://localhost:1420',
  MCP_BROWSER_HEADLESS: process.env.MCP_BROWSER_HEADLESS || '1',
};

async function run() {
  const transport = new StdioClientTransport({ command: CMD, args: ARGS, env });
  const client = new Client({ name: 'communitas-demo-client', version: '0.1.0' });
  await client.connect(transport);

  function printResult(title, res) {
    console.log(`\n== ${title} ==`);
    if (!res?.content) { console.log('No content'); return; }
    for (const c of res.content) {
      if (c.type === 'text') console.log(c.text);
      if (c.type === 'image') console.log(`[image ${c.mimeType}] ${c.data.substring(0, 40)}...`);
    }
  }

  const url = env.MCP_BROWSER_URL;
  let res;

  res = await client.callTool({ name: 'browser_navigate', arguments: { url } });
  printResult('Navigate', res);

  // Try to wait for any heading or body
  try {
    res = await client.callTool({ name: 'browser_wait_for', arguments: { selector: 'body', timeoutMs: 8000 } });
    printResult('Wait body', res);
  } catch (e) {
    console.log('wait_for body failed:', e?.message || e);
  }

  // Exercise app helpers if available
  try { printResult('app_setup_workspace', await client.callTool({ name: 'app_setup_workspace', arguments: {} })); } catch (e) { console.log('app_setup_workspace failed:', e?.message || e); }
  try { printResult('app_test_groups', await client.callTool({ name: 'app_test_groups', arguments: {} })); } catch (e) { console.log('app_test_groups failed:', e?.message || e); }
  try { printResult('app_test_group_messaging', await client.callTool({ name: 'app_test_group_messaging', arguments: {} })); } catch (e) { console.log('app_test_group_messaging failed:', e?.message || e); }

  // Snapshot the UI
  res = await client.callTool({ name: 'browser_snapshot', arguments: { fullPage: false } });
  printResult('Snapshot', res);

  await client.close();
}

run().catch((e) => { console.error('Demo failed:', e); process.exit(1); });

