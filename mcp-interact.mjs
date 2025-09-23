#!/usr/bin/env node

/**
 * Interactive MCP Client for Communitas Testing
 * Allows direct communication with the MCP server
 */

import net from 'net';
import readline from 'readline';

const MCP_SOCKET = process.env.MCP_SOCKET || '/tmp/tauri-mcp-communitas-93041.sock';

class InteractiveMCPClient {
  constructor(socketPath) {
    this.socketPath = socketPath;
    this.id = 0;
    this.pending = new Map();
    this.connected = false;
  }

  async connect() {
    return new Promise((resolve, reject) => {
      this.socket = net.createConnection(this.socketPath, () => {
        this.connected = true;
        console.log('✅ Connected to MCP server at', this.socketPath);
        resolve();
      });

      this.socket.on('data', (data) => {
        try {
          const messages = data.toString().split('\n').filter(Boolean);
          messages.forEach(msg => {
            try {
              const response = JSON.parse(msg);
              const promise = this.pending.get(response.id);
              if (promise) {
                if (response.error) {
                  promise.reject(new Error(response.error.message));
                } else {
                  promise.resolve(response.result);
                }
                this.pending.delete(response.id);
              } else {
                // Notification or unexpected response
                console.log('\n📨 Server notification:', JSON.stringify(response, null, 2));
              }
            } catch (e) {
              console.error('Failed to parse response:', e.message);
            }
          });
        } catch (error) {
          console.error('Data processing error:', error);
        }
      });

      this.socket.on('error', (err) => {
        console.error('❌ Socket error:', err.message);
        reject(err);
      });

      this.socket.on('close', () => {
        this.connected = false;
        console.log('Disconnected from MCP server');
      });
    });
  }

  async call(method, params = {}) {
    if (!this.connected) {
      throw new Error('Not connected to MCP server');
    }

    const id = ++this.id;
    const request = {
      jsonrpc: '2.0',
      method,
      params,
      id
    };

    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.socket.write(JSON.stringify(request) + '\n');

      // Timeout after 30 seconds
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error(`MCP timeout for ${method}`));
        }
      }, 30000);
    });
  }

  close() {
    if (this.socket) {
      this.socket.end();
    }
  }
}

// Helper commands for quick testing
const commands = {
  ping: async (mcp) => {
    const result = await mcp.call('ping');
    console.log('🏓 Pong:', result);
  },

  screenshot: async (mcp) => {
    console.log('📸 Taking screenshot...');
    const result = await mcp.call('take_screenshot', { format: 'png' });
    console.log('✅ Screenshot taken, length:', result?.length || 0);
  },

  dom: async (mcp, selector = 'body') => {
    const result = await mcp.call('get_dom', { selector });
    console.log('🌐 DOM:', result);
  },

  js: async (mcp, script) => {
    const result = await mcp.call('execute_js', {
      script,
      await_promise: script.includes('async') || script.includes('await')
    });
    console.log('📜 Result:', result);
  },

  // Quick test of Tauri commands
  claim: async (mcp) => {
    const script = `
      (async () => {
        const words = ['ocean', 'forest', 'moon', 'star'];
        try {
          const idHex = await window.__TAURI__.invoke('core_claim', { words });
          return { success: true, idHex, words };
        } catch (error) {
          return { success: false, error: error.toString() };
        }
      })()
    `;
    const result = await mcp.call('execute_js', { script, await_promise: true });
    console.log('🔑 Claim result:', result);
  },

  init: async (mcp) => {
    const script = `
      (async () => {
        try {
          await window.__TAURI__.invoke('core_initialize', {
            fourWords: 'ocean-forest-moon-star',
            displayName: 'Test User',
            deviceName: 'Test Device',
            deviceType: 'Desktop'
          });
          return { success: true };
        } catch (error) {
          return { success: false, error: error.toString() };
        }
      })()
    `;
    const result = await mcp.call('execute_js', { script, await_promise: true });
    console.log('🚀 Initialize result:', result);
  },

  channel: async (mcp) => {
    const script = `
      (async () => {
        try {
          const channel = await window.__TAURI__.invoke('core_create_channel', {
            name: 'test-channel',
            description: 'Test channel via MCP'
          });
          return { success: true, channel };
        } catch (error) {
          return { success: false, error: error.toString() };
        }
      })()
    `;
    const result = await mcp.call('execute_js', { script, await_promise: true });
    console.log('📢 Channel result:', result);
  },

  status: async (mcp) => {
    const script = `
      JSON.stringify({
        tauriAvailable: !!window.__TAURI__,
        location: window.location.href,
        title: document.title,
        hasUser: !!window.__COMMUNITAS_USER__,
        networkStatus: window.testNetwork?.status?.() || 'unknown'
      })
    `;
    const result = await mcp.call('execute_js', { script });
    console.log('📊 App status:', JSON.parse(result));
  },

  help: () => {
    console.log(`
Available commands:
  ping         - Test MCP connection
  screenshot   - Take a screenshot
  dom [sel]    - Get DOM content (optional selector)
  js <script>  - Execute JavaScript
  claim        - Claim Four-Word identity
  init         - Initialize CoreContext
  channel      - Create test channel
  status       - Get app status
  help         - Show this help
  exit         - Exit the client

Raw MCP calls:
  Just type the JSON-RPC request, e.g.:
  {"method": "ping"}
  {"method": "execute_js", "params": {"script": "window.location.href"}}
`);
  }
};

async function main() {
  console.log('🔌 Communitas MCP Interactive Client');
  console.log('=' .repeat(50));
  console.log('Socket:', MCP_SOCKET);
  console.log('Type "help" for commands or "exit" to quit');
  console.log('=' .repeat(50));

  const mcp = new InteractiveMCPClient(MCP_SOCKET);

  try {
    await mcp.connect();
  } catch (error) {
    console.error('❌ Failed to connect:', error.message);
    process.exit(1);
  }

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    prompt: 'mcp> '
  });

  rl.prompt();

  rl.on('line', async (line) => {
    const input = line.trim();

    if (input === 'exit' || input === 'quit') {
      mcp.close();
      rl.close();
      process.exit(0);
    }

    if (input === '') {
      rl.prompt();
      return;
    }

    try {
      // Check if it's a built-in command
      const [cmd, ...args] = input.split(' ');

      if (commands[cmd]) {
        await commands[cmd](mcp, args.join(' '));
      } else if (input.startsWith('{')) {
        // Raw JSON-RPC request
        const req = JSON.parse(input);
        const result = await mcp.call(req.method, req.params || {});
        console.log('📦 Response:', JSON.stringify(result, null, 2));
      } else if (input.startsWith('js ')) {
        // Shortcut for JavaScript execution
        await commands.js(mcp, input.substring(3));
      } else {
        console.log('❓ Unknown command. Type "help" for available commands.');
      }
    } catch (error) {
      console.error('❌ Error:', error.message);
    }

    rl.prompt();
  });

  rl.on('close', () => {
    console.log('\n👋 Goodbye!');
    mcp.close();
    process.exit(0);
  });
}

// Handle errors gracefully
process.on('unhandledRejection', (error) => {
  console.error('❌ Unhandled error:', error);
  process.exit(1);
});

main().catch(console.error);