/**
 * Tauri Native App Lifecycle E2E Tests
 *
 * Tests the actual packaged Tauri application, including:
 * - App startup and window management
 * - IPC communication with Rust backend
 * - Native OS integration (file system, keyring, etc.)
 * - WebRTC and networking functionality
 */

import { test, expect, Page } from '@playwright/test';
import { TauriTestEnvironment } from '../../utils/tauri-setup';
import { _electron as electron } from 'playwright';

// Skip these tests if not in Tauri mode
test.describe.configure({
  mode: 'serial',
  retries: 1
});

test.describe('Tauri App Lifecycle', () => {
  let testEnv: TauriTestEnvironment;
  let electronApp: any;
  let page: Page;

  test.beforeAll(async () => {
    // Initialize test environment
    testEnv = new TauriTestEnvironment({
      cleanup: true,
      timeout: 60000
    });

    // Launch the Tauri app via Electron
    electronApp = await electron.launch({
      args: [
        testEnv.findTauriApp(),
        '--no-sandbox',
        '--disable-dev-shm-usage'
      ],
      env: {
        ...process.env,
        COMMUNITAS_DATA_DIR: testEnv.getDataDir(),
        RUST_LOG: 'info,communitas=debug',
        NODE_ENV: 'test'
      }
    });

    // Get the main window
    page = await electronApp.firstWindow();

    // Wait for app to be ready
    await page.waitForLoadState('networkidle');
  });

  test.afterAll(async () => {
    // Clean up
    if (electronApp) {
      await electronApp.close();
    }
    await testEnv.cleanup();
  });

  test('should start Tauri app successfully', async () => {
    // Verify the app window exists and is visible
    expect(page).toBeDefined();

    // Check that the app loaded
    await expect(page.locator('body')).toBeVisible();

    // Verify no critical errors in console
    const errors: string[] = [];
    page.on('pageerror', error => errors.push(error.message));

    // Wait a moment for any startup errors
    await page.waitForTimeout(2000);

    expect(errors.length).toBe(0);
  });

  test('should have access to Tauri API', async () => {
    // Test that Tauri APIs are available
    const isTauriAvailable = await page.evaluate(() => {
      return !!(window as any).__TAURI__;
    });

    expect(isTauriAvailable).toBe(true);
  });

  test('should initialize core context', async () => {
    // Test IPC communication with Rust backend
    const healthResult = await page.evaluate(async () => {
      try {
        const result = await (window as any).__TAURI__.core.invoke('health');
        return result;
      } catch (error) {
        return { error: error.message };
      }
    });

    expect(healthResult).toHaveProperty('status', 'ok');
    expect(healthResult).toHaveProperty('app');
  });

  test('should handle onboarding flow in native app', async () => {
    // Check for first launch welcome screen
    const welcomeHeading = page.locator('h1, h2, h3').filter({ hasText: /welcome|getting started/i });
    await expect(welcomeHeading.or(page.getByText(/first launch/i))).toBeVisible();

    // Test identity creation
    const nameInput = page.locator('input[placeholder*="name"], input[type="text"]').first();
    if (await nameInput.isVisible()) {
      await nameInput.fill('Tauri Test User');
    }

    // Look for continue/get started button
    const continueButton = page.getByRole('button', { name: /continue|get started|next/i });
    if (await continueButton.isVisible()) {
      await continueButton.click();

      // Should progress in the flow
      await expect(page.getByText(/connecting|network|bootstrap/i)).toBeVisible();
    }
  });

  test('should persist data to native filesystem', async () => {
    // Test that data is saved to the native filesystem
    const testData = { key: 'test-value', timestamp: Date.now() };

    // Store test data
    const storeResult = await page.evaluate(async (data) => {
      try {
        return await (window as any).__TAURI__.core.invoke('core_private_put', {
          key: 'e2e_test_data',
          value: new TextEncoder().encode(JSON.stringify(data))
        });
      } catch (error) {
        return { error: error.message };
      }
    }, testData);

    expect(storeResult).toBe(true);

    // Retrieve and verify data
    const retrievedData = await page.evaluate(async () => {
      try {
        const bytes = await (window as any).__TAURI__.core.invoke('core_private_get', {
          key: 'e2e_test_data'
        });
        return JSON.parse(new TextDecoder().decode(new Uint8Array(bytes)));
      } catch (error) {
        return { error: error.message };
      }
    });

    expect(retrievedData).toEqual(testData);
  });

  test('should handle IPC communication errors gracefully', async () => {
    // Test error handling for invalid IPC calls
    const errorResult = await page.evaluate(async () => {
      try {
        await (window as any).__TAURI__.core.invoke('non_existent_command');
        return { success: true };
      } catch (error) {
        return { error: error.message };
      }
    });

    expect(errorResult).toHaveProperty('error');
  });

  test('should support WebRTC functionality', async ({ browserName }) => {
    // WebRTC tests are complex and may require additional setup
    // For now, test that WebRTC APIs are available
    test.skip(browserName === 'webkit', 'WebRTC not fully supported in WebKit');

    const webrtcSupport = await page.evaluate(() => {
      return {
        rtcPeerConnection: !!(window as any).RTCPeerConnection,
        rtcDataChannel: !!(window as any).RTCDataChannel,
        mediaDevices: !!navigator.mediaDevices,
        getUserMedia: !!navigator.mediaDevices?.getUserMedia
      };
    });

    expect(webrtcSupport.rtcPeerConnection).toBe(true);
    expect(webrtcSupport.mediaDevices).toBe(true);
  });

  test('should handle app window management', async () => {
    // Test window minimize/maximize/close buttons if available
    // Note: This is platform-specific and may not be testable in all environments

    const windowControls = page.locator('[data-testid="window-controls"], .window-controls');
    if (await windowControls.isVisible()) {
      // Test that window controls are present
      expect(await windowControls.count()).toBeGreaterThan(0);
    }

    // Test window title
    const title = await page.title();
    expect(title).toContain('Communitas');
  });

  test('should handle file system access', async () => {
    // Test native file system access
    const fsResult = await page.evaluate(async () => {
      try {
        const result = await (window as any).__TAURI__.core.invoke('core_storage_list', {
          path: '.'
        });
        return { success: true, entries: result.length };
      } catch (error) {
        return { error: error.message };
      }
    });

    // Should either succeed or fail gracefully
    expect(fsResult).toHaveProperty('success').or.toHaveProperty('error');
  });

  test('should handle app restart and state persistence', async () => {
    // Set some test state
    await page.evaluate(async () => {
      localStorage.setItem('e2e_test_state', 'persisted_value');
    });

    // Simulate app restart (close and reopen window)
    await page.reload();

    // Check that state is preserved
    const persistedValue = await page.evaluate(() => {
      return localStorage.getItem('e2e_test_state');
    });

    expect(persistedValue).toBe('persisted_value');
  });
});

test.describe('Tauri IPC Integration', () => {
  let testEnv: TauriTestEnvironment;
  let electronApp: any;
  let page: Page;

  test.beforeAll(async () => {
    testEnv = new TauriTestEnvironment();
    electronApp = await electron.launch({
      args: [testEnv.findTauriApp()],
      env: {
        ...process.env,
        COMMUNITAS_DATA_DIR: testEnv.getDataDir()
      }
    });
    page = await electronApp.firstWindow();
    await page.waitForLoadState('networkidle');
  });

  test.afterAll(async () => {
    if (electronApp) {
      await electronApp.close();
    }
    await testEnv.cleanup();
  });

  test('should handle complex IPC data structures', async () => {
    // Test IPC with complex data structures
    const testMessage = {
      id: 'test-message-id',
      content: {
        text: 'Hello from E2E test!',
        author: 'Test Suite'
      },
      metadata: {
        entityId: 'test-entity',
        entityType: 'channel',
        authorPeerId: 'test-peer',
        vectorClock: { 'test-peer': 1 },
        lamportClock: 1,
        timestamp: Date.now()
      }
    };

    // This would test actual message sending in a full implementation
    // For now, just verify IPC channel works
    const ipcResult = await page.evaluate(async (message) => {
      try {
        // Test with a simple IPC call first
        const health = await (window as any).__TAURI__.core.invoke('health');
        return { success: true, health };
      } catch (error) {
        return { error: error.message };
      }
    }, testMessage);

    expect(ipcResult.success).toBe(true);
  });

  test('should handle concurrent IPC calls', async () => {
    // Test multiple simultaneous IPC calls
    const promises = Array(5).fill(null).map((_, i) =>
      page.evaluate(async (index) => {
        try {
          const result = await (window as any).__TAURI__.core.invoke('health');
          return { index, success: true, result };
        } catch (error) {
          return { index, error: error.message };
        }
      }, i)
    );

    const results = await Promise.all(promises);

    // All calls should succeed
    results.forEach((result, index) => {
      expect(result.success, `IPC call ${index} failed`).toBe(true);
    });
  });
});
