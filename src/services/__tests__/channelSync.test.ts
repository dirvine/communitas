import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { channelSyncService } from '../channelSyncService';
import { channelService } from '../channelService';

// Mock the channelService
vi.mock('../channelService', () => ({
  channelService: {
    getChannelStateVector: vi.fn(),
    getChannelDiff: vi.fn(),
    applyChannelDiff: vi.fn(),
  },
}));

describe('ChannelSyncService', () => {
  const mockChannelId = 'channel-123';
  const mockStateVector = new Uint8Array([1, 2, 3, 4]);
  const mockDiff = new Uint8Array([5, 6, 7, 8]);

  beforeEach(() => {
    // Clear all mocks
    vi.clearAllMocks();

    // Clear service state between tests
    channelSyncService.clearAll();

    // Setup default mock implementations
    vi.mocked(channelService.getChannelStateVector).mockResolvedValue(
      mockStateVector
    );
    vi.mocked(channelService.getChannelDiff).mockResolvedValue(mockDiff);
    vi.mocked(channelService.applyChannelDiff).mockResolvedValue({
      messages_updated: 1,
      total_messages: 10,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('getLocalStateVector', () => {
    it('should return state vector from channelService', async () => {
      const stateVector =
        await channelSyncService.getLocalStateVector(mockChannelId);

      expect(stateVector).toEqual(mockStateVector);
      expect(channelService.getChannelStateVector).toHaveBeenCalledWith(
        mockChannelId
      );
    });

    it('should propagate errors from backend', async () => {
      const error = new Error('Backend error');
      vi.mocked(channelService.getChannelStateVector).mockRejectedValue(error);

      await expect(
        channelSyncService.getLocalStateVector(mockChannelId)
      ).rejects.toThrow('Backend error');
    });
  });

  describe('generateDiffForPeer', () => {
    it('should generate diff using channelService', async () => {
      const peerStateVector = new Uint8Array([9, 10, 11, 12]);

      const diff = await channelSyncService.generateDiffForPeer(
        mockChannelId,
        peerStateVector
      );

      expect(diff).toEqual(mockDiff);
      expect(channelService.getChannelDiff).toHaveBeenCalledWith(
        mockChannelId,
        peerStateVector
      );
    });

    it('should return empty diff when no changes', async () => {
      vi.mocked(channelService.getChannelDiff).mockResolvedValue(
        new Uint8Array([])
      );

      const diff = await channelSyncService.generateDiffForPeer(
        mockChannelId,
        mockStateVector
      );

      expect(diff.length).toBe(0);
    });

    it('should handle large diff generation', async () => {
      const largeDiff = new Uint8Array(1024 * 1024); // 1MB
      vi.mocked(channelService.getChannelDiff).mockResolvedValue(largeDiff);

      const diff = await channelSyncService.generateDiffForPeer(
        mockChannelId,
        mockStateVector
      );

      expect(diff.length).toBe(1024 * 1024);
    });
  });

  describe('applyDiffFromPeer', () => {
    it('should apply diff successfully', async () => {
      const result = await channelSyncService.applyDiffFromPeer(
        mockChannelId,
        mockDiff
      );

      expect(result.messages_updated).toBe(1);
      expect(result.total_messages).toBe(10);
      expect(channelService.applyChannelDiff).toHaveBeenCalledWith(
        mockChannelId,
        mockDiff
      );
    });

    it('should update sync state after applying diff', async () => {
      await channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff);

      const syncState = channelSyncService.getSyncState(mockChannelId);
      expect(syncState?.last_sync_at).toBeGreaterThan(0);
      expect(syncState?.state_vector).toEqual(mockStateVector);
    });

    it('should throw error when apply fails', async () => {
      const error = new Error('Invalid diff format');
      vi.mocked(channelService.applyChannelDiff).mockRejectedValue(error);

      await expect(
        channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff)
      ).rejects.toThrow('Invalid diff format');
    });

    it('should handle empty diff', async () => {
      vi.mocked(channelService.applyChannelDiff).mockResolvedValue({
        messages_updated: 0,
        total_messages: 5,
      });

      const result = await channelSyncService.applyDiffFromPeer(
        mockChannelId,
        new Uint8Array([])
      );

      expect(result.messages_updated).toBe(0);
    });
  });

  describe('syncWithPeer', () => {
    const peerStateVector = new Uint8Array([1, 2, 3]);

    it('should successfully sync with a peer', async () => {
      const result = await channelSyncService.syncWithPeer(
        mockChannelId,
        peerStateVector
      );

      expect(result.messages_updated).toBe(1);
      expect(result.total_messages).toBe(10);
      expect(channelService.getChannelDiff).toHaveBeenCalledWith(
        mockChannelId,
        peerStateVector
      );
      expect(channelService.applyChannelDiff).toHaveBeenCalledWith(
        mockChannelId,
        mockDiff
      );
    });

    it('should prevent concurrent syncs for the same channel', async () => {
      // Mock slow operation
      vi.mocked(channelService.getChannelDiff).mockImplementation(
        () =>
          new Promise((resolve) =>
            setTimeout(() => resolve(mockDiff), 100)
          ) as Promise<Uint8Array>
      );

      // Start first sync
      const firstSync = channelSyncService.syncWithPeer(
        mockChannelId,
        peerStateVector
      );

      // Try to start second sync immediately
      await expect(
        channelSyncService.syncWithPeer(mockChannelId, peerStateVector)
      ).rejects.toThrow('Sync already in progress');

      // Wait for first sync to complete
      await firstSync;
    });

    it('should reset sync_in_progress flag on error', async () => {
      vi.mocked(channelService.getChannelDiff).mockRejectedValueOnce(
        new Error('Network error')
      );

      await expect(
        channelSyncService.syncWithPeer(mockChannelId, peerStateVector)
      ).rejects.toThrow('Network error');

      // Verify flag was reset
      const state = channelSyncService.getSyncState(mockChannelId);
      expect(state?.sync_in_progress).toBe(false);

      // Should allow retry
      vi.mocked(channelService.getChannelDiff).mockResolvedValue(mockDiff);
      const result = await channelSyncService.syncWithPeer(
        mockChannelId,
        peerStateVector
      );
      expect(result).toBeDefined();
    });

    it('should update lastSyncTimestamp on successful sync', async () => {
      const beforeSync = Date.now();

      await channelSyncService.syncWithPeer(mockChannelId, peerStateVector);

      const syncState = channelSyncService.getSyncState(mockChannelId);
      expect(syncState?.last_sync_at).toBeGreaterThanOrEqual(beforeSync);
      expect(syncState?.last_sync_at).toBeLessThanOrEqual(Date.now());
    });

    it('should update state vector after sync', async () => {
      await channelSyncService.syncWithPeer(mockChannelId, peerStateVector);

      const syncState = channelSyncService.getSyncState(mockChannelId);
      expect(syncState?.state_vector).toEqual(mockStateVector);
    });
  });

  describe('getSyncState', () => {
    it('should return undefined for unknown channel', () => {
      const state = channelSyncService.getSyncState('unknown-channel');
      expect(state).toBeUndefined();
    });

    it('should return state after sync', async () => {
      await channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff);

      const state = channelSyncService.getSyncState(mockChannelId);
      expect(state?.channel_id).toBe(mockChannelId);
      expect(state?.last_sync_at).toBeGreaterThan(0);
      expect(state?.state_vector).toEqual(mockStateVector);
    });

    it('should return same state object on repeated calls', async () => {
      await channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff);

      const state1 = channelSyncService.getSyncState(mockChannelId);
      const state2 = channelSyncService.getSyncState(mockChannelId);

      expect(state1).toBe(state2);
    });

    it('should track multiple channels independently', async () => {
      const channel1 = 'channel-1';
      const channel2 = 'channel-2';

      await channelSyncService.applyDiffFromPeer(channel1, mockDiff);
      await channelSyncService.applyDiffFromPeer(channel2, mockDiff);

      const state1 = channelSyncService.getSyncState(channel1);
      const state2 = channelSyncService.getSyncState(channel2);

      expect(state1?.channel_id).toBe(channel1);
      expect(state2?.channel_id).toBe(channel2);
      expect(state1).not.toBe(state2);
    });
  });

  describe('clearAll', () => {
    it('should clear all sync state', async () => {
      // Create state for multiple channels
      await channelSyncService.applyDiffFromPeer('channel-1', mockDiff);
      await channelSyncService.applyDiffFromPeer('channel-2', mockDiff);

      channelSyncService.clearAll();

      // States should now be undefined
      const state1 = channelSyncService.getSyncState('channel-1');
      const state2 = channelSyncService.getSyncState('channel-2');

      expect(state1).toBeUndefined();
      expect(state2).toBeUndefined();
    });

    it('should allow syncing after clearAll', async () => {
      await channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff);
      channelSyncService.clearAll();

      const result = await channelSyncService.applyDiffFromPeer(
        mockChannelId,
        mockDiff
      );
      expect(result.messages_updated).toBe(1);
    });
  });

  describe('error handling', () => {
    it('should handle getChannelStateVector errors', async () => {
      vi.mocked(channelService.getChannelStateVector).mockRejectedValue(
        new Error('State vector error')
      );

      await expect(
        channelSyncService.getLocalStateVector(mockChannelId)
      ).rejects.toThrow('State vector error');
    });

    it('should handle getChannelDiff errors', async () => {
      vi.mocked(channelService.getChannelDiff).mockRejectedValue(
        new Error('Diff generation error')
      );

      await expect(
        channelSyncService.generateDiffForPeer(mockChannelId, mockStateVector)
      ).rejects.toThrow('Diff generation error');
    });

    it('should handle applyChannelDiff errors', async () => {
      vi.mocked(channelService.applyChannelDiff).mockRejectedValue(
        new Error('Apply diff error')
      );

      await expect(
        channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff)
      ).rejects.toThrow('Apply diff error');
    });
  });

  describe('timestamp management', () => {
    it('should set timestamp on first sync', async () => {
      const before = Date.now();

      await channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff);

      const state = channelSyncService.getSyncState(mockChannelId);
      expect(state?.last_sync_at).toBeGreaterThanOrEqual(before);
      expect(state?.last_sync_at).toBeLessThanOrEqual(Date.now());
    });

    it('should update timestamp on subsequent syncs', async () => {
      await channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff);
      const timestamp1 =
        channelSyncService.getSyncState(mockChannelId)?.last_sync_at;

      // Wait a bit to ensure different timestamp
      await new Promise((resolve) => setTimeout(resolve, 10));

      await channelSyncService.applyDiffFromPeer(mockChannelId, mockDiff);
      const timestamp2 =
        channelSyncService.getSyncState(mockChannelId)?.last_sync_at;

      expect(timestamp2).toBeGreaterThan(timestamp1!);
    });
  });
});
