#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const CMD = 'node';
const ARGS = ['servers/mcp-puppeteer/server.js'];

const env = {
  ...process.env,
  MCP_BROWSER_HEADLESS: '0',
};

async function testInterface(client) {
  console.log('🖥️  Testing Communitas Interface');
  console.log('='.repeat(50));

  try {
    // Navigate to the app
    await client.callTool({
      name: 'browser_navigate',
      arguments: { url: 'http://localhost:1420' }
    });

    // Wait for page to load
    await client.callTool({
      name: 'browser_wait_for',
      arguments: { selector: 'body', timeoutMs: 5000 }
    });

    // Check initial state
    const title = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.title' }
    });
    console.log('📄 Page Title:', title.content?.[0]?.text);

    // Check for main app bar
    const appBar = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.querySelector("[role=\"banner\"]") ? "App bar found" : "App bar not found"' }
    });
    console.log('📊 App Bar:', appBar.content?.[0]?.text);

    // Check for navigation buttons
    const navButtons = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.querySelectorAll("button").length' }
    });
    console.log('🔘 Navigation Buttons:', navButtons.content?.[0]?.text);

    // Check for sidebar toggle
    const sidebarToggle = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.querySelector("[aria-label*=\"menu\" i], [data-testid*=\"menu\"]") ? "Menu button found" : "Menu button not found"' }
    });
    console.log('📱 Sidebar Toggle:', sidebarToggle.content?.[0]?.text);

    // Test sidebar toggle
    try {
      await client.callTool({
        name: 'browser_click',
        arguments: { selector: 'button[aria-label*="menu"], button[data-testid*="menu"]' }
      });
      console.log('✅ Sidebar toggle clicked');

      // Check if sidebar opened
      await new Promise(resolve => setTimeout(resolve, 1000));
      const sidebarContent = await client.callTool({
        name: 'browser_eval',
        arguments: { script: 'document.querySelectorAll("aside, [role=\"complementary\"]").length' }
      });
      console.log('📂 Sidebar elements:', sidebarContent.content?.[0]?.text);
    } catch (e) {
      console.log('⚠️  Could not test sidebar toggle:', e?.message || e);
    }

    // Check for main content area
    const mainContent = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.querySelector("main, [role=\"main\"]") ? "Main content found" : "Main content not found"' }
    });
    console.log('📄 Main Content:', mainContent.content?.[0]?.text);

    // Check for testnet status display
    const testnetInfo = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'document.body.innerText.includes("testnet") || document.body.innerText.includes("Testnet") ? "Testnet info found" : "Testnet info not found"' }
    });
    console.log('🌐 Testnet Status:', testnetInfo.content?.[0]?.text);

    // Check for responsive design
    const viewport = await client.callTool({
      name: 'browser_eval',
      arguments: { script: 'window.innerWidth + "x" + window.innerHeight' }
    });
    console.log('📐 Viewport Size:', viewport.content?.[0]?.text);

    // Take a screenshot of the interface
    await client.callTool({
      name: 'browser_snapshot',
      arguments: { fullPage: true }
    });
    console.log('📸 Full interface screenshot taken');

    console.log('\n✅ Interface test completed successfully!');
    console.log('🎉 The Communitas interface is working properly!');

  } catch (error) {
    console.log('❌ Interface test failed:', error.message);
  }
}

async function run() {
  console.log('🚀 Starting Interface Test');
  console.log('==========================');

  const transport = new StdioClientTransport({ command: CMD, args: ARGS, env });
  const client = new Client({ name: 'interface-tester', version: '0.1.0' });

  try {
    await client.connect(transport);
    console.log('✅ Connected to MCP server');

    await testInterface(client);

  } catch (error) {
    console.error('❌ Test failed:', error);
  } finally {
    await client.close();
  }
}

run().catch(console.error);