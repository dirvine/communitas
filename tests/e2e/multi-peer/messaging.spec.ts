/**
 * Multi-Peer E2E Tests
 *
 * Tests multi-peer messaging scenarios by spawning communitas-headless
 * nodes and verifying message delivery between UI and headless peers.
 *
 * Prerequisites:
 * - Run `npm run build` to build frontend
 * - Build headless: `cargo build --release -p communitas-headless`
 */

import { test, expect } from '@playwright/test';
import { spawn, ChildProcess } from 'child_process';
import { join } from 'path';
import { tmpdir } from 'os';
import { mkdirSync } from 'fs';

test.describe('Multi-Peer Messaging', () => {
  let headlessProcess: ChildProcess | null = null;
  let headlessPort: number;
  let headlessInstanceDir: string;

  test.beforeAll(async () => {
    // Create temp directory for headless instance
    headlessInstanceDir = join(tmpdir(), `communitas-test-${Date.now()}`);
    mkdirSync(headlessInstanceDir, { recursive: true });

    // Spawn headless node
    headlessPort = 9100 + Math.floor(Math.random() * 100);
    
    const headlessBinary = join(
      process.cwd(),
      'target',
      'release',
      'communitas-headless'
    );

    headlessProcess = spawn(headlessBinary, [
      '--instance-id',
      'test-peer',
      '--storage',
      join(headlessInstanceDir, 'storage'),
      '--config',
      join(headlessInstanceDir, 'config.toml'),
      '--port',
      headlessPort.toString(),
      '--metrics',
    ]);

    // Wait for headless to start
    await new Promise((resolve) => setTimeout(resolve, 3000));

    if (headlessProcess && !headlessProcess.killed) {
      console.log(`✅ Headless node started on port ${headlessPort}`);
    } else {
      throw new Error('Failed to start headless node');
    }
  });

  test.afterAll(async () => {
    // Cleanup headless process
    if (headlessProcess) {
      headlessProcess.kill();
      console.log('🧹 Headless node stopped');
    }
  });

  test('should send message from UI to headless peer', async ({ page }) => {
    // GIVEN: Tauri app is running and connected to headless peer
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('load');

    // Wait for Tauri API
    await page.waitForFunction(() => window.__TAURI__ !== undefined, {
      timeout: 10000,
    });

    // Create identity
    const claimButton = page.locator('button', { hasText: /claim.*identity/i });
    if (await claimButton.isVisible()) {
      await claimButton.click();
      await page.waitForTimeout(2000);
    }

    // WHEN: Send message to headless peer
    const messageInput = page.locator('textarea, input[placeholder*="message"]').first();
    await messageInput.fill('Hello from UI to headless');

    const sendButton = page.locator('button', { hasText: /send/i }).first();
    await sendButton.click();

    // THEN: Headless peer should receive message (check via metrics or logs)
    // For now, verify message appears in UI
    await expect(page.locator('text=Hello from UI to headless')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should receive message from headless peer', async ({ page }) => {
    // GIVEN: UI is connected
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('load');

    await page.waitForFunction(() => window.__TAURI__ !== undefined, {
      timeout: 10000,
    });

    // WHEN: Headless peer sends message (simulate via direct command or API call)
    // TODO: Implement headless message injection via HTTP/metrics API

    // THEN: UI should receive and display message
    // await expect(page.locator('text=Message from headless')).toBeVisible({ timeout: 10000 });
  });

  test('should sync messages after network partition', async ({ page }) => {
    // GIVEN: UI and headless both have messages
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('load');

    // Send messages from UI
    const messageInput = page.locator('textarea, input[placeholder*="message"]').first();
    await messageInput.fill('UI message 1');
    await page.locator('button', { hasText: /send/i }).first().click();
    await page.waitForTimeout(500);

    await messageInput.fill('UI message 2');
    await page.locator('button', { hasText: /send/i }).first().click();

    // WHEN: Network partition simulated (stop/start headless)
    if (headlessProcess) {
      headlessProcess.kill();
      await new Promise((resolve) => setTimeout(resolve, 1000));

      // Restart headless
      const headlessBinary = join(
        process.cwd(),
        'target',
        'release',
        'communitas-headless'
      );
      headlessProcess = spawn(headlessBinary, [
        '--instance-id',
        'test-peer',
        '--storage',
        join(headlessInstanceDir, 'storage'),
        '--config',
        join(headlessInstanceDir, 'config.toml'),
        '--port',
        headlessPort.toString(),
      ]);

      await new Promise((resolve) => setTimeout(resolve, 3000));
    }

    // THEN: Messages should sync after reconnection
    // TODO: Verify sync via message count or specific message visibility
  });

  test('should show offline indicator when headless is down', async ({ page }) => {
    // GIVEN: UI is running
    await page.goto('http://localhost:5173');
    await page.waitForLoadState('load');

    // WHEN: Headless node is killed
    if (headlessProcess) {
      headlessProcess.kill();
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    // THEN: UI should show offline indicator
    await expect(
      page.locator('[data-testid="offline-indicator"], text=/offline/i')
    ).toBeVisible({ timeout: 10000 });
  });

  test('should queue messages when offline', async ({ page }) => {
    // GIVEN: Headless is offline
    if (headlessProcess) {
      headlessProcess.kill();
      headlessProcess = null;
    }

    await page.goto('http://localhost:5173');
    await page.waitForLoadState('load');

    // WHEN: User sends message while offline
    const messageInput = page.locator('textarea, input[placeholder*="message"]').first();
    await messageInput.fill('Queued message');

    const sendButton = page.locator('button', { hasText: /send/i }).first();
    await sendButton.click();

    // THEN: Message should show as queued/pending
    await expect(
      page.locator('[data-testid="message-status"]', { hasText: /queued|pending/i })
    ).toBeVisible({ timeout: 5000 });

    // When network recovers, message should send
    // TODO: Restart headless and verify delivery
  });
});
