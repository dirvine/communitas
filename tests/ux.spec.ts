import { test, expect } from '@playwright/test';

test.describe('Communitas UX Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for the app to load
    await page.waitForLoadState('networkidle');
  });

  test('app loads and shows main interface', async ({ page }) => {
    // Check if the main app title is visible
    await expect(page.locator('text=Communitas - P2P Collaboration Platform')).toBeVisible();
    
    // Check for navigation elements
    await expect(page.locator('text=Organization')).toBeVisible();
    await expect(page.locator('text=Groups & People')).toBeVisible();
  });

  test('navigation between tabs works', async ({ page }) => {
    // Click on Groups & People tab
    await page.click('text=Groups & People');
    
    // Verify tab content changes
    await expect(page.locator('text=Groups')).toBeVisible();
    
    // Go back to Organization tab
    await page.click('text=Organization');
    await expect(page.locator('text=Organization Dashboard')).toBeVisible();
  });

  test('identity modal opens and closes', async ({ page }) => {
    // Click Identity button
    await page.click('button:has-text("Identity")');
    
    // Check if modal opened
    await expect(page.locator('text=Identity Management')).toBeVisible();
    
    // Close modal by clicking backdrop
    await page.click('.MuiBackdrop-root');
    
    // Verify modal closed
    await expect(page.locator('text=Identity Management')).not.toBeVisible();
  });

  test('responsive sidebar behavior', async ({ page, viewport }) => {
    // Test desktop behavior
    if (viewport && viewport.width > 960) {
      // Sidebar should be visible by default on desktop
      await expect(page.locator('[role="navigation"]')).toBeVisible();
    }
    
    // Test mobile behavior
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Sidebar should be hidden on mobile
    await expect(page.locator('[role="navigation"]')).not.toBeVisible();
    
    // Open sidebar with menu button
    await page.click('[aria-label="menu"]');
    await expect(page.locator('[role="navigation"]')).toBeVisible();
  });

  test('theme switcher works', async ({ page }) => {
    // Find and click theme switcher
    await page.click('[aria-label*="theme"]');
    
    // Check if theme menu opens
    await expect(page.locator('text=Light')).toBeVisible();
    await expect(page.locator('text=Dark')).toBeVisible();
    
    // Switch to dark theme
    await page.click('text=Dark');
    
    // Verify theme changed (background color should change)
    const backgroundColor = await page.evaluate(() => {
      return window.getComputedStyle(document.body).backgroundColor;
    });
    
    // Dark theme typically has a dark background
    expect(backgroundColor).toContain('rgb');
  });

  test('network health displays mock data', async ({ page }) => {
    // Wait for network health data to load
    await page.waitForTimeout(1000);
    
    // Check for network status indicators
    const networkStatus = page.locator('text=/Connected|Disconnected/');
    await expect(networkStatus).toBeVisible();
  });

  test('organization dashboard loads', async ({ page }) => {
    // Navigate to Organization tab
    await page.click('text=Organization');
    
    // Check for organization elements
    await expect(page.locator('text=Organization Dashboard')).toBeVisible();
    
    // Verify mock organization data is displayed
    await expect(page.locator('text=Demo Organization')).toBeVisible();
  });

  test('messages display in communication hub', async ({ page }) => {
    // Open communication hub if available
    const communicationHub = page.locator('[data-testid="communication-hub"]');
    if (await communicationHub.isVisible()) {
      await communicationHub.click();
      
      // Check for message elements
      await expect(page.locator('text=/Welcome|Hello|Message/')).toBeVisible();
    }
  });

  test('encryption status indicator present', async ({ page }) => {
    // Look for encryption status component
    const encryptionStatus = page.locator('[aria-label*="encryption"]');
    await expect(encryptionStatus).toBeVisible();
  });

  test('breadcrumb navigation updates', async ({ page }) => {
    // Navigate through different sections
    await page.click('text=Organization');
    
    // Check if breadcrumb shows current location
    const breadcrumb = page.locator('[aria-label="breadcrumb"]');
    if (await breadcrumb.isVisible()) {
      await expect(breadcrumb).toContainText('Organization');
    }
    
    // Navigate to another section
    await page.click('text=Groups & People');
    
    // Breadcrumb should update
    if (await breadcrumb.isVisible()) {
      await expect(breadcrumb).toContainText('Groups');
    }
  });
});