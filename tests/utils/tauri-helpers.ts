/**
 * Tauri Test Helpers
 * 
 * IMPORTANT: These tests run in WEB MODE by default!
 * - Playwright opens the Vite dev server (http://localhost:5173) in a regular browser
 * - window.__TAURI__ is NOT available (it only exists in the Tauri WebView)
 * - Tests verify UI/UX flows that work in both web and Tauri modes
 * 
 * For TRUE Tauri native testing, you need @tauri-apps/tauri-driver (WebDriver)
 * These Playwright tests focus on UI behavior that works in web mode.
 */

import { Page, Browser, chromium, BrowserContext } from '@playwright/test';
import path from 'path';
import os from 'os';
import fs from 'fs/promises';

export interface TauriTestConfig {
  dataDir?: string;
  cleanup?: boolean;
  timeout?: number;
  baseUrl?: string;
}

export class TauriTestHelper {
  private dataDir: string;
  private cleanup: boolean;
  private baseUrl: string;

  constructor(config: TauriTestConfig = {}) {
    this.dataDir = config.dataDir || path.join(os.tmpdir(), `communitas-test-${Date.now()}`);
    this.cleanup = config.cleanup ?? true;
    this.baseUrl = config.baseUrl || 'http://localhost:5173'; // Tauri dev uses Vite on 5173
  }

  getDataDir(): string {
    return this.dataDir;
  }

  getBaseUrl(): string {
    return this.baseUrl;
  }

  async setupTestEnvironment(): Promise<void> {
    await fs.mkdir(this.dataDir, { recursive: true });
    await fs.mkdir(path.join(this.dataDir, 'vaults'), { recursive: true });
    await fs.mkdir(path.join(this.dataDir, 'storage'), { recursive: true });
    await fs.mkdir(path.join(this.dataDir, 'logs'), { recursive: true });
  }

  async cleanupTestEnvironment(): Promise<void> {
    if (this.cleanup) {
      try {
        await fs.rm(this.dataDir, { recursive: true, force: true });
      } catch (error) {
        console.warn(`Failed to cleanup test directory: ${error}`);
      }
    }
  }

  /**
   * Wait for Tauri to be ready (or continue in web mode)
   * 
   * Note: Playwright tests run in web mode by default. __TAURI__ only exists
   * in the actual Tauri WebView, not when accessing via browser.
   */
  async waitForTauriReady(page: Page, timeout = 10000, required = false): Promise<void> {
    try {
      await page.waitForFunction(
        () => !!(window as any).__TAURI__,
        { timeout: 2000 } // Short timeout since we don't expect it
      );
      console.log('✅ Tauri API detected - running in native Tauri mode');
    } catch (e) {
      console.log('ℹ️  Running in web mode (Playwright → Vite dev server)');
      console.log('   This is expected! Tests verify UI flows in web mode.');
      
      if (required) {
        throw new Error('REQUIRE_TAURI set but __TAURI__ not found. For native Tauri testing, use @tauri-apps/tauri-driver instead of Playwright.');
      }
    }
  }

  /**
   * Check if we're in Tauri mode
   */
  async isTauriMode(page: Page): Promise<boolean> {
    return page.evaluate(() => {
      return !!(window as any).__TAURI__;
    });
  }

  /**
   * Check if we're in dev mode
   */
  async isDevMode(page: Page): Promise<boolean> {
    return page.evaluate(() => {
      return !!(window as any).__TAURI_INTERNALS__?.metadata?.dev;
    });
  }

  /**
   * Invoke Tauri command safely (returns null if not in Tauri mode)
   */
  async invokeCommand<T>(
    page: Page,
    command: string,
    args?: Record<string, any>
  ): Promise<T | null> {
    const isTauri = await this.isTauriMode(page);
    if (!isTauri) {
      console.log(`⚠️  Cannot invoke '${command}' - not in Tauri mode`);
      return null;
    }

    return page.evaluate(
      async ({ cmd, params }) => {
        try {
          const result = await (window as any).__TAURI__.core.invoke(cmd, params);
          return { success: true, data: result };
        } catch (error: any) {
          return { success: false, error: error.message || String(error) };
        }
      },
      { cmd: command, params: args }
    ).then((result: any) => {
      if (!result.success) {
        throw new Error(`Tauri command '${command}' failed: ${result.error}`);
      }
      return result.data;
    });
  }

  /**
   * Wait for Tauri event
   */
  async waitForEvent(
    page: Page,
    eventName: string,
    timeout = 5000
  ): Promise<any> {
    return page.evaluate(
      ({ event, maxTimeout }) => {
        return new Promise((resolve, reject) => {
          const timer = setTimeout(() => {
            reject(new Error(`Timeout waiting for event: ${event}`));
          }, maxTimeout);

          (window as any).__TAURI__.event.once(event, (payload: any) => {
            clearTimeout(timer);
            resolve(payload);
          });
        });
      },
      { event: eventName, maxTimeout: timeout }
    );
  }

  /**
   * Clear app state
   */
  async clearAppState(page: Page): Promise<void> {
    await page.evaluate(() => {
      localStorage.clear();
      sessionStorage.clear();
    });
  }

  /**
   * Generate test four-word identity
   */
  generateTestIdentity(): string {
    const words = ['Test', 'User', 'Demo', 'Sample', 'Ocean', 'Forest', 'Mountain', 'River'];
    const shuffle = () => words.sort(() => Math.random() - 0.5);
    return shuffle().slice(0, 4).join(' ');
  }

  /**
   * Take screenshot for debugging
   */
  async screenshot(
    page: Page,
    name: string,
    fullPage = false
  ): Promise<void> {
    const screenshotDir = path.join(this.dataDir, 'screenshots');
    await fs.mkdir(screenshotDir, { recursive: true });
    
    const filename = `${name}-${Date.now()}.png`;
    await page.screenshot({
      path: path.join(screenshotDir, filename),
      fullPage
    });
    
    console.log(`📸 Screenshot saved: ${filename}`);
  }

  /**
   * Get app health status
   */
  async getHealthStatus(page: Page): Promise<any> {
    return this.invokeCommand(page, 'health');
  }
}

/**
 * Create test identity
 */
export async function createTestIdentity(
  page: Page,
  helper: TauriTestHelper,
  name?: string
): Promise<string> {
  const fourWords = name || helper.generateTestIdentity();
  
  await helper.invokeCommand(page, 'core_claim', {
    words: fourWords.split(' ')
  });
  
  return fourWords;
}

/**
 * Initialize core context
 */
export async function initializeCoreContext(
  page: Page,
  helper: TauriTestHelper
): Promise<void> {
  await helper.invokeCommand(page, 'core_initialize', {
    data_dir: helper.getDataDir()
  });
}

/**
 * Wait for network to be ready
 */
export async function waitForNetworkReady(
  page: Page,
  timeout = 10000
): Promise<void> {
  await page.waitForFunction(
    () => {
      const status = (window as any).__APP_STATE__?.networkStatus;
      return status === 'connected' || status === 'ready';
    },
    { timeout }
  ).catch(() => {
    console.log('ℹ️ Network not ready, continuing anyway');
  });
}

/**
 * Setup fake media devices for WebRTC testing
 */
export async function setupFakeMediaDevices(page: Page): Promise<void> {
  await page.addInitScript(() => {
    // Note: Chromium with --use-fake-device-for-media-stream already provides fake devices
    // This is just for additional control if needed
    
    const originalGetUserMedia = navigator.mediaDevices.getUserMedia.bind(
      navigator.mediaDevices
    );

    (navigator.mediaDevices.getUserMedia as any) = async function(constraints: any) {
      console.log('📹 getUserMedia called with:', constraints);
      return originalGetUserMedia(constraints);
    };
  });
}
