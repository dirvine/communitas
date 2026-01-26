/**
 * MCP Bridge - Communication layer for MCP Apps
 *
 * This library enables UI widgets to communicate with the MCP server
 * via postMessage JSON-RPC protocol. It handles:
 * - Tool invocations
 * - Resource fetching
 * - Sending messages to update model context
 * - Receiving events from the MCP host
 *
 * @see https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/
 */

class McpBridge {
    constructor() {
        this.requestId = 0;
        this.pendingRequests = new Map();
        this.eventHandlers = {
            toolInput: [],
            toolResult: [],
            ready: [],
        };
        this.isReady = false;
        this.messageQueue = [];

        // Set up message listener
        window.addEventListener('message', this._handleMessage.bind(this));

        // Initiate handshake with host
        this._sendHandshake();
    }

    /**
     * Call an MCP tool and return the result
     * @param {string} name - Tool name (e.g., 'list_contacts')
     * @param {object} args - Tool arguments
     * @returns {Promise<object>} Tool result
     */
    async callTool(name, args = {}) {
        return this._sendRequest('tools/call', {
            name,
            arguments: args,
        });
    }

    /**
     * Read an MCP resource
     * @param {string} uri - Resource URI (e.g., 'ui://communitas/contacts')
     * @returns {Promise<object>} Resource content
     */
    async readResource(uri) {
        return this._sendRequest('resources/read', { uri });
    }

    /**
     * Send a message to update the model context
     * This allows the UI to provide information back to the conversation
     * @param {object|string} content - Message content
     */
    sendMessage(content) {
        this._postMessage({
            jsonrpc: '2.0',
            method: 'ui/message',
            params: {
                content: typeof content === 'string' ? content : JSON.stringify(content),
            },
        });
    }

    /**
     * Register handler for tool input events
     * Called when the host sends tool input to the UI
     * @param {function} callback - Handler function
     */
    onToolInput(callback) {
        this.eventHandlers.toolInput.push(callback);
    }

    /**
     * Register handler for tool result events
     * Called when a tool execution completes
     * @param {function} callback - Handler function
     */
    onToolResult(callback) {
        this.eventHandlers.toolResult.push(callback);
    }

    /**
     * Register handler for ready event
     * Called when the bridge is ready for communication
     * @param {function} callback - Handler function
     */
    onReady(callback) {
        if (this.isReady) {
            callback();
        } else {
            this.eventHandlers.ready.push(callback);
        }
    }

    // Private methods

    _sendHandshake() {
        this._postMessage({
            jsonrpc: '2.0',
            method: 'ui/initialize',
            id: this._nextId(),
            params: {
                capabilities: {
                    toolCalls: true,
                    resourceReads: true,
                    messaging: true,
                },
            },
        });
    }

    _handleMessage(event) {
        // In production, validate origin
        // For now, accept all messages from parent
        const message = event.data;

        if (!message || typeof message !== 'object') return;

        // Handle JSON-RPC responses
        if (message.id !== undefined && this.pendingRequests.has(message.id)) {
            const { resolve, reject } = this.pendingRequests.get(message.id);
            this.pendingRequests.delete(message.id);

            if (message.error) {
                reject(new Error(message.error.message || 'Unknown error'));
            } else {
                resolve(message.result);
            }
            return;
        }

        // Handle events
        switch (message.method) {
            case 'ui/initialized':
                this.isReady = true;
                this.eventHandlers.ready.forEach(cb => cb());
                // Process queued messages
                this.messageQueue.forEach(msg => this._postMessage(msg));
                this.messageQueue = [];
                break;

            case 'ui/toolInput':
                this.eventHandlers.toolInput.forEach(cb => cb(message.params));
                break;

            case 'ui/toolResult':
                this.eventHandlers.toolResult.forEach(cb => cb(message.params));
                break;
        }
    }

    _sendRequest(method, params) {
        return new Promise((resolve, reject) => {
            const id = this._nextId();
            this.pendingRequests.set(id, { resolve, reject });

            const message = {
                jsonrpc: '2.0',
                method,
                id,
                params,
            };

            if (this.isReady) {
                this._postMessage(message);
            } else {
                this.messageQueue.push(message);
            }

            // Timeout after 30 seconds
            setTimeout(() => {
                if (this.pendingRequests.has(id)) {
                    this.pendingRequests.delete(id);
                    reject(new Error('Request timeout'));
                }
            }, 30000);
        });
    }

    _postMessage(message) {
        // Post to parent window (MCP host)
        if (window.parent && window.parent !== window) {
            window.parent.postMessage(message, '*');
        }
    }

    _nextId() {
        return ++this.requestId;
    }
}

// Export for use in modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = McpBridge;
}
