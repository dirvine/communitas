// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

/**
 * Saorsa Sites Service - DNS-free website publishing
 *
 * Provides TypeScript bindings for Saorsa Sites Tauri commands.
 * Enables publishing and fetching content-addressed websites via
 * the rendezvous protocol (SPEC2.md §5).
 */

import { invoke } from '@tauri-apps/api/core';

export interface AssetData {
  path: string;
  content_base64: string;
}

export interface SiteData {
  site_id: string;
  assets: AssetData[];
}

export class SitesService {
  /**
   * Publish a site with multiple assets
   *
   * @param assets - Array of assets with paths and base64-encoded content
   * @returns Site ID as hex string
   *
   * @example
   * ```typescript
   * const assets = [
   *   { path: 'index.html', content_base64: btoa('<html>...</html>') },
   *   { path: 'style.css', content_base64: btoa('body { ... }') }
   * ];
   * const siteId = await sitesService.publish(assets);
   * console.log('Published site:', siteId);
   * ```
   */
  async publish(assets: AssetData[]): Promise<string> {
    try {
      const siteId = await invoke<string>('gossip_site_publish', { assets });
      return siteId;
    } catch (error) {
      throw new Error(`Failed to publish site: ${error}`);
    }
  }

  /**
   * Fetch a site from the network
   *
   * @param siteIdHex - Site ID as hex string (from publish() or discovery)
   * @returns Site data with all assets
   *
   * @example
   * ```typescript
   * const site = await sitesService.fetch('a1b2c3...');
   * for (const asset of site.assets) {
   *   const content = atob(asset.content_base64);
   *   console.log(`${asset.path}: ${content.length} bytes`);
   * }
   * ```
   */
  async fetch(siteIdHex: string): Promise<SiteData> {
    try {
      const site = await invoke<SiteData>('gossip_site_fetch', {
        site_id_hex: siteIdHex
      });
      return site;
    } catch (error) {
      throw new Error(`Failed to fetch site: ${error}`);
    }
  }

  /**
   * List all published sites
   *
   * @returns Array of site IDs as hex strings
   *
   * @example
   * ```typescript
   * const sites = await sitesService.list();
   * console.log(`Found ${sites.length} sites`);
   * ```
   */
  async list(): Promise<string[]> {
    try {
      const sites = await invoke<string[]>('gossip_site_list');
      return sites;
    } catch (error) {
      throw new Error(`Failed to list sites: ${error}`);
    }
  }

  /**
   * Get providers for a site
   *
   * @param siteIdHex - Site ID as hex string
   * @returns Array of provider peer IDs as hex strings
   *
   * @example
   * ```typescript
   * const providers = await sitesService.getProviders('a1b2c3...');
   * console.log(`Found ${providers.length} providers`);
   * ```
   */
  async getProviders(siteIdHex: string): Promise<string[]> {
    try {
      const providers = await invoke<string[]>('gossip_site_providers', {
        site_id_hex: siteIdHex
      });
      return providers;
    } catch (error) {
      throw new Error(`Failed to get providers: ${error}`);
    }
  }

  /**
   * Helper: Create AssetData from File
   *
   * @param path - Asset path in site (e.g., 'index.html')
   * @param file - File object from input or fetch
   * @returns Promise resolving to AssetData
   *
   * @example
   * ```typescript
   * const fileInput = document.querySelector('input[type="file"]');
   * const file = fileInput.files[0];
   * const asset = await SitesService.fromFile('index.html', file);
   * ```
   */
  static async fromFile(path: string, file: File): Promise<AssetData> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const arrayBuffer = reader.result as ArrayBuffer;
        const bytes = new Uint8Array(arrayBuffer);
        const base64 = btoa(String.fromCharCode(...bytes));
        resolve({ path, content_base64: base64 });
      };
      reader.onerror = () => reject(reader.error);
      reader.readAsArrayBuffer(file);
    });
  }

  /**
   * Helper: Create AssetData from string content
   *
   * @param path - Asset path in site
   * @param content - String content
   * @returns AssetData with base64-encoded content
   *
   * @example
   * ```typescript
   * const asset = SitesService.fromString('index.html', '<html>...</html>');
   * ```
   */
  static fromString(path: string, content: string): AssetData {
    const base64 = btoa(content);
    return { path, content_base64: base64 };
  }

  /**
   * Helper: Decode asset content to string
   *
   * @param asset - AssetData with base64 content
   * @returns Decoded string content
   *
   * @example
   * ```typescript
   * const html = SitesService.toString(asset);
   * console.log(html);
   * ```
   */
  static toString(asset: AssetData): string {
    return atob(asset.content_base64);
  }

  /**
   * Helper: Decode asset content to Uint8Array
   *
   * @param asset - AssetData with base64 content
   * @returns Decoded byte array
   *
   * @example
   * ```typescript
   * const bytes = SitesService.toBytes(asset);
   * const blob = new Blob([bytes]);
   * ```
   */
  static toBytes(asset: AssetData): Uint8Array {
    const binary = atob(asset.content_base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  }
}

// Singleton instance
export const sitesService = new SitesService();
