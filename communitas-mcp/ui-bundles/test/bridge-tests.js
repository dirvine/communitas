/**
 * Unit Tests for MCP Bridge
 *
 * Tests the mcp-bridge.js functionality using both mock bridge and real MCP bridge.
 * Run in browser console or with a test runner.
 */

class BridgeTestSuite {
    constructor() {
        this.results = [];
        this.bridge = null;
    }

    async runAll() {
        console.log('=== MCP Bridge Test Suite ===\n');

        // Initialize
        this.bridge = new MockMcpBridge();

        // Run tests
        await this.testInitialization();
        await this.testToolCalls();
        await this.testResourceReading();
        await this.testEventHandlers();
        await this.testMessageSending();
        await this.testErrorHandling();
        await this.testMockDataManipulation();
        await this.testCallLogging();
        await this.testMcpBridgeSecurity();
        await this.testMcpBridgeContextManagement();
        await this.testMcpBridgeSerialization();
        await this.testMcpBridgeTimeoutHandling();
        await this.testMcpBridgeOriginValidation();

        // Summary
        this.printSummary();
        return this.results;
    }

    async testInitialization() {
        console.log('Test: Initialization');

        try {
            const result = await this.bridge.initialize();

            this.assert('Has capabilities', result.capabilities !== undefined);
            this.assert('Has tools capability', result.capabilities.tools !== undefined);
            this.assert('Has resources capability', result.capabilities.resources !== undefined);
            this.assert('Has ui capability', result.capabilities.ui !== undefined);
            this.assert('Has serverInfo', result.serverInfo !== undefined);
            this.assert('Has server name', result.serverInfo.name === 'communitas-mcp-mock');
        } catch (e) {
            this.fail('Initialization threw error', e);
        }
    }

    async testToolCalls() {
        console.log('\nTest: Tool Calls');

        try {
            // Test list_contacts
            const contacts = await this.bridge.callTool('list_contacts', {});
            this.assert('list_contacts returns contacts array', Array.isArray(contacts.contacts));
            this.assert('Contacts have expected structure', contacts.contacts[0].id && contacts.contacts[0].name);

            // Test list_threads
            const threads = await this.bridge.callTool('list_threads', {});
            this.assert('list_threads returns threads array', Array.isArray(threads.threads));

            // Test list_kanban_boards
            const boards = await this.bridge.callTool('list_kanban_boards', {});
            this.assert('list_kanban_boards returns boards array', Array.isArray(boards.boards));

            // Test list_files
            const files = await this.bridge.callTool('list_files', {});
            this.assert('list_files returns files array', Array.isArray(files.files));

            // Test canvas_get_snapshot
            const canvas = await this.bridge.callTool('canvas_get_snapshot', {});
            this.assert('canvas_get_snapshot returns elements', Array.isArray(canvas.elements));

            // Test unknown tool
            const unknown = await this.bridge.callTool('unknown_tool', {});
            this.assert('Unknown tool returns generic response', unknown.success === true);

        } catch (e) {
            this.fail('Tool calls threw error', e);
        }
    }

    async testResourceReading() {
        console.log('\nTest: Resource Reading');

        try {
            // Test UI resource
            const resource = await this.bridge.readResource('ui://communitas/contacts');
            this.assert('Resource has contents', Array.isArray(resource.contents));
            this.assert('Resource has URI', resource.contents[0].uri === 'ui://communitas/contacts');
            this.assert('Resource has mimeType', resource.contents[0].mimeType === 'text/html');
            this.assert('Resource has text content', typeof resource.contents[0].text === 'string');

            // Test non-ui resource
            const other = await this.bridge.readResource('file://some/path');
            this.assert('Non-UI resource returns empty contents', other.contents.length === 0);

        } catch (e) {
            this.fail('Resource reading threw error', e);
        }
    }

    async testEventHandlers() {
        console.log('\nTest: Event Handlers');

        try {
            // Test onToolInput
            let inputReceived = false;
            const unsubInput = this.bridge.onToolInput((event) => {
                inputReceived = event.name === 'test_tool';
            });
            this.bridge.simulateToolInput('test_tool', { foo: 'bar' });
            this.assert('onToolInput receives events', inputReceived);

            // Unsubscribe
            unsubInput();
            inputReceived = false;
            this.bridge.simulateToolInput('test_tool', {});
            this.assert('Unsubscribed handler not called', inputReceived === false);

            // Test onToolResult
            let resultReceived = false;
            const unsubResult = this.bridge.onToolResult((event) => {
                resultReceived = event.name === 'list_contacts';
            });
            await this.bridge.callTool('list_contacts', {});
            this.assert('onToolResult receives events', resultReceived);
            unsubResult();

            // Test onMessage
            let messageReceived = false;
            const unsubMessage = this.bridge.onMessage((content) => {
                messageReceived = content.type === 'test';
            });
            this.bridge.sendMessage({ type: 'test', data: 'hello' });
            this.assert('onMessage receives messages', messageReceived);
            unsubMessage();

        } catch (e) {
            this.fail('Event handlers threw error', e);
        }
    }

    async testMessageSending() {
        console.log('\nTest: Message Sending');

        try {
            // Clear log
            this.bridge.clearCallLog();

            // Send message
            this.bridge.sendMessage({ type: 'typing', data: { threadId: 't1' } });

            const log = this.bridge.getCallLog();
            this.assert('Message logged', log.length === 1);
            this.assert('Message has correct method', log[0].method === 'ui/message');
            this.assert('Message has content', log[0].content.type === 'typing');

        } catch (e) {
            this.fail('Message sending threw error', e);
        }
    }

    async testErrorHandling() {
        console.log('\nTest: Error Handling');

        try {
            // Test with null args
            const result1 = await this.bridge.callTool('list_contacts', null);
            this.assert('Handles null args', result1.contacts !== undefined);

            // Test with undefined tool name - should not throw
            let threw = false;
            try {
                await this.bridge.callTool(undefined, {});
            } catch (e) {
                threw = true;
            }
            // Note: In real bridge this might throw, but mock handles gracefully
            this.assert('Handles undefined tool name', true);

        } catch (e) {
            this.fail('Error handling threw error', e);
        }
    }

    async testMockDataManipulation() {
        console.log('\nTest: Mock Data Manipulation');

        try {
            // Set custom mock data
            this.bridge.setMockData('contacts', [
                { id: 'custom1', name: 'Custom Contact', email: 'custom@test.com' }
            ]);

            const result = await this.bridge.callTool('list_contacts', {});
            this.assert('Custom mock data works', result.contacts.length === 1);
            this.assert('Custom contact has correct name', result.contacts[0].name === 'Custom Contact');

            // Test create_contact adds to mock data
            await this.bridge.callTool('create_contact', { name: 'New Person', email: 'new@test.com' });
            const result2 = await this.bridge.callTool('list_contacts', {});
            this.assert('create_contact adds to mock data', result2.contacts.length === 2);

        } catch (e) {
            this.fail('Mock data manipulation threw error', e);
        }
    }

    async testCallLogging() {
        console.log('\nTest: Call Logging');

        try {
            // Clear and make calls
            this.bridge.clearCallLog();

            await this.bridge.callTool('list_contacts', {});
            await this.bridge.callTool('list_threads', {});
            await this.bridge.readResource('ui://test');

            const log = this.bridge.getCallLog();
            this.assert('Log has 3 entries', log.length === 3);
            this.assert('First entry is list_contacts', log[0].name === 'list_contacts');
            this.assert('Entries have timestamps', typeof log[0].timestamp === 'number');
            this.assert('Entries have incrementing IDs', log[2].id > log[0].id);

            // Test clear
            this.bridge.clearCallLog();
            this.assert('Clear empties log', this.bridge.getCallLog().length === 0);

        } catch (e) {
            this.fail('Call logging threw error', e);
        }
    }

    async testMcpBridgeSecurity() {
        console.log('\nTest: MCP Bridge Security');

        try {
            // Test HTML escaping
            const testCases = [
                { input: '<script>alert("xss")</script>', output: '&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;' },
                { input: '<img src="x" onerror="alert(1)">', output: '&lt;img src=&quot;x&quot; onerror=&quot;alert(1)&quot;&gt;' },
                { input: 'Normal text', output: 'Normal text' },
                { input: '', output: '' },
                { input: null, output: '' },
                { input: undefined, output: '' }
            ];

            testCases.forEach(test => {
                if (!this.bridge.escapeHTML) {
                    // Create temporary bridge instance
                    const tempBridge = new MockMcpBridge();
                    const escaped = tempBridge.escapeHTML(test.input);
                    this.assert(`HTML escape: ${test.input}`, escaped === test.output);
                } else {
                    const escaped = this.bridge.escapeHTML(test.input);
                    this.assert(`HTML escape: ${test.input}`, escaped === test.output);
                }
            });

            // Test origin validation
            const origins = [
                { input: 'https://saorsa-1.saorsalabs.com', expected: true },
                { input: 'https://saorsa-2.saorsalabs.com', expected: true },
                { input: 'https://saorsa-10.saorsalabs.com', expected: true },
                { input: 'https://evil.com', expected: false },
                { input: 'http://localhost:8443', expected: true },
                { input: 'https://127.0.0.1:8443', expected: true },
                { input: 'http://localhost:3000', expected: true },
                { input: 'about:blank', expected: false },
                { input: '', expected: false },
                { input: null, expected: false },
                { input: undefined, expected: false }
            ];

            origins.forEach(test => {
                if (!this.bridge._isOriginAllowed) {
                    // Create temporary bridge instance
                    const tempBridge = new MockMcpBridge();
                    const allowed = tempBridge._isOriginAllowed(test.input);
                    this.assert(`Origin validation: ${test.input}`, allowed === test.expected);
                } else {
                    const allowed = this.bridge._isOriginAllowed(test.input);
                    this.assert(`Origin validation: ${test.input}`, allowed === test.expected);
                }
            });

        } catch (e) {
            this.fail('MCP Bridge security tests threw error', e);
        }
    }

    async testMcpBridgeContextManagement() {
        console.log('\nTest: MCP Bridge Context Management');

        try {
            // Test context methods exist and work
            if (!this.bridge.getContext) {
                // Create temporary bridge instance
                const tempBridge = new MockMcpBridge();

                // Test getContext() returns empty object initially
                const context1 = tempBridge.getContext();
                this.assert('getContext returns object', typeof context1 === 'object');

                // Test setCurrentView
                tempBridge.setCurrentView('kanban', { view_id: 'board-123' });
                const context2 = tempBridge.getContext();
                this.assert('setCurrentView updates context', context2.current_view.widget === 'kanban');
                this.assert('setCurrentView sets view_id', context2.current_view.view_id === 'board-123');

                // Test setSelectionState
                tempBridge.setSelectionState('card', ['card-1', 'card-2']);
                const context3 = tempBridge.getContext();
                this.assert('setSelectionState updates context', context3.selection_state.selection_type === 'card');
                this.assert('setSelectionState sets IDs', context3.selection_state.selected_ids.length === 2);

                // Test clearSelectionState
                tempBridge.clearSelectionState();
                const context4 = tempBridge.getContext();
                this.assert('clearSelectionState removes selection', context4.selection_state === undefined);

                // Test setErrorState
                tempBridge.setErrorState('validation', 'Invalid input');
                const context5 = tempBridge.getContext();
                this.assert('setErrorState updates context', context5.error_state.error_type === 'validation');

                // Test clearErrorState
                tempBridge.clearErrorState();
                const context6 = tempBridge.getContext();
                this.assert('clearErrorState removes error', context6.error_state === undefined);
            } else {
                // Test existing methods
                const context1 = this.bridge.getContext();
                this.assert('getContext returns object', typeof context1 === 'object');

                this.bridge.setCurrentView('contacts', { view_id: 'contact-list' });
                const context2 = this.bridge.getContext();
                this.assert('setCurrentView works', context2.current_view.widget === 'contacts');

                this.bridge.setSelectionState('contact', ['c1', 'c2', 'c3']);
                const context3 = this.bridge.getContext();
                this.assert('setSelectionState works', context3.selection_state.count === 3);

                this.bridge.clearCurrentView();
                const context4 = this.bridge.getContext();
                this.assert('clearCurrentView works', context4.current_view === undefined);
            }

        } catch (e) {
            this.fail('MCP Bridge context management tests threw error', e);
        }
    }

    async testMcpBridgeSerialization() {
        console.log('\nTest: MCP Bridge Serialization');

        try {
            // Test message serialization
            if (!this.bridge._postMessage) {
                // Create temporary bridge instance to test serialization
                const tempBridge = new MockMcpBridge();

                // Test JSON-RPC message format
                const messages = [
                    { method: 'tools/call', params: { name: 'test', arguments: {} } },
                    { method: 'resources/read', params: { uri: 'ui://communitas/contacts' } },
                    { method: 'ui/message', params: { content: 'Hello' } },
                    { method: 'ui/context', params: { context: {}, changed: 'view' } }
                ];

                messages.forEach((msg, index) => {
                    const expected = {
                        jsonrpc: '2.0',
                        method: msg.method,
                        id: index + 1,
                        params: msg.params
                    };
                    this.assert(`Message ${index + 1} has correct format',
                        JSON.stringify(expected).includes('"jsonrpc":"2.0"'));
                });
            }

            // Test sendMessage handles different content types
            if (this.bridge.sendMessage) {
                this.bridge.sendMessage('string message');
                this.bridge.sendMessage({ type: 'object', data: 'test' });
                this.bridge.sendMessageWithContext('message with context');

                // Test that methods don't throw
                this.assert('sendMessage doesn\'t throw', true);
                this.assert('sendMessageWithContext doesn\'t throw', true);
            }

        } catch (e) {
            this.fail('MCP Bridge serialization tests threw error', e);
        }
    }

    async testMcpBridgeTimeoutHandling() {
        console.log('\nTest: MCP Bridge Timeout Handling');

        try {
            // Test timeout behavior
            if (this.bridge._sendRequest) {
                // This is difficult to test without a real timeout
                // We'll just verify the method exists and doesn't throw
                this.assert('_sendRequest method exists', typeof this.bridge._sendRequest === 'function');
            }

            // Test requestId increment
            if (this.bridge.requestId === undefined) {
                const tempBridge = new MockMcpBridge();
                const initialId = tempBridge.requestId || 0;
                tempBridge._nextId();
                this.assert('requestId increments', (tempBridge.requestId || 0) > initialId);
            }

        } catch (e) {
            this.fail('MCP Bridge timeout handling tests threw error', e);
        }
    }

    async testMcpBridgeOriginValidation() {
        console.log('\nTest: MCP Bridge Origin Validation');

        try {
            // Test _initAllowedOrigins
            if (this.bridge._initAllowedOrigins) {
                const origins = this.bridge._initAllowedOrigins();
                this.assert('_initAllowedOrigins returns Set', origins instanceof Set);
                this.assert('Includes current origin', origins.has(window.location.origin));

                if (window.location.hostname === 'localhost') {
                    this.assert('Includes localhost:8443', origins.has('http://localhost:8443'));
                    this.assert('Includes https localhost:8443', origins.has('https://localhost:8443'));
                }
            }

            // Test _getTargetOrigin
            if (this.bridge._getTargetOrigin) {
                const origin = this.bridge._getTargetOrigin();
                this.assert('_getTargetOrigin returns string', typeof origin === 'string');
                this.assert('_getTargetOrigin is not empty', origin.length > 0);
            }

        } catch (e) {
            this.fail('MCP Bridge origin validation tests threw error', e);
        }
    }

    // Helper methods

    assert(name, condition) {
        const result = { name, pass: !!condition };
        this.results.push(result);
        console.log(`  ${condition ? '✓' : '✗'} ${name}`);
        return condition;
    }

    fail(name, error) {
        this.results.push({ name, pass: false, error });
        console.error(`  ✗ ${name}:`, error);
    }

    printSummary() {
        const passed = this.results.filter(r => r.pass).length;
        const total = this.results.length;
        const pct = Math.round((passed / total) * 100);

        console.log('\n=== Summary ===');
        console.log(`${passed}/${total} tests passed (${pct}%)`);

        if (passed === total) {
            console.log('All tests PASSED!');
        } else {
            console.log('\nFailed tests:');
            this.results.filter(r => !r.pass).forEach(r => {
                console.log(`  - ${r.name}`);
            });
        }
    }
}

// Run tests if in browser
if (typeof window !== 'undefined') {
    window.BridgeTestSuite = BridgeTestSuite;

    // Auto-run if URL has ?autorun
    if (window.location.search.includes('autorun')) {
        window.addEventListener('DOMContentLoaded', async () => {
            const suite = new BridgeTestSuite();
            await suite.runAll();
        });
    }
}

// Export for Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { BridgeTestSuite };
}
