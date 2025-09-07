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
    console.log(`${name} failed:`, e?.message || e);
    return null;
  }
}

async function run() {
  console.log(`Connecting to MCP HTTP server at ${BASE}`);
  const transport = new StreamableHTTPClientTransport(BASE);
  const client = new Client({ name: 'communitas-identity', version: '0.1.0' });
  await client.connect(transport);

  await call(client, 'browser_navigate', { url: APP_URL });
  await call(client, 'browser_wait_for', { selector: 'body', timeoutMs: 10000 });

  // Try to create identity via header button
  let clicked = await call(client, 'browser_click_text', { text: 'Create Identity' });
  if (!clicked || JSON.stringify(clicked).includes('No element')) {
    // Fallback: open sign in, then choose create within dialog
    await call(client, 'browser_click_text', { text: 'Sign In' });
    await call(client, 'browser_click_text', { text: 'Create Identity' });
  }

  // Fill display name
  const name = `Test User ${new Date().toISOString().slice(11,19)}`;
  await call(client, 'browser_fill', { selector: 'input[placeholder="Your Name"]', value: name });

  // Submit create
  await call(client, 'browser_click_text', { text: 'Create Identity' });

  // Give it a moment and snapshot
  await call(client, 'browser_wait_for', { selector: 'body', timeoutMs: 3000 });
  await call(client, 'browser_click_text', { text: 'Identity' });
  await call(client, 'browser_wait_text', { text: 'Identity', timeoutMs: 3000 });
  await call(client, 'browser_snapshot', { fullPage: false });

  console.log('\nIdentity flow complete.');
  await client.close();
}

run().catch((e) => { console.error('Identity test failed:', e); process.exit(1); });

