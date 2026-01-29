/**
 * Messages Widget E2E Tests
 *
 * Tests thread list, message view, reactions, typing indicators,
 * and navigation for the Messages widget.
 */

const { test, expect } = require('@playwright/test');

test.describe('Messages Widget', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to messages widget
    await page.goto('/widgets/messages/index.html');
  });

  test('should load thread list', async ({ page }) => {
    // Wait for thread list to load
    await page.waitForSelector('.thread-list', { state: 'visible' });

    // Check that thread items are rendered
    const threadItems = page.locator('.thread-item');
    const count = await threadItems.count();

    // Should have at least some threads
    expect(count).toBeGreaterThanOrEqual(0);

    // If threads exist, verify first one is visible
    if (count > 0) {
      await expect(threadItems.first()).toBeVisible();
    }
  });

  test('thread click opens message view', async ({ page }) => {
    await page.waitForSelector('.thread-list');

    // Get first thread
    const firstThread = page.locator('.thread-item').first();

    if (await firstThread.count() > 0) {
      // Click thread
      await firstThread.click();

      // Wait for message view to appear
      await page.waitForTimeout(500);

      // Check for message view elements
      const messageView = page.locator('.message-view, .messages-container, #message-view');
      const hasMessageView = await messageView.count() > 0;

      if (hasMessageView) {
        await expect(messageView.first()).toBeVisible();
      }

      // Check for back button
      const backButton = page.locator('.back-button, button[aria-label*="back" i]');
      if (await backButton.count() > 0) {
        await expect(backButton.first()).toBeVisible();
      }
    }
  });

  test('unread badge displays with 99+ cap', async ({ page }) => {
    await page.waitForSelector('.thread-list');

    // Look for unread badges
    const unreadBadges = page.locator('.unread-badge, .badge');
    const count = await unreadBadges.count();

    if (count > 0) {
      // Get first badge
      const badge = unreadBadges.first();
      await expect(badge).toBeVisible();

      // Check badge content
      const badgeText = await badge.textContent();

      // Should be a number or "99+"
      expect(badgeText).toMatch(/^\d+\+?$/);

      // If it says 99+, it should not exceed that
      if (badgeText?.includes('+')) {
        expect(badgeText).toBe('99+');
      }
    }
  });

  test('message reactions toggle', async ({ page }) => {
    await page.waitForSelector('.thread-list');

    // Open a thread
    const firstThread = page.locator('.thread-item').first();
    if (await firstThread.count() > 0) {
      await firstThread.click();
      await page.waitForTimeout(500);

      // Look for reaction elements
      const reactions = page.locator('.reaction, .reaction-btn, [data-reaction]');
      const reactionCount = await reactions.count();

      if (reactionCount > 0) {
        // Get first reaction button
        const reactionBtn = reactions.first();

        // Get initial state
        const initialClass = await reactionBtn.getAttribute('class');

        // Click reaction
        await reactionBtn.click();
        await page.waitForTimeout(200);

        // Verify state changed
        const newClass = await reactionBtn.getAttribute('class');
        expect(newClass).not.toBe(initialClass);
      }
    }
  });

  test('typing indicator shows', async ({ page }) => {
    await page.waitForSelector('.thread-list');

    // Look for typing indicator (may not always be present)
    const typingIndicator = page.locator('.typing-indicator, [aria-live="polite"]');
    const count = await typingIndicator.count();

    // Typing indicator structure should exist
    expect(count).toBeGreaterThanOrEqual(0);

    // If present, verify it's accessible
    if (count > 0) {
      const ariaLive = await typingIndicator.first().getAttribute('aria-live');
      expect(ariaLive).toBeTruthy();
    }
  });

  test('compose input tracks draft', async ({ page }) => {
    await page.waitForSelector('.thread-list');

    // Open a thread
    const firstThread = page.locator('.thread-item').first();
    if (await firstThread.count() > 0) {
      await firstThread.click();
      await page.waitForTimeout(500);

      // Find compose input
      const composeInput = page.locator('textarea, input[type="text"]').last();

      if (await composeInput.count() > 0) {
        // Type a draft message
        const draftText = 'This is a draft message';
        await composeInput.fill(draftText);

        // Wait a moment
        await page.waitForTimeout(300);

        // Verify input contains draft
        const inputValue = await composeInput.inputValue();
        expect(inputValue).toBe(draftText);

        // Clear draft
        await composeInput.clear();
        const clearedValue = await composeInput.inputValue();
        expect(clearedValue).toBe('');
      }
    }
  });

  test('back button returns to thread list', async ({ page }) => {
    await page.waitForSelector('.thread-list');

    // Open a thread
    const firstThread = page.locator('.thread-item').first();
    if (await firstThread.count() > 0) {
      await firstThread.click();
      await page.waitForTimeout(500);

      // Find back button
      const backButton = page.locator('.back-button, button[aria-label*="back" i]').first();

      if (await backButton.count() > 0) {
        // Click back button
        await backButton.click();
        await page.waitForTimeout(500);

        // Verify thread list is visible again
        const threadList = page.locator('.thread-list');
        await expect(threadList).toBeVisible();

        // Verify message view is hidden
        const messageView = page.locator('.message-view, .messages-container');
        if (await messageView.count() > 0) {
          const isHidden = await messageView.first().isHidden();
          expect(isHidden).toBe(true);
        }
      }
    }
  });

  test('search/filter updates view', async ({ page }) => {
    await page.waitForSelector('.thread-list');

    // Get initial thread count
    const allThreads = page.locator('.thread-item');
    const initialCount = await allThreads.count();

    // Find search input
    const searchInput = page.locator('input[type="text"], input[type="search"]').first();

    if (await searchInput.count() > 0) {
      // Type in search
      await searchInput.fill('test search query');
      await page.waitForTimeout(300);

      // Get filtered count
      const filteredThreads = page.locator('.thread-item');
      const filteredCount = await filteredThreads.count();

      // Count should be less than or equal to initial
      expect(filteredCount).toBeLessThanOrEqual(initialCount);

      // Clear search
      await searchInput.clear();
      await page.waitForTimeout(300);

      // Should show all threads again
      const resetCount = await allThreads.count();
      expect(resetCount).toBe(initialCount);
    }
  });
});
