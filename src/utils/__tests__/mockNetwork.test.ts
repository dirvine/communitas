/**
 * Mock Network Infrastructure for Testing
 *
 * Provides comprehensive mocking for P2P, DHT, and network operations
 * to enable reliable testing of distributed features without external dependencies.
 */

// Mock Network Types
export interface MockPeer {
  id: string;
  address: string;
  publicKey: string;
  connected: boolean;
  latency: number;
  storageCapacity: number;
  availableStorage: number;
}

export interface MockDHTEntry {
  key: string;
  value: Uint8Array;
  peers: string[];
  timestamp: number;
  ttl: number;
}

export interface MockNetworkState {
  peers: Map<string, MockPeer>;
  dht: Map<string, MockDHTEntry>;
  connections: Map<string, Set<string>>;
  messages: Array<{
    from: string;
    to: string;
    type: string;
    payload: any;
    timestamp: number;
  }>;
}

// Global mock network state
let mockNetworkState: MockNetworkState = {
  peers: new Map(),
  dht: new Map(),
  connections: new Map(),
  messages: []
};

// Mock Network Utilities
export class MockNetwork {
  private static instance: MockNetwork;
  private networkState = mockNetworkState;

  static getInstance(): MockNetwork {
    if (!MockNetwork.instance) {
      MockNetwork.instance = new MockNetwork();
    }
    return MockNetwork.instance;
  }

  // Peer Management
  addPeer(peer: MockPeer): void {
    this.networkState.peers.set(peer.id, peer);
    this.networkState.connections.set(peer.id, new Set());
  }

  removePeer(peerId: string): void {
    this.networkState.peers.delete(peerId);
    this.networkState.connections.delete(peerId);

    // Remove from all connection sets
    for (const connections of this.networkState.connections.values()) {
      connections.delete(peerId);
    }
  }

  getPeer(peerId: string): MockPeer | undefined {
    return this.networkState.peers.get(peerId);
  }

  getAllPeers(): MockPeer[] {
    return Array.from(this.networkState.peers.values());
  }

  // Connection Management
  connectPeers(peerId1: string, peerId2: string): void {
    const conn1 = this.networkState.connections.get(peerId1);
    const conn2 = this.networkState.connections.get(peerId2);

    if (conn1 && conn2) {
      conn1.add(peerId2);
      conn2.add(peerId1);
    }
  }

  disconnectPeers(peerId1: string, peerId2: string): void {
    const conn1 = this.networkState.connections.get(peerId1);
    const conn2 = this.networkState.connections.get(peerId2);

    if (conn1 && conn2) {
      conn1.delete(peerId2);
      conn2.delete(peerId1);
    }
  }

  getConnectedPeers(peerId: string): string[] {
    const connections = this.networkState.connections.get(peerId);
    return connections ? Array.from(connections) : [];
  }

  // DHT Operations
  putDHT(key: string, value: Uint8Array, peers: string[] = [], ttl: number = 3600000): void {
    const entry: MockDHTEntry = {
      key,
      value,
      peers,
      timestamp: Date.now(),
      ttl
    };
    this.networkState.dht.set(key, entry);
  }

  getDHT(key: string): MockDHTEntry | undefined {
    const entry = this.networkState.dht.get(key);
    if (entry && Date.now() - entry.timestamp > entry.ttl) {
      this.networkState.dht.delete(key);
      return undefined;
    }
    return entry;
  }

  // Message Passing
  sendMessage(from: string, to: string, type: string, payload: any): void {
    const message = {
      from,
      to,
      type,
      payload,
      timestamp: Date.now()
    };
    this.networkState.messages.push(message);
  }

  getMessagesForPeer(peerId: string): any[] {
    return this.networkState.messages.filter(msg => msg.to === peerId);
  }

  clearMessages(): void {
    this.networkState.messages = [];
  }

  // Network Simulation
  simulateLatency(minMs: number = 10, maxMs: number = 100): Promise<void> {
    const delay = Math.random() * (maxMs - minMs) + minMs;
    return new Promise(resolve => setTimeout(resolve, delay));
  }

  simulateNetworkFailure(peerId: string, durationMs: number = 5000): void {
    const peer = this.networkState.peers.get(peerId);
    if (peer) {
      peer.connected = false;
      setTimeout(() => {
        if (peer) peer.connected = true;
      }, durationMs);
    }
  }

  // Reset Network State
  reset(): void {
    this.networkState = {
      peers: new Map(),
      dht: new Map(),
      connections: new Map(),
      messages: []
    };
  }

  // Get Network Statistics
  getStats(): {
    totalPeers: number;
    totalConnections: number;
    totalDHTEntries: number;
    totalMessages: number;
  } {
    const totalConnections = Array.from(this.networkState.connections.values())
      .reduce((sum, connections) => sum + connections.size, 0) / 2; // Divide by 2 to avoid double counting

    return {
      totalPeers: this.networkState.peers.size,
      totalConnections,
      totalDHTEntries: this.networkState.dht.size,
      totalMessages: this.networkState.messages.length
    };
  }
}

// Mock WebRTC/WebSocket Transport
export class MockTransport {
  private static instance: MockTransport;
  private network = MockNetwork.getInstance();

  static getInstance(): MockTransport {
    if (!MockTransport.instance) {
      MockTransport.instance = new MockTransport();
    }
    return MockTransport.instance;
  }

  async connect(peerId: string, targetPeerId: string): Promise<boolean> {
    await this.network.simulateLatency(20, 50);

    const peer = this.network.getPeer(peerId);
    const targetPeer = this.network.getPeer(targetPeerId);

    if (peer && targetPeer && peer.connected && targetPeer.connected) {
      this.network.connectPeers(peerId, targetPeerId);
      return true;
    }

    return false;
  }

  async send(peerId: string, targetPeerId: string, data: any): Promise<boolean> {
    await this.network.simulateLatency(5, 20);

    const peer = this.network.getPeer(peerId);
    const targetPeer = this.network.getPeer(targetPeerId);

    if (peer && targetPeer && peer.connected && targetPeer.connected) {
      this.network.sendMessage(peerId, targetPeerId, 'data', data);
      return true;
    }

    return false;
  }

  disconnect(peerId: string, targetPeerId: string): void {
    this.network.disconnectPeers(peerId, targetPeerId);
  }
}

// Mock DHT Service
export class MockDHTService {
  private network = MockNetwork.getInstance();

  async put(key: string, value: Uint8Array, options: { ttl?: number } = {}): Promise<void> {
    await this.network.simulateLatency(10, 30);

    const peers = this.network.getAllPeers()
      .filter(peer => peer.connected)
      .slice(0, 3) // Simulate finding 3 closest peers
      .map(peer => peer.id);

    this.network.putDHT(key, value, peers, options.ttl);
  }

  async get(key: string): Promise<Uint8Array | null> {
    await this.network.simulateLatency(15, 40);

    const entry = this.network.getDHT(key);
    return entry ? entry.value : null;
  }

  async findPeers(key: string): Promise<string[]> {
    await this.network.simulateLatency(10, 25);

    const entry = this.network.getDHT(key);
    return entry ? entry.peers : [];
  }
}

// Test Utilities
export const createMockPeer = (id: string, overrides: Partial<MockPeer> = {}): MockPeer => ({
  id,
  address: `mock://${id}`,
  publicKey: `mock-key-${id}`,
  connected: true,
  latency: Math.floor(Math.random() * 100) + 10,
  storageCapacity: 1000000000, // 1GB
  availableStorage: 500000000, // 500MB
  ...overrides
});

export const setupMockNetwork = (peerCount: number = 5): MockNetwork => {
  const network = MockNetwork.getInstance();
  network.reset();

  // Create mock peers
  for (let i = 0; i < peerCount; i++) {
    const peer = createMockPeer(`peer-${i}`);
    network.addPeer(peer);
  }

  // Connect peers in a mesh network
  const peers = network.getAllPeers();
  for (let i = 0; i < peers.length; i++) {
    for (let j = i + 1; j < peers.length; j++) {
      network.connectPeers(peers[i].id, peers[j].id);
    }
  }

  return network;
};

// Test Setup Helpers
export const mockNetworkSetup = () => {
  // This function should be called in test files to set up network mocking
  // Usage: mockNetworkSetup() in beforeEach/afterEach blocks
  return {
    beforeEach: () => MockNetwork.getInstance().reset(),
    afterEach: () => MockNetwork.getInstance().reset()
  };
};

// Export for use in other test files
export { mockNetworkState };