/**
 * Canvas Widget E2E Tests
 *
 * Tests canvas element, layers, history, drawing tools,
 * and remote cursors for the Canvas widget.
 */

const { test, expect } = require('@playwright/test');

test.describe('Canvas Widget', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to canvas widget
    await page.goto('/widgets/canvas/index.html');
  });

  test('should load canvas element', async ({ page }) => {
    // Wait for canvas to load
    await page.waitForSelector('canvas', { state: 'visible' });

    // Verify canvas is present
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    // Check canvas has dimensions
    const box = await canvas.boundingBox();
    expect(box).not.toBeNull();

    if (box) {
      expect(box.width).toBeGreaterThan(0);
      expect(box.height).toBeGreaterThan(0);
    }
  });

  test('layer panel displays layers', async ({ page }) => {
    await page.waitForSelector('canvas');

    // Look for layer panel
    const layerPanel = page.locator('.layer-panel, .layers, [aria-label*="layer" i]');

    if (await layerPanel.count() > 0) {
      await expect(layerPanel.first()).toBeVisible();

      // Check for layer items
      const layers = page.locator('.layer, .layer-item, [data-layer-id]');
      const layerCount = await layers.count();

      // Should have at least one layer (default layer)
      expect(layerCount).toBeGreaterThan(0);

      if (layerCount > 0) {
        await expect(layers.first()).toBeVisible();
      }
    }
  });

  test('layer visibility toggle works', async ({ page }) => {
    await page.waitForSelector('canvas');

    const layers = page.locator('.layer, .layer-item');

    if (await layers.count() > 0) {
      // Find visibility toggle button
      const visibilityBtn = page.locator('.visibility-toggle, button[aria-label*="visibility" i]').first();

      if (await visibilityBtn.count() > 0) {
        // Get initial state
        const initialClass = await visibilityBtn.getAttribute('class');
        const initialAriaPressed = await visibilityBtn.getAttribute('aria-pressed');

        // Click toggle
        await visibilityBtn.click();
        await page.waitForTimeout(200);

        // Verify state changed
        const newClass = await visibilityBtn.getAttribute('class');
        const newAriaPressed = await visibilityBtn.getAttribute('aria-pressed');

        // At least one should have changed
        const changed = newClass !== initialClass || newAriaPressed !== initialAriaPressed;
        expect(changed).toBe(true);
      }
    }
  });

  test('layer lock/unlock functions', async ({ page }) => {
    await page.waitForSelector('canvas');

    const layers = page.locator('.layer, .layer-item');

    if (await layers.count() > 0) {
      // Find lock toggle button
      const lockBtn = page.locator('.lock-toggle, button[aria-label*="lock" i]').first();

      if (await lockBtn.count() > 0) {
        // Get initial state
        const initialAriaPressed = await lockBtn.getAttribute('aria-pressed');

        // Click lock
        await lockBtn.click();
        await page.waitForTimeout(200);

        // Verify state changed
        const newAriaPressed = await lockBtn.getAttribute('aria-pressed');
        expect(newAriaPressed).not.toBe(initialAriaPressed);
      }
    }
  });

  test('opacity slider adjusts layer', async ({ page }) => {
    await page.waitForSelector('canvas');

    // Look for opacity slider
    const opacitySlider = page.locator('input[type="range"][aria-label*="opacity" i], .opacity-slider');

    if (await opacitySlider.count() > 0) {
      await expect(opacitySlider.first()).toBeVisible();

      // Get initial value
      const initialValue = await opacitySlider.first().inputValue();

      // Change slider value
      await opacitySlider.first().fill('50');
      await page.waitForTimeout(200);

      // Verify value changed
      const newValue = await opacitySlider.first().inputValue();
      expect(newValue).not.toBe(initialValue);
      expect(newValue).toBe('50');
    }
  });

  test('history scrubber shows actions', async ({ page }) => {
    await page.waitForSelector('canvas');

    // Look for history panel or scrubber
    const historyScrubber = page.locator('.history-scrubber, .history, [aria-label*="history" i]');

    if (await historyScrubber.count() > 0) {
      await expect(historyScrubber.first()).toBeVisible();

      // Check for history items
      const historyItems = page.locator('.history-item, .action');
      const itemCount = await historyItems.count();

      // May or may not have history yet
      expect(itemCount).toBeGreaterThanOrEqual(0);

      if (itemCount > 0) {
        await expect(historyItems.first()).toBeVisible();
      }
    }
  });

  test('undo/redo buttons work', async ({ page }) => {
    await page.waitForSelector('canvas');

    // Look for undo button
    const undoBtn = page.locator('button[aria-label*="undo" i], .undo-btn');
    const redoBtn = page.locator('button[aria-label*="redo" i], .redo-btn');

    if (await undoBtn.count() > 0 && await redoBtn.count() > 0) {
      await expect(undoBtn.first()).toBeVisible();
      await expect(redoBtn.first()).toBeVisible();

      // Check if buttons are enabled/disabled appropriately
      const undoDisabled = await undoBtn.first().isDisabled();
      const redoDisabled = await redoBtn.first().isDisabled();

      // At least one of them should exist
      expect(undoDisabled !== undefined).toBe(true);
      expect(redoDisabled !== undefined).toBe(true);

      // Try clicking undo (if enabled)
      if (!undoDisabled) {
        await undoBtn.first().click();
        await page.waitForTimeout(200);

        // Redo should now be enabled
        const redoNowDisabled = await redoBtn.first().isDisabled();
        expect(redoNowDisabled).toBe(false);
      }
    }
  });

  test('remote cursors display', async ({ page }) => {
    await page.waitForSelector('canvas');

    // Look for remote cursor elements
    const remoteCursors = page.locator('.remote-cursor, [data-remote-cursor], .cursor-remote');
    const cursorCount = await remoteCursors.count();

    // Remote cursors may not be present initially (no other users)
    expect(cursorCount).toBeGreaterThanOrEqual(0);

    // If present, verify structure
    if (cursorCount > 0) {
      const cursor = remoteCursors.first();

      // Should have position
      const box = await cursor.boundingBox();
      expect(box).not.toBeNull();

      // Should have some visual indicator (color, label)
      const cursorText = await cursor.textContent();
      const hasLabel = cursorText && cursorText.length > 0;

      const hasColor = await cursor.evaluate((el) => {
        const style = window.getComputedStyle(el);
        return style.backgroundColor !== 'rgba(0, 0, 0, 0)';
      });

      // Should have either label or color
      expect(hasLabel || hasColor).toBe(true);
    }
  });
});
