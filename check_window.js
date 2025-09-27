import net from 'net';

const socketPath = '/tmp/tauri-mcp-communitas-20224.sock';

const client = net.createConnection(socketPath, () => {
  console.log('Connected to MCP');
  
  // Try to execute JS to check the window location
  const request = {
    command: 'execute_js',
    payload: {
      window_label: 'main',
      code: 'window.location.href'
    }
  };
  
  console.log('Checking window location...');
  client.write(JSON.stringify(request) + '\n');
});

let buffer = '';

client.on('data', (data) => {
  buffer += data.toString();
  
  try {
    const response = JSON.parse(buffer);
    console.log('Response:', response);
    
    if (response.error) {
      console.log('\n❌ JavaScript execution failed:', response.error);
      console.log('This likely means the webview is blank or has an error.');
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
  console.log('Timeout');
  client.end();
}, 5000);
