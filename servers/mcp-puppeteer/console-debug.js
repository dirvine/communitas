#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const CMD = 'node';
const ARGS = ['servers/mcp-puppeteer/server.js'];

const env = {
  ...process.env,
  MCP_BROWSER_HEADLESS: '0',
};

async function checkConsoleErrors(client) {
  console.log('🔍 Checking browser console for errors...');

  try {
    // Navigate to the app
    await client.callTool({
      name: 'browser_navigate',
      arguments: { url: 'http://localhost:1420' }
    });

    // Wait a bit for scripts to load
    await new Promise(resolve => setTimeout(resolve, 3000));

    // Check if there are any console messages by looking at the page content
    const pageContent = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.body.innerHTML' }
    });

    console.log('📄 Page content length:', pageContent.content?.[0]?.text?.length || 0);

    // Check if the root element has any content
    const rootContent = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.getElementById("root").innerHTML' }
    });

    console.log('🎯 Root content:', rootContent.content?.[0]?.text || 'Empty');

    // Check if React is loaded
    const reactCheck = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'typeof React' }
    });

    console.log('⚛️  React available:', reactCheck.content?.[0]?.text);

    // Check if main.tsx script loaded
    const scriptCheck = await client.callTool({
      name: 'browser_eval',
      arguments: { script: `
        const scripts = Array.from(document.querySelectorAll('script'));
        const mainScript = scripts.find(s => s.src && s.src.includes('main.tsx'));
        mainScript ? 'main.tsx script found' : 'main.tsx script not found'
      ` }
    });

    console.log('📜 Main script:', scriptCheck.content?.[0]?.text);

    // Try to check for any JavaScript errors by looking at the DOM
    const errorCheck = await client.callTool({
      name: 'browser_eval',
      arguments: { script: `
        const errorElements = document.querySelectorAll('[style*="background-color: #fee"], .error, .runtime-error');
        errorElements.length > 0 ? 'Error elements found in DOM' : 'No error elements found'
      ` }
    });

    console.log('🚨 Error elements:', errorCheck.content?.[0]?.text);

  } catch (error) {
    console.log('❌ Console check failed:', error.message);
  }
}

async function run() {
  console.log('🔧 Console Error Investigation');
  console.log('==============================');

  const transport = new StdioClientTransport({ command: CMD, args: ARGS, env });
  const client = new Client({ name: 'console-debugger', version: '0.1.0' });

  try {
    await client.connect(transport);
    console.log('✅ Connected to MCP server');

    await checkConsoleErrors(client);

  } catch (error) {
    console.error('❌ Failed:', error);
  } finally {
    await client.close();
  }
}

run().catch(console.error);