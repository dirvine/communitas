#!/usr/bin/env node

import net from 'net';

const SOCKET_PATH = '/tmp/tauri-mcp-communitas.sock';

console.log('Testing MCP commands directly...');

const client = net.createConnection({ path: SOCKET_PATH }, () => {
    console.log('✅ Connected to Tauri MCP socket!');
    
    // First take a screenshot
    const screenshotCmd = JSON.stringify({
        command: 'take_screenshot',
        payload: {
            window_title: 'Communitas',
            quality: 80,
            width: 1024,
            height: 768
        }
    }) + '\n';
    
    console.log('Sending screenshot command...');
    client.write(screenshotCmd);
});

client.on('data', (data) => {
    const response = data.toString();
    console.log('Response length:', response.length);
    
    try {
        const parsed = JSON.parse(response);
        if (parsed.success && parsed.data) {
            if (parsed.data.screenshot) {
                // Save screenshot to file
                const fs = require('fs');
                const base64Data = parsed.data.screenshot.replace(/^data:image\/png;base64,/, '');
                fs.writeFileSync('tauri-screenshot.png', base64Data, 'base64');
                console.log('✅ Screenshot saved to tauri-screenshot.png');
                
                // Now get DOM
                const domCmd = JSON.stringify({
                    command: 'get_dom',
                    payload: {
                        window_title: 'Communitas'
                    }
                }) + '\n';
                
                console.log('Getting DOM content...');
                client.write(domCmd);
                
            } else if (parsed.data.dom) {
                console.log('DOM Content:', parsed.data.dom.substring(0, 500) + '...');
                
                // Now execute JS to check for errors
                const jsCmd = JSON.stringify({
                    command: 'execute_js',
                    payload: {
                        window_title: 'Communitas',
                        script: `
                            const errors = [];
                            if (window.__TAURI__) {
                                errors.push('Tauri API is available');
                            } else {
                                errors.push('Tauri API is NOT available');
                            }
                            
                            // Check React root
                            const root = document.getElementById('root');
                            if (root) {
                                errors.push('Root element exists: ' + root.innerHTML.substring(0, 100));
                            } else {
                                errors.push('No root element found');
                            }
                            
                            // Get any console errors
                            errors.join(' | ');
                        `
                    }
                }) + '\n';
                
                console.log('Executing JavaScript diagnostics...');
                client.write(jsCmd);
                
            } else if (parsed.data.result !== undefined) {
                console.log('JavaScript result:', parsed.data.result);
                client.end();
                process.exit(0);
            }
        } else {
            console.log('Error response:', parsed);
        }
    } catch (err) {
        console.log('Raw response:', response.substring(0, 200));
        console.error('Parse error:', err);
    }
});

client.on('error', (err) => {
    console.error('❌ Connection error:', err.message);
    process.exit(1);
});

client.on('end', () => {
    console.log('Connection closed');
});