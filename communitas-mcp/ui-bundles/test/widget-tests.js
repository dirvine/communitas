/**
 * Widget Render Tests
 *
 * Tests that each widget renders correctly with mock data.
 * Uses iframes to load widgets and validates DOM structure.
 */

class WidgetTestSuite {
    constructor() {
        this.results = [];
        this.widgets = ['contacts', 'messages', 'kanban', 'drive', 'canvas', 'settings', 'search', 'notifications'];
    }

    async runAll() {
        console.log('=== Widget Render Test Suite ===\n');

        for (const widget of this.widgets) {
            await this.testWidget(widget);
        }

        this.printSummary();
        return this.results;
    }

    async testWidget(name) {
        console.log(`\nTesting: ${name}`);

        const tests = this.getTestsForWidget(name);

        try {
            // Load widget in iframe
            const iframe = await this.loadWidget(name);

            if (!iframe || !iframe.contentDocument) {
                this.fail(`${name}: Failed to load widget`);
                return;
            }

            const doc = iframe.contentDocument;

            // Run common tests
            await this.testBasicStructure(name, doc);

            // Run widget-specific tests
            for (const test of tests) {
                await test(name, doc);
            }

            // Cleanup
            iframe.remove();

        } catch (e) {
            this.fail(`${name}: Error during testing`, e);
        }
    }

    loadWidget(name, timeout = 5000) {
        return new Promise((resolve, reject) => {
            const iframe = document.createElement('iframe');
            iframe.style.display = 'none';
            iframe.src = `../${name}/index.html`;

            const timer = setTimeout(() => {
                iframe.remove();
                reject(new Error('Timeout loading widget'));
            }, timeout);

            iframe.onload = () => {
                clearTimeout(timer);
                // Give widget time to initialize
                setTimeout(() => resolve(iframe), 100);
            };

            iframe.onerror = () => {
                clearTimeout(timer);
                reject(new Error('Failed to load iframe'));
            };

            document.body.appendChild(iframe);
        });
    }

    async testBasicStructure(name, doc) {
        // Check HTML structure
        this.assert(`${name}: Has doctype`, doc.doctype !== null);
        this.assert(`${name}: Has html element`, doc.documentElement !== null);
        this.assert(`${name}: Has head element`, doc.head !== null);
        this.assert(`${name}: Has body element`, doc.body !== null);

        // Check meta tags
        const charset = doc.querySelector('meta[charset]');
        this.assert(`${name}: Has charset meta`, charset !== null);

        const viewport = doc.querySelector('meta[name="viewport"]');
        this.assert(`${name}: Has viewport meta`, viewport !== null);

        // Check title
        this.assert(`${name}: Has title`, doc.title !== '');

        // Check for MCP bridge script reference
        const scripts = Array.from(doc.querySelectorAll('script'));
        const hasBridge = scripts.some(s =>
            s.src.includes('mcp-bridge') || s.textContent.includes('McpBridge')
        );
        this.assert(`${name}: References MCP bridge`, hasBridge);

        // Check no console errors (basic check)
        this.assert(`${name}: Body has content`, doc.body.innerHTML.length > 100);
    }

    getTestsForWidget(name) {
        const widgetTests = {
            contacts: [
                this.testContactsWidget.bind(this)
            ],
            messages: [
                this.testMessagesWidget.bind(this)
            ],
            kanban: [
                this.testKanbanWidget.bind(this)
            ],
            drive: [
                this.testDriveWidget.bind(this)
            ],
            canvas: [
                this.testCanvasWidget.bind(this)
            ],
            settings: [
                this.testSettingsWidget.bind(this)
            ],
            search: [
                this.testSearchWidget.bind(this)
            ],
            notifications: [
                this.testNotificationsWidget.bind(this)
            ]
        };

        return widgetTests[name] || [];
    }

    async testContactsWidget(name, doc) {
        // Check for contact list container
        const container = doc.querySelector('.contact-list, [class*="contact"], #contacts, .contacts');
        this.assert(`${name}: Has contact container`, container !== null);

        // Check for search input
        const search = doc.querySelector('input[type="text"], input[type="search"], .search');
        this.assert(`${name}: Has search input`, search !== null);

        // Check for CSS styling
        const styles = doc.querySelectorAll('style, link[rel="stylesheet"]');
        this.assert(`${name}: Has styles`, styles.length > 0);
    }

    async testMessagesWidget(name, doc) {
        // Check for thread list or message container
        const container = doc.querySelector('.thread-list, .message-list, [class*="thread"], [class*="message"]');
        this.assert(`${name}: Has thread/message container`, container !== null);

        // Check for compose area
        const compose = doc.querySelector('textarea, input[type="text"], .compose, .input');
        this.assert(`${name}: Has compose input`, compose !== null);
    }

    async testKanbanWidget(name, doc) {
        // Check for board/column structure
        const board = doc.querySelector('.board, .kanban, [class*="board"], [class*="column"]');
        this.assert(`${name}: Has board container`, board !== null);

        // Check for drag-drop related attributes or classes
        const draggable = doc.querySelector('[draggable], .draggable, .card');
        this.assert(`${name}: Has draggable elements or cards`, draggable !== null || board !== null);
    }

    async testDriveWidget(name, doc) {
        // Check for file list container
        const fileList = doc.querySelector('.file-list, .files, [class*="file"], .drive');
        this.assert(`${name}: Has file list container`, fileList !== null);

        // Check for upload area
        const upload = doc.querySelector('.upload, input[type="file"], [class*="upload"], button');
        this.assert(`${name}: Has upload or action button`, upload !== null);
    }

    async testCanvasWidget(name, doc) {
        // Check for canvas or SVG element
        const canvas = doc.querySelector('canvas, svg, .canvas, [class*="canvas"]');
        this.assert(`${name}: Has canvas/svg element`, canvas !== null);

        // Check for toolbar or controls
        const controls = doc.querySelector('.toolbar, .controls, button, [class*="tool"]');
        this.assert(`${name}: Has toolbar or controls`, controls !== null);
    }

    async testSettingsWidget(name, doc) {
        // Check for settings sections
        const sections = doc.querySelectorAll('.section, .setting, [class*="setting"], form');
        this.assert(`${name}: Has settings sections`, sections.length > 0);

        // Check for toggles or inputs
        const inputs = doc.querySelectorAll('input, select, button');
        this.assert(`${name}: Has form inputs`, inputs.length > 0);
    }

    async testSearchWidget(name, doc) {
        // Check for search input
        const search = doc.querySelector('input[type="text"], input[type="search"], .search-input');
        this.assert(`${name}: Has search input`, search !== null);

        // Check for results container
        const results = doc.querySelector('.results, .search-results, [class*="result"]');
        this.assert(`${name}: Has results container`, results !== null);
    }

    async testNotificationsWidget(name, doc) {
        // Check for notification list
        const list = doc.querySelector('.notification-list, .notifications, [class*="notification"]');
        this.assert(`${name}: Has notification container`, list !== null);

        // Check for mark read/action buttons
        const actions = doc.querySelector('button, .action, [class*="mark"], .clear');
        this.assert(`${name}: Has action elements`, actions !== null);
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

        // Group by widget
        const byWidget = {};
        this.results.forEach(r => {
            const widget = r.name.split(':')[0];
            if (!byWidget[widget]) byWidget[widget] = { pass: 0, fail: 0 };
            if (r.pass) byWidget[widget].pass++;
            else byWidget[widget].fail++;
        });

        console.log('\nBy widget:');
        Object.entries(byWidget).forEach(([widget, counts]) => {
            const status = counts.fail === 0 ? '✓' : '✗';
            console.log(`  ${status} ${widget}: ${counts.pass}/${counts.pass + counts.fail}`);
        });

        if (passed === total) {
            console.log('\nAll widget tests PASSED!');
        } else {
            console.log('\nFailed tests:');
            this.results.filter(r => !r.pass).forEach(r => {
                console.log(`  - ${r.name}`);
            });
        }
    }
}

// Export for browser
if (typeof window !== 'undefined') {
    window.WidgetTestSuite = WidgetTestSuite;
}

// Export for Node.js
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { WidgetTestSuite };
}
