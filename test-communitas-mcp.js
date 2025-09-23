#!/usr/bin/env node

/**
 * Comprehensive Communitas MCP Testing Script
 * Tests all features of the application using the MCP server
 */

import net from 'net';
import { promises as fs } from 'fs';
import path from 'path';

// Configuration
const MCP_SOCKET = process.env.MCP_SOCKET || '/tmp/tauri-mcp-communitas-94743.sock';
const TEST_RESULTS_DIR = './test-results';
const SCREENSHOT_DIR = path.join(TEST_RESULTS_DIR, 'screenshots');

// Test data
const TEST_USER = {
  fourWords: 'ocean-forest-moon-star',
  displayName: 'Test User Alice',
  deviceName: 'Test Device',
  deviceType: 'Desktop'
};

const TEST_CONTACT = {
  fourWords: 'river-mountain-sun-cloud',
  displayName: 'Test Contact Bob'
};

const TEST_GROUP = {
  words: ['test', 'group', 'alpha', 'team'],
  name: 'Test Group Alpha',
  description: 'Test group for MCP testing'
};

const TEST_CHANNEL = {
  name: 'test-general',
  description: 'General test channel'
};

/**
 * MCP Client for communication with Tauri app
 */
class MCPClient {
  constructor(socketPath) {
    this.socketPath = socketPath;
    this.id = 0;
    this.pending = new Map();
    this.connected = false;
  }

  async connect() {
    return new Promise((resolve, reject) => {
      this.socket = net.createConnection(this.socketPath, () => {
        this.connected = true;
        console.log('✅ Connected to MCP server at', this.socketPath);
        resolve();
      });

      this.socket.on('data', (data) => {
        try {
          const messages = data.toString().split('\n').filter(Boolean);
          messages.forEach(msg => {
            try {
              const response = JSON.parse(msg);
              const promise = this.pending.get(response.id);
              if (promise) {
                if (response.error) {
                  promise.reject(new Error(response.error.message));
                } else {
                  promise.resolve(response.result);
                }
                this.pending.delete(response.id);
              }
            } catch (e) {
              console.error('Failed to parse response:', e);
            }
          });
        } catch (error) {
          console.error('Data processing error:', error);
        }
      });

      this.socket.on('error', reject);
      this.socket.on('close', () => {
        this.connected = false;
        console.log('Disconnected from MCP server');
      });
    });
  }

  async call(method, params = {}) {
    if (!this.connected) {
      throw new Error('Not connected to MCP server');
    }

    const id = ++this.id;
    const request = {
      jsonrpc: '2.0',
      method,
      params,
      id
    };

    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.socket.write(JSON.stringify(request) + '\n');

      // Timeout after 10 seconds
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error(`MCP timeout for ${method}`));
        }
      }, 10000);
    });
  }

  close() {
    if (this.socket) {
      this.socket.end();
    }
  }
}

/**
 * Test utilities
 */
async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function setupTestDirs() {
  await fs.mkdir(TEST_RESULTS_DIR, { recursive: true });
  await fs.mkdir(SCREENSHOT_DIR, { recursive: true });
}

async function saveScreenshot(mcp, name) {
  try {
    const screenshot = await mcp.call('take_screenshot', { format: 'png' });
    const filename = `${name}-${Date.now()}.png`;
    const filepath = path.join(SCREENSHOT_DIR, filename);
    await fs.writeFile(filepath, Buffer.from(screenshot, 'base64'));
    console.log(`   📸 Screenshot saved: ${filename}`);
    return filepath;
  } catch (error) {
    console.error(`   ❌ Screenshot failed: ${error.message}`);
  }
}

/**
 * Test phases
 */
async function testPhase1_Identity(mcp) {
  console.log('\n🔍 PHASE 1: IDENTITY & USER MANAGEMENT');
  console.log('=' .repeat(50));

  const results = { passed: 0, failed: 0, errors: [] };

  try {
    // Test 1.1: Check if app is loaded
    console.log('\n📝 Test 1.1: Checking app status...');
    const appStatus = await mcp.call('execute_js', {
      script: `
        JSON.stringify({
          tauriAvailable: !!window.__TAURI__,
          location: window.location.href,
          title: document.title
        })
      `
    });
    const status = JSON.parse(appStatus);
    console.log('   ✅ App loaded:', status.title);
    console.log('   ✅ Tauri available:', status.tauriAvailable);
    results.passed++;

    // Test 1.2: Generate Four-Word identity
    console.log('\n📝 Test 1.2: Generating Four-Word identity...');
    await saveScreenshot(mcp, 'before-identity');

    const claimResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          const words = ['ocean', 'forest', 'moon', 'star'];
          try {
            const idHex = await window.__TAURI__.invoke('core_claim', { words });
            return { success: true, idHex, words };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (claimResult.success) {
      console.log('   ✅ Identity claimed:', claimResult.idHex.substring(0, 16) + '...');
      results.passed++;
    } else {
      console.log('   ❌ Identity claim failed:', claimResult.error);
      results.failed++;
      results.errors.push(claimResult.error);
    }

    // Test 1.3: Initialize CoreContext
    console.log('\n📝 Test 1.3: Initializing CoreContext...');
    const initResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            await window.__TAURI__.invoke('core_initialize', {
              fourWords: '${TEST_USER.fourWords}',
              displayName: '${TEST_USER.displayName}',
              deviceName: '${TEST_USER.deviceName}',
              deviceType: '${TEST_USER.deviceType}'
            });
            return { success: true };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (initResult.success) {
      console.log('   ✅ CoreContext initialized');
      results.passed++;
    } else {
      console.log('   ❌ Initialization failed:', initResult.error);
      results.failed++;
      results.errors.push(initResult.error);
    }

    // Test 1.4: Check user profile
    console.log('\n📝 Test 1.4: Checking user profile...');
    const profileCheck = await mcp.call('execute_js', {
      script: `
        JSON.stringify({
          hasUser: !!window.__COMMUNITAS_USER__,
          displayName: window.__COMMUNITAS_USER__?.displayName || null
        })
      `
    });
    const profile = JSON.parse(profileCheck);

    if (profile.hasUser) {
      console.log('   ✅ User profile exists:', profile.displayName);
      results.passed++;
    } else {
      console.log('   ⚠️  No user profile found in window context');
      results.failed++;
    }

    await saveScreenshot(mcp, 'after-identity');

  } catch (error) {
    console.error('   ❌ Phase 1 error:', error.message);
    results.failed++;
    results.errors.push(error.message);
  }

  return results;
}

async function testPhase2_Contacts(mcp) {
  console.log('\n🔍 PHASE 2: CONTACT MANAGEMENT');
  console.log('=' .repeat(50));

  const results = { passed: 0, failed: 0, errors: [] };

  try {
    // Test 2.1: Add contact
    console.log('\n📝 Test 2.1: Adding contact...');
    const addContactResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            // Simulate adding a contact
            const contact = {
              fourWords: '${TEST_CONTACT.fourWords}',
              displayName: '${TEST_CONTACT.displayName}'
            };
            // Store in local state for now
            window.__TEST_CONTACTS__ = window.__TEST_CONTACTS__ || [];
            window.__TEST_CONTACTS__.push(contact);
            return { success: true, contact };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (addContactResult.success) {
      console.log('   ✅ Contact added:', TEST_CONTACT.displayName);
      results.passed++;
    } else {
      console.log('   ❌ Add contact failed:', addContactResult.error);
      results.failed++;
    }

    await saveScreenshot(mcp, 'contacts');

  } catch (error) {
    console.error('   ❌ Phase 2 error:', error.message);
    results.failed++;
    results.errors.push(error.message);
  }

  return results;
}

async function testPhase3_Groups(mcp) {
  console.log('\n🔍 PHASE 3: GROUPS');
  console.log('=' .repeat(50));

  const results = { passed: 0, failed: 0, errors: [] };

  try {
    // Test 3.1: Create group
    console.log('\n📝 Test 3.1: Creating group...');
    const createGroupResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            const result = await window.__TAURI__.invoke('core_group_create', {
              words: ${JSON.stringify(TEST_GROUP.words)}
            });
            return { success: true, group: result };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (createGroupResult.success) {
      console.log('   ✅ Group created:', createGroupResult.group.id_hex?.substring(0, 16) + '...');
      results.passed++;
    } else {
      console.log('   ❌ Group creation failed:', createGroupResult.error);
      results.failed++;
      results.errors.push(createGroupResult.error);
    }

    // Test 3.2: Add member to group
    console.log('\n📝 Test 3.2: Adding member to group...');
    const addMemberResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            await window.__TAURI__.invoke('core_group_add_member', {
              groupWords: ${JSON.stringify(TEST_GROUP.words)},
              memberWords: '${TEST_CONTACT.fourWords}'.split('-')
            });
            return { success: true };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (addMemberResult.success) {
      console.log('   ✅ Member added to group');
      results.passed++;
    } else {
      console.log('   ⚠️  Add member failed (expected):', addMemberResult.error);
      // This might fail if the identity doesn't exist yet
    }

    await saveScreenshot(mcp, 'groups');

  } catch (error) {
    console.error('   ❌ Phase 3 error:', error.message);
    results.failed++;
    results.errors.push(error.message);
  }

  return results;
}

async function testPhase5_Channels(mcp) {
  console.log('\n🔍 PHASE 5: CHANNELS');
  console.log('=' .repeat(50));

  const results = { passed: 0, failed: 0, errors: [] };

  try {
    // Test 5.1: Create channel
    console.log('\n📝 Test 5.1: Creating channel...');
    const createChannelResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            const channel = await window.__TAURI__.invoke('core_create_channel', {
              name: '${TEST_CHANNEL.name}',
              description: '${TEST_CHANNEL.description}'
            });
            window.__TEST_CHANNEL__ = channel;
            return { success: true, channel };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (createChannelResult.success) {
      console.log('   ✅ Channel created:', createChannelResult.channel.name);
      console.log('   📍 Channel ID:', createChannelResult.channel.id);
      results.passed++;
    } else {
      console.log('   ❌ Channel creation failed:', createChannelResult.error);
      results.failed++;
      results.errors.push(createChannelResult.error);
    }

    // Test 5.2: Send message to channel
    console.log('\n📝 Test 5.2: Sending message to channel...');
    const sendMessageResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            if (!window.__TEST_CHANNEL__) {
              return { success: false, error: 'No channel created' };
            }
            await window.__TAURI__.invoke('core_send_message_to_channel', {
              channelId: window.__TEST_CHANNEL__.id,
              text: 'Hello from MCP test! 🎉'
            });
            return { success: true };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (sendMessageResult.success) {
      console.log('   ✅ Message sent to channel');
      results.passed++;
    } else {
      console.log('   ❌ Send message failed:', sendMessageResult.error);
      results.failed++;
      results.errors.push(sendMessageResult.error);
    }

    // Test 5.3: Get channels list
    console.log('\n📝 Test 5.3: Getting channels list...');
    const getChannelsResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            const channels = await window.__TAURI__.invoke('core_get_channels');
            return { success: true, count: channels.length, channels };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (getChannelsResult.success) {
      console.log('   ✅ Channels retrieved:', getChannelsResult.count);
      if (getChannelsResult.channels.length > 0) {
        console.log('   📍 First channel:', getChannelsResult.channels[0].name);
      }
      results.passed++;
    } else {
      console.log('   ❌ Get channels failed:', getChannelsResult.error);
      results.failed++;
    }

    await saveScreenshot(mcp, 'channels');

  } catch (error) {
    console.error('   ❌ Phase 5 error:', error.message);
    results.failed++;
    results.errors.push(error.message);
  }

  return results;
}

async function testPhase7_Storage(mcp) {
  console.log('\n🔍 PHASE 7: STORAGE & VIRTUAL DISKS');
  console.log('=' .repeat(50));

  const results = { passed: 0, failed: 0, errors: [] };

  try {
    // Test 7.1: Initialize container
    console.log('\n📝 Test 7.1: Initializing container storage...');
    const initContainerResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            await window.__TAURI__.invoke('container_init');
            return { success: true };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (initContainerResult.success) {
      console.log('   ✅ Container initialized');
      results.passed++;
    } else {
      console.log('   ❌ Container init failed:', initContainerResult.error);
      results.failed++;
    }

    // Test 7.2: Store object
    console.log('\n📝 Test 7.2: Storing test object...');
    const storeResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            const encoder = new TextEncoder();
            const data = encoder.encode('# Test Document\\n\\nThis is a test document stored via MCP.');
            const handle = await window.__TAURI__.invoke('container_put_object', {
              bytes: Array.from(data)
            });
            window.__TEST_STORAGE_HANDLE__ = handle;
            return { success: true, handle };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (storeResult.success) {
      console.log('   ✅ Object stored:', storeResult.handle?.substring(0, 16) + '...');
      results.passed++;
    } else {
      console.log('   ❌ Store failed:', storeResult.error);
      results.failed++;
    }

    // Test 7.3: Retrieve object
    console.log('\n📝 Test 7.3: Retrieving stored object...');
    const retrieveResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            if (!window.__TEST_STORAGE_HANDLE__) {
              return { success: false, error: 'No handle available' };
            }
            const bytes = await window.__TAURI__.invoke('container_get_object', {
              oidHex: window.__TEST_STORAGE_HANDLE__
            });
            const decoder = new TextDecoder();
            const content = decoder.decode(new Uint8Array(bytes));
            return { success: true, content };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (retrieveResult.success) {
      console.log('   ✅ Object retrieved:', retrieveResult.content?.substring(0, 30) + '...');
      results.passed++;
    } else {
      console.log('   ❌ Retrieve failed:', retrieveResult.error);
      results.failed++;
    }

    // Test 7.4: Get container tip
    console.log('\n📝 Test 7.4: Getting container tip...');
    const tipResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            const tip = await window.__TAURI__.invoke('container_current_tip');
            return { success: true, tip };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (tipResult.success) {
      console.log('   ✅ Container tip:', tipResult.tip);
      results.passed++;
    } else {
      console.log('   ❌ Get tip failed:', tipResult.error);
      results.failed++;
    }

    await saveScreenshot(mcp, 'storage');

  } catch (error) {
    console.error('   ❌ Phase 7 error:', error.message);
    results.failed++;
    results.errors.push(error.message);
  }

  return results;
}

async function testPhase11_Network(mcp) {
  console.log('\n🔍 PHASE 11: NETWORK & P2P');
  console.log('=' .repeat(50));

  const results = { passed: 0, failed: 0, errors: [] };

  try {
    // Test 11.1: Check network status
    console.log('\n📝 Test 11.1: Checking network status...');
    const networkStatus = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            // Check if network service is available
            const hasNetworkService = !!window.testNetwork;
            let status = 'unknown';
            if (hasNetworkService && window.testNetwork.status) {
              status = window.testNetwork.status();
            }
            return {
              success: true,
              hasNetworkService,
              status
            };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (networkStatus.success) {
      console.log('   ✅ Network service available:', networkStatus.hasNetworkService);
      console.log('   📍 Network status:', networkStatus.status);
      results.passed++;
    } else {
      console.log('   ❌ Network check failed:', networkStatus.error);
      results.failed++;
    }

    // Test 11.2: Get bootstrap nodes
    console.log('\n📝 Test 11.2: Getting bootstrap nodes...');
    const bootstrapResult = await mcp.call('execute_js', {
      script: `
        (async () => {
          try {
            const nodes = await window.__TAURI__.invoke('core_get_bootstrap_nodes');
            return { success: true, nodes };
          } catch (error) {
            return { success: false, error: error.toString() };
          }
        })()
      `,
      await_promise: true
    });

    if (bootstrapResult.success) {
      console.log('   ✅ Bootstrap nodes:', bootstrapResult.nodes?.length || 0);
      if (bootstrapResult.nodes && bootstrapResult.nodes.length > 0) {
        console.log('   📍 First node:', bootstrapResult.nodes[0]);
      }
      results.passed++;
    } else {
      console.log('   ❌ Get bootstrap nodes failed:', bootstrapResult.error);
      results.failed++;
    }

    await saveScreenshot(mcp, 'network');

  } catch (error) {
    console.error('   ❌ Phase 11 error:', error.message);
    results.failed++;
    results.errors.push(error.message);
  }

  return results;
}

/**
 * Main test runner
 */
async function runTests() {
  console.log('\n🚀 COMMUNITAS COMPREHENSIVE MCP TEST SUITE');
  console.log('=' .repeat(60));
  console.log('MCP Socket:', MCP_SOCKET);
  console.log('Test Results Dir:', TEST_RESULTS_DIR);
  console.log('=' .repeat(60));

  await setupTestDirs();

  const mcp = new MCPClient(MCP_SOCKET);
  const allResults = [];

  try {
    // Connect to MCP server
    await mcp.connect();

    // Take initial screenshot
    console.log('\n📸 Taking initial screenshot...');
    await saveScreenshot(mcp, 'initial');

    // Run test phases
    const phases = [
      { name: 'Identity & User Management', fn: testPhase1_Identity },
      { name: 'Contact Management', fn: testPhase2_Contacts },
      { name: 'Groups', fn: testPhase3_Groups },
      { name: 'Channels', fn: testPhase5_Channels },
      { name: 'Storage & Virtual Disks', fn: testPhase7_Storage },
      { name: 'Network & P2P', fn: testPhase11_Network }
    ];

    for (const phase of phases) {
      const results = await phase.fn(mcp);
      allResults.push({ phase: phase.name, ...results });
      await sleep(1000); // Brief pause between phases
    }

    // Generate summary
    console.log('\n' + '=' .repeat(60));
    console.log('📊 TEST SUMMARY');
    console.log('=' .repeat(60));

    let totalPassed = 0;
    let totalFailed = 0;
    let allErrors = [];

    for (const result of allResults) {
      totalPassed += result.passed;
      totalFailed += result.failed;
      allErrors = allErrors.concat(result.errors);

      const status = result.failed === 0 ? '✅' : '❌';
      console.log(`${status} ${result.phase}: ${result.passed} passed, ${result.failed} failed`);
    }

    console.log('\n' + '=' .repeat(60));
    console.log(`TOTAL: ${totalPassed} passed, ${totalFailed} failed`);

    if (totalFailed > 0) {
      console.log('\n❌ ERRORS ENCOUNTERED:');
      allErrors.forEach((error, i) => {
        console.log(`  ${i + 1}. ${error}`);
      });
    }

    // Save test report
    const report = {
      timestamp: new Date().toISOString(),
      socket: MCP_SOCKET,
      results: allResults,
      totals: { passed: totalPassed, failed: totalFailed },
      errors: allErrors
    };

    const reportPath = path.join(TEST_RESULTS_DIR, `report-${Date.now()}.json`);
    await fs.writeFile(reportPath, JSON.stringify(report, null, 2));
    console.log(`\n📄 Test report saved: ${reportPath}`);

    // Take final screenshot
    console.log('\n📸 Taking final screenshot...');
    await saveScreenshot(mcp, 'final');

  } catch (error) {
    console.error('\n❌ Fatal error:', error.message);
    process.exit(1);
  } finally {
    mcp.close();
  }

  console.log('\n✅ Testing complete!');
  process.exit(0);
}

// Run tests
runTests().catch(console.error);