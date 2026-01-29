/**
 * Widget test helpers
 *
 * Reusable utilities for testing Communitas widgets
 */

const { McpMock } = require('./mcp-mock');

/**
 * Start MCP mock server
 * @returns {Promise<McpMock>} Mock instance
 */
async function startMcpMock() {
  const mock = new McpMock();

  // Register common tool responses
  mock.registerResponse('list_contacts', {
    success: true,
    contacts: [
      { id: '1', name: 'Alice', status: 'online', favorite: true },
      { id: '2', name: 'Bob', status: 'away', favorite: false },
      { id: '3', name: 'Carol', status: 'busy', favorite: false },
    ]
  });

  mock.registerResponse('list_threads', {
    success: true,
    threads: [
      { id: '1', name: 'General', unread: 5, lastMessage: 'Hello world' },
      { id: '2', name: 'Random', unread: 0, lastMessage: 'Bye' },
    ]
  });

  return mock;
}

/**
 * Load a widget in the test page
 * @param {Page} page - Playwright page
 * @param {string} widgetName - Widget name (e.g., 'contacts', 'messages')
 * @returns {Promise<void>}
 */
async function loadWidget(page, widgetName) {
  await page.goto(`/widgets/${widgetName}/index.html`);
  await page.waitForLoadState('networkidle');
}

/**
 * Wait for widget to be ready
 * @param {Page} page - Playwright page
 * @returns {Promise<void>}
 */
async function waitForWidgetReady(page) {
  await page.waitForSelector('.widget-container', { state: 'visible' });
}

/**
 * Get widget element
 * @param {Page} page - Playwright page
 * @returns {Locator} Widget container locator
 */
function getWidgetContainer(page) {
  return page.locator('.widget-container');
}

/**
 * Check if widget shows error state
 * @param {Page} page - Playwright page
 * @returns {Promise<boolean>}
 */
async function hasErrorState(page) {
  return await page.locator('.error-message').isVisible();
}

/**
 * Check if widget shows loading state
 * @param {Page} page - Playwright page
 * @returns {Promise<boolean>}
 */
async function hasLoadingState(page) {
  return await page.locator('.loading-spinner').isVisible();
}

/**
 * Check if widget shows empty state
 * @param {Page} page - Playwright page
 * @returns {Promise<boolean>}
 */
async function hasEmptyState(page) {
  return await page.locator('.empty-state').isVisible();
}

/**
 * Simulate MCP tool response
 * @param {Page} page - Playwright page
 * @param {string} toolName - Tool name
 * @param {*} response - Response data
 * @returns {Promise<void>}
 */
async function mockToolResponse(page, toolName, response) {
  await page.evaluate(([tool, data]) => {
    window.postMessage({
      type: 'mcp-response',
      tool,
      result: data
    }, '*');
  }, [toolName, response]);
}

module.exports = {
  startMcpMock,
  loadWidget,
  waitForWidgetReady,
  getWidgetContainer,
  hasErrorState,
  hasLoadingState,
  hasEmptyState,
  mockToolResponse,
};
