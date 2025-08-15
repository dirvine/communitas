// Mock Tauri API for browser development and testing
// This provides realistic mock data for all Tauri API calls

interface MockStore {
  [key: string]: any;
}

class MockTauriAPI {
  private store: MockStore = {};
  private organizationData = {
    id: 'org-001',
    name: 'Demo Organization',
    description: 'A demonstration organization for UX testing',
    members: [
      { id: 'user-001', name: 'Alice Johnson', role: 'Admin', status: 'online' },
      { id: 'user-002', name: 'Bob Smith', role: 'Member', status: 'offline' },
      { id: 'user-003', name: 'Carol White', role: 'Member', status: 'online' },
    ],
    settings: {
      privacy: 'private',
      allow_invites: true,
      require_approval: true,
    }
  };

  private networkHealth = {
    status: 'Connected',
    peer_count: 12,
    nat_type: 'Open',
    bandwidth_kbps: 2048,
    avg_latency_ms: 35,
  };

  private messages = [
    {
      id: 'msg-001',
      sender: 'Alice Johnson',
      content: 'Welcome to the demo organization!',
      timestamp: new Date(Date.now() - 3600000).toISOString(),
      encrypted: true,
    },
    {
      id: 'msg-002',
      sender: 'Bob Smith',
      content: 'Thanks for the invite. Looking forward to collaborating.',
      timestamp: new Date(Date.now() - 1800000).toISOString(),
      encrypted: true,
    },
  ];

  private identity = {
    id: 'identity-001',
    name: 'Current User',
    publicKey: 'mock-public-key-abc123',
    address: 'word1-word2-word3',
    verified: true,
  };

  async invoke(command: string, args?: any): Promise<any> {
    console.log(`[Mock Tauri] Invoking command: ${command}`, args);
    
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 100 + Math.random() * 200));

    switch (command) {
      case 'get_network_health':
        return this.getNetworkHealth();
      
      case 'get_organization':
        return this.getOrganization(args?.id);
      
      case 'list_organizations':
        return this.listOrganizations();
      
      case 'create_organization':
        return this.createOrganization(args);
      
      case 'update_organization':
        return this.updateOrganization(args);
      
      case 'get_messages':
        return this.getMessages(args);
      
      case 'send_message':
        return this.sendMessage(args);
      
      case 'get_identity':
        return this.getIdentity();
      
      case 'create_identity':
        return this.createIdentity(args);
      
      case 'store_secure':
        return this.storeSecure(args);
      
      case 'retrieve_secure':
        return this.retrieveSecure(args);
      
      case 'connect_peer':
        return this.connectPeer(args);
      
      case 'disconnect_peer':
        return this.disconnectPeer(args);
      
      case 'list_peers':
        return this.listPeers();
      
      case 'get_dht_stats':
        return this.getDhtStats();
      
      default:
        console.warn(`[Mock Tauri] Unknown command: ${command}`);
        return null;
    }
  }

  private getNetworkHealth() {
    // Simulate dynamic network changes
    this.networkHealth.peer_count = Math.floor(Math.random() * 20) + 5;
    this.networkHealth.bandwidth_kbps = Math.floor(Math.random() * 1000) + 1000;
    this.networkHealth.avg_latency_ms = Math.floor(Math.random() * 50) + 20;
    return this.networkHealth;
  }

  private getOrganization(id?: string) {
    return this.organizationData;
  }

  private listOrganizations() {
    return [
      this.organizationData,
      {
        id: 'org-002',
        name: 'Secondary Organization',
        description: 'Another test organization',
        members: [],
      }
    ];
  }

  private createOrganization(data: any) {
    const newOrg = {
      id: `org-${Date.now()}`,
      ...data,
      members: [{ id: 'user-001', name: 'Current User', role: 'Admin', status: 'online' }],
    };
    return newOrg;
  }

  private updateOrganization(data: any) {
    Object.assign(this.organizationData, data);
    return this.organizationData;
  }

  private getMessages(args: any) {
    return this.messages;
  }

  private sendMessage(args: any) {
    const newMessage = {
      id: `msg-${Date.now()}`,
      sender: 'Current User',
      content: args.content,
      timestamp: new Date().toISOString(),
      encrypted: true,
    };
    this.messages.push(newMessage);
    return newMessage;
  }

  private getIdentity() {
    return this.identity;
  }

  private createIdentity(args: any) {
    return {
      ...this.identity,
      ...args,
      id: `identity-${Date.now()}`,
    };
  }

  private storeSecure(args: any) {
    const { key, value } = args;
    this.store[key] = value;
    return { success: true };
  }

  private retrieveSecure(args: any) {
    const { key } = args;
    return this.store[key] || null;
  }

  private connectPeer(args: any) {
    return {
      success: true,
      peer_id: args.address,
      connection_id: `conn-${Date.now()}`,
    };
  }

  private disconnectPeer(args: any) {
    return { success: true };
  }

  private listPeers() {
    return [
      { id: 'peer-001', address: 'word4-word5-word6', status: 'connected', latency_ms: 25 },
      { id: 'peer-002', address: 'word7-word8-word9', status: 'connected', latency_ms: 45 },
      { id: 'peer-003', address: 'word10-word11-word12', status: 'connecting', latency_ms: null },
    ];
  }

  private getDhtStats() {
    return {
      total_keys: 1234,
      total_values: 5678,
      replication_factor: 3,
      routing_table_size: 20,
      storage_used_mb: 45.6,
    };
  }
}

// Export singleton instance
export const mockTauriApi = new MockTauriAPI();

// Function to inject mock API into window for browser development
export const injectMockTauriApi = () => {
  if (typeof window !== 'undefined' && !(window as any).__TAURI__) {
    console.log('[Mock Tauri] Injecting mock API for browser development');
    (window as any).__TAURI__ = {
      invoke: (command: string, args?: any) => mockTauriApi.invoke(command, args),
      core: {
        invoke: (command: string, args?: any) => mockTauriApi.invoke(command, args),
      },
      event: {
        listen: (event: string, handler: Function) => {
          console.log(`[Mock Tauri] Listening to event: ${event}`);
          // Return a mock unlisten function
          return Promise.resolve(() => {});
        },
        emit: (event: string, payload?: any) => {
          console.log(`[Mock Tauri] Emitting event: ${event}`, payload);
          return Promise.resolve();
        },
      },
      window: {
        getCurrent: () => ({
          setTitle: (title: string) => Promise.resolve(),
          center: () => Promise.resolve(),
          minimize: () => Promise.resolve(),
          maximize: () => Promise.resolve(),
          close: () => Promise.resolve(),
        }),
      },
    };
  }
};

// Enable mock API in development/test mode
if (process.env.NODE_ENV === 'development' && !import.meta.env.VITE_DISABLE_MOCK) {
  injectMockTauriApi();
}