import net from 'net';

const socketPath = '/tmp/tauri-mcp-communitas-17742.sock';

console.log('Testing MCP integration with Communitas app...\n');

const client = net.createConnection(socketPath, () => {
  console.log('✅ Connected to MCP server');
  
  // Test DOM capture
  const request = {
    command: 'get_dom',
    payload: {
      window_label: 'main'
    }
  };
  
  console.log('📸 Requesting DOM capture...');
  client.write(JSON.stringify(request) + '\n');
});

let buffer = '';

client.on('data', (data) => {
  buffer += data.toString();
  
  const lines = buffer.split('\n');
  for (let line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        
        if (response.success && response.data) {
          const domStr = typeof response.data === 'string' ? response.data : JSON.stringify(response.data);
          const isMock = domStr.includes('Mock DOM');
          const hasRoot = domStr.includes('id="root"');
          const hasReact = domStr.includes('React') || domStr.includes('react');
          const hasCommunitas = domStr.includes('Communitas');
          
          console.log('\n=== MCP DOM CAPTURE RESULT ===');
          console.log('✅ Success:', response.success);
          console.log('📄 DOM Length:', domStr.length, 'characters');
          console.log('🎭 Is Mock DOM:', isMock ? '❌ YES (listeners not working)' : '✅ NO (real DOM!)');
          console.log('🌳 Has React Root:', hasRoot);
          console.log('⚛️  Has React Content:', hasReact);
          console.log('🏠 Has Communitas App:', hasCommunitas);
          
          if (!isMock) {
            console.log('\n🎉 SUCCESS! MCP is capturing real DOM content!');
            console.log('\n📝 DOM Preview (first 600 chars):');
            console.log('---');
            console.log(domStr.substring(0, 600));
            console.log('---\n');
            
            // Now test if we can find UI elements
            const hasLoginForm = domStr.includes('four-words') || domStr.includes('login') || domStr.includes('Four Words');
            const hasButtons = domStr.includes('<button') || domStr.includes('Button');
            
            console.log('🔍 UI Analysis:');
            console.log('   Login/Four Words form:', hasLoginForm ? '✅ Found' : '❌ Not found');
            console.log('   Buttons:', hasButtons ? '✅ Found' : '❌ Not found');
            
            if (hasLoginForm) {
              console.log('\n✨ The app appears to be at the login/registration screen!');
              console.log('   You should be able to create a user now.');
            }
          } else {
            console.log('\n⚠️  WARNING: Still getting mock DOM');
            console.log('   The frontend MCP listeners are not active.');
            console.log('   Check the browser console for errors.');
          }
        } else if (response.error) {
          console.log('\n❌ Error:', response.error);
        }
        
        client.end();
        break;
      } catch (e) {
        // Continue buffering
      }
    }
  }
});

client.on('error', (err) => {
  console.error('❌ Connection error:', err.message);
});

client.on('close', () => {
  console.log('\n👋 Connection closed');
  process.exit(0);
});

setTimeout(() => {
  console.log('\n⏱️  Timeout reached');
  client.end();
}, 5000);
