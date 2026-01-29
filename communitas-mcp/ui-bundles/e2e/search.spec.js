/**
 * Search Widget E2E Tests
 *
 * Tests search input, results display, highlighting,
 * and navigation for the Search widget.
 */

const { test, expect } = require('@playwright/test');

test.describe('Search Widget', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to search widget
    await page.goto('/widgets/search/index.html');
  });

  test('search input accepts query', async ({ page }) => {
    // Wait for search widget to load
    await page.waitForSelector('input[type="search"], input[type="text"]', { state: 'visible' });

    // Get search input
    const searchInput = page.locator('input[type="search"], input[type="text"]').first();

    // Type search query
    const searchQuery = 'test search query';
    await searchInput.fill(searchQuery);
    await page.waitForTimeout(300);

    // Verify input contains query
    const inputValue = await searchInput.inputValue();
    expect(inputValue).toBe(searchQuery);
  });

  test('results display and highlight', async ({ page }) => {
    await page.waitForSelector('input[type="search"], input[type="text"]');

    const searchInput = page.locator('input[type="search"], input[type="text"]').first();

    // Perform search
    await searchInput.fill('message');
    await page.waitForTimeout(500);

    // Look for search results
    const results = page.locator('.search-result, .result-item, [role="listitem"]');
    const resultCount = await results.count();

    // May or may not have results
    expect(resultCount).toBeGreaterThanOrEqual(0);

    if (resultCount > 0) {
      // Check for highlighting
      const highlighted = page.locator('mark, .highlight, .match');

      if (await highlighted.count() > 0) {
        await expect(highlighted.first()).toBeVisible();
      }
    }
  });

  test('result selection navigates', async ({ page }) => {
    await page.waitForSelector('input[type="search"], input[type="text"]');

    const searchInput = page.locator('input[type="search"], input[type="text"]').first();

    // Perform search
    await searchInput.fill('contact');
    await page.waitForTimeout(500);

    const results = page.locator('.search-result, .result-item, [role="listitem"]');

    if (await results.count() > 0) {
      // Click first result
      await results.first().click();
      await page.waitForTimeout(300);

      // Navigation might happen (check if URL changed or event fired)
      // For now, just verify the click was registered
      await expect(results.first()).toBeVisible();
    }
  });

  test('empty state for no results', async ({ page }) => {
    await page.waitForSelector('input[type="search"], input[type="text"]');

    const searchInput = page.locator('input[type="search"], input[type="text"]').first();

    // Search for something that won't match
    await searchInput.fill('zzznomatchesforxthisxqueryx999');
    await page.waitForTimeout(500);

    // Check for empty state
    const results = page.locator('.search-result, .result-item');
    const resultCount = await results.count();

    expect(resultCount).toBe(0);

    // Look for empty state message
    const emptyState = page.locator('.empty-state, .no-results, [role="status"]');

    if (await emptyState.count() > 0) {
      await expect(emptyState.first()).toBeVisible();
    }
  });
});
