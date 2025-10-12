import { invoke } from '@tauri-apps/api/core';
import type {
  Channel,
  Message,
  Thread,
  CreateChannelRequest,
  SendMessageRequest,
  ChannelMember,
  AppliedDiffResult,
} from '../types/channels';

/**
 * Channel Service - Frontend interface to Tauri channel commands
 * All operations work offline via CRDT and sync automatically
 */
export class ChannelService {
  // === Channel Operations ===

  async createChannel(request: CreateChannelRequest): Promise<Channel> {
    return invoke<Channel>('create_channel', { request });
  }

  async getChannel(channelId: string): Promise<Channel | null> {
    return invoke<Channel | null>('get_channel', { channelId });
  }

  async listChannels(orgId: string): Promise<Channel[]> {
    return invoke<Channel[]>('list_channels', { orgId });
  }

  // === Message Operations ===

  async sendMessage(request: SendMessageRequest): Promise<Message> {
    return invoke<Message>('send_message', { request });
  }

  async getMessages(
    channelId: string,
    limit?: number,
    offset?: number
  ): Promise<Message[]> {
    return invoke<Message[]>('get_messages', {
      channelId,
      limit: limit ?? null,
      offset: offset ?? null,
    });
  }

  // === Thread Operations ===

  async createThread(parentMessageId: string): Promise<Thread> {
    return invoke<Thread>('create_thread', { parentMessageId });
  }

  async getThreadReplies(threadId: string): Promise<Message[]> {
    return invoke<Message[]>('get_thread_replies', { threadId });
  }

  // === Member Operations ===

  async addMember(
    channelId: string,
    userId: string,
    role: string
  ): Promise<void> {
    return invoke<void>('add_channel_member', { channelId, userId, role });
  }

  async removeMember(channelId: string, userId: string): Promise<void> {
    return invoke<void>('remove_channel_member', { channelId, userId });
  }

  async getMembers(channelId: string): Promise<[string, string][]> {
    return invoke<[string, string][]>('get_channel_members', { channelId });
  }

  // === Sync Operations ===

  /**
   * Get full CRDT update (old sync protocol)
   * @deprecated Use Phase 3 state vector methods for efficiency
   */
  async getSyncUpdate(channelId: string): Promise<Uint8Array> {
    return invoke<Uint8Array>('get_channel_sync_update', { channelId });
  }

  /**
   * Apply full CRDT update (old sync protocol)
   * @deprecated Use Phase 3 diff-based sync for efficiency
   */
  async applySyncUpdate(channelId: string, update: Uint8Array): Promise<void> {
    return invoke<void>('apply_channel_sync_update', { channelId, update });
  }

  // === Phase 3: Efficient Sync (State Vector Protocol) ===

  /**
   * Get current CRDT state vector for a channel
   * Peers exchange state vectors to determine what updates are missing
   */
  async getChannelStateVector(channelId: string): Promise<Uint8Array> {
    return invoke<Uint8Array>('get_channel_state_vector', { channelId });
  }

  /**
   * Get CRDT diff containing only updates missing from remote state vector
   * This is the core of efficient sync - only transmit what's needed
   */
  async getChannelDiff(
    channelId: string,
    remoteStateVector: Uint8Array
  ): Promise<Uint8Array> {
    return invoke<Uint8Array>('get_channel_diff', {
      channelId,
      remoteStateVector,
    });
  }

  /**
   * Apply CRDT diff from another peer
   * Returns count of messages updated/materialized
   */
  async applyChannelDiff(
    channelId: string,
    diff: Uint8Array
  ): Promise<AppliedDiffResult> {
    return invoke<AppliedDiffResult>('apply_channel_diff', {
      channelId,
      diff,
    });
  }
}

// Singleton instance
export const channelService = new ChannelService();
