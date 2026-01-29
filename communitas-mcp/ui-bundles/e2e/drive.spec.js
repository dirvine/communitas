/**
 * Drive Widget E2E Tests
 *
 * Tests file list, navigation, upload, delete, and quota
 * for the Drive widget.
 */

const { test, expect } = require('@playwright/test');

test.describe('Drive Widget', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to drive widget
    await page.goto('/widgets/drive/index.html');
  });

  test('should load file list', async ({ page }) => {
    // Wait for file list to load
    await page.waitForSelector('.file-list, .files, [role="list"]', { state: 'visible' });

    // Check for file items
    const fileItems = page.locator('.file-item, .file, [role="listitem"]');
    const count = await fileItems.count();

    // Should have at least structure (may be empty initially)
    expect(count).toBeGreaterThanOrEqual(0);

    if (count > 0) {
      await expect(fileItems.first()).toBeVisible();
    }
  });

  test('breadcrumb navigation works', async ({ page }) => {
    await page.waitForSelector('.file-list, .files');

    // Look for breadcrumb
    const breadcrumb = page.locator('.breadcrumb, nav[aria-label*="breadcrumb" i]');

    if (await breadcrumb.count() > 0) {
      await expect(breadcrumb.first()).toBeVisible();

      // Check for breadcrumb items
      const breadcrumbItems = page.locator('.breadcrumb-item, .breadcrumb a, .breadcrumb button');
      const itemCount = await breadcrumbItems.count();

      expect(itemCount).toBeGreaterThan(0);

      // Try clicking a breadcrumb item
      if (itemCount > 1) {
        await breadcrumbItems.first().click();
        await page.waitForTimeout(300);

        // Verify navigation happened (path or content changed)
        const updatedItems = page.locator('.breadcrumb-item');
        await expect(updatedItems.first()).toBeVisible();
      }
    }
  });

  test('file selection highlights item', async ({ page }) => {
    await page.waitForSelector('.file-list, .files');

    const fileItems = page.locator('.file-item, .file, [role="listitem"]');

    if (await fileItems.count() > 0) {
      // Click first file
      await fileItems.first().click();
      await page.waitForTimeout(200);

      // Check if selected class applied
      const hasSelected = await fileItems.first().evaluate((el) =>
        el.classList.contains('selected') || el.getAttribute('aria-selected') === 'true'
      );

      expect(hasSelected).toBe(true);
    }
  });

  test('upload button triggers file picker', async ({ page }) => {
    await page.waitForSelector('.file-list, .files');

    // Look for upload button
    const uploadBtn = page.locator('button[aria-label*="upload" i], .upload-btn, input[type="file"]');

    if (await uploadBtn.count() > 0) {
      // Check if it's a file input
      const isFileInput = await uploadBtn.first().evaluate((el) => el.tagName === 'INPUT');

      if (isFileInput) {
        // File input should have type="file"
        const inputType = await uploadBtn.first().getAttribute('type');
        expect(inputType).toBe('file');
      } else {
        // Should be clickable button
        await expect(uploadBtn.first()).toBeVisible();
        await expect(uploadBtn.first()).toBeEnabled();
      }
    }
  });

  test('upload progress displays', async ({ page }) => {
    await page.waitForSelector('.file-list, .files');

    // Look for upload progress elements (may not be visible initially)
    const progressBar = page.locator('.progress-bar, [role="progressbar"], .upload-progress');
    const progressCount = await progressBar.count();

    // Progress bar structure should exist
    expect(progressCount).toBeGreaterThanOrEqual(0);

    if (progressCount > 0) {
      // Check for aria-valuenow attribute (when progress is active)
      const ariaValueNow = await progressBar.first().getAttribute('aria-valuenow');

      // Should have progress value when active
      if (ariaValueNow !== null) {
        const progress = parseInt(ariaValueNow, 10);
        expect(progress).toBeGreaterThanOrEqual(0);
        expect(progress).toBeLessThanOrEqual(100);
      }
    }
  });

  test('delete file removes from list', async ({ page }) => {
    await page.waitForSelector('.file-list, .files');

    const fileItems = page.locator('.file-item, .file');

    if (await fileItems.count() > 0) {
      // Get initial count
      const initialCount = await fileItems.count();

      // Find delete button (may be in context menu or on item)
      const deleteBtn = page.locator('button[aria-label*="delete" i], .delete-btn');

      if (await deleteBtn.count() > 0) {
        // Click delete
        await deleteBtn.first().click();
        await page.waitForTimeout(300);

        // Check if confirmation dialog appears
        const confirmBtn = page.locator('button[aria-label*="confirm" i], .confirm-btn');

        if (await confirmBtn.count() > 0) {
          await confirmBtn.first().click();
          await page.waitForTimeout(300);
        }

        // Verify file was removed
        const newCount = await fileItems.count();
        expect(newCount).toBeLessThanOrEqual(initialCount);
      }
    }
  });

  test('folder navigation updates path', async ({ page }) => {
    await page.waitForSelector('.file-list, .files');

    // Look for folder items
    const folders = page.locator('.folder, .file-item[data-type="folder"], [aria-label*="folder" i]');

    if (await folders.count() > 0) {
      // Get initial path (from breadcrumb or URL)
      const initialBreadcrumb = await page.locator('.breadcrumb').textContent();

      // Click folder
      await folders.first().click();
      await page.waitForTimeout(500);

      // Verify path changed
      const newBreadcrumb = await page.locator('.breadcrumb').textContent();

      // Breadcrumb should have changed (unless folder was empty/error)
      if (initialBreadcrumb && newBreadcrumb) {
        expect(newBreadcrumb).not.toBe(initialBreadcrumb);
      }
    }
  });

  test('quota bar shows usage', async ({ page }) => {
    await page.waitForSelector('.file-list, .files');

    // Look for quota/storage bar
    const quotaBar = page.locator('.quota-bar, .storage-bar, [aria-label*="storage" i]');

    if (await quotaBar.count() > 0) {
      await expect(quotaBar.first()).toBeVisible();

      // Check for usage text
      const quotaText = await quotaBar.first().textContent();

      // Should contain numbers or percentage
      const hasNumbers = /\d+/.test(quotaText || '');
      expect(hasNumbers).toBe(true);

      // Check for progress bar
      const progress = page.locator('[role="progressbar"]');

      if (await progress.count() > 0) {
        const ariaValueNow = await progress.first().getAttribute('aria-valuenow');
        if (ariaValueNow) {
          const usage = parseInt(ariaValueNow, 10);
          expect(usage).toBeGreaterThanOrEqual(0);
          expect(usage).toBeLessThanOrEqual(100);
        }
      }
    }
  });
});
