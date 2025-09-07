/**
 * Setup Test Workspace
 * 
 * Creates a complete test environment with:
 * - Identity
 * - Organization
 * - Channels
 * - Projects
 * - Messages
 */

import { offlineStorage } from './services/storage/OfflineStorageService';

// Get invoke function (nullable in browser/offline mode)
async function getInvoke(): Promise<((cmd: string, args?: any) => Promise<any>) | null> {
  if (typeof window !== 'undefined' && (window as any).__TAURI__?.core?.invoke) {
    return (window as any).__TAURI__.core.invoke;
  }
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke;
  } catch {
    return null; // Offline/browser mode
  }
}

export async function setupTestWorkspace() {
  console.log('🚀 Setting up Test Workspace');
  console.log('============================\n');
  
  try {
    const invoke = await getInvoke();
    
    // Step 1: Initialize Identity
    console.log('1️⃣ Creating Identity...');
    const identity = {
      fourWords: 'saorsa-labs-test-workspace',
      displayName: 'David Irvine',
      deviceName: 'MacBook Pro',
      deviceType: 'Desktop'
    };
    
    if (invoke) {
      try {
        await invoke('core_initialize', identity);
        console.log('✅ Identity created via Tauri:', identity.fourWords);
      } catch (e) {
        console.log('⚠️ Tauri initialize failed, continuing offline:', e);
      }
    } else {
      console.log('ℹ️ Tauri not available, creating identity offline');
    }
    
    // Store identity in offline storage
    await offlineStorage.store('primary_identity', {
      ...identity,
      id: `id_${Date.now()}`,
      publicKey: 'generated_public_key',
      createdAt: new Date().toISOString(),
      isActive: true
    });
    
    // Step 2: Create Organization
    console.log('\n2️⃣ Creating Organization...');
    const organization = {
      id: `org_${Date.now()}`,
      name: 'Saorsa Labs',
      description: 'Building the future of decentralized collaboration',
      fourWordAddress: 'saorsa-labs-main-org',
      members: [identity.fourWords],
      createdAt: new Date().toISOString(),
      settings: {
        isPublic: false,
        allowInvites: true,
        requireApproval: true
      }
    };
    
    await offlineStorage.storeContent('organization', organization);
    console.log('✅ Organization created:', organization.name);
    
    // Step 3: Create Channels
    console.log('\n3️⃣ Creating Channels...');
    const channels = [
      {
        id: `channel_general_${Date.now()}`,
        name: 'general',
        description: 'General discussion',
        type: 'public',
        organizationId: organization.id,
        members: [identity.fourWords],
        createdAt: new Date().toISOString()
      },
      {
        id: `channel_dev_${Date.now() + 1}`,
        name: 'development',
        description: 'Development discussions',
        type: 'public',
        organizationId: organization.id,
        members: [identity.fourWords],
        createdAt: new Date().toISOString()
      },
      {
        id: `channel_random_${Date.now() + 2}`,
        name: 'random',
        description: 'Off-topic chat',
        type: 'public',
        organizationId: organization.id,
        members: [identity.fourWords],
        createdAt: new Date().toISOString()
      },
      {
        id: `channel_announcements_${Date.now() + 3}`,
        name: 'announcements',
        description: 'Important announcements',
        type: 'public',
        organizationId: organization.id,
        members: [identity.fourWords],
        pinned: true,
        createdAt: new Date().toISOString()
      }
    ];
    
    for (const channel of channels) {
      if (invoke) {
        try {
          await invoke('core_create_channel', {
            name: channel.name,
            members: channel.members
          });
          console.log(`  ✅ Created channel via Tauri: #${channel.name}`);
        } catch {
          console.log(`  ⚠️ Using offline storage for #${channel.name}`);
        }
      }
      
      // Store in offline storage regardless
      await offlineStorage.storeContent('channel', channel);
    }
    
    // Step 4: Create Projects
    console.log('\n4️⃣ Creating Projects...');
    const projects = [
      {
        id: `project_communitas_${Date.now()}`,
        name: 'Communitas Development',
        description: 'Main Communitas application development',
        organizationId: organization.id,
        status: 'active',
        team: [identity.fourWords],
        tasks: [
          { name: 'Implement P2P messaging', status: 'completed' },
          { name: 'Add file sharing', status: 'in_progress' },
          { name: 'Create mobile app', status: 'planned' }
        ],
        createdAt: new Date().toISOString()
      },
      {
        id: `project_saorsa_${Date.now() + 1}`,
        name: 'Saorsa Core',
        description: 'Core P2P networking library',
        organizationId: organization.id,
        status: 'active',
        team: [identity.fourWords],
        tasks: [
          { name: 'Implement DHT', status: 'completed' },
          { name: 'Add QUIC transport', status: 'completed' },
          { name: 'Optimize routing', status: 'in_progress' }
        ],
        createdAt: new Date().toISOString()
      },
      {
        id: `project_website_${Date.now() + 2}`,
        name: 'Website Redesign',
        description: 'New marketing website',
        organizationId: organization.id,
        status: 'planning',
        team: [identity.fourWords],
        tasks: [
          { name: 'Design mockups', status: 'in_progress' },
          { name: 'Content writing', status: 'planned' },
          { name: 'Deploy to production', status: 'planned' }
        ],
        createdAt: new Date().toISOString()
      }
    ];
    
    for (const project of projects) {
      await offlineStorage.storeContent('project', project);
      console.log(`  ✅ Created project: ${project.name}`);
    }
    
    // Step 5: Add Messages
    console.log('\n5️⃣ Adding Messages...');
    const messages = [
      {
        channelId: channels[0].id,
        text: 'Welcome to Communitas! 🎉',
        author: identity.fourWords,
        timestamp: new Date(Date.now() - 3600000).toISOString()
      },
      {
        channelId: channels[0].id,
        text: 'The P2P network is now fully operational',
        author: identity.fourWords,
        timestamp: new Date(Date.now() - 1800000).toISOString()
      },
      {
        channelId: channels[1].id,
        text: 'Just pushed the latest updates to saorsa-core v0.3.17',
        author: identity.fourWords,
        timestamp: new Date(Date.now() - 900000).toISOString()
      },
      {
        channelId: channels[1].id,
        text: 'Post-quantum cryptography is now integrated! ML-DSA and ML-KEM are working perfectly.',
        author: identity.fourWords,
        timestamp: new Date(Date.now() - 600000).toISOString()
      },
      {
        channelId: channels[2].id,
        text: 'Anyone tried the new offline mode? Works like magic! ✨',
        author: identity.fourWords,
        timestamp: new Date(Date.now() - 300000).toISOString()
      },
      {
        channelId: channels[3].id,
        text: '📢 Important: We are now fully offline-first! All data is cached locally and syncs when online.',
        author: identity.fourWords,
        timestamp: new Date(Date.now() - 60000).toISOString(),
        pinned: true
      }
    ];
    
    for (const message of messages) {
      if (invoke) {
        try {
          await invoke('core_send_message_to_channel', {
            channel_id: message.channelId,
            text: message.text
          });
        } catch {
          // Fallback to offline storage
        }
      }
      
      await offlineStorage.storeContent('message', message);
    }
    console.log(`  ✅ Added ${messages.length} messages`);
    
    // Step 6: Add Files/Documents
    console.log('\n6️⃣ Adding Documents...');
    const documents = [
      {
        id: `doc_readme_${Date.now()}`,
        name: 'README.md',
        type: 'markdown',
        projectId: projects[0].id,
        content: '# Communitas\n\nDecentralized collaboration platform',
        size: 1024,
        createdAt: new Date().toISOString()
      },
      {
        id: `doc_spec_${Date.now() + 1}`,
        name: 'SPECIFICATION.md',
        type: 'markdown',
        projectId: projects[0].id,
        content: '# Technical Specification\n\n## Architecture\n- P2P Network\n- Post-Quantum Crypto\n- Virtual Disks',
        size: 2048,
        createdAt: new Date().toISOString()
      },
      {
        id: `doc_api_${Date.now() + 2}`,
        name: 'API.md',
        type: 'markdown',
        projectId: projects[1].id,
        content: '# Saorsa Core API\n\n## Functions\n- create_identity()\n- send_message()\n- store_data()',
        size: 1536,
        createdAt: new Date().toISOString()
      }
    ];
    
    for (const doc of documents) {
      await offlineStorage.storeContent('document', doc);
      // Also store as file
      const content = new TextEncoder().encode(doc.content);
      await offlineStorage.storeFile(doc.name, content.buffer, {
        projectId: doc.projectId,
        type: doc.type,
        size: doc.size
      });
      console.log(`  ✅ Added document: ${doc.name}`);
    }
    
    // Step 7: Add Team Members (simulated)
    console.log('\n7️⃣ Adding Team Members...');
    const teamMembers = [
      {
        fourWords: 'alice-ocean-mountain-star',
        name: 'Alice Johnson',
        role: 'Developer',
        organizationId: organization.id,
        avatar: '👩‍💻',
        status: 'online'
      },
      {
        fourWords: 'bob-forest-river-moon',
        name: 'Bob Smith',
        role: 'Designer',
        organizationId: organization.id,
        avatar: '👨‍🎨',
        status: 'away'
      },
      {
        fourWords: 'charlie-valley-cloud-sun',
        name: 'Charlie Davis',
        role: 'Product Manager',
        organizationId: organization.id,
        avatar: '👨‍💼',
        status: 'offline'
      }
    ];
    
    for (const member of teamMembers) {
      await offlineStorage.storeContent('team_member', member);
      console.log(`  ✅ Added team member: ${member.name} (${member.fourWords})`);
    }
    
    // Step 8: Store workspace configuration
    console.log('\n8️⃣ Saving Workspace Configuration...');
    const workspace = {
      id: `workspace_${Date.now()}`,
      name: 'Saorsa Labs Workspace',
      primaryIdentity: identity.fourWords,
      organizationId: organization.id,
      channelIds: channels.map(c => c.id),
      projectIds: projects.map(p => p.id),
      teamMemberIds: teamMembers.map(m => m.fourWords),
      createdAt: new Date().toISOString(),
      lastAccessed: new Date().toISOString()
    };
    
    await offlineStorage.store('current_workspace', workspace, {
      encrypt: true,
      syncOnline: true
    });
    console.log('✅ Workspace configuration saved');
    
    // Summary
    console.log('\n============================');
    console.log('✨ Test Workspace Created Successfully!');
    console.log('============================');
    console.log(`  Identity: ${identity.displayName} (${identity.fourWords})`);
    console.log(`  Organization: ${organization.name}`);
    console.log(`  Channels: ${channels.length} created`);
    console.log(`  Projects: ${projects.length} created`);
    console.log(`  Messages: ${messages.length} added`);
    console.log(`  Documents: ${documents.length} added`);
    console.log(`  Team Members: ${teamMembers.length} added`);
    
    // Get storage stats
    const stats = await offlineStorage.getStats();
    console.log('\n📊 Storage Statistics:');
    console.log(`  Cache Size: ${stats.cacheSize} items`);
    console.log(`  Content Count: ${stats.contentCount}`);
    console.log(`  File Count: ${stats.fileCount}`);
    console.log(`  Network Status: ${stats.isOnline ? '🌐 Online' : '📵 Offline'}`);
    
    console.log('\n💡 You can now:');
    console.log('  • Browse channels and see messages');
    console.log('  • View and manage projects');
    console.log('  • Access documents offline');
    console.log('  • Chat with team members (when online)');
    console.log('  • Everything is cached for offline access!');
    
    return workspace;
    
  } catch (error) {
    console.error('❌ Setup failed:', error);
    throw error;
  }
}

// Function to clear all test data
export async function clearTestWorkspace() {
  console.log('🧹 Clearing Test Workspace...');
  
  await offlineStorage.clearCache();
  localStorage.removeItem('communitas-onboarded');
  localStorage.removeItem('current_workspace');
  
  console.log('✅ Workspace cleared');
}

// Function to list workspace contents
export async function listWorkspaceContents() {
  console.log('📋 Workspace Contents');
  console.log('====================\n');
  
  const workspace = await offlineStorage.get('current_workspace');
  if (!workspace) {
    console.log('No workspace found. Run setupTestWorkspace() first.');
    return;
  }
  
  console.log('📁 Organizations:');
  const orgs = await offlineStorage.getContentByType('organization');
  orgs.forEach(org => console.log(`  • ${org.content.name}`));
  
  console.log('\n💬 Channels:');
  const channels = await offlineStorage.getContentByType('channel');
  channels.forEach(ch => console.log(`  #${ch.content.name} - ${ch.content.description}`));
  
  console.log('\n📂 Projects:');
  const projects = await offlineStorage.getContentByType('project');
  projects.forEach(proj => console.log(`  • ${proj.content.name} (${proj.content.status})`));
  
  console.log('\n👥 Team Members:');
  const members = await offlineStorage.getContentByType('team_member');
  members.forEach(member => console.log(`  • ${member.content.name} - ${member.content.role} (${member.content.status})`));
  
  console.log('\n📄 Documents:');
  const docs = await offlineStorage.getContentByType('document');
  docs.forEach(doc => console.log(`  • ${doc.content.name} (${doc.content.size} bytes)`));
  
  console.log('\n💬 Recent Messages:');
  const messages = await offlineStorage.getContentByType('message');
  messages.slice(-5).forEach(msg => console.log(`  "${msg.content.text.substring(0, 50)}..."`));
  
  const stats = await offlineStorage.getStats();
  console.log('\n📊 Total Items:', stats.contentCount);
}

// Auto-expose to window
if (typeof window !== 'undefined') {
  (window as any).workspace = {
    setup: setupTestWorkspace,
    clear: clearTestWorkspace,
    list: listWorkspaceContents,
    storage: offlineStorage
  };
  
  console.log('🎯 Workspace Manager Loaded!');
  console.log('Commands:');
  console.log('  window.workspace.setup() - Create test workspace');
  console.log('  window.workspace.list()  - List workspace contents');
  console.log('  window.workspace.clear() - Clear all data');
}
