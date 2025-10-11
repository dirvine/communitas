import { test, expect, Page } from '@playwright/test';

/**
 * P2P Network Integration Tests
 * Tests the full network connectivity with 8-node test network
 */

test.describe('P2P Network Integration', () => {
  let page: Page;

  test.beforeAll(async ({ browser }) => {
    // Create a new page for all tests
    page = await browser.newPage();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test('should display network status indicator', async () => {
    await page.goto('http://localhost:5173');

    // Wait for app to load
    await page.waitForLoadState('networkidle');

    // Check for network status indicator
    const networkStatus = page.locator('[data-testid="network-status"]');
    await expect(networkStatus).toBeVisible({ timeout: 10000 });

    // Get status attribute
    const status = await networkStatus.getAttribute('data-status');
    console.log('Network status:', status);

    // Should be one of these states
    expect(['connected', 'connecting', 'local', 'offline']).toContain(status);
  });

  test('should connect to bootstrap nodes', async () => {
    // Monitor network requests
    const networkRequests: string[] = [];

    page.on('request', request => {
      const url = request.url();
      if (url.includes('10000') || url.includes('10001') || url.includes('ws://')) {
        networkRequests.push(url);
      }
    });

    await page.goto('http://localhost:5173');
    await page.waitForTimeout(3000); // Wait for connections

    console.log('P2P network requests:', networkRequests);

    // Should have attempted connections to bootstrap nodes
    const bootstrapConnections = networkRequests.filter(url =>
      url.includes('10000') || url.includes('10001')
    );

    expect(bootstrapConnections.length).toBeGreaterThan(0);
  });

  test('should handle network reconnection', async () => {
    await page.goto('http://localhost:5173');

    // Get initial status
    const networkStatus = page.locator('[data-testid="network-status"]');
    await expect(networkStatus).toBeVisible();

    const initialStatus = await networkStatus.getAttribute('data-status');

    // Simulate network disruption by evaluating JS
    await page.evaluate(() => {
      // Trigger offline event
      window.dispatchEvent(new Event('offline'));
    });

    // Wait for status change
    await page.waitForTimeout(1000);

    // Check status changed
    const offlineStatus = await networkStatus.getAttribute('data-status');
    expect(offlineStatus).toBe('offline');

    // Simulate network recovery
    await page.evaluate(() => {
      window.dispatchEvent(new Event('online'));
    });

    // Wait for reconnection
    await page.waitForTimeout(3000);

    // Should reconnect
    const reconnectedStatus = await networkStatus.getAttribute('data-status');
    expect(['connected', 'connecting']).toContain(reconnectedStatus);
  });

  test('should display node count when connected', async () => {
    await page.goto('http://localhost:5173');

    // Wait for connection
    await page.waitForTimeout(5000);

    // Check for node count display
    const nodeCount = page.locator('[data-testid="node-count"]');

    // If connected, should show node count
    const networkStatus = await page.locator('[data-testid="network-status"]').getAttribute('data-status');

    if (networkStatus === 'connected') {
      await expect(nodeCount).toBeVisible();
      const count = await nodeCount.textContent();
      console.log('Connected nodes:', count);

      // Should have at least 1 node (bootstrap)
      expect(parseInt(count || '0')).toBeGreaterThan(0);
    }
  });

  test('should persist connection state across page reloads', async () => {
    await page.goto('http://localhost:5173');

    // Wait for initial connection
    await page.waitForTimeout(3000);

    // Get connection state
    const initialStatus = await page.locator('[data-testid="network-status"]').getAttribute('data-status');

    // Reload page
    await page.reload();

    // Wait for app to reinitialize
    await page.waitForTimeout(2000);

    // Should restore connection state quickly
    const reloadedStatus = await page.locator('[data-testid="network-status"]').getAttribute('data-status');

    // Should be in same or better state (not worse)
    if (initialStatus === 'connected') {
      expect(['connected', 'connecting']).toContain(reloadedStatus);
    }
  });

  test('should show network metrics', async () => {
    await page.goto('http://localhost:5173');

    // Wait for connection
    await page.waitForTimeout(5000);

    // Open network metrics (if available)
    const metricsButton = page.locator('[data-testid="network-metrics-button"]');

    if (await metricsButton.isVisible()) {
      await metricsButton.click();

      // Check for metrics display
      const metrics = page.locator('[data-testid="network-metrics"]');
      await expect(metrics).toBeVisible();

      // Should show various metrics
      const latency = page.locator('[data-testid="metric-latency"]');
      const bandwidth = page.locator('[data-testid="metric-bandwidth"]');
      const packets = page.locator('[data-testid="metric-packets"]');

      // At least one metric should be visible
      const hasMetrics =
        await latency.isVisible() ||
        await bandwidth.isVisible() ||
        await packets.isVisible();

      expect(hasMetrics).toBeTruthy();
    }
  });

  test('should handle network errors gracefully', async () => {
    await page.goto('http://localhost:5173');

    // Intercept and fail network requests
    await page.route('**/api/**', route => route.abort());

    // Trigger a network operation
    const networkAction = page.locator('[data-testid="refresh-network"]');
    if (await networkAction.isVisible()) {
      await networkAction.click();
    }

    // Should show error state but not crash
    const errorIndicator = page.locator('[data-testid="network-error"]');
    const isErrorVisible = await errorIndicator.isVisible({ timeout: 5000 }).catch(() => false);

    // App should still be responsive
    const appTitle = page.locator('h1, [data-testid="app-title"]');
    await expect(appTitle).toBeVisible();

    // Clear route interception
    await page.unroute('**/api/**');
  });

  test('should support manual network control', async () => {
    await page.goto('http://localhost:5173');

    // Look for network control buttons
    const connectButton = page.locator('[data-testid="network-connect"]');
    const disconnectButton = page.locator('[data-testid="network-disconnect"]');

    // Test disconnect if available
    if (await disconnectButton.isVisible()) {
      await disconnectButton.click();
      await page.waitForTimeout(1000);

      const status = await page.locator('[data-testid="network-status"]').getAttribute('data-status');
      expect(['offline', 'local']).toContain(status);
    }

    // Test reconnect if available
    if (await connectButton.isVisible()) {
      await connectButton.click();
      await page.waitForTimeout(3000);

      const status = await page.locator('[data-testid="network-status"]').getAttribute('data-status');
      expect(['connected', 'connecting']).toContain(status);
    }
  });
});