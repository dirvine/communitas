#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';

const BASE = process.env.MCP_HTTP_URL || `http://${process.env.MCP_HTTP_HOST || '127.0.0.1'}:${process.env.MCP_HTTP_PORT || 8765}`;
const APP_URL = process.env.MCP_BROWSER_URL || 'http://localhost:1420';
const OUTDIR = process.env.MCP_OUTDIR || 'mcp-artifacts';

fs.mkdirSync(OUTDIR, { recursive: true });

function print(title, res) {
  console.log(`\n== ${title} ==`);
  if (!res?.content) { console.log('No content'); return; }
  for (const c of res.content) {
    if (c.type === 'text') console.log(c.text);
    if (c.type === 'image') {
      const file = path.join(OUTDIR, `${title.replace(/[^a-z0-9-_]/gi,'_')}.png`);
      fs.writeFileSync(file, Buffer.from(String(c.data), 'base64'));
      console.log(`[image saved] ${file}`);
    }
  }
}

async function run() {
  console.log(`Connecting to MCP HTTP server at ${BASE}`);
  const transport = new StreamableHTTPClientTransport(BASE);
  const client = new Client({ name: 'communitas-full-check', version: '0.1.0' });
  await client.connect(transport);

  const calls = [
    ['navigate', { name: 'browser_navigate', args: { url: APP_URL } }],
    ['wait_body', { name: 'browser_wait_for', args: { selector: 'body', timeoutMs: 8000 } }],
    ['tabs_click_org', { name: 'app_click_tab', args: { tabName: 'Organization' } }],
    ['offline_full_flow', { name: 'app_offline_full_flow', args: {} }],
    ['list_groups', { name: 'app_list_groups', args: {} }],
    ['offline_stats', { name: 'app_offline_stats', args: {} }],
    ['snapshot_home', { name: 'browser_snapshot', args: { fullPage: false } }],
  ];

  for (const [label, spec] of calls) {
    try {
      const res = await client.callTool({ name: spec.name, arguments: spec.args });
      print(label, res);
    } catch (e) {
      console.log(`${label} failed:`, e?.message || e);
    }
  }

  await client.close();
  console.log(`\nArtifacts written to ${OUTDIR}`);
}

run().catch((e) => { console.error('Full check failed:', e); process.exit(1); });
