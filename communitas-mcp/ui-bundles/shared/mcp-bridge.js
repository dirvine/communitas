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
            contextChange: [],
            message: [],
        };
        this.isReady = false;
        this.messageQueue = [];

        // AI context state (Phase 9.3)
        // This tracks the current UI state to help AI understand context
        this._uiContext = {
            current_view: null,
            selection_state: null,
            pending_actions: null,
            error_state: null,
        };

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

    /**
     * Register handler for context change events
     * Called when the AI context is updated
     * @param {function} callback - Handler function(context)
     */
    onContextChange(callback) {
        this.eventHandlers.contextChange.push(callback);
    }

    /**
     * Register handler for incoming messages from the MCP host
     * Called when the host sends a message to the UI (e.g., typing indicators)
     * @param {function} callback - Handler function(message)
     */
    onMessage(callback) {
        this.eventHandlers.message.push(callback);
    }

    // ==========================================================================
    // AI Context Methods (Phase 9.3)
    // ==========================================================================
    // These methods track the current UI state to help AI hosts understand
    // what the user is viewing, what's selected, pending changes, and errors.

    /**
     * Get the current UI context
     * @returns {object} The current context object
     */
    getContext() {
        // Return a clean copy without null values
        const ctx = {};
        if (this._uiContext.current_view) ctx.current_view = this._uiContext.current_view;
        if (this._uiContext.selection_state) ctx.selection_state = this._uiContext.selection_state;
        if (this._uiContext.pending_actions) ctx.pending_actions = this._uiContext.pending_actions;
        if (this._uiContext.error_state) ctx.error_state = this._uiContext.error_state;
        return ctx;
    }

    /**
     * Set the current view context
     * Call this when the user navigates to a different view
     *
     * @param {string} widget - Widget name (e.g., 'kanban', 'contacts', 'drive')
     * @param {object} [options] - Additional view options
     * @param {string} [options.view_id] - Specific view/board/folder ID
     * @param {string} [options.view_mode] - View mode (e.g., 'board', 'list', 'grid')
     * @param {string} [options.filter] - Active filter description
     */
    setCurrentView(widget, options = {}) {
        this._uiContext.current_view = {
            widget: widget,
            view_id: options.view_id || null,
            view_mode: options.view_mode || null,
            filter: options.filter || null,
        };
        this._notifyContextChange('current_view');
    }

    /**
     * Clear the current view context
     */
    clearCurrentView() {
        this._uiContext.current_view = null;
        this._notifyContextChange('current_view');
    }

    /**
     * Set the selection state context
     * Call this when the user selects or deselects items
     *
     * @param {string} selectionType - Type of items (e.g., 'card', 'contact', 'file')
     * @param {string[]} selectedIds - Array of selected item IDs
     */
    setSelectionState(selectionType, selectedIds = []) {
        if (selectedIds.length === 0) {
            this._uiContext.selection_state = null;
        } else {
            this._uiContext.selection_state = {
                selected_ids: selectedIds,
                selection_type: selectionType,
                count: selectedIds.length,
            };
        }
        this._notifyContextChange('selection_state');
    }

    /**
     * Clear the selection state
     */
    clearSelectionState() {
        this._uiContext.selection_state = null;
        this._notifyContextChange('selection_state');
    }

    /**
     * Set the pending actions context
     * Call this when there are unsaved changes
     *
     * @param {string} actionType - Type of action ('edit', 'create', 'delete', 'move', 'draft')
     * @param {string[]} [unsavedItems] - Array of item IDs with unsaved changes
     */
    setPendingActions(actionType, unsavedItems = []) {
        this._uiContext.pending_actions = {
            has_unsaved: true,
            unsaved_items: unsavedItems,
            action_type: actionType,
        };
        this._notifyContextChange('pending_actions');
    }

    /**
     * Clear pending actions (e.g., after save)
     */
    clearPendingActions() {
        this._uiContext.pending_actions = null;
        this._notifyContextChange('pending_actions');
    }

    /**
     * Set the error state context
     * Call this when an error occurs that the AI should know about
     *
     * @param {string} errorType - Error type ('network', 'validation', 'permission', 'timeout', 'internal', 'not_found', 'quota_exceeded', 'sync')
     * @param {string} errorMessage - Human-readable error message
     * @param {object} [options] - Additional options
     * @param {boolean} [options.recoverable=true] - Whether the error can be recovered from
     * @param {string} [options.recovery_hint] - Hint for how to recover
     */
    setErrorState(errorType, errorMessage, options = {}) {
        this._uiContext.error_state = {
            has_error: true,
            error_type: errorType,
            error_message: errorMessage,
            recoverable: options.recoverable !== false,
            recovery_hint: options.recovery_hint || null,
        };
        this._notifyContextChange('error_state');
    }

    /**
     * Clear the error state
     */
    clearErrorState() {
        this._uiContext.error_state = null;
        this._notifyContextChange('error_state');
    }

    /**
     * Send context update to the MCP host
     * This notifies the host that the UI context has changed
     * @private
     */
    _notifyContextChange(changedField) {
        // Notify local handlers
        const context = this.getContext();
        this.eventHandlers.contextChange.forEach(cb => cb(context, changedField));

        // Send context update to MCP host
        this._postMessage({
            jsonrpc: '2.0',
            method: 'ui/context',
            params: {
                context: context,
                changed: changedField,
            },
        });
    }

    /**
     * Send a message to update the model context with current UI context
     * This allows the UI to provide information back to the conversation
     * Enhanced in Phase 9.3 to include AI context automatically
     * @param {object|string} content - Message content
     * @param {boolean} [includeContext=true] - Whether to include UI context
     */
    sendMessageWithContext(content, includeContext = true) {
        const params = {
            content: typeof content === 'string' ? content : JSON.stringify(content),
        };

        if (includeContext) {
            const context = this.getContext();
            if (Object.keys(context).length > 0) {
                params.context = context;
            }
        }

        this._postMessage({
            jsonrpc: '2.0',
            method: 'ui/message',
            params: params,
        });
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

            case 'ui/message':
                this.eventHandlers.message.forEach(cb => cb(message.params));
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
