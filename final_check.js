import net from 'net';

const socketPath = '/tmp/tauri-mcp-communitas-26263.sock';

const client = net.createConnection(socketPath, () => {
  console.log('Connected to MCP');
  
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
  
  const lines = buffer.split('\n');
  for (let line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        
        if (response.data) {
          const dom = response.data;
          const isMock = dom.includes('Mock DOM');
          
          console.log('\n=== TAURI WINDOW STATUS ===');
          if (isMock) {
            console.log('❌ APP NOT LOADED - Mock DOM returned');
            console.log('\nThe Tauri webview is NOT loading the app.');
            console.log('This is why you see a blank or error window.');
          } else {
            console.log('✅ APP LOADED - Real DOM detected!');
            console.log('\nDOM preview:');
            console.log(dom.substring(0, 500));
          }
        }
        
        client.end();
        break;
      } catch (e) {}
    }
  }
});

client.on('error', (err) => {
  console.error('Error:', err.message);
});

client.on('close', () => {
  process.exit(0);
});

setTimeout(() => {
  console.log('Timeout');
  client.end();
}, 5000);
