#!/usr/bin/env node

import net from 'net';

const SOCKET_PATH = '/tmp/tauri-mcp-communitas-65268.sock';

console.log('Testing JavaScript execution in Tauri...\n');

const client = net.createConnection({ path: SOCKET_PATH }, () => {
    console.log('Connected to MCP socket\n');
    
    // Execute JavaScript to debug the app state
    const jsCmd = JSON.stringify({
        command: 'execute_js',
        payload: {
            window_label: 'main',
            code: `
                // Gather diagnostic info
                const info = {
                    url: window.location.href,
                    title: document.title,
                    hasTauri: typeof window.__TAURI__ !== 'undefined',
                    hasRoot: !!document.getElementById('root'),
                    bodyHTML: document.body.innerHTML,
                    
                    // Check for console errors
                    errors: [],
                    
                    // Check React
                    reactRoot: document.getElementById('root')?.innerHTML || 'NO ROOT',
                    
                    // Check scripts
                    scripts: Array.from(document.scripts).map(s => ({
                        src: s.src || 'inline',
                        loaded: !s.src || s.complete
                    }))
                };
                
                // Try to access window.console.error calls
                if (window.console && window.console.error) {
                    const originalError = window.console.error;
                    window.console.error = function(...args) {
                        info.errors.push(args.join(' '));
                        originalError.apply(console, args);
                    };
                }
                
                JSON.stringify(info, null, 2);
            `
        }
    }) + '\n';
    
    client.write(jsCmd);
});

let buffer = '';

client.on('data', (data) => {
    buffer += data.toString();
    
    // Process complete JSON responses
    let newlineIndex;
    while ((newlineIndex = buffer.indexOf('\n')) !== -1) {
        const jsonStr = buffer.substring(0, newlineIndex);
        buffer = buffer.substring(newlineIndex + 1);
        
        try {
            const response = JSON.parse(jsonStr);
            
            if (response.success && response.data && response.data.result) {
                console.log('JavaScript execution result:\n');
                try {
                    const result = JSON.parse(response.data.result);
                    console.log('URL:', result.url);
                    console.log('Title:', result.title);
                    console.log('Has Tauri API:', result.hasTauri);
                    console.log('Has root element:', result.hasRoot);
                    console.log('\nBody HTML:');
                    console.log(result.bodyHTML);
                    console.log('\nReact Root content:');
                    console.log(result.reactRoot);
                    console.log('\nScripts loaded:', result.scripts);
                    if (result.errors && result.errors.length > 0) {
                        console.log('\nConsole errors:', result.errors);
                    }
                } catch (e) {
                    console.log('Raw result:', response.data.result);
                }
            } else {
                console.log('Response:', JSON.stringify(response, null, 2));
            }
            
            client.end();
        } catch (err) {
            console.error('Parse error:', err.message);
            console.log('Raw data:', jsonStr.substring(0, 200));
        }
    }
});

client.on('error', (err) => {
    console.error('Connection error:', err.message);
    process.exit(1);
});

client.on('end', () => {
    console.log('\nDone');
    process.exit(0);
});