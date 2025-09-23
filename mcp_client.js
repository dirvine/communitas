#!/usr/bin/env node

import net from 'net';
import fs from 'fs';

class TauriMCPClient {
  constructor(host = '127.0.0.1', port = 9999) {
    this.host = host;
    this.port = port;
    this.client = null;
    this.connected = false;
    this.pendingRequests = new Map();
  }

  connect() {
    return new Promise((resolve, reject) => {
      this.client = net.createConnection({
        host: this.host,
        port: this.port
      }, () => {
        console.log(`Connected to Tauri MCP server at ${this.host}:${this.port}`);
        this.connected = true;
        resolve();
      });

      this.client.on('error', (err) => {
        console.error('Connection error:', err.message);
        reject(err);
      });

      this.client.on('close', () => {
        console.log('Connection closed');
        this.connected = false;
      });

      this.client.on('data', (data) => {
        this.handleResponse(data);
      });
    });
  }

  handleResponse(data) {
    const responseStr = data.toString();
    console.log('Received response:', responseStr);

    try {
      const response = JSON.parse(responseStr);

      // Find the oldest pending request
      const requestIds = Array.from(this.pendingRequests.keys()).sort();
      if (requestIds.length > 0) {
        const requestId = requestIds[0];
        const { resolve, reject } = this.pendingRequests.get(requestId);
        this.pendingRequests.delete(requestId);

        if (response.success) {
          resolve(response.data);
        } else {
          reject(new Error(response.error || 'Command failed'));
        }
      }
    } catch (err) {
      console.error('Error parsing response:', err.message);
    }
  }

  sendCommand(command, payload = {}) {
    return new Promise((resolve, reject) => {
      if (!this.connected) {
        reject(new Error('Not connected to MCP server'));
        return;
      }

      const requestId = Date.now().toString() + Math.random().toString(36).substring(2);
      this.pendingRequests.set(requestId, { resolve, reject });

      const request = JSON.stringify({
        command,
        payload
      }) + '\n';

      console.log(`Sending command: ${command}`, payload);
      this.client.write(request);

      // Timeout after 30 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(requestId)) {
          this.pendingRequests.delete(requestId);
          reject(new Error('Request timed out'));
        }
      }, 30000);
    });
  }

  async ping() {
    return this.sendCommand('ping');
  }

  async getDom(windowLabel = 'main') {
    return this.sendCommand('get_dom', { window_label: windowLabel });
  }

  async executeJs(code) {
    return this.sendCommand('execute_js', { code });
  }

  async simulateMouseMovement(x, y, click = false) {
    return this.sendCommand('simulate_mouse_movement', { x, y, click });
  }

  async simulateTextInput(text) {
    return this.sendCommand('simulate_text_input', { text });
  }

  async getElementPosition(selector, windowLabel = 'main') {
    return this.sendCommand('get_element_position', {
      selector_type: 'css',
      selector_value: selector,
      window_label: windowLabel
    });
  }

  async sendTextToElement(selector, text, windowLabel = 'main') {
    return this.sendCommand('send_text_to_element', {
      selector_type: 'css',
      selector_value: selector,
      text,
      window_label: windowLabel
    });
  }

  async takeScreenshot(windowLabel = 'main') {
    return this.sendCommand('take_screenshot', { window_label: windowLabel });
  }

  close() {
    if (this.client) {
      this.client.end();
    }
  }
}

// CLI usage
async function main() {
  const client = new TauriMCPClient();

  try {
    await client.connect();

    const args = process.argv.slice(2);
    if (args.length === 0) {
      console.log('Usage: node mcp_client.js <command> [args...]');
      console.log('Commands:');
      console.log('  ping');
      console.log('  get_dom');
      console.log('  execute_js <script>');
      console.log('  simulate_mouse_movement <x> <y> [click]');
      console.log('  simulate_text_input <text>');
      console.log('  get_element_position <selector>');
      console.log('  send_text_to_element <selector> <text>');
      console.log('  take_screenshot');
      return;
    }

    const command = args[0];
    let result;

    switch (command) {
      case 'ping':
        result = await client.ping();
        console.log('Ping result:', result);
        break;

      case 'get_dom':
        const windowLabel = args[1] || 'main';
        result = await client.getDom(windowLabel);
        console.log('DOM length:', result ? result.length : 0);
        // Only show first 500 chars to avoid flooding console
        console.log('DOM preview:', result ? result.substring(0, 500) + '...' : 'No DOM');
        break;

      case 'execute_js':
        if (args.length < 2) throw new Error('execute_js requires a code argument');
        result = await client.executeJs(args[1]);
        console.log('JS result:', result);
        break;

      case 'simulate_mouse_movement':
        if (args.length < 3) throw new Error('simulate_mouse_movement requires x and y coordinates');
        const x = parseInt(args[1]);
        const y = parseInt(args[2]);
        const click = args[3] === 'true' || args[3] === 'click';
        result = await client.simulateMouseMovement(x, y, click);
        console.log('Mouse movement result:', result);
        break;

      case 'simulate_text_input':
        if (args.length < 2) throw new Error('simulate_text_input requires text argument');
        result = await client.simulateTextInput(args[1]);
        console.log('Text input result:', result);
        break;

      case 'get_element_position':
        if (args.length < 2) throw new Error('get_element_position requires selector argument');
        const elementWindowLabel = args[2] || 'main';
        result = await client.getElementPosition(args[1], elementWindowLabel);
        console.log('Element position:', result);
        break;

      case 'send_text_to_element':
        if (args.length < 3) throw new Error('send_text_to_element requires selector and text arguments');
        const textWindowLabel = args[3] || 'main';
        result = await client.sendTextToElement(args[1], args[2], textWindowLabel);
        console.log('Send text result:', result);
        break;

      case 'take_screenshot':
        const screenshotWindowLabel = args[1] || 'main';
        result = await client.takeScreenshot(screenshotWindowLabel);
        console.log('Screenshot taken, size:', result ? result.length : 0, 'bytes');
        // Save screenshot to file
        if (result) {
          const filename = `screenshot_${Date.now()}.png`;
          fs.writeFileSync(filename, Buffer.from(result, 'base64'));
          console.log(`Screenshot saved as: ${filename}`);
        }
        break;

      default:
        throw new Error(`Unknown command: ${command}`);
    }

  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  } finally {
    client.close();
  }
}

// Run CLI if this is the main module
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export default TauriMCPClient;