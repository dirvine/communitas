import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for testing Communitas Tauri application
 * Connects via Chrome DevTools Protocol for integration testing
 */
export default defineConfig({
  testDir: './tests/integration',
  fullyParallel: false, // Run tests sequentially for network testing
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker for Tauri app testing
  reporter: 'html',

  use: {
    // Connect to Tauri app via CDP
    baseURL: 'http://localhost:5173', // Vite dev server
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'tauri-app',
      use: {
        ...devices['Desktop Chrome'],
        // Connect to existing Tauri instance via CDP
        launchOptions: {
          args: ['--remote-debugging-port=9222'],
        },
        // Or connect to existing browser
        connectOptions: {
          wsEndpoint: 'ws://localhost:9222/devtools/browser',
        },
      },
    },
  ],

  // Launch Tauri app before tests
  webServer: {
    command: 'cd .. && ./communitas-desktop/launch-with-debug.sh',
    port: 5173,
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // 2 minutes for Tauri to start
    stdout: 'pipe',
    stderr: 'pipe',
  },
});