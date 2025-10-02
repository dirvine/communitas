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
