/**
 * Smoke test for Playwright infrastructure
 *
 * Verifies that Playwright is set up correctly and can run basic tests
 */

const { test, expect } = require('@playwright/test');
const { startMcpMock, loadWidget, waitForWidgetReady } = require('./utils/widget-helpers');

test.describe('Playwright Infrastructure', () => {
  test('should be able to run tests', async () => {
    // This test just verifies Playwright works
    expect(true).toBe(true);
  });

  test('MCP mock should initialize', async () => {
    const mock = await startMcpMock();
    expect(mock).toBeDefined();
    expect(mock.tools).toBeDefined();
    expect(mock.responses).toBeDefined();
  });

  test('helper functions should be available', () => {
    expect(typeof startMcpMock).toBe('function');
    expect(typeof loadWidget).toBe('function');
    expect(typeof waitForWidgetReady).toBe('function');
  });
});
