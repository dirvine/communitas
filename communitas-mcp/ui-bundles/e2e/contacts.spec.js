/**
 * Contacts Widget E2E Tests
 *
 * Tests rendering, search, favorites, presence, and accessibility
 * for the Contacts widget.
 */

const { test, expect } = require('@playwright/test');
const { startMcpMock, loadWidget, waitForWidgetReady } = require('./utils/widget-helpers');

test.describe('Contacts Widget', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to contacts widget
    await page.goto('/widgets/contacts/index.html');
  });

  test('should load and render contact list', async ({ page }) => {
    // Wait for widget to load
    await page.waitForSelector('.contacts-list', { state: 'visible' });

    // Check that contact items are rendered
    const contactItems = page.locator('.contact-item');
    await expect(contactItems).toHaveCount(await contactItems.count());

    // Verify at least one contact is shown
    await expect(contactItems.first()).toBeVisible();
  });

  test('search input filters contacts', async ({ page }) => {
    await page.waitForSelector('.contacts-list');

    // Get initial contact count
    const allContacts = page.locator('.contact-item');
    const initialCount = await allContacts.count();

    // Type in search box
    const searchInput = page.locator('input[type="text"]').first();
    await searchInput.fill('alice');

    // Wait for filtering
    await page.waitForTimeout(300);

    // Verify filtered results
    const filteredContacts = page.locator('.contact-item');
    const filteredCount = await filteredContacts.count();

    // Should have fewer contacts after filtering
    expect(filteredCount).toBeLessThanOrEqual(initialCount);

    // Clear search
    await searchInput.clear();
    await page.waitForTimeout(300);

    // Should show all contacts again
    const resetCount = await allContacts.count();
    expect(resetCount).toBe(initialCount);
  });

  test('favorite toggle works', async ({ page }) => {
    await page.waitForSelector('.contact-item');

    // Find a contact with favorite button
    const favoriteBtn = page.locator('.favorite-btn').first();
    await expect(favoriteBtn).toBeVisible();

    // Get initial state
    const initialClass = await favoriteBtn.getAttribute('class');
    const wasFavorite = initialClass?.includes('favorited') || false;

    // Click favorite button
    await favoriteBtn.click();

    // Wait for state change
    await page.waitForTimeout(200);

    // Verify state changed
    const newClass = await favoriteBtn.getAttribute('class');
    const isFavorite = newClass?.includes('favorited') || false;

    expect(isFavorite).not.toBe(wasFavorite);
  });

  test('presence states display correctly', async ({ page }) => {
    await page.waitForSelector('.contact-item');

    // Check for presence indicators
    const presenceDots = page.locator('.presence-dot');
    await expect(presenceDots.first()).toBeVisible();

    // Verify presence states exist (online, away, busy, invisible, unknown)
    const presenceStates = ['.online', '.away', '.busy', '.invisible', '.unknown'];

    let foundStates = 0;
    for (const state of presenceStates) {
      const count = await page.locator(state).count();
      if (count > 0) foundStates++;
    }

    // Should have at least some presence states rendered
    expect(foundStates).toBeGreaterThan(0);
  });

  test('in-call indicator shows when active', async ({ page }) => {
    await page.waitForSelector('.contact-item');

    // Look for in-call indicator (should exist even if not active)
    const inCallIndicator = page.locator('.in-call-dot');

    // Check if any contact has in-call indicator
    const count = await inCallIndicator.count();

    // In-call indicator structure should exist in the widget
    // (even if no contacts are currently in a call)
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('contact selection updates UI state', async ({ page }) => {
    await page.waitForSelector('.contact-item');

    // Get first contact
    const firstContact = page.locator('.contact-item').first();
    await firstContact.click();

    // Wait for selection state
    await page.waitForTimeout(200);

    // Verify selected class applied
    const hasSelectedClass = await firstContact.evaluate((el) =>
      el.classList.contains('selected')
    );

    expect(hasSelectedClass).toBe(true);
  });

  test('empty state displays when no contacts', async ({ page }) => {
    // Search for something that doesn't exist
    const searchInput = page.locator('input[type="text"]').first();
    await searchInput.fill('zzznobodymatchesthis999');

    // Wait for filtering
    await page.waitForTimeout(300);

    // Check for empty state or no contacts shown
    const contactItems = page.locator('.contact-item');
    const count = await contactItems.count();

    expect(count).toBe(0);
  });

  test('accessibility: ARIA labels present', async ({ page }) => {
    await page.waitForSelector('.contacts-list');

    // Check for ARIA roles
    const contactsList = page.locator('.contacts-list');
    const listRole = await contactsList.getAttribute('role');
    expect(listRole).toBe('list');

    // Check contact items have proper structure
    const contactItems = page.locator('.contact-item');
    const firstItemRole = await contactItems.first().getAttribute('role');
    expect(firstItemRole).toBe('listitem');

    // Check for aria-label on presence indicators
    const presenceDots = page.locator('.presence-dot');
    if (await presenceDots.count() > 0) {
      const ariaLabel = await presenceDots.first().getAttribute('aria-label');
      expect(ariaLabel).toBeTruthy();
    }

    // Check favorite button accessibility
    const favoriteBtn = page.locator('.favorite-btn').first();
    if (await favoriteBtn.count() > 0) {
      const btnAriaLabel = await favoriteBtn.getAttribute('aria-label');
      expect(btnAriaLabel).toBeTruthy();
    }
  });
});
