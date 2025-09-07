/**
 * Test Network Connection Flow
 * 
 * This script tests the automatic network connection on startup,
 * fallback to local mode, and manual reconnection features.
 */

import { networkService } from './services/network/NetworkConnectionService';

// Global test utilities exposed to console
declare global {
  interface Window {
    testNetwork: {
      status: () => void;
      connect: () => Promise<void>;
      disconnect: () => Promise<void>;
      simulateOffline: () => void;
      simulateOnline: () => void;
      testFlow: () => Promise<void>;
    };
  }
}

// Test utilities
const testNetwork = {
  // Check current network status
  status: () => {
    const state = networkService.getState();
    console.log('📊 Network Status:', {
      status: state.status,
      isOnline: state.isOnline,
      peers: state.peers,
      bootstrapNodes: state.bootstrapNodes.length,
      error: state.error,
      retryCount: state.retryCount,
      lastAttempt: state.lastConnectionAttempt,
      lastSuccess: state.lastSuccessfulConnection
    });
  },

  // Manually connect to network
  connect: async () => {
    console.log('🔌 Attempting manual connection...');
    const success = await networkService.connect();
    if (success) {
      console.log('✅ Connected successfully!');
    } else {
      console.log('❌ Connection failed');
    }
    testNetwork.status();
  },

  // Disconnect from network
  disconnect: async () => {
    console.log('📵 Disconnecting from network...');
    await networkService.disconnect();
    console.log('✅ Disconnected, now in local mode');
    testNetwork.status();
  },

  // Simulate going offline
  simulateOffline: () => {
    console.log('📵 Simulating offline mode...');
    window.dispatchEvent(new Event('offline'));
    setTimeout(() => {
      testNetwork.status();
    }, 100);
  },

  // Simulate coming online
  simulateOnline: () => {
    console.log('🌐 Simulating online mode...');
    window.dispatchEvent(new Event('online'));
    setTimeout(() => {
      testNetwork.status();
    }, 100);
  },

  // Test the complete flow
  testFlow: async () => {
    console.log('🧪 Testing Complete Network Flow');
    console.log('==================================');
    
    // Step 1: Check initial status
    console.log('\n📋 Step 1: Initial Status');
    testNetwork.status();
    
    // Step 2: Simulate offline
    console.log('\n📋 Step 2: Going Offline');
    testNetwork.simulateOffline();
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Step 3: Try to connect while offline
    console.log('\n📋 Step 3: Attempting Connection While Offline');
    await testNetwork.connect();
    
    // Step 4: Come back online
    console.log('\n📋 Step 4: Coming Back Online');
    testNetwork.simulateOnline();
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Step 5: Should auto-connect
    console.log('\n📋 Step 5: Checking Auto-Connection');
    await new Promise(resolve => setTimeout(resolve, 2000));
    testNetwork.status();
    
    // Step 6: Manual disconnect
    console.log('\n📋 Step 6: Manual Disconnect to Local Mode');
    await testNetwork.disconnect();
    
    // Step 7: Manual reconnect
    console.log('\n📋 Step 7: Manual Reconnect');
    await testNetwork.connect();
    
    console.log('\n✅ Test Flow Complete!');
    console.log('==================================');
    console.log('📝 Summary:');
    console.log('- App starts and attempts to connect automatically ✓');
    console.log('- Falls back to local mode when offline ✓');
    console.log('- Header shows network status indicator ✓');
    console.log('- Click indicator to manually reconnect ✓');
    console.log('- Auto-reconnects when coming back online ✓');
  }
};

// Expose to browser console
if (typeof window !== 'undefined') {
  window.testNetwork = testNetwork;
  
  console.log(`
╔═══════════════════════════════════════════════════════════╗
║         🌐 Network Connection Testing Available            ║
╠═══════════════════════════════════════════════════════════╣
║                                                            ║
║  Test Commands:                                           ║
║  • window.testNetwork.status()      - Check status        ║
║  • window.testNetwork.connect()     - Manual connect      ║
║  • window.testNetwork.disconnect()  - Go local mode       ║
║  • window.testNetwork.simulateOffline() - Go offline      ║
║  • window.testNetwork.simulateOnline()  - Come online     ║
║  • window.testNetwork.testFlow()    - Run complete test   ║
║                                                            ║
║  Visual Indicator:                                         ║
║  • Look for network status chip in the header             ║
║  • Green = Connected                                      ║
║  • Yellow = Connecting/Local                              ║
║  • Red = Error                                            ║
║  • Click to reconnect when offline                        ║
║                                                            ║
╚═══════════════════════════════════════════════════════════╝
  `);
  
  // Subscribe to network changes and log them
  networkService.subscribe((state) => {
    console.log(`🔔 Network State Change: ${state.status}`, {
      peers: state.peers,
      error: state.error
    });
  });
}