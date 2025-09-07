#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';

const BASE = process.env.MCP_HTTP_URL || `http://${process.env.MCP_HTTP_HOST || '127.0.0.1'}:${process.env.MCP_HTTP_PORT || 8899}`;
const APP_URL = process.env.MCP_BROWSER_URL || 'http://localhost:1420';

function show(title, res) {
  console.log(`\n== ${title} ==`);
  if (!res?.content) { console.log('No content'); return; }
  for (const c of res.content) {
    if (c.type === 'text') console.log(c.text);
  }
}

async function call(client, name, args = {}) {
  try {
    const res = await client.callTool({ name, arguments: args });
    show(name, res);
    return res;
  } catch (e) {
    console.log(name + ' failed:', e?.message || e);
    return null;
  }
}

async function snapshot(client, label) {
  try {
    const res = await client.callTool({ name: 'browser_snapshot', arguments: { fullPage: false } });
    show(`snapshot:${label}`, res);
  } catch (e) {
    console.log(`snapshot:${label} failed:`, e?.message || e);
  }
}

async function run() {
  console.log(`Connecting to MCP HTTP server at ${BASE}`);
  const transport = new StreamableHTTPClientTransport(BASE);
  const client = new Client({ name: 'communitas-suite', version: '0.2.0' });
  await client.connect(transport);

  await call(client, 'browser_navigate', { url: APP_URL });
  await call(client, 'browser_wait_for', { selector: 'body', timeoutMs: 10000 });

  // Ensure offline data exists
  await call(client, 'app_offline_full_flow', {});

  // Create Organization flow (if available)
  await call(client, 'browser_click_text', { text: 'Create Organization' });
  // Try dialog or panel; fill by label
  await call(client, 'browser_fill_by_label', { label: 'Organization Name', value: 'Test Org ' + new Date().toISOString().slice(11,19) });
  await call(client, 'browser_fill_by_label', { label: 'Description', value: 'Automated test org' });
  await call(client, 'browser_click_text', { text: 'Create Organization' });
  // Small wait and snapshot
  await call(client, 'browser_wait_for', { selector: 'body', timeoutMs: 1000 });
  await snapshot(client, 'org_created');

  // Tabs to exercise (best-effort)
  const tabs = ['Organization', 'Messages', 'Files', 'Documents', 'Network', 'Storage', 'Diagnostics', 'Calling', 'Website', 'Identity'];
  for (const tab of tabs) {
    await call(client, 'browser_click_text', { text: tab });
    await snapshot(client, tab.toLowerCase());
  }

  // Spot-check some texts for basic health
  await call(client, 'browser_query_all_texts', { selector: 'h6' });
  await call(client, 'browser_query_all_texts', { selector: 'button' });

  console.log('\nSuite complete. Snapshots saved to mcp-artifacts/.');
  await client.close();
}

run().catch((e) => { console.error('Suite failed:', e); process.exit(1); });
