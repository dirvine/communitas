/**
 * UpdateService - Handles Tauri app updates
 *
 * This service provides automatic update checking and installation using
 * Tauri's built-in updater system with GitHub releases.
 */

import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import { getVersion } from '@tauri-apps/api/app';

export interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion?: string;
  body?: string; // Release notes
  date?: string;
}

export class UpdateService {
  private static instance: UpdateService;
  private checkInProgress = false;

  private constructor() {}

  static getInstance(): UpdateService {
    if (!UpdateService.instance) {
      UpdateService.instance = new UpdateService();
    }
    return UpdateService.instance;
  }

  /**
   * Check for updates
   *
   * @returns Update information
   */
  async checkForUpdates(): Promise<UpdateInfo> {
    if (this.checkInProgress) {
      console.log('Update check already in progress');
      return { available: false, currentVersion: 'unknown' };
    }

    this.checkInProgress = true;

    try {
      console.log('Checking for updates...');

      const update = await check();

      if (update) {
        console.log(
          `Update available: ${update.currentVersion} -> ${update.version}`
        );

        return {
          available: true,
          currentVersion: update.currentVersion,
          latestVersion: update.version,
          body: update.body,
          date: update.date,
        };
      }

      console.log('No updates available');
      const currentVersion = await getVersion();
      return {
        available: false,
        currentVersion: currentVersion || 'unknown',
      };
    } catch (error) {
      console.error('Failed to check for updates:', error);
      throw error;
    } finally {
      this.checkInProgress = false;
    }
  }

  /**
   * Download and install update
   *
   * This will download the update, install it, and prompt for restart.
   * The installation happens in the background.
   *
   * @param onProgress - Optional callback for download progress
   */
  async installUpdate(
    onProgress?: (downloaded: number, total: number) => void
  ): Promise<void> {
    try {
      console.log('Checking for updates to install...');
      const update = await check();

      if (!update) {
        console.log('No updates available to install');
        return;
      }

      console.log('Downloading and installing update...');

      // Download and install - this returns once download is complete
      let downloaded = 0;
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          downloaded = 0;
          console.log(`Download started: ${event.data.contentLength || 0} bytes`);
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength ?? 0;
          const total = (event.data as any).contentLength ?? 0;
          if (onProgress && total > 0) {
            onProgress(downloaded, total);
          }
        } else if (event.event === 'Finished') {
          console.log('Update download and installation complete');
        }
      });

      console.log('Update installed successfully');
    } catch (error) {
      console.error('Failed to install update:', error);
      throw error;
    }
  }

  /**
   * Restart the application to apply updates
   *
   * This should only be called after successfully installing an update.
   */
  async restartApp(): Promise<void> {
    try {
      console.log('Restarting application to apply update...');
      await relaunch();
    } catch (error) {
      console.error('Failed to restart application:', error);
      throw error;
    }
  }

  /**
   * Check for updates on startup
   *
   * This is a convenience method that can be called during app initialization
   * to automatically check for updates.
   *
   * @param autoInstall - If true, automatically download and install updates
   * @returns True if update was found (and installed if autoInstall=true)
   */
  async checkOnStartup(autoInstall = false): Promise<boolean> {
    try {
      const info = await this.checkForUpdates();

      if (info.available) {
        console.log(
          `Update available: ${info.currentVersion} -> ${info.latestVersion}`
        );

        if (autoInstall) {
          await this.installUpdate();
          console.log('Update installed. Please restart the application.');
          return true;
        }

        return true;
      }

      console.log('No updates available');
      return false;
    } catch (error) {
      console.warn('Failed to check for updates on startup:', error);
      return false;
    }
  }
}

// Singleton instance
export const updateService = UpdateService.getInstance();
