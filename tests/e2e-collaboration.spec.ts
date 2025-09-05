import { test, expect } from '@playwright/test';

/**
 * End-to-End Collaboration Tests
 * Tests complete user journeys including real-time features, security, and UX
 */

test.describe('Communitas E2E Collaboration Suite', () => {
  test.beforeEach(async ({ page }) => {
    // Set up test environment
    await page.goto('/');
    await page.waitForLoadState('networkidle');
  });

  test('complete user registration and authentication flow', async ({ page }) => {
    // Test user registration
    await page.click('[data-testid="register-button"]');

    // Fill registration form
    await page.fill('[data-testid="username-input"]', 'testuser');
    await page.fill('[data-testid="password-input"]', 'SecurePass123!');
    await page.fill('[data-testid="confirm-password-input"]', 'SecurePass123!');

    // Submit registration
    await page.click('[data-testid="submit-registration"]');

    // Verify successful registration
    await expect(page.locator('[data-testid="registration-success"]')).toBeVisible();

    // Test login
    await page.fill('[data-testid="login-username"]', 'testuser');
    await page.fill('[data-testid="login-password"]', 'SecurePass123!');
    await page.click('[data-testid="login-submit"]');

    // Verify successful login
    await expect(page.locator('[data-testid="dashboard"]')).toBeVisible();
  });

  test('real-time messaging and presence indicators', async ({ page, context }) => {
    // Create two browser contexts for simulating multiple users
    const user1Page = page;
    const user2Context = await context.newPage();

    // User 1: Login and join chat
    await user1Page.fill('[data-testid="login-username"]', 'alice');
    await user1Page.fill('[data-testid="login-password"]', 'password123');
    await user1Page.click('[data-testid="login-submit"]');

    // Navigate to chat
    await user1Page.click('[data-testid="chat-room-general"]');

    // User 2: Login and join same chat
    await user2Context.goto('/');
    await user2Context.fill('[data-testid="login-username"]', 'bob');
    await user2Context.fill('[data-testid="login-password"]', 'password123');
    await user2Context.click('[data-testid="login-submit"]');
    await user2Context.click('[data-testid="chat-room-general"]');

    // Verify presence indicators
    await expect(user1Page.locator('[data-testid="presence-bob"]')).toBeVisible();
    await expect(user2Context.locator('[data-testid="presence-alice"]')).toBeVisible();

    // Test real-time messaging
    await user1Page.fill('[data-testid="message-input"]', 'Hello from Alice!');
    await user1Page.click('[data-testid="send-message"]');

    // Verify message appears for both users
    await expect(user1Page.locator('[data-testid="message-content"]').last()).toContainText('Hello from Alice!');
    await expect(user2Context.locator('[data-testid="message-content"]').last()).toContainText('Hello from Alice!');

    // Test typing indicators
    await user2Context.fill('[data-testid="message-input"]', 'Bob is typing...');
    await expect(user1Page.locator('[data-testid="typing-indicator"]')).toContainText('Bob is typing');

    // Test message status (sent, delivered, read)
    await expect(user1Page.locator('[data-testid="message-status"]').last()).toHaveAttribute('data-status', 'delivered');
  });

  test('file sharing and collaboration features', async ({ page }) => {
    // Login
    await page.fill('[data-testid="login-username"]', 'testuser');
    await page.fill('[data-testid="login-password"]', 'password123');
    await page.click('[data-testid="login-submit"]');

    // Navigate to file sharing area
    await page.click('[data-testid="file-sharing-tab"]');

    // Upload a file
    const fileInput = page.locator('[data-testid="file-upload-input"]');
    await fileInput.setInputFiles({
      name: 'test-document.pdf',
      mimeType: 'application/pdf',
      buffer: Buffer.from('Test PDF content')
    });

    // Verify file upload
    await expect(page.locator('[data-testid="uploaded-file-test-document.pdf"]')).toBeVisible();

    // Share file with collaborators
    await page.click('[data-testid="share-file-test-document.pdf"]');
    await page.fill('[data-testid="share-with-input"]', 'alice');
    await page.click('[data-testid="confirm-share"]');

    // Verify sharing notification
    await expect(page.locator('[data-testid="share-success-notification"]')).toBeVisible();
  });

  test('security features and input validation', async ({ page }) => {
    // Test XSS prevention
    await page.fill('[data-testid="message-input"]', '<script>alert("XSS")</script>');
    await page.click('[data-testid="send-message"]');

    // Verify script is sanitized
    await expect(page.locator('[data-testid="message-content"]').last()).not.toContainText('<script>');

    // Test SQL injection prevention
    await page.fill('[data-testid="search-input"]', "'; DROP TABLE users; --");
    await page.click('[data-testid="search-submit"]');

    // Verify no error and safe handling
    await expect(page.locator('[data-testid="error-message"]')).not.toBeVisible();

    // Test file upload security
    const fileInput = page.locator('[data-testid="file-upload-input"]');
    await fileInput.setInputFiles({
      name: '../../../etc/passwd',
      mimeType: 'text/plain',
      buffer: Buffer.from('malicious content')
    });

    // Verify path traversal is blocked
    await expect(page.locator('[data-testid="upload-error"]')).toContainText('Invalid file path');
  });

  test('responsive design and mobile experience', async ({ page, browser }) => {
    // Test mobile viewport
    const mobileContext = await browser.newContext({
      viewport: { width: 375, height: 667 },
      userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15'
    });
    const mobilePage = await mobileContext.newPage();

    await mobilePage.goto('/');
    await mobilePage.fill('[data-testid="login-username"]', 'mobileuser');
    await mobilePage.fill('[data-testid="login-password"]', 'password123');
    await mobilePage.click('[data-testid="login-submit"]');

    // Verify mobile layout
    await expect(mobilePage.locator('[data-testid="mobile-navigation"]')).toBeVisible();
    await expect(mobilePage.locator('[data-testid="chat-interface"]')).toHaveCSS('width', '375px');

    // Test touch interactions
    await mobilePage.click('[data-testid="mobile-menu-toggle"]');
    await expect(mobilePage.locator('[data-testid="mobile-menu"]')).toBeVisible();

    await mobileContext.close();
  });

  test('performance and accessibility compliance', async ({ page }) => {
    // Login
    await page.fill('[data-testid="login-username"]', 'perfuser');
    await page.fill('[data-testid="login-password"]', 'password123');
    await page.click('[data-testid="login-submit"]');

    // Measure page load performance
    const startTime = Date.now();
    await page.click('[data-testid="chat-room-general"]');
    const loadTime = Date.now() - startTime;

    // Verify sub-100ms interaction (allowing some margin for test environment)
    expect(loadTime).toBeLessThan(500);

    // Test accessibility
    const accessibilitySnapshot = await page.accessibility.snapshot();
    expect(accessibilitySnapshot).toBeDefined();

    // Verify ARIA labels
    await expect(page.locator('[aria-label="Send message"]')).toBeVisible();
    await expect(page.locator('[aria-label="File upload"]')).toBeVisible();

    // Test keyboard navigation
    await page.keyboard.press('Tab');
    await expect(page.locator(':focus')).toHaveAttribute('data-testid', 'message-input');

    await page.keyboard.press('Tab');
    await expect(page.locator(':focus')).toHaveAttribute('data-testid', 'send-button');
  });

  test('error handling and recovery', async ({ page }) => {
    // Simulate network disconnection
    await page.context().setOffline(true);

    // Try to send message
    await page.fill('[data-testid="message-input"]', 'Test message');
    await page.click('[data-testid="send-message"]');

    // Verify offline handling
    await expect(page.locator('[data-testid="offline-indicator"]')).toBeVisible();
    await expect(page.locator('[data-testid="message-status"]').last()).toHaveAttribute('data-status', 'pending');

    // Reconnect
    await page.context().setOffline(false);

    // Verify message sync
    await expect(page.locator('[data-testid="message-status"]').last()).toHaveAttribute('data-status', 'sent');

    // Test error recovery for failed operations
    await expect(page.locator('[data-testid="retry-button"]')).not.toBeVisible();
  });

  test('data persistence and synchronization', async ({ page, context }) => {
    // Login and create content
    await page.fill('[data-testid="login-username"]', 'syncuser');
    await page.fill('[data-testid="login-password"]', 'password123');
    await page.click('[data-testid="login-submit"]');

    // Create a message
    await page.fill('[data-testid="message-input"]', 'Persistent message');
    await page.click('[data-testid="send-message"]');

    // Verify message appears
    await expect(page.locator('[data-testid="message-content"]').last()).toContainText('Persistent message');

    // Simulate page refresh
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify data persistence
    await expect(page.locator('[data-testid="message-content"]').last()).toContainText('Persistent message');

    // Test cross-device synchronization
    const device2Page = await context.newPage();
    await device2Page.goto('/');
    await device2Page.fill('[data-testid="login-username"]', 'syncuser');
    await device2Page.fill('[data-testid="login-password"]', 'password123');
    await device2Page.click('[data-testid="login-submit"]');

    // Verify same data appears on second device
    await expect(device2Page.locator('[data-testid="message-content"]').last()).toContainText('Persistent message');
  });
});