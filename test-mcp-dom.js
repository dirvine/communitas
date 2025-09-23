#!/usr/bin/env node

import net from 'net';
import fs from 'fs';

const SOCKET_PATH = '/tmp/tauri-mcp-communitas-65268.sock';

console.log('Connecting to MCP socket:', SOCKET_PATH);

const commands = [
    {
        name: 'get_dom',
        cmd: {
            command: 'get_dom',
            payload: {
                window_label: 'main'
            }
        }
    },
    {
        name: 'execute_js',
        cmd: {
            command: 'execute_js',
            payload: {
                window_label: 'main',
                script: `
                    JSON.stringify({
                        hasTauri: typeof window.__TAURI__ !== 'undefined',
                        hasRoot: !!document.getElementById('root'),
                        rootHTML: (document.getElementById('root')?.innerHTML || '').substring(0, 500),
                        bodyHTML: document.body.innerHTML.substring(0, 500),
                        url: window.location.href,
                        title: document.title,
                        scripts: Array.from(document.scripts).map(s => s.src || 'inline').slice(0, 5)
                    })
                `
            }
        }
    },
    {
        name: 'ping',
        cmd: {
            command: 'ping',
            payload: {}
        }
    }
];

let currentCommandIndex = 0;
let buffer = '';

const client = net.createConnection({ path: SOCKET_PATH }, () => {
    console.log('✅ Connected to Tauri MCP socket!\n');
    sendNextCommand();
});

function sendNextCommand() {
    if (currentCommandIndex >= commands.length) {
        client.end();
        return;
    }
    
    const current = commands[currentCommandIndex];
    console.log(`Sending command: ${current.name}`);
    client.write(JSON.stringify(current.cmd) + '\n');
}

client.on('data', (data) => {
    buffer += data.toString();
    
    let newlineIndex;
    while ((newlineIndex = buffer.indexOf('\n')) !== -1) {
        const jsonStr = buffer.substring(0, newlineIndex);
        buffer = buffer.substring(newlineIndex + 1);
        
        try {
            const response = JSON.parse(jsonStr);
            const cmdName = commands[currentCommandIndex].name;
            
            console.log(`\nResponse for ${cmdName}:`);
            console.log('Success:', response.success);
            
            if (response.error) {
                console.log('Error:', response.error);
            }
            
            if (response.data) {
                if (response.data.dom) {
                    const dom = response.data.dom;
                    console.log('DOM length:', dom.length);
                    console.log('First 300 chars:', dom.substring(0, 300));
                    
                    // Save full DOM
                    fs.writeFileSync('tauri-dom.html', dom);
                    console.log('Full DOM saved to tauri-dom.html');
                    
                } else if (response.data.result) {
                    try {
                        const jsResult = JSON.parse(response.data.result);
                        console.log('JavaScript result:', JSON.stringify(jsResult, null, 2));
                    } catch (e) {
                        console.log('Raw result:', response.data.result);
                    }
                } else {
                    console.log('Data:', JSON.stringify(response.data, null, 2));
                }
            }
            
            currentCommandIndex++;
            sendNextCommand();
            
        } catch (err) {
            console.error('Parse error:', err.message);
        }
    }
});

client.on('error', (err) => {
    console.error('❌ Connection error:', err.message);
    process.exit(1);
});

client.on('end', () => {
    console.log('\n✅ All commands completed');
});