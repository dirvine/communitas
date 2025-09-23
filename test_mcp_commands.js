import net from 'net';

function sendCommand(socket, command, payload = {}) {
  const message = JSON.stringify({ command, payload }) + '\n';
  console.log(`Sending: ${command}`, payload);
  socket.write(message);
}

function testCommand(command, payload = {}) {
  return new Promise((resolve, reject) => {
    const client = net.createConnection({ host: '127.0.0.1', port: 9999 }, () => {
      console.log(`\n=== Testing ${command} ===`);
      sendCommand(client, command, payload);
    });

    let response = '';
    client.on('data', (data) => {
      response += data.toString();
      if (response.includes('\n')) {
        console.log('Received:', response.trim());
        client.end();
        resolve(JSON.parse(response.trim()));
      }
    });

    client.on('error', (err) => {
      console.error('Connection error:', err.message);
      reject(err);
    });

    client.on('end', () => {
      console.log('Disconnected');
    });

    // Timeout after 10 seconds
    setTimeout(() => {
      client.end();
      reject(new Error('Timeout'));
    }, 10000);
  });
}

async function runTests() {
  try {
    // Test ping
    await testCommand('ping');

    // Test execute_js to see if webview is responsive
    try {
      const result = await testCommand('execute_js', {
        window_label: 'main',
        code: 'console.log("Test from MCP"); return "Hello from webview";'
      });
      console.log('execute_js result:', result);
    } catch (e) {
      console.log('execute_js failed:', e.message);
    }

    // Test get_dom (might fail if window not ready)
    try {
      await testCommand('get_dom', { window_label: 'main' });
    } catch (e) {
      console.log('get_dom failed (expected if window not ready):', e.message);
    }

    // Test manage_window to see if window operations work
    try {
      const result = await testCommand('manage_window', {
        window_label: 'main',
        operation: 'focus'
      });
      console.log('manage_window result:', result);
    } catch (e) {
      console.log('manage_window failed:', e.message);
    }

    // Test take_screenshot (might fail if window not ready)
    try {
      await testCommand('take_screenshot', { window_label: 'main' });
    } catch (e) {
      console.log('take_screenshot failed (expected if window not ready):', e.message);
    }

  } catch (error) {
    console.error('Test failed:', error);
  }
}

runTests();