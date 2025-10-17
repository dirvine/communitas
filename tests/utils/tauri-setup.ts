/**
 * Tauri App Testing Setup
 *
 * This module handles setup and teardown for testing the native Tauri application.
 * It manages app lifecycle, data directories, and cleanup.
 */

import { execSync, spawn } from 'child_process';
import { promises as fs } from 'fs';
import { existsSync } from 'fs';
import path from 'path';
import { promisify } from 'util';

const exec = promisify(execSync);

export interface TauriTestConfig {
  appPath?: string;
  dataDir: string;
  cleanup: boolean;
  timeout: number;
}

export class TauriTestEnvironment {
  private config: TauriTestConfig;
  private appProcess?: any;
  private dataDir: string;

  constructor(config: Partial<TauriTestConfig> = {}) {
    this.config = {
      appPath: config.appPath || this.findTauriApp(),
      dataDir: config.dataDir || path.join(process.cwd(), 'test-data'),
      cleanup: config.cleanup ?? true,
      timeout: config.timeout || 30000,
      ...config
    };

    // Create unique data directory for this test run
    const timestamp = Date.now();
    const randomId = Math.random().toString(36).substr(2, 9);
    this.dataDir = path.join(this.config.dataDir, `tauri-test-${timestamp}-${randomId}`);
  }

  /**
   * Find the built Tauri application
   */
  private findTauriApp(): string {
    const platform = process.platform;
    let appPath: string;

    if (platform === 'darwin') {
      appPath = path.join(process.cwd(), 'src-tauri/target/release/bundle/macos/Communitas.app');
    } else if (platform === 'win32') {
      appPath = path.join(process.cwd(), 'src-tauri/target/release/bundle/msi/Communitas.msi');
    } else {
      appPath = path.join(process.cwd(), 'src-tauri/target/release/bundle/appimage/Communitas.AppImage');
    }

    if (!existsSync(appPath)) {
      throw new Error(`Tauri app not found at ${appPath}. Run 'npm run tauri:build' first.`);
    }

    return appPath;
  }

  /**
   * Start the Tauri application
   */
  async startApp(): Promise<void> {
    // Create clean data directory
    await fs.mkdir(this.dataDir, { recursive: true });

    // Set environment variables for the app
    const env = {
      ...process.env,
      COMMUNITAS_DATA_DIR: this.dataDir,
      RUST_LOG: 'info,communitas=debug',
      NODE_ENV: 'test'
    };

    console.log(`🚀 Starting Tauri app from: ${this.config.appPath}`);
    console.log(`📁 Using data directory: ${this.dataDir}`);

    // Start the app process
    if (process.platform === 'darwin') {
      // macOS .app bundle
      this.appProcess = spawn('open', ['-W', '-n', this.config.appPath], { env });
    } else if (process.platform === 'win32') {
      // Windows executable
      const exePath = path.join(path.dirname(this.config.appPath), 'Communitas.exe');
      this.appProcess = spawn(exePath, [], { env });
    } else {
      // Linux AppImage
      this.appProcess = spawn(this.config.appPath, [], { env });
    }

    // Wait for app to start (basic health check)
    await this.waitForAppReady();

    console.log('✅ Tauri app started successfully');
  }

  /**
   * Wait for the Tauri app to be ready
   */
  private async waitForAppReady(): Promise<void> {
    // Simple wait - in a real implementation, you might:
    // 1. Poll a health endpoint
    // 2. Check for a specific window
    // 3. Wait for a specific log message
    await new Promise(resolve => setTimeout(resolve, 5000));
  }

  /**
   * Stop the Tauri application
   */
  async stopApp(): Promise<void> {
    if (this.appProcess) {
      console.log('🛑 Stopping Tauri app');

      if (process.platform === 'darwin') {
        // Kill the app by bundle ID or process name
        try {
          execSync('pkill -f Communitas');
        } catch (error) {
          // Ignore if process not found
        }
      } else {
        this.appProcess.kill('SIGTERM');
      }

      // Wait for cleanup
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  }

  /**
   * Clean up test data
   */
  async cleanup(): Promise<void> {
    if (this.config.cleanup) {
      console.log(`🧹 Cleaning up test data: ${this.dataDir}`);
      try {
        await fs.rm(this.dataDir, { recursive: true, force: true });
      } catch (error) {
        console.warn(`Failed to cleanup ${this.dataDir}:`, error);
      }
    }
  }

  /**
   * Get the app's WebSocket URL for testing
   */
  getAppUrl(): string {
    // In a real implementation, you'd get this from the app's logs or config
    // For now, return a placeholder
    return 'ws://localhost:3000';
  }

  /**
   * Get the data directory path
   */
  getDataDir(): string {
    return this.dataDir;
  }
}

// Global setup function for Playwright
export default async function globalSetup(): Promise<void> {
  console.log('🔧 Setting up Tauri test environment');

  const testEnv = new TauriTestEnvironment({
    cleanup: !process.env.KEEP_TEST_DATA,
  });

  // Store the test environment in global state for tests to use
  (global as any).tauriTestEnv = testEnv;

  // Note: We don't start the app here as Playwright will handle test-specific setup
  // Individual tests will start/stop the app as needed

  console.log('✅ Tauri test environment setup complete');
}
