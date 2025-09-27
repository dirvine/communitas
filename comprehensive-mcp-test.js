#!/usr/bin/env node
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const APP_URL = 'http://127.0.0.1:5003';
const ARTIFACT_DIR = path.resolve('mcp-artifacts/chrome-devtools');
fs.mkdirSync(ARTIFACT_DIR, { recursive: true });

function log(message) {
  console.log(`[comprehensive-test] ${message}`);
}

const proc = spawn('npx', ['chrome-devtools-mcp@latest', '--headless', '--isolated'], {
  stdio: ['pipe', 'pipe', 'pipe'],
});

proc.stderr.setEncoding('utf8');
proc.stderr.on('data', (chunk) => {
  chunk
    .split(/\r?\n/)
    .filter(Boolean)
    .forEach((line) => log(`stderr: ${line}`));
});

let buffer = '';
const pending = new Map();
let nextId = 1;

proc.stdout.setEncoding('utf8');
proc.stdout.on('data', (chunk) => {
  buffer += chunk;
  let idx;
  while ((idx = buffer.indexOf('\n')) >= 0) {
    const raw = buffer.slice(0, idx).trim();
    buffer = buffer.slice(idx + 1);
    if (!raw) continue;
    let message;
    try {
      message = JSON.parse(raw);
    } catch (error) {
      log(`stdout: ${raw}`);
      continue;
    }
    if (message.id && pending.has(message.id)) {
      const { resolve, reject } = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) {
        reject(new Error(message.error.message ?? 'Unknown MCP error'));
      } else {
        resolve(message.result);
      }
    } else if (message.method) {
      log(`notification: ${JSON.stringify(message)}`);
    }
  }
});

function call(method, params = {}) {
  const id = nextId++;
  const payload = { jsonrpc: '2.0', id, method, params };
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    proc.stdin.write(`${JSON.stringify(payload)}\n`);
  });
}

function callTool(name, args = {}) {
  return call('tools/call', { name, arguments: args });
}

function extractTextContent(result) {
  if (!result?.content) return '';
  for (const item of result.content) {
    if (item.type === 'text' && typeof item.text === 'string') {
      return item.text;
    }
  }
  return '';
}

function saveArtifact(filename, data) {
  const filepath = path.join(ARTIFACT_DIR, filename);
  fs.writeFileSync(filepath, data, 'utf8');
  log(`Saved artifact: ${filepath}`);
}

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

(async () => {
  const testResults = {
    navigation: null,
    screenshot: null,
    performance: null,
    console: null,
    network: null,
    react: null,
    authentication: null,
    theme: null,
    memory: null,
    errors: []
  };

  try {
    // Initialize MCP
    await call('initialize', {
      protocolVersion: '0.1.0',
      clientInfo: { name: 'ComprehensiveCommunitas', version: '1.0.0' },
      capabilities: {},
    });
    log('✅ Initialized MCP session');

    // List available tools
    const tools = await call('tools/list');
    log(`Available tools: ${tools.tools.map((t) => t.name).join(', ')}`);

    // Step 1: Navigate to application
    log('🔗 Step 1: Navigating to application...');
    await callTool('new_page', { url: APP_URL });
    await sleep(3000); // Wait for page load
    testResults.navigation = { success: true, url: APP_URL };
    log('✅ Navigation completed');

    // Step 2: Take screenshot
    log('📸 Step 2: Taking screenshot...');
    const screenshot = await callTool('take_screenshot', { format: 'png' });
    const image = screenshot.content?.find((item) => item.type === 'image');
    if (image?.data) {
      const screenshotPath = path.join(ARTIFACT_DIR, 'comprehensive-screenshot.png');
      fs.writeFileSync(screenshotPath, Buffer.from(image.data, 'base64'));
      testResults.screenshot = { success: true, path: screenshotPath };
      log('✅ Screenshot saved');
    }

    // Step 3: Check performance metrics
    log('⚡ Step 3: Checking performance metrics...');
    try {
      await callTool('performance_start_trace', { categories: ['devtools.timeline'] });
      await sleep(2000); // Let some activity happen
      const perfResult = await callTool('performance_stop_trace');
      const perfInsight = await callTool('performance_analyze_insight');

      testResults.performance = {
        success: true,
        insight: extractTextContent(perfInsight)
      };
      saveArtifact('performance-analysis.txt', testResults.performance.insight);
      log('✅ Performance metrics captured');
    } catch (err) {
      testResults.performance = { success: false, error: err.message };
      testResults.errors.push(`Performance: ${err.message}`);
    }

    // Step 4: Check console logs
    log('🔍 Step 4: Checking console logs...');
    try {
      const consoleLogs = await callTool('list_console_messages');
      const consoleText = extractTextContent(consoleLogs);
      testResults.console = {
        success: true,
        logs: consoleText
      };
      saveArtifact('console-logs.txt', consoleText);

      // Count errors
      const errorCount = (consoleText.match(/error/gi) || []).length;
      const warningCount = (consoleText.match(/warning/gi) || []).length;
      log(`📊 Found ${errorCount} errors and ${warningCount} warnings in console`);
    } catch (err) {
      testResults.console = { success: false, error: err.message };
    }

    // Step 5: Check network requests
    log('🌐 Step 5: Checking network requests...');
    try {
      const networkRequests = await callTool('list_network_requests');
      const networkText = extractTextContent(networkRequests);
      testResults.network = {
        success: true,
        requests: networkText
      };
      saveArtifact('network-requests.txt', networkText);
      log('✅ Network requests captured');
    } catch (err) {
      testResults.network = { success: false, error: err.message };
    }

    // Step 6: Verify React components
    log('⚛️ Step 6: Verifying React components...');
    try {
      const reactCheck = await callTool('evaluate_script', {
        expression: `
          JSON.stringify({
            hasReact: typeof React !== 'undefined',
            hasReactDom: typeof ReactDOM !== 'undefined',
            hasRoot: document.getElementById('root') !== null,
            title: document.title,
            bodyClasses: document.body.className,
            reactVersions: window.React ? React.version : 'undefined'
          })
        `
      });

      const reactInfo = JSON.parse(extractTextContent(reactCheck));
      testResults.react = {
        success: true,
        info: reactInfo
      };
      saveArtifact('react-info.json', JSON.stringify(reactInfo, null, 2));
      log(`✅ React check: ${reactInfo.hasRoot ? 'Root found' : 'No root'}, Title: "${reactInfo.title}"`);
    } catch (err) {
      testResults.react = { success: false, error: err.message };
    }

    // Step 7: Check authentication state
    log('🔐 Step 7: Checking authentication state...');
    try {
      const authCheck = await callTool('evaluate_script', {
        expression: `
          JSON.stringify({
            hasSignInButton: !!document.querySelector('[contains(text(), "SIGN IN")]'),
            hasUserProfile: !!document.querySelector('.user-profile'),
            isOfflineMode: document.body.textContent.includes('Offline'),
            hasCreateIdentity: document.body.textContent.includes('Create Identity'),
            currentUrl: window.location.href
          })
        `
      });

      const authInfo = JSON.parse(extractTextContent(authCheck));
      testResults.authentication = {
        success: true,
        state: authInfo
      };
      saveArtifact('auth-state.json', JSON.stringify(authInfo, null, 2));
      log(`✅ Auth state: Offline=${authInfo.isOfflineMode}, HasSignIn=${authInfo.hasSignInButton}`);
    } catch (err) {
      testResults.authentication = { success: false, error: err.message };
    }

    // Step 8: Test theme switching (if button exists)
    log('🎨 Step 8: Testing theme switching...');
    try {
      // Check if theme toggle exists
      const themeToggleCheck = await callTool('evaluate_script', {
        expression: `!!document.querySelector('[aria-label*="theme"], [title*="theme"], button[class*="theme"]')`
      });

      if (extractTextContent(themeToggleCheck) === 'true') {
        // Click theme toggle
        await callTool('click', { selector: '[aria-label*="theme"], [title*="theme"], button[class*="theme"]' });
        await sleep(1000);

        // Take screenshot after theme change
        const themeScreenshot = await callTool('take_screenshot', { format: 'png' });
        const themeImage = themeScreenshot.content?.find((item) => item.type === 'image');
        if (themeImage?.data) {
          const themePath = path.join(ARTIFACT_DIR, 'theme-switched.png');
          fs.writeFileSync(themePath, Buffer.from(themeImage.data, 'base64'));
        }

        testResults.theme = { success: true, toggled: true };
        log('✅ Theme switching tested');
      } else {
        testResults.theme = { success: true, toggled: false, reason: 'No theme toggle found' };
        log('ℹ️ No theme toggle button found');
      }
    } catch (err) {
      testResults.theme = { success: false, error: err.message };
    }

    // Step 9: Check memory usage
    log('💾 Step 9: Checking memory usage...');
    try {
      const memoryCheck = await callTool('evaluate_script', {
        expression: `
          JSON.stringify({
            usedJSHeapSize: performance.memory ? performance.memory.usedJSHeapSize : 'unavailable',
            totalJSHeapSize: performance.memory ? performance.memory.totalJSHeapSize : 'unavailable',
            jsHeapSizeLimit: performance.memory ? performance.memory.jsHeapSizeLimit : 'unavailable',
            timing: performance.timing ? {
              domContentLoaded: performance.timing.domContentLoadedEventEnd - performance.timing.navigationStart,
              loadComplete: performance.timing.loadEventEnd - performance.timing.navigationStart
            } : 'unavailable'
          })
        `
      });

      const memoryInfo = JSON.parse(extractTextContent(memoryCheck));
      testResults.memory = {
        success: true,
        usage: memoryInfo
      };
      saveArtifact('memory-usage.json', JSON.stringify(memoryInfo, null, 2));

      if (memoryInfo.usedJSHeapSize !== 'unavailable') {
        const usedMB = Math.round(memoryInfo.usedJSHeapSize / 1024 / 1024);
        log(`✅ Memory usage: ${usedMB}MB`);
      } else {
        log('ℹ️ Memory API not available');
      }
    } catch (err) {
      testResults.memory = { success: false, error: err.message };
    }

    // Step 10: Take final snapshot
    log('📋 Step 10: Taking final snapshot...');
    const finalSnapshot = await callTool('take_snapshot');
    const finalText = extractTextContent(finalSnapshot);
    saveArtifact('final-snapshot.txt', finalText);

    // Generate comprehensive report
    const report = `
# Communitas Chrome DevTools MCP Test Report
Generated: ${new Date().toISOString()}
URL: ${APP_URL}

## Test Results Summary
${Object.entries(testResults).map(([test, result]) =>
  `- ${test}: ${result?.success ? '✅ PASS' : '❌ FAIL'}`
).join('\n')}

## Detailed Results

### Navigation
${testResults.navigation?.success ? `✅ Successfully navigated to ${testResults.navigation.url}` : '❌ Navigation failed'}

### Screenshot
${testResults.screenshot?.success ? `✅ Screenshot saved to ${testResults.screenshot.path}` : '❌ Screenshot failed'}

### Performance
${testResults.performance?.success ?
  `✅ Performance metrics captured:\n${testResults.performance.insight}` :
  `❌ Performance check failed: ${testResults.performance?.error}`}

### Console Logs
${testResults.console?.success ?
  `✅ Console logs captured (see console-logs.txt)` :
  `❌ Console check failed: ${testResults.console?.error}`}

### Network Requests
${testResults.network?.success ?
  `✅ Network requests captured (see network-requests.txt)` :
  `❌ Network check failed: ${testResults.network?.error}`}

### React Components
${testResults.react?.success ?
  `✅ React verification:\n${JSON.stringify(testResults.react.info, null, 2)}` :
  `❌ React check failed: ${testResults.react?.error}`}

### Authentication State
${testResults.authentication?.success ?
  `✅ Authentication state:\n${JSON.stringify(testResults.authentication.state, null, 2)}` :
  `❌ Auth check failed: ${testResults.authentication?.error}`}

### Theme Switching
${testResults.theme?.success ?
  (testResults.theme.toggled ? '✅ Theme switching tested successfully' : `ℹ️ ${testResults.theme.reason}`) :
  `❌ Theme test failed: ${testResults.theme?.error}`}

### Memory Usage
${testResults.memory?.success ?
  `✅ Memory usage:\n${JSON.stringify(testResults.memory.usage, null, 2)}` :
  `❌ Memory check failed: ${testResults.memory?.error}`}

## Error Summary
${testResults.errors.length === 0 ? 'No errors detected' : testResults.errors.join('\n')}

## Artifacts Location
All test artifacts saved to: ${ARTIFACT_DIR}
`;

    saveArtifact('comprehensive-test-report.md', report);
    log('✅ Comprehensive test completed!');
    console.log(report);

  } catch (error) {
    log(`❌ Test suite failed: ${error.message}`);
    console.error(error);
  } finally {
    try {
      proc.stdin.end();
    } catch (err) {
      // ignore
    }
    await new Promise((resolve) => {
      let resolved = false;
      const finish = () => {
        if (!resolved) {
          resolved = true;
          resolve();
        }
      };
      proc.once('exit', finish);
      try {
        proc.kill('SIGTERM');
      } catch (err) {
        log(`stderr: failed to terminate MCP process: ${err}`);
        finish();
        return;
      }
      setTimeout(() => {
        if (!resolved) {
          try {
            proc.kill('SIGKILL');
          } catch (err) {
            log(`stderr: failed to SIGKILL MCP process: ${err}`);
          }
          finish();
        }
      }, 1500);
    });
  }
})()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });