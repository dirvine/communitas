/**
 * Test Tauri Group Commands
 * 
 * Tests the group management functionality via Tauri MCP
 */

import { offlineStorage } from './services/storage/OfflineStorageService';

// Get invoke function
async function getInvoke() {
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.core?.invoke) {
    return (window as any).__TAURI__.core.invoke;
  }
  
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke;
  } catch {
    throw new Error('Tauri not available');
  }
}

export async function testTauriGroups() {
  console.log('🧪 Testing Tauri Group Commands');
  console.log('================================\n');
  
  try {
    const invoke = await getInvoke();
    const results = [];
    
    // Make sure identity is initialized first
    console.log('0️⃣ Initializing identity...');
    try {
      await invoke('core_initialize', {
        fourWords: 'test-group-admin-user',
        displayName: 'Group Admin',
        deviceName: 'Test Device',
        deviceType: 'Desktop'
      });
      console.log('✅ Identity initialized');
    } catch (e) {
      console.log('⚠️ Identity already initialized or error:', e);
    }
    
    // Test 1: Create a group (four-word identity)
    console.log('\n1️⃣ Creating group...');
    // Generate simple four-word address (alphanumeric words only)
    const suffix = Date.now().toString().slice(-6);
    const groupWords: [string, string, string, string] = [
      'testgroup', 'alpha', 'beta', `g${suffix}`
    ];
    try {
      const groupResult: { id_hex: string; words: [string, string, string, string] } = await invoke('core_group_create', {
        words: groupWords,
      });
      console.log('✅ Group created:', groupResult);
      results.push({ test: 'create_group', success: true, data: groupResult });

      // Store in offline storage
      await offlineStorage.storeContent('group', {
        id_hex: groupResult.id_hex,
        words: groupResult.words,
        isPublic: false,
        members: ['test-group-admin-user'],
        createdAt: new Date().toISOString(),
      });
    } catch (error) {
      console.error('❌ Group creation failed:', error);
      results.push({ test: 'create_group', success: false, error });
    }
    
    // Test 2: Add members to group
    console.log('\n2️⃣ Adding members to group...');
    const testMembers = [
      'alice-test-member-one',
      'bob-test-member-two',
      'charlie-test-member-three',
    ];
    
    for (const member of testMembers) {
      try {
        const parts = member.split('-');
        if (parts.length !== 4) {
          throw new Error(`Invalid member four-words: ${member}`);
        }
        await invoke('core_group_add_member', {
          group_words: groupWords,
          member_words: [parts[0], parts[1], parts[2], parts[3]],
        });
        console.log(`  ✅ Added member: ${member}`);
        results.push({ test: 'add_member', success: true, member });
      } catch (error) {
        console.log(`  ⚠️ Failed to add ${member}:`, error);
        results.push({ test: 'add_member', success: false, member, error });
      }
    }
    
    // Test 3: Remove a member
    console.log('\n3️⃣ Removing a member...');
    try {
      const rem = testMembers[2].split('-');
      await invoke('core_group_remove_member', {
        group_words: groupWords,
        member_words: [rem[0], rem[1], rem[2], rem[3]],
      });
      console.log(`✅ Removed member: ${testMembers[2]}`);
      results.push({ test: 'remove_member', success: true });
    } catch (error) {
      console.log('⚠️ Member removal failed:', error);
      results.push({ test: 'remove_member', success: false, error });
    }
    
    // Test 4: Create multiple groups for organization
    console.log('\n4️⃣ Creating organizational groups...');
    const orgGroups = [
      { words: ['engineering', 'team', 'private', `g${suffix}`] as [string, string, string, string], isPublic: false },
      { words: ['marketing', 'team', 'private', `g${suffix}`] as [string, string, string, string], isPublic: false },
      { words: ['product', 'team', 'private', `g${suffix}`] as [string, string, string, string], isPublic: false },
      { words: ['company', 'announcements', 'public', `g${suffix}`] as [string, string, string, string], isPublic: true },
    ];
    
    for (const group of orgGroups) {
      try {
        const res: { id_hex: string; words: [string, string, string, string] } = await invoke('core_group_create', {
          words: group.words,
        });

        console.log(`  ✅ Created group: ${group.words.join('-')}`);

        // Store in offline storage
        await offlineStorage.storeContent('group', {
          id_hex: res.id_hex,
          words: res.words,
          isPublic: group.isPublic,
          members: ['test-group-admin-user'],
          createdAt: new Date().toISOString(),
        });

        results.push({ test: 'create_org_group', success: true, group: group.words.join('-') });
      } catch (error) {
        console.log(`  ⚠️ Failed to create ${group.words.join('-')}:`, error);
        results.push({ test: 'create_org_group', success: false, group: group.words.join('-'), error });
      }
    }
    
    // Test 5: Test private storage with groups
    console.log('\n5️⃣ Testing private storage for groups...');
    try {
      const groupData = {
        settings: {
          notifications: true,
          autoJoin: false,
          moderators: ['test-group-admin-user']
        },
        metadata: {
          created: new Date().toISOString(),
          lastActivity: new Date().toISOString(),
          messageCount: 0
        }
      };
      
      await invoke('core_private_put', {
        key: `group_settings_${groupWords.join('-')}`,
        content: new TextEncoder().encode(JSON.stringify(groupData)),
      });

      const retrieved: Uint8Array = await invoke('core_private_get', {
        key: `group_settings_${groupWords.join('-')}`,
      });
      
      const decoded = new TextDecoder().decode(new Uint8Array(retrieved));
      const parsed = JSON.parse(decoded);
      
      console.log('✅ Group settings stored and retrieved');
      results.push({ test: 'group_storage', success: true });
    } catch (error) {
      console.log('⚠️ Group storage failed:', error);
      results.push({ test: 'group_storage', success: false, error });
    }
    
    // Summary
    console.log('\n================================');
    console.log('📊 Test Results Summary');
    console.log('================================');
    
    const successful = results.filter(r => r.success).length;
    const failed = results.filter(r => !r.success).length;
    
    console.log(`✅ Successful: ${successful}`);
    console.log(`❌ Failed: ${failed}`);
    
    // List all groups created
    console.log('\n📋 Groups in storage:');
    const groups = await offlineStorage.getContentByType('group');
    groups.forEach(g => {
      const label = g.content.words ? g.content.words.join('-') : g.content.id_hex || 'unknown-group';
      console.log(`  • ${label}`);
      console.log(`    Members: ${g.content.members?.length || 0}`);
      console.log(`    Public: ${g.content.isPublic ? 'Yes' : 'No'}`);
    });
    
    return results;
    
  } catch (error) {
    console.error('❌ Test suite failed:', error);
    throw error;
  }
}

// Test group messaging
export async function testGroupMessaging() {
  console.log('\n💬 Testing Group Messaging');
  console.log('==========================\n');
  
  try {
    const invoke = await getInvoke();
    
    // Get groups from storage
    const groups = await offlineStorage.getContentByType('group');
    if (groups.length === 0) {
      console.log('No groups found. Run testTauriGroups() first.');
      return;
    }
    
    const group = groups[0].content;
    const groupLabel = group.words ? group.words.join('-') : group.id_hex || 'unknown-group';
    console.log(`Using group: ${groupLabel}`);

    // Ensure a channel exists to send messages through
    let channelId: string | null = null;
    try {
      const channel = await (await getInvoke())('core_create_channel', {
        name: `grp-${Date.now().toString().slice(-6)}`,
        description: 'Test channel for group messaging',
      });
      channelId = channel.id; // Channel has id field from saorsa_core::chat::Channel
      console.log(`📡 Created channel: ${channel.name} (${channelId})`);
    } catch (e) {
      console.log('⚠️ Failed to create channel, messaging test may be limited:', e);
    }
    
    // Send messages to group
    const messages = [
      'Hello group members! 👋',
      'This is a test of group messaging.',
      'Everything is working great with offline support!',
      'Post-quantum crypto keeps our messages secure.'
    ];
    
    for (const text of messages) {
      try {
        // Try to send via messaging service (requires channel_id)
        if (!channelId) throw new Error('Missing channelId');
        const recipients: string[] = group.members && Array.isArray(group.members)
          ? group.members
          : ['test-recipient-one-two'];
        const messageId: string = await invoke('core_send_message_to_recipients', {
          channel_id: channelId,
          recipients,
          text,
        });
        
        console.log(`✅ Sent: "${text}"`);
        
        // Store in offline storage
        await offlineStorage.storeContent('group_message', {
          groupId: group.id_hex ?? (group.words ? group.words.join('-') : 'unknown-group'),
          messageId,
          text,
          author: 'test-group-admin-user',
          timestamp: new Date().toISOString()
        });
        
      } catch (error) {
        console.log(`⚠️ Message failed (storing offline): "${text}"`);
        
        // Store offline anyway
        await offlineStorage.storeContent('group_message', {
          groupId: group.id_hex ?? (group.words ? group.words.join('-') : 'unknown-group'),
          text,
          author: 'test-group-admin-user',
          timestamp: new Date().toISOString(),
          pending: true
        });
      }
    }
    
    // Retrieve messages
    console.log('\n📥 Group messages in storage:');
    const groupMessages = await offlineStorage.getContentByType('group_message');
    groupMessages.forEach(msg => {
      const time = new Date(msg.content.timestamp).toLocaleTimeString();
      console.log(`  [${time}] ${msg.content.text}`);
      if (msg.content.pending) {
        console.log('    ⏳ Pending sync');
      }
    });
    
  } catch (error) {
    console.error('❌ Group messaging test failed:', error);
  }
}

// Auto-expose to window
if (typeof window !== 'undefined') {
  (window as any).tauriGroups = {
    test: testTauriGroups,
    testMessaging: testGroupMessaging,
    listGroups: async () => {
      const groups = await offlineStorage.getContentByType('group');
      console.log('📋 Groups:');
      groups.forEach(g => console.log(`  • ${g.content.name}`));
      return groups;
    }
  };
  
  console.log('🎯 Tauri Groups Test Loaded!');
  console.log('Commands:');
  console.log('  window.tauriGroups.test()         - Test group creation');
  console.log('  window.tauriGroups.testMessaging() - Test group messaging');
  console.log('  window.tauriGroups.listGroups()   - List all groups');
}
