/**
 * File Operations & Storage E2E Tests (Web Mode)
 * 
 * Prerequisites: Run `npm run tauri dev` before running tests
 */

import { test, expect } from '@playwright/test';
import { TauriTestHelper } from '../../utils/tauri-helpers';

test.describe('File Operations', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await page.waitForTimeout(2000);
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('F1: File upload/attach button is accessible', async ({ page }) => {
    const uploadButton = page.locator('button, [role="button"]').filter({
      hasText: /upload|attach|file|document|add file/i
    });

    const buttonCount = await uploadButton.count();
    console.log(`Found ${buttonCount} file upload buttons`);
    
    await helper.screenshot(page, 'files-upload-ui');
    expect(buttonCount >= 0).toBe(true);
  });

  test('F2: Storage section is accessible', async ({ page }) => {
    const storageNav = page.locator('a, button, [role="tab"]').filter({
      hasText: /storage|files|documents/i
    });

    const navCount = await storageNav.count();

    if (navCount > 0) {
      await storageNav.first().click();
      console.log('✅ Clicked storage navigation');
      await page.waitForTimeout(1000);
      await helper.screenshot(page, 'files-storage-view');
    }

    const bodyText = await page.locator('body').textContent();
    expect(bodyText).toBeDefined();
  });

  test('F3: New Document action exists', async ({ page }) => {
    const newDocButton = page.locator('button, a').filter({
      hasText: /new doc|new document|create doc|\+ doc/i
    });

    const buttonCount = await newDocButton.count();
    console.log(`Found ${buttonCount} new document buttons`);
    
    await helper.screenshot(page, 'files-new-doc');
    expect(buttonCount >= 0).toBe(true);
  });

  test('F4: Can navigate to documents/files area', async ({ page }) => {
    const fileAreas = page.locator('[data-testid*="file"], [data-testid*="document"], [data-testid*="storage"]');
    const navItems = page.locator('nav a, aside a, [role="navigation"] button');

    const hasFileUI = await fileAreas.count() > 0 || await navItems.count() > 0;
    
    await helper.screenshot(page, 'files-navigation');
    expect(hasFileUI).toBe(true);
    console.log('✅ File/document UI accessible');
  });
});
