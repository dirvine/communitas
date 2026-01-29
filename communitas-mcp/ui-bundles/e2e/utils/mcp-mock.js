/**
 * Mock MCP server for E2E testing
 *
 * Provides a lightweight mock of the MCP JSON-RPC interface
 * for testing widgets in isolation without a full MCP server.
 */

class McpMock {
  constructor() {
    this.tools = new Map();
    this.responses = new Map();
  }

  /**
   * Register a tool handler
   * @param {string} name - Tool name
   * @param {Function} handler - Function that returns mock response
   */
  registerTool(name, handler) {
    this.tools.set(name, handler);
  }

  /**
   * Register a canned response for a tool
   * @param {string} name - Tool name
   * @param {*} response - Response data
   */
  registerResponse(name, response) {
    this.responses.set(name, response);
  }

  /**
   * Handle a tool call
   * @param {string} name - Tool name
   * @param {*} params - Tool parameters
   * @returns {*} Response data
   */
  async handleToolCall(name, params) {
    // Check for canned response first
    if (this.responses.has(name)) {
      return this.responses.get(name);
    }

    // Check for handler
    if (this.tools.has(name)) {
      const handler = this.tools.get(name);
      return await handler(params);
    }

    // Default: return success
    return { success: true, data: {} };
  }

  /**
   * Inject mock into page context
   * @param {Page} page - Playwright page object
   */
  async injectIntoPage(page) {
    await page.addInitScript(() => {
      window.mcpBridge = {
        callTool: async (name, params) => {
          // Post message to test context
          window.postMessage({ type: 'mcp-call', name, params }, '*');

          // Wait for response
          return new Promise((resolve) => {
            const handler = (event) => {
              if (event.data.type === 'mcp-response') {
                window.removeEventListener('message', handler);
                resolve(event.data.result);
              }
            };
            window.addEventListener('message', handler);
          });
        }
      };
    });

    // Handle tool calls from page
    await page.exposeFunction('__mcpMockCall', async (name, params) => {
      return await this.handleToolCall(name, params);
    });
  }
}

module.exports = { McpMock };
