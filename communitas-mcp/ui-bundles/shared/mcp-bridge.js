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

        // Configure allowed origins for postMessage validation
        // In production, this should be configured based on the actual MCP host origin
        this._allowedOrigins = this._initAllowedOrigins();

        // Set up message listener
        window.addEventListener('message', this._handleMessage.bind(this));

        // Initiate handshake with host
        this._sendHandshake();
    }

    /**
     * HTML escape utility to prevent XSS
     * @param {string} str - The string to escape
     * @returns {string} - The escaped string safe for innerHTML
     */
    escapeHTML(str) {
        const div = document.createElement('div');
        div.textContent = str || '';
        return div.innerHTML;
    }

    /**
     * Initialize allowed origins for postMessage validation
     * @private
     */
    _initAllowedOrigins() {
        const origins = new Set();

        // Always allow same origin
        origins.add(window.location.origin);

        // Allow localhost for development (with various ports)
        if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
            origins.add('http://localhost:8443');
            origins.add('https://localhost:8443');
            origins.add('http://127.0.0.1:8443');
            origins.add('https://127.0.0.1:8443');
        }

        // Allow configured Saorsa Labs domains
        const saorsaDomains = [
            'https://saorsa-1.saorsalabs.com',
            'https://saorsa-2.saorsalabs.com',
            'https://saorsa-3.saorsalabs.com',
            'https://saorsa-4.saorsalabs.com',
            'https://saorsa-5.saorsalabs.com',
            'https://saorsa-6.saorsalabs.com',
            'https://saorsa-7.saorsalabs.com',
            'https://saorsa-8.saorsalabs.com',
            'https://saorsa-9.saorsalabs.com',
            'https://saorsa-10.saorsalabs.com',
        ];
        saorsaDomains.forEach(domain => origins.add(domain));

        // Allow parent window origin if it exists (for iframe embedding)
        // This is safe because we're in an iframe and the parent is the MCP host
        try {
            if (window.parent && window.parent !== window) {
                // We'll validate during message handling instead of here
                // to avoid cross-origin access issues
            }
        } catch (e) {
            // Cross-origin restriction - will validate per-message
        }

        return origins;
    }

    /**
     * Validate postMessage origin
     * @private
     * @param {string} origin - The origin to validate
     * @returns {boolean} True if origin is allowed
     */
    _isOriginAllowed(origin) {
        if (!origin) return false;

        // Check if origin is in allowed set
        if (this._allowedOrigins.has(origin)) {
            return true;
        }

        // For development, allow localhost on any port
        if (origin.match(/^https?:\/\/localhost:\d+$/) || origin.match(/^https?:\/\/127\.0\.0\.1:\d+$/)) {
            return true;
        }

        // Log rejected origin for security monitoring
        console.error('MCP Bridge: Rejected message from untrusted origin:', origin);

        return false;
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
        // SECURITY: Validate origin before processing message
        if (!this._isOriginAllowed(event.origin)) {
            console.error('MCP Bridge: Rejected message from untrusted origin:', event.origin);
            return;
        }

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
            // SECURITY: Use specific origin instead of wildcard
            // For iframe communication, we use the parent's origin
            // In production, this should be configured to the exact MCP host origin
            const targetOrigin = this._getTargetOrigin();
            window.parent.postMessage(message, targetOrigin);
        }
    }

    /**
     * Get the target origin for postMessage
     * @private
     * @returns {string} The target origin
     */
    _getTargetOrigin() {
        // Try to get parent window origin
        try {
            // In same-origin case, we can access parent.location.origin
            if (window.parent && window.parent !== window && window.parent.location) {
                return window.parent.location.origin;
            }
        } catch (e) {
            // Cross-origin restriction - use configured origins
        }

        // For cross-origin iframe, use the first allowed origin that's not our own
        for (const origin of this._allowedOrigins) {
            if (origin !== window.location.origin) {
                return origin;
            }
        }

        // Fallback to same origin (should not happen in production)
        return window.location.origin;
    }

    _nextId() {
        return ++this.requestId;
    }
}

// Export for use in modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = McpBridge;
}
