// Environment-aware API service that works in both browser and Tauri
import { invoke } from '@tauri-apps/api/core';

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
      console.error('Failed to store message:', error);
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
      console.error('Failed to add member:', error);
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
      console.error('Failed to remove member:', error);
      return false;
    }
  }

  async createThread(messageId: string, entityType: string, entityId: string): Promise<string> {
    // For browser, just return a mock thread ID
    return `thread_${Date.now()}`;
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
        switch (entityType) {
          case 'group':
            return await invoke('core_group_get_messages', { groupId: entityId });
          case 'channel':
            return await invoke('core_channel_get_messages', { channelId: entityId });
          case 'user':
            return await invoke('core_get_direct_messages', { userId: entityId });
          case 'project':
            return await invoke('core_project_get_messages', { projectId: entityId });
          case 'organization':
            return await invoke('core_organization_get_messages', { organizationId: entityId });
          default:
            return [];
        }
      } catch (error) {
        console.warn('Tauri backend call failed, falling back to browser API:', error);
        return this.browserAPI.getMessages(entityType, entityId);
      }
    } else {
      return this.browserAPI.getMessages(entityType, entityId);
    }
  }

  async sendMessage(entityType: string, entityId: string, content: string, replyToId?: string): Promise<any> {
    if (isTauri()) {
      try {
        switch (entityType) {
          case 'group':
            return await invoke('core_send_message_to_group', { 
              groupId: entityId, content, replyToId 
            });
          case 'channel':
            return await invoke('core_send_message_to_channel', { 
              channelId: entityId, content, replyToId 
            });
          case 'user':
            return await invoke('core_send_direct_message', { 
              userId: entityId, content, replyToId 
            });
          case 'project':
            return await invoke('core_send_message_to_project', { 
              projectId: entityId, content, replyToId 
            });
          case 'organization':
            return await invoke('core_send_message_to_organization', { 
              organizationId: entityId, content, replyToId 
            });
          default:
            throw new Error(`Unsupported entity type: ${entityType}`);
        }
      } catch (error) {
        console.warn('Tauri backend call failed, falling back to browser API:', error);
        return this.browserAPI.sendMessage(entityType, entityId, content, replyToId);
      }
    } else {
      return this.browserAPI.sendMessage(entityType, entityId, content, replyToId);
    }
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
        console.warn('Tauri backend call failed, falling back to browser API:', error);
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
        console.warn('Tauri backend call failed, falling back to browser API:', error);
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
        console.warn('Tauri backend call failed, falling back to browser API:', error);
        return this.browserAPI.removeMember(entityType, entityId, memberId);
      }
    } else {
      return this.browserAPI.removeMember(entityType, entityId, memberId);
    }
  }

  async createThread(messageId: string, entityType: string, entityId: string): Promise<string> {
    if (isTauri()) {
      try {
        // Use entity-specific thread creation commands for better backend integration
        switch (entityType) {
          case 'group':
            return await invoke<string>('core_create_group_thread', { 
              groupId: entityId, messageId 
            });
          case 'channel':
            return await invoke<string>('core_create_channel_thread', { 
              channelId: entityId, messageId 
            });
          case 'user':
            return await invoke<string>('core_create_direct_thread', { 
              userId: entityId, messageId 
            });
          case 'project':
            return await invoke<string>('core_create_project_thread', { 
              projectId: entityId, messageId 
            });
          case 'organization':
            return await invoke<string>('core_create_organization_thread', { 
              organizationId: entityId, messageId 
            });
          default:
            // Fallback to generic thread creation
            return await invoke<string>('core_create_message_thread', { 
              messageId, entityType, entityId 
            });
        }
      } catch (error) {
        console.warn('Tauri backend call failed, falling back to browser API:', error);
        return this.browserAPI.createThread(messageId, entityType, entityId);
      }
    } else {
      return this.browserAPI.createThread(messageId, entityType, entityId);
    }
  }
}

// Export singleton instance
export const backendService = new BackendService();
export default backendService;