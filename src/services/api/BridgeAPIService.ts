// BridgeAPIService - HTTP client for communitas-bridge server
// Enables browser-based testing with real P2P networking via REST endpoints
import { logger } from '../LoggingService';

interface BridgeConfig {
  baseUrl: string;
  timeout: number;
}

const DEFAULT_CONFIG: BridgeConfig = {
  baseUrl: 'http://localhost:3030',
  timeout: 10000, // 10 seconds
};

export class BridgeAPIService {
  private config: BridgeConfig;

  constructor(config?: Partial<BridgeConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Make HTTP request to bridge server with timeout and error handling
   */
  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.config.baseUrl}${endpoint}`;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: 'Unknown error' }));
        throw new Error(error.error || `HTTP ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof Error) {
        logger.error('Bridge API request failed', { endpoint, error: error.message });
        throw error;
      }
      throw new Error('Unknown bridge API error');
    }
  }

  /**
   * Health check - verify bridge server is running
   */
  async healthCheck(): Promise<boolean> {
    try {
      const result = await this.request<{ status: string }>('/health');
      return result.status === 'ok';
    } catch {
      return false;
    }
  }

  /**
   * Initialize core context with four-word identity
   */
  async initialize(
    fourWords: string,
    displayName: string,
    deviceName: string
  ): Promise<void> {
    await this.request('/api/core/initialize', {
      method: 'POST',
      body: JSON.stringify({
        four_words: fourWords,
        display_name: displayName,
        device_name: deviceName,
      }),
    });
  }

  /**
   * Check if core is initialized
   */
  async getStatus(): Promise<{ initialized: boolean }> {
    return this.request('/api/core/status');
  }

  /**
   * Create a new channel
   */
  async createChannel(
    name: string,
    description: string
  ): Promise<{ id: string; name: string; description: string; created_at: string }> {
    return this.request('/api/channels', {
      method: 'POST',
      body: JSON.stringify({ name, description }),
    });
  }

  /**
   * List all channels
   */
  async listChannels(): Promise<{ channels: Array<{ id: string; name: string; description: string; created_at: string }> }> {
    return this.request('/api/channels');
  }

  /**
   * Get channel messages (stubbed in bridge - pending saorsa-core API)
   */
  async getChannelMessages(
    channelId: string,
    limit?: number
  ): Promise<{ messages: any[]; note?: string }> {
    const params = limit ? `?limit=${limit}` : '';
    return this.request(`/api/channels/${channelId}/messages${params}`);
  }

  /**
   * Send message to channel
   */
  async sendChannelMessage(
    channelId: string,
    content: string,
    recipients: string[],
    replyToId?: string
  ): Promise<{ success: boolean; message_id: string }> {
    return this.request(`/api/channels/${channelId}/messages`, {
      method: 'POST',
      body: JSON.stringify({
        content,
        recipients,
        reply_to_id: replyToId,
      }),
    });
  }

  /**
   * Create a thread from a message
   */
  async createThread(
    channelId: string,
    parentMessageId: string
  ): Promise<{ thread_id: string; channel_id: string; parent_message_id: string }> {
    return this.request('/api/threads/create', {
      method: 'POST',
      body: JSON.stringify({
        channel_id: channelId,
        parent_message_id: parentMessageId,
      }),
    });
  }

  /**
   * Get thread messages (stubbed in bridge - pending saorsa-core API)
   */
  async getThreadMessages(threadId: string): Promise<{ messages: any[]; note?: string }> {
    return this.request(`/api/threads/${threadId}/messages`);
  }

  /**
   * Get members for entity (stubbed in bridge)
   */
  async getMembers(
    entityType: string,
    entityId: string
  ): Promise<{ members: any[] }> {
    return this.request(`/api/${entityType}/${entityId}/members`);
  }

  /**
   * Add member to entity (stubbed in bridge)
   */
  async addMember(
    entityType: string,
    entityId: string,
    fourWordAddress: string,
    role: string
  ): Promise<{ success: boolean }> {
    return this.request(`/api/${entityType}/${entityId}/members`, {
      method: 'POST',
      body: JSON.stringify({
        four_word_address: fourWordAddress,
        role,
      }),
    });
  }
}

// Export singleton instance
export const bridgeAPI = new BridgeAPIService();
export default bridgeAPI;
