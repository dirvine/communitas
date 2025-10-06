// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

/**
 * Connection Service - Manages P2P network connection status
 *
 * Provides TypeScript bindings for gossip overlay connection management.
 * Shows user's identity, online/offline status, and bootstrap peer management.
 */

import { invoke } from '@tauri-apps/api/core';

export interface ConnectionStatus {
  online: boolean;
  four_words: string;
  peer_count: number;
}

export interface BootstrapPeer {
  four_words: string;
  peer_id: string;
  last_seen: number;
  success_rate: number;
}

export class ConnectionService {
  /**
   * Get user's own four-word identity
   *
   * @returns Four-word address (e.g., "ocean-forest-moon-star")
   *
   * @example
   * ```typescript
   * const identity = await connectionService.getOwnIdentity();
   * console.log('My identity:', identity);
   * ```
   */
  async getOwnIdentity(): Promise<string> {
    try {
      return await invoke<string>('gossip_get_own_identity');
    } catch (error) {
      throw new Error(`Failed to get identity: ${error}`);
    }
  }

  /**
   * Get current connection status
   *
   * @returns Connection status with online/offline state and peer count
   *
   * @example
   * ```typescript
   * const status = await connectionService.getStatus();
   * console.log(`Status: ${status.online ? 'Online' : 'Offline'}`);
   * console.log(`Connected to ${status.peer_count} peers`);
   * ```
   */
  async getStatus(): Promise<ConnectionStatus> {
    try {
      return await invoke<ConnectionStatus>('gossip_get_connection_status');
    } catch (error) {
      throw new Error(`Failed to get connection status: ${error}`);
    }
  }

  /**
   * Add friend's four-word identity for bootstrap
   *
   * Allows connecting to the network via a known peer.
   * Useful for:
   * - Initial bootstrap when first joining
   * - Reconnecting after being offline
   * - Building a trusted peer list
   *
   * @param fourWords - Friend's four-word address
   *
   * @example
   * ```typescript
   * await connectionService.addBootstrapPeer('alpha-bravo-charlie-delta');
   * ```
   */
  async addBootstrapPeer(fourWords: string): Promise<void> {
    try {
      await invoke('gossip_add_bootstrap_peer', { four_words: fourWords });
    } catch (error) {
      throw new Error(`Failed to add bootstrap peer: ${error}`);
    }
  }

  /**
   * Get list of known contacts/peers
   *
   * Returns all cached peers that can be used for bootstrap.
   * Peers are sorted by connection quality (success rate).
   *
   * @returns Array of known peers with their metadata
   *
   * @example
   * ```typescript
   * const peers = await connectionService.getCachedPeers();
   * peers.forEach(p => {
   *   console.log(`${p.four_words}: ${(p.success_rate * 100).toFixed(0)}% success`);
   * });
   * ```
   */
  async getCachedPeers(): Promise<BootstrapPeer[]> {
    try {
      return await invoke<BootstrapPeer[]>('gossip_get_cached_peers');
    } catch (error) {
      throw new Error(`Failed to get cached peers: ${error}`);
    }
  }

  /**
   * Format last seen timestamp as human-readable string
   *
   * @param timestamp - Unix timestamp in seconds
   * @returns Human-readable time ago string
   */
  static formatLastSeen(timestamp: number): string {
    if (timestamp === 0) return 'Never';

    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;

    if (diff < 60) return 'Just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  /**
   * Get connection quality indicator (0-100)
   *
   * @param peerCount - Number of connected peers
   * @returns Quality score: 0 (offline) to 100 (excellent)
   */
  static getConnectionQuality(peerCount: number): number {
    if (peerCount === 0) return 0;
    if (peerCount === 1) return 40;
    if (peerCount <= 3) return 60;
    if (peerCount <= 5) return 80;
    return 100;
  }

  /**
   * Get connection status color
   *
   * @param online - Whether currently online
   * @param peerCount - Number of connected peers
   * @returns Color: 'success' (green), 'warning' (yellow), or 'error' (red)
   */
  static getStatusColor(online: boolean, peerCount: number): 'success' | 'warning' | 'error' {
    if (!online || peerCount === 0) return 'error';
    if (peerCount <= 2) return 'warning';
    return 'success';
  }
}

// Singleton instance
export const connectionService = new ConnectionService();
