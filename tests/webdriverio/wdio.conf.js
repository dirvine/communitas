import path from 'node:path';
import os from 'node:os';
import fs from 'node:fs';
import { spawn } from 'node:child_process';

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..', '..');
const defaultBinary = path.join(repoRoot, 'target', 'debug', 'communitas-dioxus');
const tauriBinary = process.env.TAURI_APP_BINARY ?? defaultBinary;
const driverBinary =
  process.env.TAURI_DRIVER_BIN ?? path.join(os.homedir(), '.cargo', 'bin', 'tauri-driver');
const driverPort = Number.parseInt(process.env.TAURI_DRIVER_PORT ?? '4444', 10);
const nativePort = Number.parseInt(process.env.TAURI_DRIVER_NATIVE_PORT ?? '4445', 10);
const logsDir = path.join(path.dirname(new URL(import.meta.url).pathname), 'logs');

// Ensure logs directory exists
if (!fs.existsSync(logsDir)) {
  fs.mkdirSync(logsDir, { recursive: true });
}

let driverProcess;

export const config = {
  hostname: '127.0.0.1',
  port: driverPort,
  path: '/',
  runner: 'local',
  specs: ['./specs/**/*.js'],
  maxInstances: 1,
  capabilities: [
    {
      browserName: process.env.WEBKIT_BROWSER ?? 'webkit',
      'tauri:options': {
        application: tauriBinary
      }
    }
  ],
  logLevel: 'info',
  bail: 0,
  baseUrl: 'tauri://localhost',
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 1,
  framework: 'mocha',
  reporters: [
    'spec',
    ['json', { outputDir: logsDir, outputFileFormat: () => 'results.json' }]
  ],
  mochaOpts: {
    timeout: 60000
  },
  outputDir: logsDir,
  screenshotPath: logsDir,
  afterTest: async function(test, context, { error }) {
    // Capture screenshot on test failure
    if (error) {
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const screenshotName = `${test.parent}-${test.title}-${timestamp}.png`.replace(/\s+/g, '_');
      const screenshotPath = path.join(logsDir, screenshotName);
      try {
        await browser.saveScreenshot(screenshotPath);
        console.log(`Screenshot saved: ${screenshotPath}`);
      } catch (e) {
        console.log(`Failed to save screenshot: ${e.message}`);
      }
    }
  },
  onPrepare() {
    driverProcess = spawn(
      driverBinary,
      ['--port', `${driverPort}`, '--native-port', `${nativePort}`],
      { stdio: 'inherit' }
    );
    process.on('exit', () => {
      if (driverProcess) {
        driverProcess.kill();
      }
    });
  },
  onComplete() {
    if (driverProcess) {
      driverProcess.kill();
      driverProcess = undefined;
    }
  }
};
