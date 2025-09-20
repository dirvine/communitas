#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const CMD = 'node';
const ARGS = ['servers/mcp-puppeteer/server.js'];

const env = {
  ...process.env,
  MCP_BROWSER_HEADLESS: '0',
};

async function testLoginFlow(client) {
  console.log('🔐 Testing Login & Identity Flow');
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

    console.log('✅ App loaded successfully');

    // Check for login/identity related elements
    const loginElements = await client.callTool({
      name: 'browser_eval',
      arguments: { script: `
        const elements = document.querySelectorAll('*');
        let loginFound = false;
        let identityFound = false;
        let authFound = false;

        for (let el of elements) {
          const text = el.textContent?.toLowerCase() || '';
          if (text.includes('login') || text.includes('sign in')) loginFound = true;
          if (text.includes('identity') || text.includes('four word')) identityFound = true;
          if (text.includes('auth') || text.includes('authenticate')) authFound = true;
        }

        return { loginFound, identityFound, authFound };
      ` }
    });

    const authStatus = JSON.parse(loginElements.content?.[0]?.text || '{}');
    console.log('🔍 Auth Elements Found:', authStatus);

    // Check for navigation to identity section
    try {
      // Look for identity/dashboard navigation
      const identityNav = await client.callTool({
        name: 'browser_eval',
        arguments: { script: `
          const buttons = Array.from(document.querySelectorAll('button'));
          const identityBtn = buttons.find(btn =>
            btn.textContent?.toLowerCase().includes('identity') ||
            btn.textContent?.toLowerCase().includes('dashboard')
          );
          return identityBtn ? 'Identity/Dashboard button found' : 'No identity navigation found';
        ` }
      });
      console.log('🧭 Identity Navigation:', identityNav.content?.[0]?.text);
    } catch (e) {
      console.log('⚠️  Could not check identity navigation');
    }

    // Test sidebar navigation to identity
    try {
      // Try to click menu button first
      await client.callTool({
        name: 'browser_click',
        arguments: { selector: 'button' }
      });
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Look for identity option in sidebar
      const sidebarIdentity = await client.callTool({
        name: 'browser_eval',
        arguments: { script: `
          const allText = document.body.textContent?.toLowerCase() || '';
          return allText.includes('identity') ? 'Identity section accessible' : 'Identity section not found';
        ` }
      });
      console.log('📂 Sidebar Identity Access:', sidebarIdentity.content?.[0]?.text);
    } catch (e) {
      console.log('⚠️  Could not test sidebar navigation');
    }

    // Check for testnet node information display
    const testnetDisplay = await client.callTool({
      name: 'browser_eval',
      arguments: { script: `
        const text = document.body.textContent || '';
        const hasNode1 = text.includes('philosophy-truth-prevent-wound') || text.includes('9000');
        const hasNode2 = text.includes('donna-jewish-scorpion-socrates') || text.includes('9010');
        const hasNode3 = text.includes('bike-in-porto-napkin') || text.includes('9020');
        return { hasNode1, hasNode2, hasNode3 };
      ` }
    });

    const nodes = JSON.parse(testnetDisplay.content?.[0]?.text || '{}');
    console.log('🌐 Testnet Nodes Displayed:', nodes);

    // Take screenshot of current state
    await client.callTool({
      name: 'browser_snapshot',
      arguments: { fullPage: true }
    });
    console.log('📸 Interface screenshot taken');

    console.log('\n✅ Login & Identity flow test completed!');
    console.log('🎯 Ready for login and identity feature development!');

  } catch (error) {
    console.log('❌ Login flow test failed:', error.message);
  }
}

async function run() {
  console.log('🚀 Starting Login & Identity Test');
  console.log('==================================');

  const transport = new StdioClientTransport({ command: CMD, args: ARGS, env });
  const client = new Client({ name: 'login-identity-tester', version: '0.1.0' });

  try {
    await client.connect(transport);
    console.log('✅ Connected to MCP server');

    await testLoginFlow(client);

  } catch (error) {
    console.error('❌ Test failed:', error);
  } finally {
    await client.close();
  }
}

run().catch(console.error);