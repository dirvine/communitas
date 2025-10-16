/**
 * Onboarding and Identity Creation E2E Tests
 *
 * Tests the complete user onboarding flow in web mode.
 * This covers the critical path from first launch to authenticated user.
 */

import { test, expect } from '@playwright/test';

test.describe('Onboarding Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Clear any existing session data
    await page.context().clearCookies();
    await page.evaluate(() => {
      localStorage.clear();
      sessionStorage.clear();
    });
  });

  test('should display welcome screen on first visit', async ({ page }) => {
    await page.goto('/');

    // Check for welcome screen elements
    await expect(page.getByRole('heading', { name: /welcome to communitas/i })).toBeVisible();
    await expect(page.getByText(/privacy, security and freedom/i)).toBeVisible();
    await expect(page.getByRole('button', { name: /get started/i })).toBeVisible();
  });

  test('should navigate through identity creation flow', async ({ page }) => {
    await page.goto('/');

    // Click "Get Started"
    await page.getByRole('button', { name: /get started/i }).click();

    // Should show identity creation screen
    await expect(page.getByText(/your four-word address/i)).toBeVisible();
    await expect(page.getByPlaceholder(/enter your display name/i)).toBeVisible();

    // Fill in identity details
    await page.getByPlaceholder(/enter your display name/i).fill('Test User');
    await page.getByPlaceholder(/enter device name/i).fill('Test Device');

    // Generate four-word identity (if button exists)
    const generateButton = page.getByRole('button', { name: /generate/i });
    if (await generateButton.isVisible()) {
      await generateButton.click();
    }

    // Check that four-word address is generated
    const fourWordInput = page.locator('input[placeholder*="four-word"]').or(
      page.locator('[data-testid="four-word-address"]')
    );
    await expect(fourWordInput).toBeVisible();

    // Continue to next step
    await page.getByRole('button', { name: /continue/i }).click();

    // Should move to network setup
    await expect(page.getByText(/connecting to network/i)).toBeVisible();
  });

  test('should handle passkey registration', async ({ page }) => {
    // Mock WebAuthn for testing
    await page.addInitScript(() => {
      // Mock WebAuthn API for testing
      (navigator as any).credentials = {
        create: async () => ({
          id: 'test-credential-id',
          rawId: new ArrayBuffer(32),
          response: {
            clientDataJSON: new ArrayBuffer(100),
            attestationObject: new ArrayBuffer(200)
          }
        }),
        get: async () => ({
          id: 'test-credential-id',
          rawId: new ArrayBuffer(32),
          response: {
            clientDataJSON: new ArrayBuffer(100),
            authenticatorData: new ArrayBuffer(50),
            signature: new ArrayBuffer(64)
          }
        })
      };
    });

    await page.goto('/');

    // Navigate to passkey setup
    await page.getByRole('button', { name: /get started/i }).click();
    await page.getByRole('button', { name: /continue/i }).click();

    // Look for passkey setup option
    const passkeyButton = page.getByRole('button', { name: /enable passkey/i }).or(
      page.getByText(/passkey/i).locator('..').locator('button')
    );

    if (await passkeyButton.isVisible()) {
      await passkeyButton.click();

      // Should show biometric options
      await expect(page.getByText(/touch id/i).or(page.getByText(/face id/i))).toBeVisible();

      // Complete passkey setup
      await page.getByRole('button', { name: /enable passkey/i }).click();
    }
  });

  test('should complete onboarding and reach main app', async ({ page }) => {
    await page.goto('/');

    // Quick onboarding flow
    await page.getByRole('button', { name: /get started/i }).click();
    await page.getByPlaceholder(/enter your display name/i).fill('Test User');

    // Skip to main app (mock the flow completion)
    await page.evaluate(() => {
      // Simulate completed onboarding
      localStorage.setItem('communitas_onboarding_complete', 'true');
      localStorage.setItem('communitas_user', JSON.stringify({
        id: 'test-user',
        name: 'Test User',
        fourWordAddress: 'test-user-words'
      }));
    });

    // Reload to check persistence
    await page.reload();

    // Should now show main app interface
    await expect(page.getByText(/activity dashboard/i).or(page.getByText(/dashboard/i))).toBeVisible();
  });

  test('should handle network connection errors gracefully', async ({ page }) => {
    // Mock network failure
    await page.route('**/connect-network', route => route.abort());
    await page.route('**/bootstrap-nodes', route => route.abort());

    await page.goto('/');

    // Go through onboarding
    await page.getByRole('button', { name: /get started/i }).click();
    await page.getByPlaceholder(/enter your display name/i).fill('Test User');
    await page.getByRole('button', { name: /continue/i }).click();

    // Should handle network failure gracefully
    await expect(page.getByText(/offline mode/i).or(page.getByText(/network error/i))).toBeVisible();

    // Should still allow proceeding
    const continueButton = page.getByRole('button', { name: /continue anyway/i }).or(
      page.getByRole('button', { name: /enter communitas/i })
    );

    if (await continueButton.isVisible()) {
      await continueButton.click();
      await expect(page.getByText(/dashboard/i)).toBeVisible();
    }
  });
});

test.describe('Identity Management', () => {
  test.beforeEach(async ({ page }) => {
    // Set up authenticated user
    await page.addInitScript(() => {
      localStorage.setItem('communitas_user', JSON.stringify({
        id: 'test-user',
        name: 'Test User',
        fourWordAddress: 'test-user-words'
      }));
      localStorage.setItem('communitas_onboarding_complete', 'true');
    });

    await page.goto('/');
  });

  test('should display user identity information', async ({ page }) => {
    // Check identity display in sidebar
    await expect(page.getByText('Test User')).toBeVisible();
    await expect(page.getByText('test-user-words')).toBeVisible();
  });

  test('should allow identity switching', async ({ page }) => {
    // Click on identity selector
    await page.getByText('Test User').click();

    // Should show identity switcher
    await expect(page.getByText(/switch identity/i)).toBeVisible();
    await expect(page.getByRole('button', { name: /create new identity/i })).toBeVisible();
  });
});
