#!/usr/bin/env node
import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const APP_URL = process.env.APP_URL ?? 'http://127.0.0.1:1420';
const ARTIFACT_DIR = path.resolve('mcp-artifacts/chrome-devtools');
fs.mkdirSync(ARTIFACT_DIR, { recursive: true });

function log(msg) {
  console.log(`[chrome-mcp] ${msg}`);
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
    const line = buffer.slice(0, idx).trim();
    buffer = buffer.slice(idx + 1);
    if (!line) continue;
    let message;
    try {
      message = JSON.parse(line);
    } catch (error) {
      log(`stdout: ${line}`);
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
      continue;
    }
    if (message.method) {
      log(`notification: ${JSON.stringify(message)}`);
    }
  }
});

function call(method, params = {}) {
  const id = nextId++;
  const payload = {
    jsonrpc: '2.0',
    id,
    method,
    params,
  };
  const json = JSON.stringify(payload);
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    proc.stdin.write(`${json}\n`);
  });
}

async function callTool(name, args = {}) {
  return call('tools/call', {
    name,
    arguments: args,
  });
}

function extractTextContent(result) {
  if (!result?.content) return '';
  for (const item of result.content) {
    if (item.type === 'text') {
      return item.text ?? '';
    }
  }
  return '';
}

(async () => {
  try {
    await call('initialize', {
      protocolVersion: '0.1.0',
      clientInfo: { name: 'CommunitasChromeMCP', version: '0.1.0' },
      capabilities: {},
    });
    log('Initialized MCP session');

    const tools = await call('tools/list');
    log(`Available tools: ${tools.tools.map((t) => t.name).join(', ')}`);

    await callTool('new_page', { url: APP_URL });
    log(`Opened page ${APP_URL}`);

    try {
      await callTool('wait_for', { text: 'Create Identity' });
      log('Found "Create Identity" text on page');
    } catch (err) {
      log(`wait_for("Create Identity") failed: ${err.message}`);
    }

    const snapshot = await callTool('take_snapshot');
    const snapshotText = extractTextContent(snapshot);
    const snapshotPath = path.join(ARTIFACT_DIR, 'initial_snapshot.txt');
    fs.writeFileSync(snapshotPath, snapshotText, 'utf8');
    log(`Saved initial snapshot to ${snapshotPath}`);

    const pageList = await callTool('list_pages');
    const pagesText = extractTextContent(pageList);
    const pagesPath = path.join(ARTIFACT_DIR, 'pages.txt');
    fs.writeFileSync(pagesPath, pagesText, 'utf8');
    log(`Saved pages info to ${pagesPath}`);

    const screenshot = await callTool('take_screenshot', { format: 'png' });
    const image = screenshot.content?.find((item) => item.type === 'image');
    if (image?.data) {
      const screenshotPath = path.join(ARTIFACT_DIR, 'initial.png');
      fs.writeFileSync(screenshotPath, Buffer.from(image.data, 'base64'));
      log(`Saved screenshot to ${screenshotPath}`);
    }
  } catch (error) {
    log(`Error: ${error.message}`);
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
