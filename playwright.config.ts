import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for Communitas Tauri App E2E testing
 *
 * Supports both web mode testing and native Tauri app testing:
 * - Web mode: Test the React app in browsers (existing setup)
 * - Tauri mode: Test the packaged native application
 */
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false, // Sequential for Tauri testing
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 1,
  workers: process.env.CI ? 1 : 1, // Single worker for stability
  reporter: process.env.CI ? 'github' : 'html',

  use: {
    // Base URL for web mode testing
    baseURL: process.env.TAURI_MODE ? undefined : 'http://localhost:1420',

    // Collect trace when retrying the failed test
    trace: 'on-first-retry',

    // Take screenshot on failure
    screenshot: 'only-on-failure',

    // Video on failure
    video: 'retain-on-failure',

    // Timeout settings for Tauri app
    actionTimeout: 10000,
    navigationTimeout: 30000,
  },

  // Configure projects for different testing modes
  projects: [
    // Web Mode Testing (existing React app in browsers)
    {
      name: 'web-chromium',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: 'http://localhost:1420'
      },
      testMatch: '**/web-mode/**/*.spec.ts',
    },
    {
      name: 'web-firefox',
      use: {
        ...devices['Desktop Firefox'],
        baseURL: 'http://localhost:1420'
      },
      testMatch: '**/web-mode/**/*.spec.ts',
    },
    {
      name: 'web-webkit',
      use: {
        ...devices['Desktop Safari'],
        baseURL: 'http://localhost:1420'
      },
      testMatch: '**/web-mode/**/*.spec.ts',
    },

    // Native Tauri App Testing
    {
      name: 'tauri-native',
      use: {
        // Custom Tauri configuration will be handled in test fixtures
      },
      testMatch: '**/tauri-mode/**/*.spec.ts',
    },

    // Mobile web testing
    {
      name: 'mobile-chrome',
      use: {
        ...devices['Pixel 5'],
        baseURL: 'http://localhost:1420'
      },
      testMatch: '**/web-mode/**/*.spec.ts',
    },
  ],

  // Web server for web mode testing
  webServer: process.env.TAURI_MODE ? undefined : {
    command: 'npm run dev:browser',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },

  // Global setup for Tauri testing
  globalSetup: process.env.TAURI_MODE ? './tests/utils/tauri-setup.ts' : undefined,

  // Test timeout for Tauri operations
  timeout: 60000, // 60 seconds for Tauri app startup
});