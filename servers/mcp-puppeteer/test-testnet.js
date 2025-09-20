#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const CMD = 'node';
const ARGS = ['servers/mcp-puppeteer/server.js'];

const env = {
  ...process.env,
  MCP_BROWSER_HEADLESS: '0', // Make browser visible
};

const testnetNodes = [
  { name: 'Node 1: philosophy-truth-prevent-wound', port: 9000, url: 'http://localhost:9000' },
  { name: 'Node 2: donna-jewish-scorpion-socrates', port: 9010, url: 'http://localhost:9010' },
  { name: 'Node 3: bike-in-porto-napkin', port: 9020, url: 'http://localhost:9020' },
  { name: 'Node 4: congratulate-twice-tonga-hurt', port: 9030, url: 'http://localhost:9030' },
  { name: 'Node 5: sponsor-biker-simon-leipzig', port: 9040, url: 'http://localhost:9040' },
];

async function testNode(client, node) {
  console.log(`\n🚀 Testing ${node.name} (${node.url})`);
  console.log('='.repeat(60));

  try {
    // Navigate to the node
    const navResult = await client.callTool({
      name: 'browser_navigate',
      arguments: { url: node.url }
    });
    console.log('✅ Navigation:', navResult.content?.[0]?.text || 'Success');

    // Wait for page to load
    try {
      const waitResult = await client.callTool({
        name: 'browser_wait_for',
        arguments: { selector: 'body', timeoutMs: 10000 }
      });
      console.log('✅ Page loaded:', waitResult.content?.[0]?.text || 'Success');
    } catch (e) {
      console.log('⚠️  Page load timeout:', e?.message || e);
    }

    // Check for Communitas app elements
    try {
      const titleResult = await client.callTool({
        name: 'browser_eval',
        arguments: { script: 'document.title' }
      });
      console.log('📄 Page title:', titleResult.content?.[0]?.text || 'Unknown');
    } catch (e) {
      console.log('❌ Could not get page title:', e?.message || e);
    }

    // Look for common app elements
    try {
      const bodyText = await client.callTool({
        name: 'browser_eval',
        arguments: { script: 'document.body ? document.body.innerText.substring(0, 200) : "No body"' }
      });
      console.log('📝 Page content preview:', bodyText.content?.[0]?.text || 'No content');
    } catch (e) {
      console.log('❌ Could not get page content:', e?.message || e);
    }

    // Try to take a screenshot
    try {
      await client.callTool({
        name: 'browser_snapshot',
        arguments: { fullPage: false }
      });
      console.log('📸 Screenshot taken successfully');
    } catch (e) {
      console.log('❌ Screenshot failed:', e?.message || e);
    }

    // Test Communitas app functions if available
    console.log('\n🔧 Testing Communitas app functions...');

    const appTests = [
      { name: 'app_test_identity', args: {} },
      { name: 'app_setup_workspace', args: {} },
      { name: 'app_test_groups', args: {} },
      { name: 'app_list_groups', args: {} },
    ];

    for (const test of appTests) {
      try {
        const result = await client.callTool({
          name: test.name,
          arguments: test.args
        });
        console.log(`✅ ${test.name}:`, result.content?.[0]?.text?.substring(0, 100) || 'Success');
      } catch (e) {
        console.log(`❌ ${test.name}:`, e?.message || e);
      }
    }

  } catch (error) {
    console.log('❌ Test failed:', error.message);
  }
}

async function run() {
  console.log('🌐 Starting Testnet Health Check with Visible Browser');
  console.log('==================================================');

  const transport = new StdioClientTransport({ command: CMD, args: ARGS, env });
  const client = new Client({ name: 'testnet-health-check', version: '0.1.0' });

  try {
    await client.connect(transport);
    console.log('✅ Connected to MCP Puppeteer server');

    for (const node of testnetNodes) {
      await testNode(client, node);

      // Brief pause between nodes
      await new Promise(resolve => setTimeout(resolve, 2000));
    }

    console.log('\n🎉 Testnet health check completed!');
    console.log('==================================================');

  } catch (error) {
    console.error('❌ Test failed:', error);
  } finally {
    await client.close();
  }
}

run().catch((e) => {
  console.error('💥 Script failed:', e);
  process.exit(1);
});