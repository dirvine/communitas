#!/usr/bin/env node

/**
 * Collaborative Editing Test Script
 *
 * This script helps test the collaborative editing functionality by:
 * 1. Opening multiple browser tabs/windows
 * 2. Providing test scenarios
 * 3. Monitoring synchronization
 */

import { exec } from 'child_process';

console.log('🚀 Communitas Collaborative Editing Test Suite');
console.log('==============================================\n');

// Test scenarios
const scenarios = [
  {
    name: 'Single User Test',
    description: 'Test basic editor functionality with one user',
    url: 'http://localhost:1422/test/collaboration',
    instructions: [
      'Open the collaborative editor',
      'Type some text and verify it saves',
      'Test the preview and split view modes',
      'Try the toolbar controls (save, fullscreen, etc.)'
    ]
  },
  {
    name: 'Multi-User Synchronization Test',
    description: 'Test real-time collaboration between multiple users',
    url: 'http://localhost:1422/test/collaboration',
    instructions: [
      'Open multiple browser tabs/windows',
      'Start typing in different sections simultaneously',
      'Verify changes sync between all tabs',
      'Test cursor visibility and user awareness',
      'Try conflicting edits to test conflict resolution'
    ]
  },
  {
    name: 'Offline Persistence Test',
    description: 'Test offline editing and synchronization',
    url: 'http://localhost:1422/test/collaboration',
    instructions: [
      'Type some content while online',
      'Close the browser tab',
      'Reopen the tab (content should persist)',
      'Make changes offline',
      'Reconnect and verify synchronization'
    ]
  }
];

function printScenario(scenario, index) {
  console.log(`${index + 1}. ${scenario.name}`);
  console.log(`   ${scenario.description}`);
  console.log(`   URL: ${scenario.url}`);
  console.log('   Instructions:');
  scenario.instructions.forEach(instruction => {
    console.log(`     • ${instruction}`);
  });
  console.log('');
}

function openBrowser(url) {
  const command = process.platform === 'darwin'
    ? `open "${url}"`
    : process.platform === 'win32'
    ? `start "${url}"`
    : `xdg-open "${url}"`;

  exec(command, (error) => {
    if (error) {
      console.log(`Please manually open: ${url}`);
    }
  });
}

console.log('📋 Available Test Scenarios:\n');

scenarios.forEach((scenario, index) => {
  printScenario(scenario, index);
});

console.log('🔧 Quick Test Commands:');
console.log('  npm run dev                    # Start development server');
console.log('  node test-collaboration.js     # Show this help');
console.log('');

console.log('🎯 Recommended Testing Flow:');
console.log('1. Start the development server: npm run dev');
console.log('2. Open http://localhost:1422 in your browser');
console.log('3. Click "🧪 Test Collaborative Editing" button');
console.log('4. Test single-user functionality first');
console.log('5. Open multiple tabs for multi-user testing');
console.log('6. Test offline persistence by closing/reopening tabs');
console.log('');

console.log('📊 What to Look For:');
console.log('✅ Real-time text synchronization');
console.log('✅ User cursor visibility');
console.log('✅ Conflict-free editing');
console.log('✅ Offline content persistence');
console.log('✅ Professional editor UI');
console.log('✅ Markdown preview functionality');
console.log('');

console.log('🛠️  Troubleshooting:');
console.log('• If synchronization doesn\'t work, check browser console for errors');
console.log('• Ensure all tabs use the same room/session');
console.log('• Try refreshing tabs if connection issues occur');
console.log('• Check IndexedDB for offline persistence');
console.log('');

console.log('🎉 Ready to test collaborative editing!');
console.log('   Visit: http://localhost:1422/test/collaboration');

// Auto-open browser if requested
if (process.argv.includes('--open')) {
  console.log('\n🌐 Opening browser tabs...');
  setTimeout(() => {
    openBrowser('http://localhost:1422');
    setTimeout(() => {
      openBrowser('http://localhost:1422/test/collaboration');
    }, 1000);
  }, 2000);
}

if (process.argv.includes('--help') || process.argv.includes('-h')) {
  console.log('\n📖 Usage:');
  console.log('  node test-collaboration.js           # Show help');
  console.log('  node test-collaboration.js --open    # Auto-open browser');
  console.log('  node test-collaboration.js --help    # Show this help');
}