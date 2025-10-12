// Channel types matching Rust implementation

export interface Channel {
  id: string;
  org_id: string;
  name: string;
  description?: string;
  created_at: number;
  created_by: string;
}

export interface Message {
  id: string;
  channel_id: string;
  thread_id?: string;
  author_id: string;
  content: string;
  created_at: number;
  updated_at?: number;
}

export interface Thread {
  id: string;
  parent_message_id: string;
  channel_id: string;
  reply_count: number;
  last_reply_at?: number;
}

export interface CreateChannelRequest {
  org_id: string;
  name: string;
  description?: string;
  created_by: string;
}

export interface SendMessageRequest {
  channel_id: string;
  author_id: string;
  content: string;
  thread_id?: string;
}

export type ChannelMemberRole = 'owner' | 'admin' | 'member';

export interface ChannelMember {
  user_id: string;
  role: ChannelMemberRole;
}

/**
 * Result of applying a CRDT diff to a channel
 * Returned by apply_channel_diff command
 */
export interface AppliedDiffResult {
  messages_updated: number;
  total_messages: number;
}

/**
 * Channel sync state tracking
 * Used by ChannelSyncService to manage sync operations
 */
export interface ChannelSyncState {
  channel_id: string;
  last_sync_at: number;
  state_vector: Uint8Array | null;
  sync_in_progress: boolean;
}
