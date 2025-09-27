#!/usr/bin/env node
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const APP_URL = process.env.APP_URL ?? 'http://127.0.0.1:1420';
const ARTIFACT_DIR = path.resolve('mcp-artifacts/chrome-devtools');
fs.mkdirSync(ARTIFACT_DIR, { recursive: true });

function log(message) {
  console.log(`[flows] ${message}`);
}

function createMcpProcess() {
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
  return proc;
}

const proc = createMcpProcess();
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

function parseJsonFromContent(result) {
  const text = extractTextContent(result);
  const match = text.match(/```json\n([\s\S]*?)\n```/);
  if (match) {
    try {
      return JSON.parse(match[1]);
    } catch (err) {
      return null;
    }
  }
  return null;
}

async function takeSnapshot(label) {
  const response = await callTool('take_snapshot');
  const text = extractTextContent(response);
  const filePath = path.join(ARTIFACT_DIR, `snapshot-${label}.txt`);
  fs.writeFileSync(filePath, text ?? '', 'utf8');
  log(`Snapshot '${label}' saved to ${filePath}`);
  return text;
}

function findUid(snapshotText, { text, role, exact = false }) {
  if (!snapshotText) return null;
  const lines = snapshotText.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed.startsWith('uid=')) continue;
    if (role && !trimmed.includes(` ${role} `)) continue;
    if (text) {
      const quotedMatch = trimmed.match(/"([^"]*)"/);
      const label = quotedMatch ? quotedMatch[1] : '';
      const matches = exact
        ? label === text
        : label.toLowerCase().includes(text.toLowerCase());
      if (!matches) continue;
    }
    const uidMatch = trimmed.match(/uid=([^\s]+)/);
    if (uidMatch) {
      return uidMatch[1];
    }
  }
  return null;
}

async function run() {
  await call('initialize', {
    protocolVersion: '0.1.0',
    clientInfo: { name: 'CommunitasChromeFlows', version: '0.1.0' },
    capabilities: {},
  });
  log('Initialized session');

  const tools = await call('tools/list');
  log(`Tools: ${tools.tools.map((t) => t.name).join(', ')}`);

  await callTool('new_page', { url: APP_URL });
  log(`Navigated to ${APP_URL}`);

  await callTool('wait_for', { text: 'Communitas' }).catch(() => {});
  await callTool('wait_for', { text: 'SIGN IN' }).catch(() => {});

  const homeSnapshot = await takeSnapshot('home');

  const buttonListResponse = await callTool('evaluate_script', {
    function: `() => {
      return Array.from(document.querySelectorAll('button')).map(btn => ({
        text: (btn.innerText || '').trim(),
        ariaLabel: btn.getAttribute('aria-label'),
        role: btn.getAttribute('role'),
        id: btn.id || null,
      }));
    }`,
  });
  const buttonList = parseJsonFromContent(buttonListResponse) ?? [];
  const buttonListPath = path.join(ARTIFACT_DIR, 'buttons-home.json');
  fs.writeFileSync(buttonListPath, JSON.stringify(buttonList, null, 2));
  log(`Captured home buttons to ${buttonListPath}`);

  const signInUid = findUid(homeSnapshot, { text: 'SIGN IN', role: 'button', exact: true });
  if (!signInUid) {
    throw new Error('Could not find SIGN IN button');
  }
  await callTool('click', { uid: signInUid });
  await callTool('evaluate_script', {
    function: `() => {
      const buttons = Array.from(document.querySelectorAll('button'));
      const target = buttons.find(btn => (btn.innerText || '').trim() === 'SIGN IN');
      if (target) {
        target.click();
        return true;
      }
      return false;
    }`,
  });
  log('Clicked SIGN IN button');

  await new Promise((resolve) => setTimeout(resolve, 1000));
  await callTool('wait_for', { text: 'Create Identity' }).catch(() => {});
  await callTool('wait_for', { text: 'Sign In' }).catch(() => {});

  const consoleMessages = await callTool('list_console_messages', {});
  const consoleText = extractTextContent(consoleMessages);
  fs.writeFileSync(path.join(ARTIFACT_DIR, 'console-after-signin.txt'), consoleText ?? '', 'utf8');

  const dialogInfo = await callTool('evaluate_script', {
    function: `() => {
      const dialogs = Array.from(document.querySelectorAll('[role="dialog"], .MuiDialog-root'));
      return dialogs.map((dialog, index) => ({
        index,
        ariaLabel: dialog.getAttribute('aria-label') ?? null,
        text: dialog.textContent?.slice(0, 500) ?? '',
      }));
    }`,
  });
  const dialogData = parseJsonFromContent(dialogInfo);
  if (dialogData) {
    const infoPath = path.join(ARTIFACT_DIR, 'dialog-elements.json');
    fs.writeFileSync(infoPath, JSON.stringify(dialogData, null, 2));
    log(`Captured dialog metadata to ${infoPath}`);
  } else {
    log('No dialog metadata captured');
  }

  const dialogSnapshot = await takeSnapshot('login-dialog');

  const dialogScreenshot = await callTool('take_screenshot', { format: 'png' });
  const image = dialogScreenshot.content?.find((item) => item.type === 'image');
  if (image?.data) {
    const screenshotPath = path.join(ARTIFACT_DIR, 'login-dialog.png');
    fs.writeFileSync(screenshotPath, Buffer.from(image.data, 'base64'));
    log(`Saved login dialog screenshot to ${screenshotPath}`);
  }

  let createIdentityResult = { success: false, error: 'Sign Up tab not found' };

  const signUpUid = findUid(dialogSnapshot, { text: 'Sign Up', role: 'button' });
  if (signUpUid) {
    await callTool('click', { uid: signUpUid });
    log('Switched to Sign Up mode');

    await new Promise((resolve) => setTimeout(resolve, 1000));
    await callTool('wait_for', { text: 'Display Name' }).catch(() => {});

    const signupSnapshot = await takeSnapshot('signup-dialog');

    const displayNameUid = findUid(signupSnapshot, { text: 'Display Name', role: 'textbox' });
    const emailUid = findUid(signupSnapshot, { text: 'Email (Optional)', role: 'textbox' });
    const passwordUid = findUid(signupSnapshot, { text: 'Password', role: 'textbox' });
    const confirmUid = findUid(signupSnapshot, { text: 'Confirm Password', role: 'textbox' });

    if (displayNameUid && passwordUid && confirmUid) {
      await callTool('fill', { uid: displayNameUid, value: 'MCP Test User' });
      if (emailUid) {
        await callTool('fill', { uid: emailUid, value: 'mcp-test@example.com' });
      }
      await callTool('fill', { uid: passwordUid, value: 'TestPassword123!' });
        await callTool('fill', { uid: confirmUid, value: 'TestPassword123!' });
      log('Filled Sign Up form fields');

      await takeSnapshot('signup-filled');

      const createButtonUid =
        findUid(signupSnapshot, { text: 'Create Identity', role: 'button' }) ??
        findUid(signupSnapshot, { text: 'CREATE IDENTITY', role: 'button' });

      if (createButtonUid) {
        await callTool('click', { uid: createButtonUid });
        log('Clicked Create Identity button');

        await new Promise((resolve) => setTimeout(resolve, 5000));

        const postSnapshot = await takeSnapshot('signup-post-submit');

        const consoleAfterSubmit = await callTool('list_console_messages', {});
        const consoleAfterText = extractTextContent(consoleAfterSubmit);
        fs.writeFileSync(
          path.join(ARTIFACT_DIR, 'console-after-submit.txt'),
          consoleAfterText ?? '',
          'utf8'
        );

        const localStorageState = await callTool('evaluate_script', {
          function: `() => ({
            fourWords: localStorage.getItem('communitas-four-words'),
            userName: localStorage.getItem('communitas-user-name'),
            hasVault: localStorage.getItem('communitas-has-vault'),
            identity: localStorage.getItem('communitas-identity')
          })`,
        });
        const storageData = parseJsonFromContent(localStorageState);
        fs.writeFileSync(
          path.join(ARTIFACT_DIR, 'local-storage-after-submit.json'),
          JSON.stringify(storageData, null, 2)
        );

        if (storageData?.userName) {
          createIdentityResult = { success: true, error: null };
        } else {
          createIdentityResult = {
            success: false,
            error: 'Identity creation appears incomplete (localStorage missing communitas-user-name)',
          };
        }

        const cancelUid =
          findUid(postSnapshot, { text: 'Cancel', role: 'button' }) ??
          findUid(postSnapshot, { text: 'CANCEL', role: 'button' });
        if (cancelUid) {
          await callTool('click', { uid: cancelUid });
          await new Promise((resolve) => setTimeout(resolve, 500));
        }
      } else {
        createIdentityResult = { success: false, error: 'Create Identity button not found' };
      }
    } else {
      createIdentityResult = { success: false, error: 'Required Sign Up inputs not found' };
    }
  }

  fs.writeFileSync(
    path.join(ARTIFACT_DIR, 'results.json'),
    JSON.stringify(createIdentityResult, null, 2)
  );

  // Navigate to website panel route
  const websiteRoute = `${APP_URL}/dev/website`;
  await callTool('navigate_page', { url: websiteRoute });
  log(`Navigated to website panel route ${websiteRoute}`);

  await new Promise((resolve) => setTimeout(resolve, 2000));
  await callTool('wait_for', { text: 'Save' }).catch(() => {});

  const websiteSnapshot = await takeSnapshot('website-panel');
  const websiteScreenshot = await callTool('take_screenshot', { format: 'png' });
  const websiteImage = websiteScreenshot.content?.find((item) => item.type === 'image');
  if (websiteImage?.data) {
    const screenshotPath = path.join(ARTIFACT_DIR, 'website-panel.png');
    fs.writeFileSync(screenshotPath, Buffer.from(websiteImage.data, 'base64'));
    log(`Saved website panel screenshot to ${screenshotPath}`);
  }

  const websiteConsole = await callTool('list_console_messages', {});
  const websiteConsoleText = extractTextContent(websiteConsole);
  fs.writeFileSync(
    path.join(ARTIFACT_DIR, 'console-website.txt'),
    websiteConsoleText ?? '',
    'utf8'
  );

  // Attempt login with placeholder identity (expected to fail without backend)
  await callTool('navigate_page', { url: APP_URL });
  await new Promise((resolve) => setTimeout(resolve, 1000));
  await callTool('wait_for', { text: 'SIGN IN' }).catch(() => {});

  const homeSnapshotForLogin = await takeSnapshot('home-before-login');
  const loginButtonUid = findUid(homeSnapshotForLogin, { text: 'SIGN IN', role: 'button', exact: true });
  if (loginButtonUid) {
    await callTool('click', { uid: loginButtonUid });
    await new Promise((resolve) => setTimeout(resolve, 500));
    await callTool('wait_for', { text: 'Four-Word Address' }).catch(() => {});

    const loginSnapshot = await takeSnapshot('login-attempt-dialog');
    const addressUid = findUid(loginSnapshot, { text: 'Four-Word Address', role: 'textbox' });
    if (addressUid) {
      await callTool('fill', { uid: addressUid, value: 'brave-ocean-gentle-mountain' });
      const signInSubmitUid =
        findUid(loginSnapshot, { text: 'Sign In', role: 'button' }) ??
        findUid(loginSnapshot, { text: 'SIGN IN', role: 'button' });
      if (signInSubmitUid) {
        await callTool('click', { uid: signInSubmitUid });
        await new Promise((resolve) => setTimeout(resolve, 2000));
        const loginResultSnapshot = await takeSnapshot('login-attempt-result');
        const loginConsole = await callTool('list_console_messages', {});
        const loginConsoleText = extractTextContent(loginConsole);
        fs.writeFileSync(
          path.join(ARTIFACT_DIR, 'console-login-attempt.txt'),
          loginConsoleText ?? '',
          'utf8'
        );
      }
    }
  }
}

run()
  .catch((error) => {
    log(`Flow error: ${error.message}`);
    fs.writeFileSync(path.join(ARTIFACT_DIR, 'results.json'), JSON.stringify({ success: false, error: error.message }, null, 2));
  })
  .finally(async () => {
    try {
      proc.stdin.end();
    } catch {}
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
      } catch {
        finish();
        return;
      }
      setTimeout(() => {
        if (!resolved) {
          try {
            proc.kill('SIGKILL');
          } catch {}
          finish();
        }
      }, 1500);
    });
  })
  .then(() => process.exit(0));
