import net from 'net';
import fs from 'fs';

const socketPath = '/tmp/tauri-mcp-communitas-20224.sock';

const client = net.createConnection(socketPath, () => {
  console.log('Connected to MCP');
  
  const request = {
    command: 'take_screenshot',
    payload: {
      window_label: 'main',
      format: 'png'
    }
  };
  
  console.log('Taking screenshot...');
  client.write(JSON.stringify(request) + '\n');
});

let buffer = '';

client.on('data', (data) => {
  buffer += data.toString();
  
  // Try to find complete JSON
  const lines = buffer.split('\n');
  
  for (let line of lines) {
    if (line.trim()) {
      try {
        const response = JSON.parse(line);
        
        if (response.success && response.data && response.data.data) {
          console.log('✅ Screenshot captured!');
          
          // Save the screenshot
          const base64Data = response.data.data;
          const imageBuffer = Buffer.from(base64Data, 'base64');
          
          fs.writeFileSync('tauri-window.png', imageBuffer);
          console.log('Screenshot saved to tauri-window.png');
          console.log('You can open this file to see what the Tauri window is showing.');
          
          client.end();
          return;
        } else if (response.error) {
          console.log('❌ Error:', response.error);
          client.end();
          return;
        }
      } catch (e) {
        // Not complete JSON yet
      }
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
}, 10000);
