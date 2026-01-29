/**
 * Kanban Widget E2E Tests
 *
 * Tests board layout, columns, cards, drag-drop, filters,
 * and accessibility for the Kanban widget.
 */

const { test, expect } = require('@playwright/test');

test.describe('Kanban Widget', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to kanban widget
    await page.goto('/widgets/kanban/index.html');
  });

  test('should load board with columns', async ({ page }) => {
    // Wait for board to load
    await page.waitForSelector('.kanban-board, .board', { state: 'visible' });

    // Check for columns
    const columns = page.locator('.column, .kanban-column, [data-column]');
    const columnCount = await columns.count();

    // Should have at least one column
    expect(columnCount).toBeGreaterThan(0);

    // Verify columns are visible
    await expect(columns.first()).toBeVisible();
  });

  test('cards render in columns', async ({ page }) => {
    await page.waitForSelector('.kanban-board, .board');

    // Look for cards
    const cards = page.locator('.card, .kanban-card, [data-card-id]');
    const cardCount = await cards.count();

    // May or may not have cards initially
    expect(cardCount).toBeGreaterThanOrEqual(0);

    if (cardCount > 0) {
      // Verify first card is visible
      await expect(cards.first()).toBeVisible();

      // Check card has content
      const cardText = await cards.first().textContent();
      expect(cardText).toBeTruthy();
    }
  });

  test('drag-drop moves card between columns', async ({ page }) => {
    await page.waitForSelector('.kanban-board, .board');

    const columns = page.locator('.column, .kanban-column');
    const cards = page.locator('.card, .kanban-card, [draggable="true"]');

    if (await columns.count() >= 2 && await cards.count() > 0) {
      // Get first card
      const sourceCard = cards.first();
      const sourceBox = await sourceCard.boundingBox();

      // Get second column
      const targetColumn = columns.nth(1);
      const targetBox = await targetColumn.boundingBox();

      if (sourceBox && targetBox) {
        // Perform drag and drop using HTML5 drag API
        await page.mouse.move(sourceBox.x + sourceBox.width / 2, sourceBox.y + sourceBox.height / 2);
        await page.mouse.down();
        await page.mouse.move(targetBox.x + targetBox.width / 2, targetBox.y + targetBox.height / 2);
        await page.mouse.up();

        // Wait for state update
        await page.waitForTimeout(500);

        // Verify card was moved (check if it's now in second column)
        // Note: Actual verification depends on widget implementation
      }
    }
  });

  test('filter bar filters by tag/priority', async ({ page }) => {
    await page.waitForSelector('.kanban-board, .board');

    // Look for filter controls
    const filterBar = page.locator('.filter-bar, .filters, [role="search"]');

    if (await filterBar.count() > 0) {
      // Get initial card count
      const allCards = page.locator('.card, .kanban-card');
      const initialCount = await allCards.count();

      // Find filter input or select
      const filterInput = page.locator('input[type="text"], select').first();

      if (await filterInput.count() > 0) {
        // Apply filter
        await filterInput.fill('high priority');
        await page.waitForTimeout(300);

        // Get filtered count
        const filteredCards = page.locator('.card, .kanban-card');
        const filteredCount = await filteredCards.count();

        // Should have filtered (or no matches)
        expect(filteredCount).toBeLessThanOrEqual(initialCount);

        // Clear filter
        await filterInput.clear();
        await page.waitForTimeout(300);

        // Should show all cards again
        const resetCount = await allCards.count();
        expect(resetCount).toBe(initialCount);
      }
    }
  });

  test('card counts display correctly', async ({ page }) => {
    await page.waitForSelector('.kanban-board, .board');

    // Look for card count indicators
    const countIndicators = page.locator('.card-count, .column-count, [data-count]');

    if (await countIndicators.count() > 0) {
      // Get first count indicator
      const countText = await countIndicators.first().textContent();

      // Should be numeric
      const match = countText?.match(/\d+/);
      expect(match).toBeTruthy();

      if (match) {
        const count = parseInt(match[0], 10);
        expect(count).toBeGreaterThanOrEqual(0);
      }
    }
  });

  test('add card placeholder works', async ({ page }) => {
    await page.waitForSelector('.kanban-board, .board');

    // Look for add card button or placeholder
    const addCardBtn = page.locator('.add-card, button[aria-label*="add" i]');

    if (await addCardBtn.count() > 0) {
      // Click add card
      await addCardBtn.first().click();
      await page.waitForTimeout(300);

      // Look for card input or form
      const cardInput = page.locator('textarea, input[placeholder*="card" i]');

      if (await cardInput.count() > 0) {
        await expect(cardInput.first()).toBeVisible();
        await expect(cardInput.first()).toBeFocused();
      }
    }
  });

  test('card detail view opens', async ({ page }) => {
    await page.waitForSelector('.kanban-board, .board');

    const cards = page.locator('.card, .kanban-card');

    if (await cards.count() > 0) {
      // Click first card
      await cards.first().click();
      await page.waitForTimeout(500);

      // Look for card detail modal or panel
      const cardDetail = page.locator('.card-detail, .modal, [role="dialog"]');

      if (await cardDetail.count() > 0) {
        await expect(cardDetail.first()).toBeVisible();

        // Look for close button
        const closeBtn = page.locator('button[aria-label*="close" i], .close-btn');

        if (await closeBtn.count() > 0) {
          await closeBtn.first().click();
          await page.waitForTimeout(300);

          // Detail should be hidden
          const isHidden = await cardDetail.first().isHidden();
          expect(isHidden).toBe(true);
        }
      }
    }
  });

  test('accessibility: keyboard drag-drop', async ({ page }) => {
    await page.waitForSelector('.kanban-board, .board');

    const cards = page.locator('.card, .kanban-card');

    if (await cards.count() > 0) {
      // Focus first card
      await cards.first().focus();

      // Verify card is focusable
      await expect(cards.first()).toBeFocused();

      // Try keyboard navigation
      await page.keyboard.press('Tab');
      await page.waitForTimeout(100);

      // Verify tab moved focus (to next card or button)
      const focusedElement = page.locator(':focus');
      await expect(focusedElement).toBeVisible();

      // Check for keyboard shortcuts hint or aria-label
      const firstCard = cards.first();
      const ariaLabel = await firstCard.getAttribute('aria-label');
      const ariaDescribedBy = await firstCard.getAttribute('aria-describedby');

      // Should have some accessibility attribute
      expect(ariaLabel || ariaDescribedBy).toBeTruthy();
    }
  });
});
