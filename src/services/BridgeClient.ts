/**
 * Bridge Client - HTTP/REST client for Communitas Bridge Server
 *
 * Provides browser-compatible API access to Communitas core functionality
 * when Tauri backend is not available.
 */

const BRIDGE_URL = import.meta.env.VITE_BRIDGE_URL || 'http://localhost:3030';

export interface BridgeResponse<T = any> {
  success?: boolean;
  data?: T;
  error?: string;
}

export interface InitializeRequest {
  four_words: string;
  display_name: string;
  device_name: string;
}

export interface CreateChannelRequest {
  name: string;
  description: string;
}

export interface Channel {
  id: string;
  name: string;
  description: string;
  created_at: string;
}

export interface NetworkInfo {
  four_word_id: string;
  listen_addr: string;
  peer_count: number;
  is_listening: boolean;
}

class BridgeClient {
  private baseUrl: string;

  constructor(baseUrl: string = BRIDGE_URL) {
    this.baseUrl = baseUrl;
  }

  /**
   * Check if bridge server is available
   */
  async isAvailable(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/health`, {
        method: 'GET',
        signal: AbortSignal.timeout(2000)
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Health check
   */
  async health(): Promise<{ status: string; service: string }> {
    const response = await fetch(`${this.baseUrl}/health`);
    return response.json();
  }

  /**
   * Initialize core with four-word identity
   */
  async initialize(request: InitializeRequest): Promise<BridgeResponse> {
    const response = await fetch(`${this.baseUrl}/api/core/initialize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request)
    });

    if (!response.ok) {
      throw new Error(`Initialization failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Get core status
   */
  async getStatus(): Promise<{ initialized: boolean }> {
    const response = await fetch(`${this.baseUrl}/api/core/status`);
    return response.json();
  }

  /**
   * Create a channel
   */
  async createChannel(request: CreateChannelRequest): Promise<Channel> {
    const response = await fetch(`${this.baseUrl}/api/channels`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request)
    });

    if (!response.ok) {
      throw new Error(`Channel creation failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * List all channels
   */
  async listChannels(): Promise<{ channels: Channel[] }> {
    const response = await fetch(`${this.baseUrl}/api/channels`);
    return response.json();
  }

  /**
   * Get network connection info
   */
  async getNetworkInfo(): Promise<NetworkInfo> {
    const response = await fetch(`${this.baseUrl}/api/network/connection-info`);
    return response.json();
  }

  /**
   * Send a message to a channel
   */
  async sendMessage(channelId: string, content: string, recipients: string[]): Promise<any> {
    const response = await fetch(`${this.baseUrl}/api/channels/${channelId}/messages`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        content,
        recipients,
        reply_to_id: null
      })
    });

    if (!response.ok) {
      throw new Error(`Message send failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Get messages for a channel
   */
  async getMessages(channelId: string, limit: number = 50): Promise<any> {
    const response = await fetch(
      `${this.baseUrl}/api/channels/${channelId}/messages?limit=${limit}`
    );
    return response.json();
  }

  /**
   * Connect to a peer by four-word address
   */
  async connectToPeer(fourWordAddr: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/api/network/connect`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ four_word_addr: fourWordAddr })
    });

    if (!response.ok) {
      throw new Error(`Peer connection failed: ${response.statusText}`);
    }

    return response.json();
  }

  /**
   * Start networking
   */
  async startNetworking(): Promise<{ connection_identity: string; listen_address: string }> {
    const response = await fetch(`${this.baseUrl}/api/network/start`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}'
    });

    if (!response.ok) {
      throw new Error(`Failed to start networking: ${response.statusText}`);
    }

    return response.json();
  }

  async getConnectionInfo(): Promise<{ four_word_id: string; is_listening: boolean; listen_addr: string; peer_count: number }> {
    const response = await fetch(`${this.baseUrl}/api/network/connection-info`);
    if (!response.ok) {
      throw new Error(`Failed to get connection info: ${response.statusText}`);
    }
    return response.json();
  }

  /**
   * Get list of connected peers
   */
  async getConnectedPeers(): Promise<{ peers: any[] }> {
    const response = await fetch(`${this.baseUrl}/api/network/peers`);
    return response.json();
  }
}

// Singleton instance
export const bridgeClient = new BridgeClient();

// Export for testing/mocking
export { BridgeClient };
