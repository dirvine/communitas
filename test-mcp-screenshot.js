#!/usr/bin/env node

import net from 'net';
import fs from 'fs';

const SOCKET_PATH = '/tmp/tauri-mcp-communitas-65268.sock';

console.log('Connecting to MCP socket:', SOCKET_PATH);

const client = net.createConnection({ path: SOCKET_PATH }, () => {
    console.log('✅ Connected to Tauri MCP socket!');
    
    // Take a screenshot
    const screenshotCmd = JSON.stringify({
        command: 'take_screenshot',
        payload: {
            window_label: 'main',  // Tauri default window label
            quality: 90
        }
    }) + '\n';
    
    console.log('Sending screenshot command...');
    client.write(screenshotCmd);
});

let buffer = '';

client.on('data', (data) => {
    buffer += data.toString();
    
    // Try to find complete JSON responses
    let newlineIndex;
    while ((newlineIndex = buffer.indexOf('\n')) !== -1) {
        const jsonStr = buffer.substring(0, newlineIndex);
        buffer = buffer.substring(newlineIndex + 1);
        
        try {
            const response = JSON.parse(jsonStr);
            console.log('Response success:', response.success);
            
            if (response.success && response.data && response.data.screenshot) {
                // Save screenshot
                const base64Data = response.data.screenshot.replace(/^data:image\/png;base64,/, '');
                fs.writeFileSync('tauri-app-screenshot.png', base64Data, 'base64');
                console.log('✅ Screenshot saved to tauri-app-screenshot.png');
                console.log('Image size:', base64Data.length, 'bytes (base64)');
                
                // Now get DOM
                const domCmd = JSON.stringify({
                    command: 'get_dom',
                    payload: {
                        window_label: 'main'
                    }
                }) + '\n';
                
                console.log('\nGetting DOM content...');
                client.write(domCmd);
                
            } else if (response.data && response.data.dom !== undefined) {
                const dom = response.data.dom;
                console.log('DOM length:', dom.length);
                console.log('First 500 chars:', dom.substring(0, 500));
                
                // Save DOM to file for inspection
                fs.writeFileSync('tauri-app-dom.html', dom);
                console.log('✅ DOM saved to tauri-app-dom.html');
                
                // Execute JS to check app state
                const jsCmd = JSON.stringify({
                    command: 'execute_js',
                    payload: {
                        window_label: 'main',
                        script: `
                            try {
                                const result = {
                                    hasTauri: typeof window.__TAURI__ !== 'undefined',
                                    hasRoot: !!document.getElementById('root'),
                                    rootContent: document.getElementById('root')?.innerHTML?.substring(0, 200) || 'no root',
                                    bodyContent: document.body.innerHTML.substring(0, 200),
                                    url: window.location.href,
                                    title: document.title
                                };
                                JSON.stringify(result);
                            } catch (e) {
                                JSON.stringify({ error: e.toString() });
                            }
                        `
                    }
                }) + '\n';
                
                console.log('\nExecuting JavaScript diagnostics...');
                client.write(jsCmd);
                
            } else if (response.data && response.data.result !== undefined) {
                console.log('\nJavaScript diagnostic result:');
                try {
                    const jsResult = JSON.parse(response.data.result);
                    console.log(JSON.stringify(jsResult, null, 2));
                } catch (e) {
                    console.log(response.data.result);
                }
                
                client.end();
                process.exit(0);
            } else {
                console.log('Other response:', JSON.stringify(response, null, 2));
            }
        } catch (err) {
            console.error('Parse error:', err.message);
            console.log('Failed to parse:', jsonStr.substring(0, 100));
        }
    }
});

client.on('error', (err) => {
    console.error('❌ Connection error:', err.message);
    process.exit(1);
});

client.on('end', () => {
    console.log('\nConnection closed');
});