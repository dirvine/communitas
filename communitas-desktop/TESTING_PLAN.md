# Comprehensive Testing Plan for Encrypted Storage & Bootstrap Management

## Test Environment Setup

### Prerequisites
- Tauri development server running (`npm run tauri dev`)
- Chrome DevTools MCP configured and connected
- Web frontend accessible at `http://localhost:5173/`
- Console access for JavaScript testing

## 1. Authentication Flow Testing

### 1.1 Initial Vault Creation
```javascript
// Test first-time user registration with vault creation
const testInitialSetup = async () => {
  // Generate new four-word identity
  const identity = await window.__TAURI__.invoke('generate_four_word_identity');
  console.log('Generated identity:', identity);

  // Initialize encrypted storage
  const result = await window.__TAURI__.invoke('core_storage_initialize', {
    password: 'TestPassword123!@#',
    displayName: 'Test User'
  });

  console.assert(result.success, 'Vault creation should succeed');
  console.assert(result.session_id, 'Should return session ID');
  console.assert(result.vault_id, 'Should return vault ID');

  return result;
};
```

### 1.2 Multi-Account Login Testing
```javascript
// Test login with multiple accounts
const testMultiAccountLogin = async () => {
  // Login to first account
  const account1 = await window.__TAURI__.invoke('core_storage_login', {
    fourWords: 'ocean-forest-moon-star',
    password: 'Account1Password!'
  });

  // Login to second account (should create new session)
  const account2 = await window.__TAURI__.invoke('core_storage_login', {
    fourWords: 'mountain-river-sun-cloud',
    password: 'Account2Password!'
  });

  // Get active sessions
  const sessions = await window.__TAURI__.invoke('core_storage_get_sessions');
  console.assert(sessions.length >= 2, 'Should have multiple sessions');

  return { account1, account2, sessions };
};
```

### 1.3 Password-Only Login (Familiar Device)
```javascript
// Test password-only login after initial authentication
const testFamiliarDeviceLogin = async () => {
  // First logout
  await window.__TAURI__.invoke('core_storage_logout');

  // Try password-only login
  const result = await window.__TAURI__.invoke('core_storage_password_login', {
    password: 'TestPassword123!@#'
  });

  console.assert(result.success, 'Password-only login should work on familiar device');
  console.assert(!result.require_full_auth, 'Should not require full auth');

  return result;
};
```

### 1.4 Session Expiration Testing
```javascript
// Test session timeout and cleanup
const testSessionExpiration = async () => {
  // Create short-lived session (1 second for testing)
  // Note: Would need backend modification to support custom timeout

  // Get initial sessions
  const before = await window.__TAURI__.invoke('core_storage_get_sessions');

  // Wait for expiration
  await new Promise(resolve => setTimeout(resolve, 2000));

  // Check sessions after expiration
  const after = await window.__TAURI__.invoke('core_storage_get_sessions');

  console.log('Sessions before:', before.length, 'after:', after.length);

  return { before, after };
};
```

## 2. Bootstrap Connectivity Testing

### 2.1 Bootstrap Node Management
```javascript
// Test bootstrap node operations
const testBootstrapManagement = async () => {
  // Get current bootstrap nodes
  const currentNodes = await window.__TAURI__.invoke('core_get_bootstrap_nodes');
  console.log('Current bootstrap nodes:', currentNodes);

  // Add custom node with four-word address
  await window.__TAURI__.invoke('core_add_bootstrap_node', {
    address: 'ocean-forest-moon-star'
  });

  // Add IPv6 node with extended words (more than 4)
  await window.__TAURI__.invoke('core_add_bootstrap_node', {
    address: 'ocean-forest-moon-star-mountain-river'
  });

  // Update bootstrap configuration
  await window.__TAURI__.invoke('core_update_bootstrap_nodes', {
    nodes: [
      'ocean-forest-moon-star',
      'test-node-alpha-beta',
      '192.168.1.100:7000',
      '[2001:db8::1]:7000'
    ]
  });

  // Get statistics
  const stats = await window.__TAURI__.invoke('core_get_bootstrap_stats');
  console.log('Bootstrap statistics:', stats);

  return { currentNodes, stats };
};
```

### 2.2 Network Reconnection Testing
```javascript
// Test persistence across restart
const testNetworkPersistence = async () => {
  // Add custom nodes
  await window.__TAURI__.invoke('core_add_bootstrap_node', {
    address: 'persistent-test-node-one'
  });

  // Get current state
  const beforeRestart = await window.__TAURI__.invoke('core_get_bootstrap_nodes');
  console.log('Nodes before restart:', beforeRestart);

  // Simulate app restart (manual step required)
  console.log('Please restart the app and run testAfterRestart()');

  return beforeRestart;
};

const testAfterRestart = async () => {
  // Check if nodes persisted
  const afterRestart = await window.__TAURI__.invoke('core_get_bootstrap_nodes');
  console.log('Nodes after restart:', afterRestart);

  // Verify custom nodes still exist
  const hasCustom = afterRestart.custom_nodes?.length > 0;
  console.assert(hasCustom, 'Custom nodes should persist');

  return afterRestart;
};
```

### 2.3 Connection Quality Testing
```javascript
// Test connection quality scoring
const testConnectionQuality = async () => {
  const stats = await window.__TAURI__.invoke('core_get_bootstrap_stats');

  console.log('Connection statistics:');
  console.log('- Total attempts:', stats.total_attempts);
  console.log('- Successful:', stats.successful_connections);
  console.log('- Failed:', stats.failed_connections);
  console.log('- Average latency:', stats.average_latency_ms);
  console.log('- Cache size:', stats.cache_size);
  console.log('- Top peers:', stats.top_peers);

  // Test quality thresholds
  const goodPeers = stats.top_peers?.filter(p => p.quality_score > 0.8);
  console.log('High quality peers:', goodPeers);

  return stats;
};
```

## 3. Offline-First Features Testing

### 3.1 Encrypted Storage Operations
```javascript
// Test storing and retrieving encrypted data
const testEncryptedStorage = async () => {
  // Store without FEC
  const testData = {
    message: 'Test encrypted data',
    timestamp: Date.now(),
    sensitive: 'Secret information'
  };

  await window.__TAURI__.invoke('core_storage_store', {
    key: 'test/basic',
    dataBase64: btoa(JSON.stringify(testData)),
    useFec: false
  });

  // Store with FEC for critical data
  await window.__TAURI__.invoke('core_storage_store', {
    key: 'test/critical',
    dataBase64: btoa(JSON.stringify(testData)),
    useFec: true
  });

  // Retrieve data
  const retrieved = await window.__TAURI__.invoke('core_storage_retrieve', {
    key: 'test/basic'
  });

  const decoded = JSON.parse(atob(retrieved));
  console.assert(decoded.message === testData.message, 'Data should match');

  // List all keys
  const keys = await window.__TAURI__.invoke('core_storage_list_keys');
  console.log('Stored keys:', keys);

  return { stored: testData, retrieved: decoded, keys };
};
```

### 3.2 FEC Resilience Testing
```javascript
// Test Forward Error Correction
const testFECResilience = async () => {
  // Store large data with FEC
  const largeData = new Array(1000).fill('A').join('').repeat(100);

  await window.__TAURI__.invoke('core_storage_store', {
    key: 'test/large-fec',
    dataBase64: btoa(largeData),
    useFec: true
  });

  // Get storage statistics
  const stats = await window.__TAURI__.invoke('core_storage_get_stats');
  console.log('Storage stats:', stats);

  // Verify FEC metadata
  console.assert(stats.fec_protected_keys > 0, 'Should have FEC protected keys');

  return stats;
};
```

### 3.3 Offline Queue Testing
```javascript
// Test offline operation queueing
const testOfflineQueue = async () => {
  // Simulate offline mode
  console.log('Testing offline queue (simulated)');

  // Queue operations while offline
  const operations = [];
  for (let i = 0; i < 5; i++) {
    operations.push(
      window.__TAURI__.invoke('core_storage_store', {
        key: `offline/test-${i}`,
        dataBase64: btoa(`Offline data ${i}`),
        useFec: false
      }).catch(e => ({ error: e, index: i }))
    );
  }

  const results = await Promise.allSettled(operations);
  console.log('Offline operations:', results);

  return results;
};
```

## 4. Session Management Testing

### 4.1 Account Switching
```javascript
// Test switching between multiple accounts
const testAccountSwitching = async () => {
  // Ensure multiple accounts exist
  await testMultiAccountLogin();

  // Get current sessions
  const sessions = await window.__TAURI__.invoke('core_storage_get_sessions');
  console.log('Available sessions:', sessions);

  // Switch to different vault
  for (const fourWords of sessions) {
    await window.__TAURI__.invoke('core_storage_switch_vault', {
      fourWords
    });

    // Verify switch
    const keys = await window.__TAURI__.invoke('core_storage_list_keys');
    console.log(`Keys for ${fourWords}:`, keys);
  }

  return sessions;
};
```

### 4.2 Session Persistence
```javascript
// Test session persistence across app restarts
const testSessionPersistence = async () => {
  // Get current sessions
  const before = await window.__TAURI__.invoke('core_storage_get_sessions');
  console.log('Sessions before restart:', before);

  // Store session markers
  for (const fourWords of before) {
    await window.__TAURI__.invoke('core_storage_switch_vault', { fourWords });
    await window.__TAURI__.invoke('core_storage_store', {
      key: 'session-marker',
      dataBase64: btoa(new Date().toISOString()),
      useFec: false
    });
  }

  console.log('Restart app and run testSessionsAfterRestart()');

  return before;
};

const testSessionsAfterRestart = async () => {
  // Check persisted sessions
  const after = await window.__TAURI__.invoke('core_storage_get_sessions');
  console.log('Sessions after restart:', after);

  // Verify session data persisted
  for (const fourWords of after) {
    await window.__TAURI__.invoke('core_storage_switch_vault', { fourWords });

    try {
      const marker = await window.__TAURI__.invoke('core_storage_retrieve', {
        key: 'session-marker'
      });
      console.log(`Session ${fourWords} marker:`, atob(marker));
    } catch (e) {
      console.error(`Session ${fourWords} missing marker`);
    }
  }

  return after;
};
```

### 4.3 Concurrent Session Testing
```javascript
// Test concurrent operations on multiple sessions
const testConcurrentSessions = async () => {
  const sessions = await window.__TAURI__.invoke('core_storage_get_sessions');

  // Perform operations on all sessions concurrently
  const operations = sessions.map(async (fourWords, index) => {
    await window.__TAURI__.invoke('core_storage_switch_vault', { fourWords });

    // Store unique data per session
    await window.__TAURI__.invoke('core_storage_store', {
      key: `concurrent/test-${index}`,
      dataBase64: btoa(`Data for session ${index}`),
      useFec: false
    });

    return { fourWords, index };
  });

  const results = await Promise.all(operations);
  console.log('Concurrent operations completed:', results);

  return results;
};
```

## 5. Import/Export Testing

### 5.1 Vault Backup and Restore
```javascript
// Test vault export and import
const testVaultBackup = async () => {
  // Export current vault with data
  const backup = await window.__TAURI__.invoke('core_storage_export_vault', {
    includeData: true
  });

  console.log('Backup size:', backup.length);

  // Store backup reference
  localStorage.setItem('vault_backup', backup);

  // Clear and reimport
  await window.__TAURI__.invoke('core_storage_logout');

  // Import backup
  await window.__TAURI__.invoke('core_storage_import_vault', {
    backupBase64: backup,
    password: 'TestPassword123!@#'
  });

  // Verify restoration
  const keys = await window.__TAURI__.invoke('core_storage_list_keys');
  console.log('Restored keys:', keys);

  return { backupSize: backup.length, restoredKeys: keys };
};
```

### 5.2 Identity Storage Testing
```javascript
// Test identity storage with FEC
const testIdentityStorage = async () => {
  const identity = {
    fourWords: 'test-identity-alpha-beta',
    publicKey: 'dummy_public_key_base64',
    privateKey: 'dummy_private_key_base64_encrypted',
    metadata: {
      created: new Date().toISOString(),
      deviceName: 'Test Device',
      deviceType: 'Desktop'
    }
  };

  // Store identity with FEC protection
  await window.__TAURI__.invoke('core_storage_store_identity', {
    identityDataBase64: btoa(JSON.stringify(identity))
  });

  // Retrieve identity
  const stored = await window.__TAURI__.invoke('core_storage_retrieve', {
    key: 'identity'
  });

  const retrieved = JSON.parse(atob(stored));
  console.assert(retrieved.fourWords === identity.fourWords, 'Identity should match');

  return retrieved;
};
```

## 6. Performance Testing

### 6.1 Large Data Handling
```javascript
// Test performance with large data sets
const testLargeDataPerformance = async () => {
  const results = [];
  const sizes = [1, 10, 100, 1000]; // KB

  for (const size of sizes) {
    const data = new Array(size * 1024).fill('A').join('');
    const start = performance.now();

    await window.__TAURI__.invoke('core_storage_store', {
      key: `perf/test-${size}kb`,
      dataBase64: btoa(data),
      useFec: size > 100 // Use FEC for large data
    });

    const storeTime = performance.now() - start;

    const retrieveStart = performance.now();
    await window.__TAURI__.invoke('core_storage_retrieve', {
      key: `perf/test-${size}kb`
    });
    const retrieveTime = performance.now() - retrieveStart;

    results.push({
      size: `${size}KB`,
      storeTime: `${storeTime.toFixed(2)}ms`,
      retrieveTime: `${retrieveTime.toFixed(2)}ms`
    });
  }

  console.table(results);
  return results;
};
```

### 6.2 Concurrent Operations Performance
```javascript
// Test concurrent storage operations
const testConcurrentPerformance = async () => {
  const operations = 100;
  const start = performance.now();

  const promises = [];
  for (let i = 0; i < operations; i++) {
    promises.push(
      window.__TAURI__.invoke('core_storage_store', {
        key: `concurrent/op-${i}`,
        dataBase64: btoa(`Operation ${i}`),
        useFec: false
      })
    );
  }

  await Promise.all(promises);
  const elapsed = performance.now() - start;

  console.log(`${operations} concurrent operations: ${elapsed.toFixed(2)}ms`);
  console.log(`Average: ${(elapsed / operations).toFixed(2)}ms per operation`);

  return { operations, elapsed, average: elapsed / operations };
};
```

## 7. Error Handling Testing

### 7.1 Invalid Credentials
```javascript
// Test error handling for invalid credentials
const testInvalidCredentials = async () => {
  try {
    await window.__TAURI__.invoke('core_storage_login', {
      fourWords: 'invalid-words-not-exist',
      password: 'WrongPassword'
    });
    console.error('Should have thrown error');
  } catch (error) {
    console.log('Expected error:', error);
    console.assert(error.includes('vault not found') || error.includes('invalid'), 'Should indicate invalid credentials');
  }

  return 'Passed';
};
```

### 7.2 Storage Limits
```javascript
// Test storage limits and quotas
const testStorageLimits = async () => {
  const stats = await window.__TAURI__.invoke('core_storage_get_stats');

  console.log('Storage statistics:');
  console.log('- Total keys:', stats.total_keys);
  console.log('- Total size:', stats.total_size_bytes);
  console.log('- FEC keys:', stats.fec_protected_keys);
  console.log('- Compression ratio:', stats.compression_ratio);

  // Test approaching limits
  if (stats.total_size_bytes > 100 * 1024 * 1024) { // 100MB
    console.warn('Approaching storage limit');
  }

  return stats;
};
```

## 8. Chrome DevTools MCP Integration Testing

### 8.1 MCP Command Verification
```javascript
// Verify MCP integration with storage commands
const testMCPIntegration = async () => {
  console.log('Run these commands through Chrome DevTools MCP:');
  console.log('1. take_screenshot - Capture current state');
  console.log('2. execute_js with storage commands');
  console.log('3. get_dom to verify UI updates');
  console.log('4. manage_local_storage to check persistence');

  // Example MCP test
  const testScript = `
    (async () => {
      const result = await window.__TAURI__.invoke('core_storage_get_sessions');
      return { sessions: result, timestamp: Date.now() };
    })()
  `;

  console.log('MCP test script:', testScript);

  return 'See MCP results';
};
```

## Test Execution Plan

### Phase 1: Basic Functionality (30 minutes)
1. Run `testInitialSetup()` - Vault creation
2. Run `testEncryptedStorage()` - Basic storage operations
3. Run `testInvalidCredentials()` - Error handling
4. Run `testBootstrapManagement()` - Bootstrap nodes

### Phase 2: Multi-Account Features (30 minutes)
1. Run `testMultiAccountLogin()` - Multiple accounts
2. Run `testAccountSwitching()` - Vault switching
3. Run `testFamiliarDeviceLogin()` - Password-only auth
4. Run `testConcurrentSessions()` - Concurrent operations

### Phase 3: Persistence & Reliability (45 minutes)
1. Run `testNetworkPersistence()` - Bootstrap persistence
2. Restart app manually
3. Run `testAfterRestart()` - Verify persistence
4. Run `testSessionPersistence()` - Session persistence
5. Restart app manually
6. Run `testSessionsAfterRestart()` - Verify sessions
7. Run `testVaultBackup()` - Backup/restore

### Phase 4: Performance & Stress Testing (30 minutes)
1. Run `testLargeDataPerformance()` - Large data handling
2. Run `testConcurrentPerformance()` - Concurrent operations
3. Run `testFECResilience()` - FEC protection
4. Run `testStorageLimits()` - Storage statistics

### Phase 5: Integration Testing (30 minutes)
1. Run `testIdentityStorage()` - Identity management
2. Run `testOfflineQueue()` - Offline operations
3. Run `testConnectionQuality()` - Network quality
4. Run `testMCPIntegration()` - Chrome DevTools MCP

## Success Criteria

### ✅ Authentication & Security
- [ ] Vault creation with strong password
- [ ] Multi-account login working
- [ ] Password-only login on familiar device
- [ ] Session expiration and cleanup
- [ ] Invalid credentials properly rejected

### ✅ Data Storage & Encryption
- [ ] Data encrypted with ChaCha20-Poly1305
- [ ] FEC protection for critical data
- [ ] Large data handling efficient
- [ ] Concurrent operations stable
- [ ] Import/export functionality working

### ✅ Network & Bootstrap
- [ ] Bootstrap nodes persist across restarts
- [ ] Custom nodes can be added/removed
- [ ] Connection statistics accurate
- [ ] IPv6 and extended addresses supported
- [ ] Quality scoring functional

### ✅ Performance
- [ ] Storage operations < 100ms for small data
- [ ] Storage operations < 1s for 1MB data
- [ ] 100 concurrent operations stable
- [ ] Memory usage reasonable
- [ ] No memory leaks detected

### ✅ Integration
- [ ] Chrome DevTools MCP commands work
- [ ] All Tauri commands accessible
- [ ] Frontend can use all features
- [ ] Error messages helpful and clear
- [ ] Logging provides debugging info

## Troubleshooting Guide

### Common Issues

1. **"Vault not found" error**
   - Ensure vault is initialized first
   - Check if logged in with correct four-words
   - Verify session hasn't expired

2. **"Session expired" error**
   - Re-authenticate with password
   - Check session timeout settings
   - Verify system time is correct

3. **Bootstrap connection failures**
   - Check network connectivity
   - Verify bootstrap nodes are reachable
   - Check firewall settings
   - Try adding custom bootstrap nodes

4. **Storage performance issues**
   - Check available disk space
   - Verify FEC is only used for critical data
   - Monitor concurrent operation count
   - Check compression settings

5. **MCP connection issues**
   - Verify Tauri dev server is running
   - Check MCP socket path
   - Ensure Chrome DevTools is connected
   - Check for port conflicts

## Automated Test Suite

Save this as `test-suite.js` and run in browser console:

```javascript
// Automated test suite
async function runFullTestSuite() {
  console.log('🚀 Starting Communitas Test Suite');

  const results = {
    passed: [],
    failed: [],
    skipped: []
  };

  const tests = [
    { name: 'Initial Setup', fn: testInitialSetup },
    { name: 'Encrypted Storage', fn: testEncryptedStorage },
    { name: 'Multi-Account Login', fn: testMultiAccountLogin },
    { name: 'Account Switching', fn: testAccountSwitching },
    { name: 'Bootstrap Management', fn: testBootstrapManagement },
    { name: 'Invalid Credentials', fn: testInvalidCredentials },
    { name: 'Large Data Performance', fn: testLargeDataPerformance },
    { name: 'Concurrent Performance', fn: testConcurrentPerformance },
    { name: 'Vault Backup', fn: testVaultBackup },
    { name: 'Identity Storage', fn: testIdentityStorage }
  ];

  for (const test of tests) {
    try {
      console.log(`\n📝 Running: ${test.name}`);
      const result = await test.fn();
      console.log(`✅ Passed: ${test.name}`, result);
      results.passed.push(test.name);
    } catch (error) {
      console.error(`❌ Failed: ${test.name}`, error);
      results.failed.push({ name: test.name, error });
    }
  }

  console.log('\n📊 Test Results:');
  console.log(`✅ Passed: ${results.passed.length}`);
  console.log(`❌ Failed: ${results.failed.length}`);
  console.log(`⏭️ Skipped: ${results.skipped.length}`);

  if (results.failed.length > 0) {
    console.log('\nFailed tests:', results.failed);
  }

  return results;
}

// Run the suite
// runFullTestSuite();
```

## Next Steps

After completing all tests:

1. **Document any issues found**
2. **Create GitHub issues for bugs**
3. **Update test cases based on findings**
4. **Add automated CI/CD tests**
5. **Create user documentation**
6. **Plan performance optimizations**
7. **Schedule security audit**

## Notes

- All test functions are designed to be run in the browser console
- Chrome DevTools MCP provides additional testing capabilities
- Tests can be run individually or as a complete suite
- Results are logged to console for easy debugging
- Test data uses predictable keys for easy cleanup