#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';

const BASE = process.env.MCP_HTTP_URL || `http://${process.env.MCP_HTTP_HOST || '127.0.0.1'}:${process.env.MCP_HTTP_PORT || 8765}`;
const APP_URL = process.env.MCP_BROWSER_URL || 'http://localhost:1420';

async function run() {
  console.log(`Connecting to MCP HTTP server at ${BASE}`);
  const transport = new StreamableHTTPClientTransport(BASE);
  const client = new Client({ name: 'communitas-http-demo', version: '0.1.0' });
  await client.connect(transport);

  function show(title, res) {
    console.log(`\n== ${title} ==`);
    if (!res?.content) { console.log('No content'); return; }
    for (const c of res.content) {
      if (c.type === 'text') console.log(c.text);
      if (c.type === 'image') console.log(`[image ${c.mimeType}] ${String(c.data).substring(0, 40)}...`);
    }
  }

  try { show('navigate', await client.callTool({ name: 'browser_navigate', arguments: { url: APP_URL } })); } catch (e) { console.error('navigate failed:', e?.message || e); }
  try { show('wait body', await client.callTool({ name: 'browser_wait_for', arguments: { selector: 'body', timeoutMs: 8000 } })); } catch (e) { console.error('wait failed:', e?.message || e); }
  try { show('app_list_groups', await client.callTool({ name: 'app_list_groups', arguments: {} })); } catch (e) { console.error('app_list_groups failed:', e?.message || e); }
  try { show('snapshot', await client.callTool({ name: 'browser_snapshot', arguments: { fullPage: false } })); } catch (e) { console.error('snapshot failed:', e?.message || e); }

  await client.close();
}

run().catch((e) => { console.error('HTTP demo failed:', e); process.exit(1); });
