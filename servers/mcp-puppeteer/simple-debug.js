#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const CMD = 'node';
const ARGS = ['servers/mcp-puppeteer/server.js'];

const env = {
  ...process.env,
  MCP_BROWSER_HEADLESS: '0',
};

async function simpleDebug(client, url) {
  console.log(`🔍 Simple debug at ${url}`);
  console.log('='.repeat(40));

  try {
    // Navigate
    await client.callTool({
      name: 'browser_navigate',
      arguments: { url }
    });
    console.log('✅ Navigated');

    // Wait for body
    await client.callTool({
      name: 'browser_wait_for',
      arguments: { selector: 'body', timeoutMs: 5000 }
    });
    console.log('✅ Body loaded');

    // Check title
    const title = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.title' }
    });
    console.log('📄 Title:', title.content?.[0]?.text);

    // Check if root element exists
    const rootCheck = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.getElementById("root") ? "Root found" : "Root missing"' }
    });
    console.log('🎯 Root element:', rootCheck.content?.[0]?.text);

    // Check for script tags
    const scriptCheck = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.querySelectorAll("script").length' }
    });
    console.log('📜 Scripts found:', scriptCheck.content?.[0]?.text);

    // Check for any visible content
    const contentCheck = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.body.innerText.substring(0, 100)' }
    });
    console.log('📝 Visible text:', contentCheck.content?.[0]?.text);

    // Take screenshot
    await client.callTool({
      name: 'browser_snapshot',
      arguments: { fullPage: false }
    });
    console.log('📸 Screenshot taken');

  } catch (error) {
    console.log('❌ Debug failed:', error.message);
  }
}

async function run() {
  console.log('🔧 Simple Debug Session');
  console.log('=======================');

  const transport = new StdioClientTransport({ command: CMD, args: ARGS, env });
  const client = new Client({ name: 'simple-debugger', version: '0.1.0' });

  try {
    await client.connect(transport);
    console.log('✅ Connected to MCP server');

    await simpleDebug(client, 'http://localhost:1420');

  } catch (error) {
    console.error('❌ Failed:', error);
  } finally {
    await client.close();
  }
}

run().catch(console.error);