/**
 * Mock MCP Bridge for Widget Testing
 *
 * Simulates the MCP host postMessage API for local widget development.
 * Use this when testing widgets outside of a real MCP host.
 */

class MockMcpBridge {
    constructor() {
        this.requestId = 0;
        this.eventHandlers = {
            toolInput: [],
            toolResult: [],
            message: []
        };
        this.mockData = this._getDefaultMockData();
        this.callLog = [];
    }

    /**
     * Initialize the mock bridge
     */
    async initialize() {
        console.log('[MockBridge] Initialized');
        return {
            capabilities: {
                tools: { supported: true },
                resources: { supported: true },
                ui: { supported: true }
            },
            serverInfo: {
                name: 'communitas-mcp-mock',
                version: '0.8.2'
            }
        };
    }

    /**
     * Simulate a tool call
     */
    async callTool(name, args = {}) {
        const id = ++this.requestId;
        this.callLog.push({ id, method: 'tools/call', name, args, timestamp: Date.now() });

        console.log(`[MockBridge] callTool: ${name}`, args);

        // Simulate network delay
        await this._delay(100);

        // Return mock data based on tool name
        const result = this._getMockToolResult(name, args);

        // Notify toolResult handlers
        this.eventHandlers.toolResult.forEach(cb => cb({ name, args, result }));

        return result;
    }

    /**
     * Simulate reading a resource
     */
    async readResource(uri) {
        const id = ++this.requestId;
        this.callLog.push({ id, method: 'resources/read', uri, timestamp: Date.now() });

        console.log(`[MockBridge] readResource: ${uri}`);

        await this._delay(50);

        return this._getMockResource(uri);
    }

    /**
     * Send a message to update model context
     */
    sendMessage(content) {
        const id = ++this.requestId;
        this.callLog.push({ id, method: 'ui/message', content, timestamp: Date.now() });

        console.log(`[MockBridge] sendMessage:`, content);

        // Notify message handlers
        this.eventHandlers.message.forEach(cb => cb(content));
    }

    /**
     * Register tool input handler
     */
    onToolInput(callback) {
        this.eventHandlers.toolInput.push(callback);
        return () => {
            const idx = this.eventHandlers.toolInput.indexOf(callback);
            if (idx >= 0) this.eventHandlers.toolInput.splice(idx, 1);
        };
    }

    /**
     * Register tool result handler
     */
    onToolResult(callback) {
        this.eventHandlers.toolResult.push(callback);
        return () => {
            const idx = this.eventHandlers.toolResult.indexOf(callback);
            if (idx >= 0) this.eventHandlers.toolResult.splice(idx, 1);
        };
    }

    /**
     * Register message handler
     */
    onMessage(callback) {
        this.eventHandlers.message.push(callback);
        return () => {
            const idx = this.eventHandlers.message.indexOf(callback);
            if (idx >= 0) this.eventHandlers.message.splice(idx, 1);
        };
    }

    /**
     * Simulate tool input from AI host
     */
    simulateToolInput(name, args) {
        console.log(`[MockBridge] Simulating tool input: ${name}`);
        this.eventHandlers.toolInput.forEach(cb => cb({ name, args }));
    }

    /**
     * Get call history
     */
    getCallLog() {
        return [...this.callLog];
    }

    /**
     * Clear call history
     */
    clearCallLog() {
        this.callLog = [];
    }

    /**
     * Update mock data
     */
    setMockData(key, data) {
        this.mockData[key] = data;
    }

    // Private methods

    _delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    _getDefaultMockData() {
        return {
            contacts: [
                { id: 'c1', name: 'Alice Smith', email: 'alice@example.com', favorite: true, presence: 'online' },
                { id: 'c2', name: 'Bob Johnson', email: 'bob@example.com', favorite: false, presence: 'away' },
                { id: 'c3', name: 'Carol Davis', email: 'carol@example.com', favorite: true, presence: 'offline' }
            ],
            threads: [
                { id: 't1', name: 'Team Chat', unread: 3, lastMessage: 'Meeting at 3pm', timestamp: Date.now() - 300000 },
                { id: 't2', name: 'Project Discussion', unread: 0, lastMessage: 'Sounds good!', timestamp: Date.now() - 3600000 }
            ],
            messages: [
                { id: 'm1', threadId: 't1', sender: 'Alice', content: 'Hello team!', timestamp: Date.now() - 600000 },
                { id: 'm2', threadId: 't1', sender: 'Bob', content: 'Hi Alice!', timestamp: Date.now() - 500000 },
                { id: 'm3', threadId: 't1', sender: 'Alice', content: 'Meeting at 3pm', timestamp: Date.now() - 300000 }
            ],
            boards: [
                { id: 'b1', name: 'Product Roadmap', columns: ['Todo', 'In Progress', 'Done'], cardCount: 12 },
                { id: 'b2', name: 'Sprint Planning', columns: ['Backlog', 'Sprint', 'Review'], cardCount: 8 }
            ],
            cards: [
                { id: 'card1', boardId: 'b1', column: 'Todo', title: 'Design new feature', dueDate: '2026-02-01', tags: ['design'] },
                { id: 'card2', boardId: 'b1', column: 'In Progress', title: 'Implement API', dueDate: null, tags: ['backend'] },
                { id: 'card3', boardId: 'b1', column: 'Done', title: 'Fix login bug', dueDate: null, tags: ['bug'] }
            ],
            files: [
                { id: 'f1', name: 'README.md', type: 'file', size: 1024, modified: Date.now() - 86400000 },
                { id: 'f2', name: 'src', type: 'directory', children: 3, modified: Date.now() - 3600000 },
                { id: 'f3', name: 'logo.png', type: 'file', size: 51200, modified: Date.now() - 7200000 }
            ],
            canvasElements: [
                { id: 'e1', type: 'rect', x: 100, y: 100, width: 200, height: 100, fill: '#3498db' },
                { id: 'e2', type: 'text', x: 150, y: 140, content: 'Hello', fontSize: 24 },
                { id: 'e3', type: 'circle', x: 400, y: 200, radius: 50, fill: '#e74c3c' }
            ]
        };
    }

    _getMockToolResult(name, args) {
        switch (name) {
            case 'list_contacts':
                return { contacts: this.mockData.contacts };
            case 'get_contact':
                return { contact: this.mockData.contacts.find(c => c.id === args.id) };
            case 'create_contact':
                const newContact = { id: `c${Date.now()}`, ...args, presence: 'offline' };
                this.mockData.contacts.push(newContact);
                return { contact: newContact };

            case 'list_threads':
                return { threads: this.mockData.threads };
            case 'list_messages':
                return { messages: this.mockData.messages.filter(m => m.threadId === args.threadId) };
            case 'send_message':
                const newMsg = { id: `m${Date.now()}`, ...args, timestamp: Date.now() };
                this.mockData.messages.push(newMsg);
                return { message: newMsg };

            case 'list_kanban_boards':
                return { boards: this.mockData.boards };
            case 'get_kanban_board':
                const board = this.mockData.boards.find(b => b.id === args.boardId);
                const cards = this.mockData.cards.filter(c => c.boardId === args.boardId);
                return { board, cards };
            case 'create_kanban_card':
                const newCard = { id: `card${Date.now()}`, ...args };
                this.mockData.cards.push(newCard);
                return { card: newCard };
            case 'move_kanban_card':
                const card = this.mockData.cards.find(c => c.id === args.cardId);
                if (card) card.column = args.toColumn;
                return { card };

            case 'list_files':
                return { files: this.mockData.files };
            case 'get_file_preview':
                return { preview: { url: 'data:image/png;base64,...', mimeType: 'image/png' } };

            case 'canvas_get_snapshot':
                return { elements: this.mockData.canvasElements, layers: [{ id: 'layer1', name: 'Default', visible: true }] };
            case 'canvas_get_history':
                return { history: [{ id: 'h1', action: 'create', timestamp: Date.now() - 1000 }] };

            default:
                return { success: true, message: `Mock response for ${name}` };
        }
    }

    _getMockResource(uri) {
        if (uri.startsWith('ui://')) {
            return {
                contents: [{
                    uri: uri,
                    mimeType: 'text/html',
                    text: `<html><body><h1>Mock UI Resource: ${uri}</h1></body></html>`
                }]
            };
        }
        return { contents: [] };
    }
}

// Export for module systems
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { MockMcpBridge };
}

// Make available globally for browser
if (typeof window !== 'undefined') {
    window.MockMcpBridge = MockMcpBridge;
}
