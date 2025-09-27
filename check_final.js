import net from 'net';

const socketPath = '/tmp/tauri-mcp-communitas-32206.sock';

const client = net.createConnection(socketPath, () => {
  console.log('Connected to MCP');
  
  const request = {
    command: 'get_dom',
    payload: {
      window_label: 'main'
    }
  };
  
  console.log('Getting DOM from Tauri window...');
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
          
          console.log('\n🎯 TAURI WINDOW CHECK:');
          if (isMock) {
            console.log('❌ APP STILL NOT LOADED!');
            console.log('The webview is not showing the React app.');
          } else {
            console.log('✅ APP IS LOADED SUCCESSFULLY!');
            
            // Check for login elements
            const hasLogin = dom.includes('login') || dom.includes('Login') || dom.includes('four-words') || dom.includes('Four Words');
            const hasInput = dom.includes('<input') || dom.includes('input type');
            const hasButton = dom.includes('<button') || dom.includes('Button');
            
            console.log('\n📱 UI Elements:');
            console.log('- Has login/registration:', hasLogin);
            console.log('- Has input fields:', hasInput);
            console.log('- Has buttons:', hasButton);
            
            console.log('\n📄 DOM Preview (first 600 chars):');
            console.log(dom.substring(0, 600));
            
            if (hasLogin || hasInput) {
              console.log('\n🎉 You can now create a user!');
              console.log('Look for the Four Words input field in the Tauri window.');
            }
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
