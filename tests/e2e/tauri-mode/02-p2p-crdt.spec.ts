import { test, expect } from '@playwright/test';
import { setupFakeMediaDevices } from '../../utils/tauri-helpers';

test.describe.serial('P2P and CRDT E2E', () => {
  test.beforeEach(async ({ page }) => {
    await setupFakeMediaDevices(page);
  });

  test('P2P presence advertisement', async ({ page, context }) => {
    await page.goto('tauri://localhost');
    // Claim identity
    await page.fill('[data-testid="four-words-input"]', 'ocean forest moon star');
    await page.click('[data-testid="claim-identity"]');
    await expect(page.locator('[data-testid="presence-status"]')).toContainText('Online');

    // Verify bootstrap connect (mock env for CI)
    await expect(page.locator('[data-testid="network-peers"]')).toHaveText(/1 peer/);
  });

  test('Channel creation and sync', async ({ page }) => {
    // Assume identity claimed
    await page.click('[data-testid="create-channel"]');
    await page.fill('[data-testid="channel-name"]', 'Test Channel');
    await page.click('[data-testid="create-channel-btn"]');
    await expect(page.locator('[data-testid="channel-list"]')).toContainText('Test Channel');

    // Sync: Wait for CRDT metadata
    await expect(page.locator('[data-testid="channel-metadata"]')).toContainText('Synced');
  });

  test('Message send/receive', async ({ page }) => {
    // Select channel
    await page.click('[data-testid="channel-Test Channel"]');
    await page.fill('[data-testid="message-input"]', 'Hello P2P!');
    await page.click('[data-testid="send-message"]');
    await expect(page.locator('[data-testid="messages"]')).toContainText('Hello P2P!');

    // Receive sim (mock peer message)
    await page.evaluate(() => window.dispatchEvent(new CustomEvent('message-received', { detail: { text: 'Echo back!' } })));
    await expect(page.locator('[data-testid="messages"]')).toContainText('Echo back!');
  });

  test('CRDT collaborative edit', async ({ page }) => {
    // Open doc
    await page.click('[data-testid="open-doc"]');
    await page.fill('[data-testid="doc-editor"]', 'Initial text');
    await page.click('[data-testid="save-doc"]');

    // Sim remote edit (yjs delta)
    await page.evaluate(() => {
      // Mock yjs apply update
      const doc = new Y.Doc();
      const text = doc.getText('shared');
      text.insert(0, 'Remote: ');
      y.applyUpdate(doc, new Uint8Array([/* mock delta */]));
    });
    await expect(page.locator('[data-testid="doc-content"]')).toContainText('Remote: Initial text');
  });

  test('Offline-online sync', async ({ page }) => {
    // Go offline
    await page.evaluate(() => navigator.onLine = false);
    await page.fill('[data-testid="message-input"]', 'Offline msg');
    await page.click('[data-testid="send-message"]'); // Queues
    await expect(page.locator('[data-testid="pending-sync"]')).toHaveCount(1);

    // Go online
    await page.evaluate(() => navigator.onLine = true);
    await expect(page.locator('[data-testid="pending-sync"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="messages"]')).toContainText('Offline msg');
  });
});