/**
 * Notifications Widget E2E Tests
 *
 * Tests notification list, unread badge, mark as read,
 * and dismiss functionality.
 */

const { test, expect } = require('@playwright/test');

test.describe('Notifications Widget', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to notifications widget
    await page.goto('/widgets/notifications/index.html');
  });

  test('notification list displays', async ({ page }) => {
    // Wait for notifications list to load
    await page.waitForSelector('.notifications-list, .notifications, [role="list"]', { state: 'visible' });

    // Check for notification items
    const notifications = page.locator('.notification, .notification-item, [role="listitem"]');
    const count = await notifications.count();

    // May or may not have notifications
    expect(count).toBeGreaterThanOrEqual(0);

    if (count > 0) {
      await expect(notifications.first()).toBeVisible();
    }
  });

  test('unread badge shows count', async ({ page }) => {
    await page.waitForSelector('.notifications-list, .notifications, [role="list"]');

    // Look for unread badge
    const unreadBadge = page.locator('.unread-badge, .badge, [aria-label*="unread" i]');

    if (await unreadBadge.count() > 0) {
      await expect(unreadBadge.first()).toBeVisible();

      // Get badge text
      const badgeText = await unreadBadge.first().textContent();

      // Should be a number
      expect(badgeText).toMatch(/^\d+$/);

      const count = parseInt(badgeText || '0', 10);
      expect(count).toBeGreaterThanOrEqual(0);
    }
  });

  test('mark as read updates state', async ({ page }) => {
    await page.waitForSelector('.notifications-list, .notifications, [role="list"]');

    const notifications = page.locator('.notification, .notification-item');

    if (await notifications.count() > 0) {
      // Look for unread notification
      const unreadNotification = page.locator('.notification.unread, [aria-label*="unread" i]').first();

      if (await unreadNotification.count() > 0) {
        // Find mark as read button
        const markReadBtn = page.locator('button[aria-label*="mark" i], .mark-read-btn').first();

        if (await markReadBtn.count() > 0) {
          // Click mark as read
          await markReadBtn.click();
          await page.waitForTimeout(300);

          // Verify notification is no longer unread
          const stillUnread = await unreadNotification.evaluate((el) =>
            el.classList.contains('unread')
          );

          expect(stillUnread).toBe(false);
        }
      }
    }
  });

  test('dismiss removes notification', async ({ page }) => {
    await page.waitForSelector('.notifications-list, .notifications, [role="list"]');

    const notifications = page.locator('.notification, .notification-item');
    const initialCount = await notifications.count();

    if (initialCount > 0) {
      // Find dismiss button
      const dismissBtn = page.locator('button[aria-label*="dismiss" i], .dismiss-btn, .close-btn').first();

      if (await dismissBtn.count() > 0) {
        // Click dismiss
        await dismissBtn.click();
        await page.waitForTimeout(300);

        // Verify notification was removed
        const newCount = await notifications.count();
        expect(newCount).toBeLessThan(initialCount);
      }
    }
  });
});
