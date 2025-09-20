#!/usr/bin/env node
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

const CMD = 'node';
const ARGS = ['servers/mcp-puppeteer/server.js'];

const env = {
  ...process.env,
  MCP_BROWSER_HEADLESS: '0', // Make browser visible for debugging
};

async function debugBlankScreen(client, url) {
  console.log(`🔍 Debugging blank screen at ${url}`);
  console.log('='.repeat(60));

  try {
    // Navigate to the app
    const navResult = await client.callTool({
      name: 'browser_navigate',
      arguments: { url }
    });
    console.log('✅ Navigation result:', navResult.content?.[0]?.text || 'Success');

    // Wait for page to load
    try {
      await client.callTool({
        name: 'browser_wait_for',
        arguments: { selector: 'body', timeoutMs: 10000 }
      });
      console.log('✅ Page body loaded');
    } catch (e) {
      console.log('⚠️  Body load timeout:', e?.message || e);
    }

    // Check page title
    try {
      const titleResult = await client.callTool({
        name: 'browser_eval',
        arguments: { script: 'document.title' }
      });
      console.log('📄 Page title:', titleResult.content?.[0]?.text || 'Unknown');
    } catch (e) {
      console.log('❌ Could not get title:', e?.message || e);
    }

    // Check if body has content
    try {
      const bodyContent = await client.callTool({
        name: 'browser_eval',
        arguments: { script: 'document.body ? document.body.innerHTML.substring(0, 500) : "No body element"' }
      });
      console.log('📝 Body HTML preview:', bodyContent.content?.[0]?.text || 'No content');
    } catch (e) {
      console.log('❌ Could not get body content:', e?.message || e);
    }

    // Check for JavaScript errors in console
    try {
      const consoleErrors = await client.callTool({
        name: 'browser_eval',
        arguments: { script: `
          if (window.console && window.console.errors) {
            return window.console.errors.join('\\n');
          }
          return 'No console errors captured';
        ` }
      });
      console.log('🔍 Console errors:', consoleErrors.content?.[0]?.text || 'None found');
    } catch (e) {
      console.log('❌ Could not check console errors:', e?.message || e);
    }

    // Check for React root element
    try {
      const reactRoot = await client.callTool({
        name: 'browser_eval',
        arguments: { script: `
          const root = document.getElementById('root');
          if (root) {
            return 'React root found: ' + root.innerHTML.substring(0, 200);
          }
          return 'No React root element found';
        ` }
      });
      console.log('⚛️  React root check:', reactRoot.content?.[0]?.text || 'Not found');
    } catch (e) {
      console.log('❌ Could not check React root:', e?.message || e);
    }

    // Check for script loading errors
    try {
      const scriptErrors = await client.callTool({
        name: 'browser_eval',
        arguments: { script: `
          const scripts = Array.from(document.querySelectorAll('script'));
          const failedScripts = scripts.filter(s => s.src && (!s.onload && !s.onreadystatechange));
          return failedScripts.length > 0 ? 'Failed scripts: ' + failedScripts.map(s => s.src).join(', ') : 'All scripts loaded successfully';
        ` }
      });
      console.log('📜 Script loading check:', scriptErrors.content?.[0]?.text || 'OK');
    } catch (e) {
      console.log('❌ Could not check scripts:', e?.message || e);
    }

    // Check network requests
    try {
      const networkCheck = await client.callTool({
        name: 'browser_eval',
        arguments: { script: `
          if (window.performance && window.performance.getEntriesByType) {
            const resources = window.performance.getEntriesByType('resource');
            const failed = resources.filter(r => r.transferSize === 0 && r.decodedBodySize === 0);
            return failed.length > 0 ? 'Failed network requests: ' + failed.map(r => r.name).join(', ') : 'All network requests successful';
          }
          return 'Performance API not available';
        ` }
      });
      console.log('🌐 Network requests check:', networkCheck.content?.[0]?.text || 'OK');
    } catch (e) {
      console.log('❌ Could not check network:', e?.message || e);
    }

    // Check for common error patterns
    try {
      const errorPatterns = await client.callTool({
        name: 'browser_eval',
        arguments: { script: `
          const html = document.documentElement.innerHTML;
          const errors = [];
          if (html.includes('ERR_')) errors.push('Network errors detected');
          if (html.includes('ReferenceError')) errors.push('JavaScript reference errors');
          if (html.includes('TypeError')) errors.push('JavaScript type errors');
          if (html.includes('SyntaxError')) errors.push('JavaScript syntax errors');
          if (html.includes('Module not found')) errors.push('Module loading errors');
          if (html.includes('Cannot resolve module')) errors.push('Module resolution errors');
          return errors.length > 0 ? 'Detected errors: ' + errors.join(', ') : 'No common error patterns found';
        ` }
      });
      console.log('🚨 Error pattern check:', errorPatterns.content?.[0]?.text || 'No errors detected');
    } catch (e) {
      console.log('❌ Could not check error patterns:', e?.message || e);
    }

    // Take a screenshot for visual inspection
    try {
      await client.callTool({
        name: 'browser_snapshot',
        arguments: { fullPage: true }
      });
      console.log('📸 Screenshot taken for visual inspection');
    } catch (e) {
      console.log('❌ Screenshot failed:', e?.message || e);
    }

    // Check if the app is a React app and inspect React-specific issues
    try {
      const reactCheck = await client.callTool({
        name: 'browser_eval',
        arguments: { script: `
          if (window.React) {
            return 'React ' + window.React.version + ' detected';
          }
          if (window.__REACT_DEVTOOLS_GLOBAL_HOOK__) {
            return 'React DevTools detected';
          }
          return 'No React detected';
        ` }
      });
      console.log('⚛️  React detection:', reactCheck.content?.[0]?.text || 'Not detected');
    } catch (e) {
      console.log('❌ Could not check React:', e?.message || e);
    }

  } catch (error) {
    console.log('❌ Debug failed:', error.message);
  }
}

async function run() {
  console.log('🔧 Starting Blank Screen Diagnostic');
  console.log('=====================================');

  const transport = new StdioClientTransport({ command: CMD, args: ARGS, env });
  const client = new Client({ name: 'blank-screen-debugger', version: '0.1.0' });

  try {
    await client.connect(transport);
    console.log('✅ Connected to MCP Puppeteer server');

    // Check the main app URL
    await debugBlankScreen(client, 'http://localhost:1420');

    // Also check if there are any test pages
    console.log('\n🔍 Also checking test pages...');
    await debugBlankScreen(client, 'http://localhost:1420/test_app.html');
    await debugBlankScreen(client, 'http://localhost:1420/debug_app.html');

  } catch (error) {
    console.error('❌ Diagnostic failed:', error);
  } finally {
    await client.close();
  }
}

run().catch((e) => {
  console.error('💥 Script failed:', e);
  process.exit(1);
});