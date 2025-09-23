#!/usr/bin/env node

import net from 'net';

const SOCKET_PATH = '/tmp/tauri-mcp-communitas.sock';

console.log('Testing connection to Tauri MCP socket...');

const client = net.createConnection({ path: SOCKET_PATH }, () => {
    console.log('✅ Connected to Tauri MCP socket!');
    
    // Test ping command
    const pingCommand = JSON.stringify({
        id: 1,
        method: 'ping',
        params: {}
    });
    
    console.log('Sending ping command...');
    client.write(pingCommand);
});

client.on('data', (data) => {
    console.log('Received response:', data.toString());
    
    // Test screenshot command
    const screenshotCommand = JSON.stringify({
        id: 2,
        method: 'take_screenshot',
        params: {
            window_title: 'Communitas',
            quality: 80,
            width: 800,
            height: 600
        }
    });
    
    console.log('Sending screenshot command...');
    client.write(screenshotCommand);
    
    setTimeout(() => {
        client.end();
        process.exit(0);
    }, 2000);
});

client.on('error', (err) => {
    console.error('❌ Connection error:', err.message);
    process.exit(1);
});

client.on('end', () => {
    console.log('Connection closed');
});