import net from 'net';

const socketPath = '/tmp/tauri-mcp-communitas-20224.sock';

const client = net.createConnection(socketPath, () => {
  console.log('Connected to MCP');
  
  // Try to get DOM to see what's actually loaded
  const request = {
    command: 'get_dom',
    payload: {
      window_label: 'main'
    }
  };
  
  console.log('Getting DOM...');
  client.write(JSON.stringify(request) + '\n');
});

let buffer = '';

client.on('data', (data) => {
  buffer += data.toString();
  
  try {
    const response = JSON.parse(buffer);
    
    if (response.success && response.data) {
      const dom = response.data;
      console.log('\n=== CURRENT DOM IN TAURI WINDOW ===');
      console.log('Length:', dom.length);
      
      // Check what's actually there
      if (dom.includes('Mock DOM')) {
        console.log('\n❌ MOCK DOM - This means the frontend is NOT receiving events!');
        console.log('The Tauri window might be blank or showing an error.');
      } else {
        console.log('\n✅ Real DOM detected!');
      }
      
      console.log('\nFirst 1000 characters:');
      console.log(dom.substring(0, 1000));
    }
    
    client.end();
  } catch (e) {
    // Continue buffering
  }
});

client.on('error', (err) => {
  console.error('Error:', err.message);
});

client.on('close', () => {
  process.exit(0);
});

setTimeout(() => {
  console.log('Timeout - no response from get_dom');
  client.end();
}, 5000);
