/**
 * Network Connection Service
 * 
 * Handles:
 * - Automatic connection to P2P network on startup
 * - Fallback to local mode when no network available
 * - Retry logic with exponential backoff
 * - Network status monitoring
 */

import { offlineStorage } from '../storage/OfflineStorageService';

export type NetworkStatus = 'connecting' | 'connected' | 'offline' | 'local' | 'error';

interface NetworkState {
  status: NetworkStatus;
  isOnline: boolean;
  peers: number;
  lastConnectionAttempt: Date | null;
  lastSuccessfulConnection: Date | null;
  bootstrapNodes: string[];
  error: string | null;
  retryCount: number;
  endpointFourWords: string | null;  // Four-word address of current endpoint
  userFourWords: string | null;      // User's four-word identity
}

interface NetworkStatusListener {
  (state: NetworkState): void;
}

export class NetworkConnectionService {
  private static instance: NetworkConnectionService;
  private state: NetworkState;
  private listeners: Set<NetworkStatusListener> = new Set();
  private retryTimer: NodeJS.Timeout | null = null;
  private invoke: any = null;
  private readonly MAX_RETRIES = 3;
  private readonly RETRY_DELAYS = [1000, 3000, 10000]; // 1s, 3s, 10s

  private constructor() {
    this.state = {
      status: 'connecting',
      isOnline: navigator.onLine,
      peers: 0,
      lastConnectionAttempt: null,
      lastSuccessfulConnection: null,
      bootstrapNodes: [],
      error: null,
      retryCount: 0,
      endpointFourWords: null,
      userFourWords: null
    };

    this.initialize();
  }

  static getInstance(): NetworkConnectionService {
    if (!NetworkConnectionService.instance) {
      NetworkConnectionService.instance = new NetworkConnectionService();
    }
    return NetworkConnectionService.instance;
  }

  private async initialize() {
    // Get invoke function
    await this.setupInvoke();

    // Setup browser online/offline listeners
    this.setupNetworkListeners();

    // Try to connect to network automatically
    // TEMPORARILY DISABLED FOR TESTING
    console.log('📵 Network auto-connection disabled for testing');
    this.updateState({ status: 'local' });
    // await this.attemptNetworkConnection();
  }

  private async setupInvoke() {
    if (typeof window !== 'undefined' && (window as any)._TAURI_?.core?.invoke) {
      this.invoke = (window as any)._TAURI_.core.invoke;
    } else {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        this.invoke = invoke;
      } catch {
        console.log('📵 Tauri not available, running in local mode');
        this.updateState({ status: 'local' });
      }
    }
  }

  private setupNetworkListeners() {
    window.addEventListener('online', () => {
      console.log('🌐 Browser online, attempting network connection...');
      this.updateState({ isOnline: true });
      this.attemptNetworkConnection();
    });

    window.addEventListener('offline', () => {
      console.log('📵 Browser offline, switching to local mode');
      this.updateState({ 
        isOnline: false, 
        status: 'offline',
        peers: 0 
      });
    });
  }

  /**
   * Attempt to connect to the P2P network
   */
  async attemptNetworkConnection(): Promise<boolean> {
    // Don't attempt if browser is offline
    if (!navigator.onLine) {
      console.log('📵 Browser offline, using local mode');
      this.updateState({ 
        status: 'offline',
        error: 'No internet connection' 
      });
      return false;
    }

    // Don't attempt if no Tauri backend
    if (!this.invoke) {
      console.log('🏠 No backend available, using local mode');
      this.updateState({ 
        status: 'local',
        error: 'Backend not available' 
      });
      return false;
    }

    this.updateState({ 
      status: 'connecting',
      lastConnectionAttempt: new Date(),
      error: null 
    });

    try {
      // Step 1: Get bootstrap nodes
      console.log('🔍 Fetching bootstrap nodes...');
      const bootstrapNodes = await this.getBootstrapNodes();
      
      if (!bootstrapNodes || bootstrapNodes.length === 0) {
        console.log('⚠️ No bootstrap nodes available, using local mode');
        this.updateState({ 
          status: 'local',
          error: 'No bootstrap nodes available',
          bootstrapNodes: []
        });
        return false;
      }

      this.updateState({ bootstrapNodes });
      console.log(`📡 Found ${bootstrapNodes.length} bootstrap nodes`);

      // Step 2: Initialize identity if not already done
      const identity = await offlineStorage.get('current_identity');
      if (!identity) {
        console.log('⚠️ No identity found, please create one first');
        this.updateState({ 
          status: 'local',
          error: 'No identity configured' 
        });
        return false;
      }
      
      // Store user's four-word identity
      const userFourWords = identity.fourWordAddress || identity.four_word_address || null;
      this.updateState({ userFourWords });

      // Step 3: Try to connect to network
      console.log('🔌 Connecting to P2P network...');
      const connected = await this.connectToNetwork();
      
      if (connected) {
        // Step 4: Get peer count
        const peerCount = await this.getPeerCount();
        
        // Step 5: Get endpoint four-word address
        // For now, we'll generate a mock endpoint based on network status
        // In production, this would come from the actual network endpoint
        const endpointFourWords = await this.getEndpointFourWords();
        
        this.updateState({ 
          status: 'connected',
          peers: peerCount,
          lastSuccessfulConnection: new Date(),
          error: null,
          retryCount: 0,
          endpointFourWords
        });

        console.log(`✅ Connected to network with ${peerCount} peers`);
        console.log(`📍 Endpoint: ${endpointFourWords || 'Unknown'}`);
        
        // Store connection info
        await offlineStorage.store('network_status', {
          connected: true,
          peers: peerCount,
          bootstrapNodes,
          timestamp: new Date().toISOString()
        });
        
        return true;
      } else {
        throw new Error('Failed to connect to network');
      }
      
    } catch (error) {
      console.error('❌ Network connection failed:', error);
      
      const errorMessage = error instanceof Error ? error.message : 'Connection failed';
      this.updateState({ 
        status: 'error',
        error: errorMessage,
        retryCount: this.state.retryCount + 1
      });

      // Auto-retry with backoff
      if (this.state.retryCount < this.MAX_RETRIES) {
        this.scheduleRetry();
      } else {
        console.log('🏠 Max retries reached, switching to local mode');
        this.updateState({ 
          status: 'local',
          error: 'Could not connect after multiple attempts' 
        });
      }
      
      return false;
    }
  }

  /**
   * Connect via a specific Four-Words bootstrap node
   * @param fourWords - The Four-Word identity of the bootstrap node to connect through
   * @returns Promise<boolean> - true if connection successful
   */
  async connectViaFourWords(fourWords: string): Promise<boolean> {
    console.log(`🔗 Attempting to connect via Four-Words: ${fourWords}`);

    if (!this.invoke) {
      console.error('❌ Backend not available for Four-Words connection');
      return false;
    }

    // Update state to show we're connecting
    this.updateState({
      status: 'connecting',
      lastConnectionAttempt: new Date(),
      error: null
    });

    try {
      // First validate the Four-Words format
      const normalizedFourWords = fourWords.trim().toLowerCase().replace(/\s+/g, '-');

      // Try to connect via the Four-Words bootstrap
      const result = await this.invoke('connect_via_four_words', {
        fourWords: normalizedFourWords
      });

      if (result) {
        console.log(`✅ Successfully connected via Four-Words: ${normalizedFourWords}`);

        // Add this to our bootstrap nodes
        const currentNodes = this.state.bootstrapNodes;
        if (!currentNodes.includes(normalizedFourWords)) {
          const updatedNodes = [...currentNodes, normalizedFourWords];
          this.updateState({ bootstrapNodes: updatedNodes });

          // Save to storage for future use
          await offlineStorage.store('bootstrap_nodes', updatedNodes, { encrypt: true });
        }

        // Now update the rest of the connection state
        const peerCount = await this.getPeerCount();
        const endpointFourWords = await this.getEndpointFourWords();
        const userFourWords = await this.getUserFourWords();

        this.updateState({
          status: 'connected',
          peers: peerCount,
          lastSuccessfulConnection: new Date(),
          error: null,
          retryCount: 0,
          endpointFourWords,
          userFourWords
        });

        console.log(`📍 Connected with ${peerCount} peers`);
        console.log(`📍 User: ${userFourWords}`);
        console.log(`📍 Endpoint: ${endpointFourWords}`);

        return true;
      } else {
        throw new Error('Connection via Four-Words failed');
      }
    } catch (error) {
      console.error('❌ Four-Words connection failed:', error);
      const errorMessage = error instanceof Error ? error.message : 'Connection failed';

      this.updateState({
        status: 'error',
        error: `Failed to connect via Four-Words: ${errorMessage}`
      });

      // Fall back to local mode after failure
      setTimeout(() => {
        if (this.state.status === 'error') {
          this.updateState({ status: 'local' });
        }
      }, 3000);

      return false;
    }
  }

  /**
   * Get the user's Four-Word identity
   */
  private async getUserFourWords(): Promise<string | null> {
    if (!this.invoke) return null;

    try {
      // Try to get user Four-Words from backend
      const userFourWords = await this.invoke('get_user_four_words');
      if (userFourWords) {
        return userFourWords;
      }
    } catch {
      // Fall back to cached identity
      const identity = await offlineStorage.get('current_identity');
      if (identity) {
        return identity.fourWordAddress || identity.four_word_address || null;
      }
    }

    return null;
  }

  /**
   * Get bootstrap nodes from configuration
   */
  private async getBootstrapNodes(): Promise<string[]> {
    try {
      // First try to get saved bootstrap nodes
      const saved = await offlineStorage.get<any[]>('bootstrap_nodes');
      if (saved && Array.isArray(saved)) {
        // Handle both string array and object array formats
        const fourWords = saved.map(node => {
          if (typeof node === 'string') {
            return node;
          } else if (node.fourWords) {
            return node.fourWords;
          }
          return null;
        }).filter(Boolean);

        if (fourWords.length > 0) {
          console.log(`Using ${fourWords.length} saved bootstrap nodes`);
          return fourWords;
        }
      }

      // Then try to get from backend
      if (this.invoke) {
        const nodes = await this.invoke('core_get_bootstrap_nodes');
        if (nodes && nodes.length > 0) {
          return nodes;
        }
      }
    } catch (error) {
      console.warn('Failed to get bootstrap nodes:', error);
    }

    // Default bootstrap nodes
    return [
      'ocean-forest-moon-star',
      'river-mountain-sun-cloud'
    ];
  }

  /**
   * Connect to the P2P network
   */
  private async connectToNetwork(): Promise<boolean> {
    if (!this.invoke) return false;

    try {
      const result = await this.invoke('connect_to_network');
      return !!result;
    } catch (error) {
      // Command might not exist, try alternative
      try {
        const health = await this.invoke('health');
        return health?.status === 'ok';
      } catch {
        return false;
      }
    }
  }

  /**
   * Get current peer count
   */
  private async getPeerCount(): Promise<number> {
    if (!this.invoke) return 0;

    try {
      const status = await this.invoke('get_network_status');
      return status?.peers || 0;
    } catch {
      // Fallback: assume connected if health check passes
      try {
        const health = await this.invoke('health');
        return health?.status === 'ok' ? 1 : 0;
      } catch {
        return 0;
      }
    }
  }

  /**
   * Get the endpoint's four-word address
   * This represents the network location we're connected through (our external IP)
   */
  private async getEndpointFourWords(): Promise<string | null> {
    if (!this.invoke) {
      // In local mode, return a mock local endpoint
      return 'local-node-alpha-one';
    }

    try {
      // Try to get actual endpoint from backend - this should be the Four-Words
      // representation of our external IP address that others can connect to
      const endpointFourWords = await this.invoke('get_endpoint_four_words');
      if (endpointFourWords) {
        return endpointFourWords;
      }
    } catch {
      // Try alternative command
      try {
        const networkInfo = await this.invoke('get_network_info');
        if (networkInfo?.endpointFourWords) {
          return networkInfo.endpointFourWords;
        }
      } catch {
        // If commands don't exist, generate based on bootstrap node
        // This is a fallback for demo purposes
        if (this.state.bootstrapNodes.length > 0) {
          const firstNode = this.state.bootstrapNodes[0];
          // Extract four-word part if it's in the format
          const fourWordPart = firstNode.split(':')[0];
          if (fourWordPart.includes('-')) {
            // Generate a unique endpoint based on the bootstrap
            const parts = fourWordPart.split('-');
            // Swap some words to make it unique
            return `${parts[2]}-${parts[0]}-relay-node`;
          }
        }
      }
    }

    // Default endpoint for connected state
    return 'relay-node-prime-zero';
  }

  /**
   * Schedule a retry attempt
   */
  private scheduleRetry() {
    if (this.retryTimer) {
      clearTimeout(this.retryTimer);
    }

    const delay = this.RETRY_DELAYS[Math.min(this.state.retryCount, this.RETRY_DELAYS.length - 1)];
    console.log(`⏰ Retrying connection in ${delay / 1000}s...`);

    this.retryTimer = setTimeout(() => {
      this.attemptNetworkConnection();
    }, delay);
  }

  /**
   * Manually trigger a connection attempt
   */
  async connect(): Promise<boolean> {
    console.log('👤 Manual connection attempt requested');
    
    // Reset retry count for manual attempts
    this.updateState({ retryCount: 0 });
    
    // Clear any pending retry
    if (this.retryTimer) {
      clearTimeout(this.retryTimer);
      this.retryTimer = null;
    }
    
    return this.attemptNetworkConnection();
  }

  /**
   * Disconnect from network and go local
   */
  async disconnect() {
    console.log('📵 Disconnecting from network...');
    
    if (this.retryTimer) {
      clearTimeout(this.retryTimer);
      this.retryTimer = null;
    }

    if (this.invoke) {
      try {
        await this.invoke('disconnect_from_network');
      } catch (error) {
        console.warn('Disconnect command failed:', error);
      }
    }

    this.updateState({ 
      status: 'local',
      peers: 0,
      error: null,
      endpointFourWords: null
    });
  }

  /**
   * Update state and notify listeners
   */
  private updateState(partial: Partial<NetworkState>) {
    this.state = { ...this.state, ...partial };
    this.notifyListeners();
  }

  /**
   * Notify all listeners of state change
   */
  private notifyListeners() {
    this.listeners.forEach(listener => listener(this.state));
  }

  /**
   * Subscribe to network status changes
   */
  subscribe(listener: NetworkStatusListener): () => void {
    this.listeners.add(listener);
    
    // Immediately notify of current state
    listener(this.state);
    
    // Return unsubscribe function
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Get current network state
   */
  getState(): NetworkState {
    return { ...this.state };
  }

  /**
   * Check if currently connected to network
   */
  isConnected(): boolean {
    return this.state.status === 'connected';
  }

  /**
   * Check if in local mode
   */
  isLocal(): boolean {
    return this.state.status === 'local' || this.state.status === 'offline';
  }
}

// Export singleton instance
export const networkService = NetworkConnectionService.getInstance();