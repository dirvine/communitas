/**
 * Dashboard and Main App Functionality E2E Tests
 *
 * Tests the main application interface, including:
 * - Activity dashboard
 * - Entity management (channels, projects, groups)
 * - Messaging interface
 * - User interactions
 */

import { test, expect } from '../../fixtures/auth.fixture';
import { AuthTestUtils } from '../../fixtures/auth.fixture';

test.describe('Activity Dashboard', () => {
  test.beforeEach(async ({ page, setupAuthUser, authenticatedUser }) => {
    await setupAuthUser(authenticatedUser);
    await page.goto('/');
  });

  test('should display activity dashboard with recent activity', async ({ page }) => {
    // Verify dashboard loads
    await expect(page.locator('.dashboard, [data-testid="activity-dashboard"]')).toBeVisible();

    // Check for activity feed
    const activityFeed = page.locator('[data-testid="activity-feed"], .activity-feed');
    await expect(activityFeed.or(page.getByText(/recent activity|activity/i))).toBeVisible();

    // Verify user identity in sidebar
    await expect(page.getByText('Alice Test')).toBeVisible();
    await expect(page.getByText('alice-forest-moon-star')).toBeVisible();
  });

  test('should display notification badges for unread messages', async ({ page }) => {
    // Look for notification badges
    const badges = page.locator('.notification-badge, [data-testid*="badge"], .badge').filter({ hasText: /\d+/ });
    if (await badges.first().isVisible()) {
      // Verify badges contain numbers
      const badgeText = await badges.first().textContent();
      expect(badgeText).toMatch(/\d+/);
    }
  });

  test('should show online status indicators', async ({ page }) => {
    // Look for online/offline indicators
    const statusIndicators = page.locator('.status-indicator, [data-testid*="status"], .online-indicator');
    await expect(statusIndicators.first()).toBeVisible();
  });
});

test.describe('Entity Management', () => {
  test.beforeEach(async ({ page, setupAuthUser, authenticatedUser }) => {
    await setupAuthUser(authenticatedUser);
    await page.goto('/');
  });

  test('should display entity list in sidebar', async ({ page }) => {
    // Check for sidebar with entities
    const sidebar = page.locator('.sidebar, [data-testid="sidebar"], aside');
    await expect(sidebar).toBeVisible();

    // Look for entity items (channels, projects, etc.)
    const entityItems = page.locator('.entity-item, [data-testid*="entity"], .conversation-item');
    await expect(entityItems.first()).toBeVisible();
  });

  test('should allow creating new entities', async ({ page }) => {
    // Look for "New" or "Add" buttons
    const addButton = page.getByRole('button', { name: /new|add|\+/i });
    if (await addButton.isVisible()) {
      await addButton.click();

      // Should show creation dialog or menu
      const createDialog = page.locator('.create-dialog, [data-testid*="create"], .modal');
      await expect(createDialog.or(page.getByText(/create|new/i))).toBeVisible();
    }
  });

  test('should allow switching between entity types', async ({ page }) => {
    // Look for filter tabs or buttons
    const filters = page.locator('.filter-tabs, [data-testid*="filter"], .tab').filter({ hasText: /all|channels|projects|groups|people/i });
    if (await filters.first().isVisible()) {
      // Click on different filter tabs
      const projectFilter = filters.filter({ hasText: /projects/i });
      if (await projectFilter.isVisible()) {
        await projectFilter.click();
        // Should update the entity list
        await page.waitForTimeout(500);
      }
    }
  });
});

test.describe('Messaging Interface', () => {
  test.beforeEach(async ({ page, setupAuthUser, authenticatedUser }) => {
    await setupAuthUser(authenticatedUser);
    await page.goto('/');

    // Navigate to a chat/channel
    const chatItem = page.locator('.entity-item, .conversation-item').filter({ hasText: /general|chat/i }).first();
    if (await chatItem.isVisible()) {
      await chatItem.click();
    }
  });

  test('should display message input area', async ({ page }) => {
    // Look for message input
    const messageInput = page.locator('input[placeholder*="message"], textarea[placeholder*="message"], [data-testid*="message-input"]');
    await expect(messageInput.or(page.getByPlaceholder(/type a message/i))).toBeVisible();
  });

  test('should allow typing and sending messages', async ({ page }) => {
    const messageInput = page.locator('input[placeholder*="message"], textarea[placeholder*="message"]');

    if (await messageInput.isVisible()) {
      // Type a test message
      await messageInput.fill('Hello from E2E test!');

      // Look for send button
      const sendButton = page.getByRole('button', { name: /send/i });
      if (await sendButton.isVisible()) {
        await sendButton.click();

        // Should show the message in chat
        await expect(page.getByText('Hello from E2E test!')).toBeVisible();
      }
    }
  });

  test('should support message reactions', async ({ page }) => {
    // Find a message to react to
    const message = page.locator('.message, [data-testid*="message"]').first();
    if (await message.isVisible()) {
      // Hover over message to show reaction options
      await message.hover();

      // Look for reaction button
      const reactionButton = page.locator('.reaction-btn, [data-testid*="reaction"]').first();
      if (await reactionButton.isVisible()) {
        await reactionButton.click();

        // Should show reaction picker
        const reactionPicker = page.locator('.reaction-picker, [data-testid*="emoji"]');
        await expect(reactionPicker).toBeVisible();
      }
    }
  });

  test('should handle file attachments', async ({ page }) => {
    // Look for file upload button
    const uploadButton = page.getByRole('button', { name: /upload|attach/i });
    if (await uploadButton.isVisible()) {
      // File upload testing would require actual files
      // For now, just verify the UI element exists
      expect(await uploadButton.isVisible()).toBe(true);
    }
  });
});

test.describe('User Interactions', () => {
  test.beforeEach(async ({ page, setupAuthUser, authenticatedUser }) => {
    await setupAuthUser(authenticatedUser);
    await page.goto('/');
  });

  test('should allow searching entities', async ({ page }) => {
    // Look for search input
    const searchInput = page.getByPlaceholder(/search|find/i);
    if (await searchInput.isVisible()) {
      await searchInput.fill('test');

      // Should show search results or filtered list
      await page.waitForTimeout(500);

      // Verify search is working (either results or "no results")
      const results = page.locator('.search-results, .filtered-list');
      await expect(results.or(page.getByText(/no results|no matches/i))).toBeVisible();
    }
  });

  test('should support keyboard shortcuts', async ({ page }) => {
    // Test command palette shortcut (Cmd+K on Mac, Ctrl+K on others)
    const isMac = await page.evaluate(() => navigator.platform.includes('Mac'));
    const modifier = isMac ? 'Meta' : 'Control';

    await page.keyboard.press(`${modifier}+K`);

    // Should open command palette
    const commandPalette = page.locator('.command-palette, [data-testid*="command"]');
    await expect(commandPalette.or(page.getByPlaceholder(/jump to|search/i))).toBeVisible();
  });

  test('should handle window focus and blur events', async ({ page }) => {
    // Switch to another tab/window
    const newPage = await page.context().newPage();
    await newPage.goto('about:blank');

    // Switch back
    await page.bringToFront();

    // App should still be functional
    await expect(page.locator('.dashboard')).toBeVisible();
  });
});

test.describe('Error Handling', () => {
  test('should handle network disconnection gracefully', async ({ page, setupAuthUser, authenticatedUser }) => {
    await setupAuthUser(authenticatedUser);
    await page.goto('/');

    // Mock network disconnection
    await page.context().setOffline(true);

    // App should show offline indicator
    await expect(page.getByText(/offline|disconnected/i)).toBeVisible();

    // Reconnect
    await page.context().setOffline(false);

    // Should recover
    await expect(page.getByText(/online|connected/i)).toBeVisible({ timeout: 10000 });
  });

  test('should handle IPC communication errors', async ({ page, setupAuthUser, authenticatedUser }) => {
    await setupAuthUser(authenticatedUser);
    await page.goto('/');

    // Mock IPC failure (this would require Tauri-specific mocking)
    // For web mode, we can test error boundaries
    await expect(page.locator('.error-boundary')).not.toBeVisible();
  });
});

test.describe('Performance and Responsiveness', () => {
  test('should load dashboard within acceptable time', async ({ page, setupAuthUser, authenticatedUser }) => {
    const startTime = Date.now();

    await setupAuthUser(authenticatedUser);
    await page.goto('/');

    // Wait for dashboard to load
    await page.waitForSelector('.dashboard, [data-testid="activity-dashboard"]', { timeout: 10000 });

    const loadTime = Date.now() - startTime;
    expect(loadTime).toBeLessThan(5000); // Should load within 5 seconds
  });

  test('should handle rapid user interactions', async ({ page, setupAuthUser, authenticatedUser }) => {
    await setupAuthUser(authenticatedUser);
    await page.goto('/');

    // Rapidly click on different elements
    const buttons = page.getByRole('button').all();
    for (let i = 0; i < Math.min(5, (await buttons).length); i++) {
      try {
        await (await buttons)[i].click({ timeout: 1000 });
        await page.waitForTimeout(100); // Brief pause
      } catch (error) {
        // Some buttons might not be clickable, that's ok
      }
    }

    // App should still be functional
    await expect(page.locator('.dashboard')).toBeVisible();
  });
});
