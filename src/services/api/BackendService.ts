// Environment-aware API service that works in both browser and Tauri
import { invoke } from '@tauri-apps/api/core';
import { logger } from '../LoggingService';

// Detect runtime environment
const isTauri = () => {
  return typeof window !== 'undefined' && 
         typeof (window as any).__TAURI__ !== 'undefined';
};

// Browser fallback API - simulates backend calls with localStorage or mock data
class BrowserAPIService {
  async getMessages(entityType: string, entityId: string): Promise<any[]> {
    const key = `messages-${entityType}-${entityId}`;
    try {
      const stored = localStorage.getItem(key);
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  async sendMessage(entityType: string, entityId: string, content: string, replyToId?: string): Promise<any> {
    const message = {
      id: `msg_${Date.now()}`,
      sender_id: 'current-user',
      sender_name: 'You',
      sender_four_words: 'your-current-four-words',
      content,
      timestamp: new Date().toISOString(),
      status: 'sent',
      reply_to_id: replyToId
    };

    // Store in localStorage
    const key = `messages-${entityType}-${entityId}`;
    try {
      const existing = localStorage.getItem(key);
      const messages = existing ? JSON.parse(existing) : [];
      messages.push(message);
      localStorage.setItem(key, JSON.stringify(messages.slice(-50))); // Keep last 50
    } catch (error) {
      logger.error('Failed to store message', { error, entityType, entityId });
    }

    return message;
  }

  async getMembers(entityType: string, entityId: string): Promise<any[]> {
    const key = `members-${entityType}-${entityId}`;
    try {
      const stored = localStorage.getItem(key);
      if (stored) {
        return JSON.parse(stored);
      }
    } catch {
      // Fallback to demo members
    }

    // Return demo members
    return [
      {
        user_id: 'user1',
        display_name: 'Alice Johnson',
        four_words: 'ocean-forest-moon-star',
        role: 'admin',
        status: 'online',
        last_seen: new Date().toISOString()
      },
      {
        user_id: 'user2', 
        display_name: 'Bob Chen',
        four_words: 'mountain-river-cloud-wind',
        role: 'member',
        status: 'away',
        last_seen: new Date(Date.now() - 15 * 60 * 1000).toISOString()
      }
    ];
  }

  async addMember(entityType: string, entityId: string, fourWordAddress: string, role: string): Promise<boolean> {
    const key = `members-${entityType}-${entityId}`;
    try {
      const existing = localStorage.getItem(key);
      const members = existing ? JSON.parse(existing) : [];
      
      // Check if member already exists
      if (members.some((m: any) => m.four_words === fourWordAddress)) {
        return false;
      }

      const newMember = {
        user_id: `user_${Date.now()}`,
        display_name: `User ${fourWordAddress.split('-')[0]}`,
        four_words: fourWordAddress,
        role,
        status: 'offline',
        last_seen: new Date().toISOString()
      };

      members.push(newMember);
      localStorage.setItem(key, JSON.stringify(members));
      return true;
    } catch (error) {
      logger.error('Failed to add member', { error, entityType, entityId, fourWordAddress });
      return false;
    }
  }

  async removeMember(entityType: string, entityId: string, memberId: string): Promise<boolean> {
    const key = `members-${entityType}-${entityId}`;
    try {
      const existing = localStorage.getItem(key);
      const members = existing ? JSON.parse(existing) : [];
      const filtered = members.filter((m: any) => m.user_id !== memberId);
      localStorage.setItem(key, JSON.stringify(filtered));
      return true;
    } catch (error) {
      logger.error('Failed to remove member', { error, entityType, entityId, memberId });
      return false;
    }
  }

  async createThread(messageId: string, entityType: string, entityId: string): Promise<string> {
    // For browser, create a mock thread and seed local storage for consistency
    const threadId = `thread_${Date.now()}`;
    try {
      const key = `messages-thread-${threadId}`;
      localStorage.setItem(key, JSON.stringify([]));
    } catch (error) {
      logger.warn('Failed to seed local thread storage', { error, threadId });
    }
    return threadId;
  }

  async getThreadMessages(threadId: string): Promise<any[]> {
    const key = `messages-thread-${threadId}`;
    try {
      const stored = localStorage.getItem(key);
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }
}

// Unified backend service
class BackendService {
  private browserAPI: BrowserAPIService;

  constructor() {
    this.browserAPI = new BrowserAPIService();
  }

  async getMessages(entityType: string, entityId: string): Promise<any[]> {
    if (isTauri()) {
      try {
        if (entityType === 'channel') {
          return await invoke('core_channel_get_messages', {
            channel_id: entityId,
            limit: 200,
          });
        }

        if (entityType === 'thread') {
          return await invoke('core_thread_get_messages', {
            thread_id: entityId,
          });
        }

        // Generic fallback for other entity types (not yet supported natively)
        const fallback = await invoke('core_messages_list', {
          entityId,
          limit: 100,
          offset: 0,
        });
        if (Array.isArray(fallback) && fallback.length > 0) {
          return fallback;
        }

        return this.browserAPI.getMessages(entityType, entityId);
      } catch (error) {
        logger.warn('Tauri backend call failed, falling back to browser API', { error, entityType, entityId });
        return this.browserAPI.getMessages(entityType, entityId);
      }
    } else {
      return this.browserAPI.getMessages(entityType, entityId);
    }
  }

  async sendMessage(entityType: string, entityId: string, content: string, replyToId?: string): Promise<any> {
    if (isTauri()) {
      try {
        if (entityType === 'channel') {
          // Get current user identity for author_id
          const identity = await invoke('gossip_get_own_identity') as { four_words: string; public_key: string };
          const authorId = identity.public_key; // Use public key as author_id

          // Use working send_message command from org_commands.rs
          return await invoke('send_message', {
            request: {
              channel_id: entityId,
              author_id: authorId,
              content: content,
              thread_id: replyToId || null, // reply_to_id becomes thread_id
            },
          });
        }

        logger.warn('Send not implemented for entity type, falling back to browser API', { entityType });
        return this.browserAPI.sendMessage(entityType, entityId, content, replyToId);
      } catch (error) {
        logger.warn('Tauri backend call failed, falling back to browser API', { error, entityType, entityId });
        return this.browserAPI.sendMessage(entityType, entityId, content, replyToId);
      }
    } else {
      return this.browserAPI.sendMessage(entityType, entityId, content, replyToId);
    }
  }

  async getThreadMessages(threadId: string): Promise<any[]> {
    if (isTauri()) {
      try {
        const threadMessages = await invoke('core_thread_get_messages', {
          thread_id: threadId,
        });
        if (Array.isArray(threadMessages) && threadMessages.length > 0) {
          return threadMessages;
        }
        return this.browserAPI.getThreadMessages(threadId);
      } catch (error) {
        logger.warn('Tauri thread fetch failed, falling back to browser API', { error, threadId });
        return this.browserAPI.getThreadMessages(threadId);
      }
    }

    return this.browserAPI.getThreadMessages(threadId);
  }

  async getMembers(entityType: string, entityId: string): Promise<any[]> {
    if (isTauri()) {
      try {
        switch (entityType) {
          case 'group':
            return await invoke('core_group_list_members', { groupId: entityId });
          case 'channel':
            return await invoke('core_channel_list_members', { channelId: entityId });
          case 'project':
            return await invoke('core_project_list_members', { projectId: entityId });
          case 'organization':
            return await invoke('core_organization_list_members', { organizationId: entityId });
          default:
            return [];
        }
      } catch (error) {
        logger.warn('Tauri backend call failed, falling back to browser API', { error, entityType, entityId });
        return this.browserAPI.getMembers(entityType, entityId);
      }
    } else {
      return this.browserAPI.getMembers(entityType, entityId);
    }
  }

  async addMember(entityType: string, entityId: string, fourWordAddress: string, role: string): Promise<boolean> {
    if (isTauri()) {
      try {
        switch (entityType) {
          case 'group':
            await invoke('core_group_add_member', { 
              groupWords: entityId.split('-'), 
              memberWords: fourWordAddress.split('-') 
            });
            return true;
          case 'channel':
            await invoke('core_channel_add_member', { 
              channelId: entityId, fourWordAddress, role 
            });
            return true;
          case 'project':
            await invoke('core_project_add_member', { 
              projectId: entityId, fourWordAddress, role 
            });
            return true;
          case 'organization':
            await invoke('core_organization_add_member', { 
              organizationId: entityId, fourWordAddress, role 
            });
            return true;
          default:
            return false;
        }
      } catch (error) {
        logger.warn('Tauri backend call failed, falling back to browser API', { error, entityType, entityId, fourWordAddress });
        return this.browserAPI.addMember(entityType, entityId, fourWordAddress, role);
      }
    } else {
      return this.browserAPI.addMember(entityType, entityId, fourWordAddress, role);
    }
  }

  async removeMember(entityType: string, entityId: string, memberId: string, memberFourWords?: string): Promise<boolean> {
    if (isTauri()) {
      try {
        switch (entityType) {
          case 'group':
            if (memberFourWords) {
              await invoke('core_group_remove_member', { 
                groupWords: entityId.split('-'), 
                memberWords: memberFourWords.split('-') 
              });
              return true;
            }
            return false;
          case 'channel':
            await invoke('core_channel_remove_member', { channelId: entityId, memberId });
            return true;
          case 'project':
            await invoke('core_project_remove_member', { projectId: entityId, memberId });
            return true;
          case 'organization':
            await invoke('core_organization_remove_member', { organizationId: entityId, memberId });
            return true;
          default:
            return false;
        }
      } catch (error) {
        logger.warn('Tauri backend call failed, falling back to browser API', { error, entityType, entityId, memberId });
        return this.browserAPI.removeMember(entityType, entityId, memberId);
      }
    } else {
      return this.browserAPI.removeMember(entityType, entityId, memberId);
    }
  }

  async createThread(messageId: string, entityType: string, entityId: string): Promise<string> {
    if (isTauri()) {
      try {
        if (entityType === 'channel') {
          const thread = await invoke<any>('core_create_thread', {
            channel_id: entityId,
            parent_message_id: messageId,
          });

          if (typeof thread === 'string') {
            return thread;
          }

          if (thread) {
            if (typeof thread.id === 'string') {
              return thread.id;
            }
            if (typeof thread.thread_id === 'string') {
              return thread.thread_id;
            }
            if (typeof thread.threadId === 'string') {
              return thread.threadId;
            }
          }
        }
      } catch (error) {
        logger.warn('Tauri backend call failed, falling back to browser API', { error, messageId, entityType, entityId });
      }
    }

    return this.browserAPI.createThread(messageId, entityType, entityId);
  }
}

// Export singleton instance
export const backendService = new BackendService();
export default backendService;
