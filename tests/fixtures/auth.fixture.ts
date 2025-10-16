/**
 * Authentication Test Fixtures
 *
 * Provides utilities for setting up authenticated test users
 * and managing test authentication state.
 */

import { test as base, expect } from '@playwright/test';

export interface TestUser {
  id: string;
  name: string;
  fourWordAddress: string;
  publicKey: string;
  isAuthenticated: boolean;
}

export const testUsers = {
  alice: {
    id: 'alice-test-id',
    name: 'Alice Test',
    fourWordAddress: 'alice-forest-moon-star',
    publicKey: 'test-public-key-alice',
    isAuthenticated: true
  },
  bob: {
    id: 'bob-test-id',
    name: 'Bob Test',
    fourWordAddress: 'bob-mountain-river-sun',
    publicKey: 'test-public-key-bob',
    isAuthenticated: true
  },
  charlie: {
    id: 'charlie-test-id',
    name: 'Charlie Test',
    fourWordAddress: 'charlie-ocean-cloud-peak',
    publicKey: 'test-public-key-charlie',
    isAuthenticated: false
  }
};

export const authFixtures = base.extend<{
  authenticatedUser: TestUser;
  setupAuthUser: (user: TestUser) => Promise<void>;
}>({
  authenticatedUser: testUsers.alice,

  setupAuthUser: async ({ page }, use) => {
    const setupUser = async (user: TestUser) => {
      // Set up authentication state in localStorage
      await page.evaluate((userData) => {
        localStorage.setItem('communitas_user', JSON.stringify(userData));
        localStorage.setItem('communitas_auth_token', 'test-token-123');
        localStorage.setItem('communitas_onboarding_complete', 'true');

        // Set up mock Tauri API responses
        if ((window as any).__TAURI__) {
          // Mock successful auth responses
          const originalInvoke = (window as any).__TAURI__.core.invoke;
          (window as any).__TAURI__.core.invoke = async (cmd: string, args?: any) => {
            if (cmd === 'auth_get_session') {
              return {
                session_id: userData.id,
                four_words: userData.fourWordAddress,
                display_name: userData.name
              };
            }
            if (cmd === 'core_get_user_info') {
              return {
                peer_id: userData.fourWordAddress,
                display_name: userData.name,
                device_name: 'Test Device'
              };
            }
            // Fall back to original invoke for other commands
            return originalInvoke(cmd, args);
          };
        }
      }, user);

      // Navigate to trigger auth state loading
      await page.reload();
      await page.waitForLoadState('networkidle');
    };

    await use(setupUser);
  }
});

// Test utilities for authentication flows
export class AuthTestUtils {
  static async mockWebAuthn(page: any) {
    await page.addInitScript(() => {
      // Mock WebAuthn API for testing
      (navigator as any).credentials = {
        create: async () => ({
          id: 'test-credential-id',
          rawId: new Uint8Array(32),
          response: {
            clientDataJSON: new Uint8Array(100),
            attestationObject: new Uint8Array(200)
          }
        }),
        get: async () => ({
          id: 'test-credential-id',
          rawId: new Uint8Array(32),
          response: {
            clientDataJSON: new Uint8Array(100),
            authenticatorData: new Uint8Array(50),
            signature: new Uint8Array(64)
          }
        })
      };
    });
  }

  static async mockNetworkRequests(page: any) {
    // Mock network-related API calls
    await page.route('**/connect-network', route => route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ connected: true, peers: 5 })
    }));

    await page.route('**/bootstrap-nodes', route => route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(['node1.example.com', 'node2.example.com'])
    }));
  }

  static async completeOnboarding(page: any, user: TestUser) {
    // Click through onboarding flow
    await page.getByRole('button', { name: /get started/i }).click();
    await page.waitForTimeout(500);

    // Fill identity form
    const nameInput = page.getByPlaceholder(/display name|your name/i).first();
    if (await nameInput.isVisible()) {
      await nameInput.fill(user.name);
    }

    // Fill device name if present
    const deviceInput = page.getByPlaceholder(/device name/i);
    if (await deviceInput.isVisible()) {
      await deviceInput.fill('Test Device');
    }

    // Continue through flow
    const continueButton = page.getByRole('button', { name: /continue|next/i });
    if (await continueButton.isVisible()) {
      await continueButton.click();
      await page.waitForTimeout(1000);
    }

    // Skip network setup for faster tests
    const skipNetworkButton = page.getByRole('button', { name: /skip|continue anyway/i });
    if (await skipNetworkButton.isVisible()) {
      await skipNetworkButton.click();
    }

    // Wait for main app to load
    await page.waitForSelector('[data-testid="main-app"], .dashboard, [data-testid="activity-feed"]', {
      timeout: 10000
    });
  }

  static async verifyAuthenticatedState(page: any, user: TestUser) {
    // Verify user identity is displayed
    await expect(page.getByText(user.name)).toBeVisible();
    await expect(page.getByText(user.fourWordAddress)).toBeVisible();

    // Verify main app elements are present
    await expect(page.locator('.dashboard, [data-testid="activity-dashboard"]')).toBeVisible();

    // Verify auth state in localStorage
    const storedUser = await page.evaluate(() => {
      const userData = localStorage.getItem('communitas_user');
      return userData ? JSON.parse(userData) : null;
    });

    expect(storedUser).toEqual(user);
  }

  static async logoutUser(page: any) {
    // Find and click logout button
    const logoutButton = page.getByRole('button', { name: /logout|sign out/i });
    if (await logoutButton.isVisible()) {
      await logoutButton.click();
    } else {
      // Try user menu
      await page.getByText(/test user/i).click();
      await page.getByRole('button', { name: /logout|sign out/i }).click();
    }

    // Verify logged out state
    await expect(page.getByText(/welcome|login|sign in/i)).toBeVisible();
    await expect(page.getByText('Test User')).not.toBeVisible();
  }
}

// Export the extended test function
export const test = authFixtures;
export { expect } from '@playwright/test';
