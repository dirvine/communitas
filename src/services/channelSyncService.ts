import { channelService } from './channelService';
import type { AppliedDiffResult, ChannelSyncState } from '../types/channels';

/**
 * ChannelSyncService - Orchestrates efficient CRDT synchronization
 *
 * Phase 3 Sync Protocol:
 * 1. Exchange state vectors between peers
 * 2. Generate diffs containing only missing updates
 * 3. Apply diffs and materialize to SQL
 *
 * This is much more efficient than the old protocol which always
 * sent the full CRDT state.
 */
export class ChannelSyncService {
  private syncStates = new Map<string, ChannelSyncState>();

  /**
   * Perform full sync with a remote peer
   *
   * @param channelId - Channel to sync
   * @param remoteStateVector - State vector from remote peer
   * @returns Result of applying the diff
   */
  async syncWithPeer(
    channelId: string,
    remoteStateVector: Uint8Array
  ): Promise<AppliedDiffResult> {
    const state = this.getOrCreateState(channelId);

    if (state.sync_in_progress) {
      throw new Error(`Sync already in progress for channel ${channelId}`);
    }

    try {
      state.sync_in_progress = true;

      // Generate diff containing only updates missing from remote
      const diff = await channelService.getChannelDiff(
        channelId,
        remoteStateVector
      );

      // Apply the diff locally (if we received updates from remote)
      // Note: In a real P2P scenario, we'd send our diff to the remote peer
      // and they would call this. For testing, we can apply locally.
      const result = await channelService.applyChannelDiff(channelId, diff);

      // Update sync state
      state.last_sync_at = Date.now();
      state.state_vector = await channelService.getChannelStateVector(
        channelId
      );

      return result;
    } finally {
      state.sync_in_progress = false;
    }
  }

  /**
   * Get our current state vector for a channel
   * Remote peer uses this to determine what updates we're missing
   */
  async getLocalStateVector(channelId: string): Promise<Uint8Array> {
    return channelService.getChannelStateVector(channelId);
  }

  /**
   * Generate diff for remote peer
   * Contains only updates missing from their state vector
   */
  async generateDiffForPeer(
    channelId: string,
    remoteStateVector: Uint8Array
  ): Promise<Uint8Array> {
    return channelService.getChannelDiff(channelId, remoteStateVector);
  }

  /**
   * Apply diff received from remote peer
   */
  async applyDiffFromPeer(
    channelId: string,
    diff: Uint8Array
  ): Promise<AppliedDiffResult> {
    const state = this.getOrCreateState(channelId);

    const result = await channelService.applyChannelDiff(channelId, diff);

    // Update local state after successful apply
    state.last_sync_at = Date.now();
    state.state_vector = await channelService.getChannelStateVector(channelId);

    return result;
  }

  /**
   * Get sync state for a channel
   */
  getSyncState(channelId: string): ChannelSyncState | undefined {
    return this.syncStates.get(channelId);
  }

  /**
   * Clear all sync state (for testing)
   */
  clearAll(): void {
    this.syncStates.clear();
  }

  private getOrCreateState(channelId: string): ChannelSyncState {
    let state = this.syncStates.get(channelId);
    if (!state) {
      state = {
        channel_id: channelId,
        last_sync_at: 0,
        state_vector: null,
        sync_in_progress: false,
      };
      this.syncStates.set(channelId, state);
    }
    return state;
  }
}

// Singleton instance
export const channelSyncService = new ChannelSyncService();
