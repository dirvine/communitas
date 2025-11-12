import { test, expect } from '@playwright/test';

test.describe('P2P Sync E2E', () => {
  test('offline message syncs on reconnect', async ({ page }) => {
    // GIVEN: App loaded, channel open
    await page.goto('tauri://localhost');
    await page.fill('[data-testid="four-words-input"]', 'ocean forest moon star');
    await page.click('[data-testid="claim-identity"]');
    await page.click('[data-testid="create-channel"]');
    await page.fill('[data-testid="channel-name"]', 'Sync Test');
    await page.click('[data-testid="create-channel-btn"]');

    // WHEN: Send offline message
    await page.evaluate(() => navigator.onLine = false); // Mock offline
    await page.fill('[data-testid="message-input"]', 'Offline msg');
    await page.click('[data-testid="send-message"]');
    await expect(page.locator('[data-testid="pending-sync"]')).toBeVisible(); // FAIL: No such locator yet

    // Reconnect
    await page.evaluate(() => navigator.onLine = true);
    await expect(page.locator('[data-testid="messages"]')).toContainText('Offline msg'); // FAIL
  });
});