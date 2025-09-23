#!/usr/bin/env node

import net from 'net';
import fs from 'fs';

// Find the latest socket
const socketFiles = fs.readdirSync('/tmp').filter(f => f.startsWith('tauri-mcp-communitas-') && f.endsWith('.sock'));
const latestSocket = socketFiles.sort().reverse()[0];
const SOCKET_PATH = `/tmp/${latestSocket}`;

console.log('Using socket:', SOCKET_PATH);

const client = net.createConnection({ path: SOCKET_PATH }, () => {
    console.log('Connected!\n');
    
    // Try multiple commands to see what works
    const commands = [
        // First, simple ping
        { command: 'ping', payload: {} },
        
        // Try to get DOM with correct field names
        { command: 'get_dom', payload: { window_label: 'main' } },
        
        // Try execute_js with simple code
        { command: 'execute_js', payload: { 
            window_label: 'main', 
            code: '"test"' 
        }},
        
        // Try screenshot with minimal params
        { command: 'take_screenshot', payload: { 
            window_label: 'main'
        }},
    ];
    
    let cmdIndex = 0;
    let buffer = '';
    
    function sendNext() {
        if (cmdIndex < commands.length) {
            const cmd = commands[cmdIndex];
            console.log(`Sending: ${cmd.command}`);
            client.write(JSON.stringify(cmd) + '\n');
            cmdIndex++;
        } else {
            client.end();
        }
    }
    
    client.on('data', (data) => {
        buffer += data.toString();
        
        // Process newline-delimited JSON
        let lines = buffer.split('\n');
        buffer = lines.pop() || '';
        
        for (const line of lines) {
            if (line.trim()) {
                try {
                    const resp = JSON.parse(line);
                    console.log('Response:', JSON.stringify(resp, null, 2).substring(0, 500));
                    
                    // Save any useful data
                    if (resp.data?.screenshot) {
                        const base64 = resp.data.screenshot.replace(/^data:image\/png;base64,/, '');
                        fs.writeFileSync('mcp-screenshot.png', base64, 'base64');
                        console.log('Screenshot saved to mcp-screenshot.png');
                    }
                    if (resp.data?.dom) {
                        fs.writeFileSync('mcp-dom.html', resp.data.dom);
                        console.log('DOM saved to mcp-dom.html');
                    }
                    if (resp.data?.result) {
                        console.log('JS Result:', resp.data.result);
                    }
                } catch (e) {
                    console.log('Raw:', line.substring(0, 200));
                }
                
                // Send next command
                setTimeout(sendNext, 100);
            }
        }
    });
    
    // Start sending commands
    sendNext();
});

client.on('error', (err) => {
    console.error('Error:', err.message);
    
    // Try to list available sockets
    console.log('\nAvailable MCP sockets:');
    const sockets = fs.readdirSync('/tmp').filter(f => f.includes('tauri-mcp'));
    sockets.forEach(s => console.log('  -', s));
});

client.on('end', () => {
    console.log('\nDone');
});