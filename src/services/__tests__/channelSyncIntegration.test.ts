import { describe, it, expect, beforeEach, vi } from 'vitest';
import { channelSyncService } from '../channelSyncService';
import { channelService } from '../channelService';
import type { AppliedDiffResult } from '../../types/channels';

vi.mock('../channelService');

describe('ChannelSync Integration', () => {
  beforeEach(() => {
    channelSyncService.clearAll();
    vi.clearAllMocks();
  });

  it('should complete two-peer sync simulation', async () => {
    const channelId = 'test-channel';

    // Peer A's state vector
    const peerAStateVector = new Uint8Array([1, 2, 3, 4]);
    // Peer B's state vector
    const peerBStateVector = new Uint8Array([1, 2, 3]); // Missing update 4

    // Mock Peer A operations
    vi.mocked(channelService.getChannelStateVector)
      .mockResolvedValueOnce(peerAStateVector) // Peer A gets their state
      .mockResolvedValueOnce(new Uint8Array([1, 2, 3, 4, 5])); // After sync

    // Peer A generates diff for Peer B (containing update 4)
    const diffForPeerB = new Uint8Array([100, 101, 102]);
    vi.mocked(channelService.getChannelDiff).mockResolvedValueOnce(
      diffForPeerB
    );

    // Peer B applies the diff
    const applyResult: AppliedDiffResult = {
      messages_updated: 1,
      total_messages: 10,
    };
    vi.mocked(channelService.applyChannelDiff).mockResolvedValueOnce(
      applyResult
    );

    // Execute sync flow
    // 1. Peer A gets their state vector
    const stateVectorA = await channelSyncService.getLocalStateVector(
      channelId
    );
    expect(stateVectorA).toEqual(peerAStateVector);

    // 2. Peer A generates diff for Peer B
    const diff = await channelSyncService.generateDiffForPeer(
      channelId,
      peerBStateVector
    );
    expect(diff).toEqual(diffForPeerB);

    // 3. Peer B applies the diff
    const result = await channelSyncService.applyDiffFromPeer(channelId, diff);
    expect(result.messages_updated).toBe(1);
    expect(result.total_messages).toBe(10);

    // Verify all mocks were called correctly
    expect(channelService.getChannelStateVector).toHaveBeenCalledTimes(2);
    expect(channelService.getChannelDiff).toHaveBeenCalledWith(
      channelId,
      peerBStateVector
    );
    expect(channelService.applyChannelDiff).toHaveBeenCalledWith(
      channelId,
      diffForPeerB
    );
  });

  it('should handle sync with no updates needed', async () => {
    const channelId = 'test-channel';
    const sameStateVector = new Uint8Array([1, 2, 3, 4]);

    // Both peers have same state - diff is empty
    vi.mocked(channelService.getChannelDiff).mockResolvedValueOnce(
      new Uint8Array([])
    );

    vi.mocked(channelService.applyChannelDiff).mockResolvedValueOnce({
      messages_updated: 0,
      total_messages: 5,
    });

    vi.mocked(channelService.getChannelStateVector).mockResolvedValueOnce(
      sameStateVector
    );

    const diff = await channelSyncService.generateDiffForPeer(
      channelId,
      sameStateVector
    );
    expect(diff.length).toBe(0);

    const result = await channelSyncService.applyDiffFromPeer(channelId, diff);
    expect(result.messages_updated).toBe(0);
    expect(result.total_messages).toBe(5);

    // Verify mocks
    expect(channelService.getChannelDiff).toHaveBeenCalledWith(
      channelId,
      sameStateVector
    );
    expect(channelService.applyChannelDiff).toHaveBeenCalledWith(
      channelId,
      new Uint8Array([])
    );
  });

  it('should handle bidirectional sync where both peers have unique updates', async () => {
    const channelId = 'test-channel';

    // Peer A has [1, 2, 3, 4] (unique update 4)
    const peerAStateVector = new Uint8Array([1, 2, 3, 4]);
    // Peer B has [1, 2, 3, 5] (unique update 5)
    const peerBStateVector = new Uint8Array([1, 2, 3, 5]);

    // Final state both peers should have
    const finalStateVector = new Uint8Array([1, 2, 3, 4, 5]);

    // Mock Peer A generating diff for Peer B (sends update 4)
    const diffAtoB = new Uint8Array([100, 101]); // Update 4 encoded
    const diffBtoA = new Uint8Array([200, 201]); // Update 5 encoded

    vi.mocked(channelService.getChannelDiff)
      .mockResolvedValueOnce(diffAtoB) // A->B: contains update 4
      .mockResolvedValueOnce(diffBtoA); // B->A: contains update 5

    // Mock Peer B applying diff from A
    vi.mocked(channelService.applyChannelDiff)
      .mockResolvedValueOnce({
        messages_updated: 1,
        total_messages: 11,
      }) // B applies A's diff
      .mockResolvedValueOnce({
        messages_updated: 1,
        total_messages: 11,
      }); // A applies B's diff

    // Mock state vector updates
    vi.mocked(channelService.getChannelStateVector)
      .mockResolvedValueOnce(finalStateVector) // B's state after applying A's diff
      .mockResolvedValueOnce(finalStateVector); // A's state after applying B's diff

    // --- Round 1: Peer A sends update to Peer B ---

    // 1. A generates diff for B
    const diffFromA = await channelSyncService.generateDiffForPeer(
      channelId,
      peerBStateVector
    );
    expect(diffFromA).toEqual(diffAtoB);

    // 2. B applies A's diff
    const resultB = await channelSyncService.applyDiffFromPeer(
      channelId,
      diffFromA
    );
    expect(resultB.messages_updated).toBe(1);
    expect(resultB.total_messages).toBe(11);

    // --- Round 2: Peer B sends update to Peer A ---

    // 3. B generates diff for A
    const diffFromB = await channelSyncService.generateDiffForPeer(
      channelId,
      peerAStateVector
    );
    expect(diffFromB).toEqual(diffBtoA);

    // 4. A applies B's diff
    const resultA = await channelSyncService.applyDiffFromPeer(
      channelId,
      diffFromB
    );
    expect(resultA.messages_updated).toBe(1);
    expect(resultA.total_messages).toBe(11);

    // Verify both peers received exactly 1 update each
    expect(resultA.messages_updated).toBe(1);
    expect(resultB.messages_updated).toBe(1);

    // Verify mock call counts
    expect(channelService.getChannelDiff).toHaveBeenCalledTimes(2);
    expect(channelService.applyChannelDiff).toHaveBeenCalledTimes(2);
    expect(channelService.getChannelStateVector).toHaveBeenCalledTimes(2);

    // Verify correct arguments
    expect(channelService.getChannelDiff).toHaveBeenNthCalledWith(
      1,
      channelId,
      peerBStateVector
    );
    expect(channelService.getChannelDiff).toHaveBeenNthCalledWith(
      2,
      channelId,
      peerAStateVector
    );
  });
});
