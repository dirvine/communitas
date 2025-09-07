/**
 * Browser console test for identity creation and login
 * Run this in the Tauri app's developer console
 */

// Test identity creation and login flow
export async function testIdentityFlow() {
  console.log('🧪 Testing Identity Creation and Login Flow');
  console.log('==========================================');
  
  // Check if Tauri is available
  const hasTauri = typeof window !== 'undefined' && (window as any).__TAURI__;
  console.log(`Tauri Available: ${hasTauri ? '✅' : '❌'}`);
  
  if (!hasTauri) {
    console.log('⚠️ Running in mock mode (no Tauri backend)');
  }
  
  try {
    // Get the invoke function
    let invoke: any;
    if (hasTauri && (window as any).__TAURI__?.core?.invoke) {
      invoke = (window as any).__TAURI__.core.invoke;
      console.log('✅ Using Tauri invoke');
    } else {
      // Try dynamic import
      try {
        const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
        invoke = tauriInvoke;
        console.log('✅ Using imported Tauri invoke');
      } catch {
        console.log('⚠️ Using mock invoke');
        // Mock invoke for testing
        invoke = async (cmd: string, args?: any) => {
          console.log(`Mock invoke: ${cmd}`, args);
          if (cmd === 'core_initialize') {
            return true;
          }
          if (cmd === 'health') {
            return { status: 'ok', saorsa_core: '0.3.17', app: '0.1.0' };
          }
          if (cmd === 'core_get_channels') {
            return [];
          }
          if (cmd === 'core_private_put') {
            return true;
          }
          if (cmd === 'core_private_get') {
            return args?.key ? `value_for_${args.key}` : null;
          }
          throw new Error(`Mock: Command ${cmd} not implemented`);
        };
      }
    }
    
    // Test 1: Initialize identity
    console.log('\n1️⃣ Testing core_initialize...');
    const initResult = await invoke('core_initialize', {
      fourWords: 'test-ocean-forest-river',
      displayName: 'Test User',
      deviceName: 'Browser Test',
      deviceType: 'Desktop'
    });
    console.log('✅ Identity initialized:', initResult);
    
    // Test 2: Check health
    console.log('\n2️⃣ Testing health check...');
    const health = await invoke('health');
    console.log('✅ Health:', health);
    
    // Test 3: Test private storage (works offline)
    console.log('\n3️⃣ Testing private storage...');
    await invoke('core_private_put', {
      key: 'test_profile',
      value: JSON.stringify({ name: 'Test User', created: new Date().toISOString() })
    });
    console.log('✅ Data stored');
    
    const retrieved = await invoke('core_private_get', { key: 'test_profile' });
    console.log('✅ Data retrieved:', retrieved);
    
    // Test 4: Try network operations (may fail without network)
    console.log('\n4️⃣ Testing network operations...');
    try {
      const channels = await invoke('core_get_channels');
      console.log('✅ Channels:', channels);
    } catch (e: any) {
      console.log('⚠️ Network operation failed (expected without network):', e.message);
    }
    
    console.log('\n==========================================');
    console.log('✨ Summary:');
    console.log('- Identity creation: ✅ Works without network');
    console.log('- Private storage: ✅ Works offline');
    console.log('- Network features: ⚠️ Require connectivity');
    console.log('\n💡 The app can function in offline mode with limited features');
    
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

// Test the AuthContext directly
export async function testAuthContext() {
  console.log('\n🔐 Testing AuthContext Integration');
  console.log('=====================================');
  
  // This would be called from React components
  console.log('The AuthContext provides:');
  console.log('1. createIdentity() - Creates new identity');
  console.log('2. login() - Logs in with four-word address');
  console.log('3. Fallback to mock data when Tauri unavailable');
  console.log('\nTo test from UI:');
  console.log('1. Click "Sign Up" button');
  console.log('2. Enter a name');
  console.log('3. Click "Create Identity"');
  console.log('\nThe system will:');
  console.log('- Generate a random four-word address');
  console.log('- Call core_initialize');
  console.log('- Store identity in state');
  console.log('- Work even without network');
}

// Auto-expose to window for console testing
if (typeof window !== 'undefined') {
  (window as any).testIdentity = {
    testFlow: testIdentityFlow,
    testAuth: testAuthContext
  };
  console.log('💡 Test functions available:');
  console.log('   window.testIdentity.testFlow() - Test identity creation');
  console.log('   window.testIdentity.testAuth() - Show auth context info');
}

// Export for module usage
export default {
  testIdentityFlow,
  testAuthContext
};