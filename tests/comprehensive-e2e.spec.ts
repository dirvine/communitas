/**
 * Comprehensive End-to-End Tests for Communitas
 *
 * Tests complete user journeys from authentication through
 * collaboration features, security validation, and performance.
 */

import { test, expect, Page } from '@playwright/test'

// Test data and utilities
const TEST_USERS = {
  alice: { username: 'alice_test', password: 'SecurePass123!' },
  bob: { username: 'bob_test', password: 'SecurePass456!' },
  charlie: { username: 'charlie_test', password: 'SecurePass789!' }
}

const TEST_DATA = {
  document: 'This is a test document for collaboration',
  message: 'Hello from automated test!',
  fileName: 'test-document.txt'
}

// Helper functions
async function loginUser(page: Page, user: typeof TEST_USERS.alice) {
  await page.goto('/')
  await page.waitForLoadState('networkidle')

  // Wait for login form to be visible
  await page.waitForSelector('[data-testid="login-username"]', { timeout: 10000 })

  await page.fill('[data-testid="login-username"]', user.username)
  await page.fill('[data-testid="login-password"]', user.password)
  await page.click('[data-testid="login-submit"]')

  // Wait for successful login
  await page.waitForSelector('[data-testid="dashboard"]', { timeout: 10000 })
}

async function createTestDocument(page: Page, content: string) {
  // Navigate to document creation
  await page.click('[data-testid="new-document"]')

  // Wait for editor to load
  await page.waitForSelector('[data-testid="document-editor"]', { timeout: 5000 })

  // Enter content
  await page.fill('[data-testid="document-editor"]', content)

  // Save document
  await page.click('[data-testid="save-document"]')

  // Wait for save confirmation
  await page.waitForSelector('[data-testid="save-success"]', { timeout: 5000 })
}

async function sendTestMessage(page: Page, message: string) {
  // Navigate to chat
  await page.click('[data-testid="chat-tab"]')

  // Wait for chat interface
  await page.waitForSelector('[data-testid="message-input"]', { timeout: 5000 })

  // Send message
  await page.fill('[data-testid="message-input"]', message)
  await page.click('[data-testid="send-message"]')

  // Wait for message to appear
  await page.waitForSelector(`[data-testid="message-content"]:has-text("${message}")`, { timeout: 5000 })
}

test.describe('Communitas Comprehensive E2E Suite', () => {
  test.beforeEach(async ({ page }) => {
    // Set up test environment
    await page.context().addInitScript(() => {
      // Mock any external dependencies
      window.localStorage.setItem('test-mode', 'true')
    })
  })

  test('complete user registration and authentication flow', async ({ page }) => {
    const newUser = {
      username: `testuser_${Date.now()}`,
      password: 'SecurePass123!',
      confirmPassword: 'SecurePass123!'
    }

    await page.goto('/')

    // Navigate to registration
    await page.click('[data-testid="register-button"]')

    // Fill registration form
    await page.fill('[data-testid="username-input"]', newUser.username)
    await page.fill('[data-testid="password-input"]', newUser.password)
    await page.fill('[data-testid="confirm-password-input"]', newUser.confirmPassword)

    // Submit registration
    await page.click('[data-testid="submit-registration"]')

    // Verify successful registration
    await expect(page.locator('[data-testid="registration-success"]')).toBeVisible()

    // Test login with new account
    await page.fill('[data-testid="login-username"]', newUser.username)
    await page.fill('[data-testid="login-password"]', newUser.password)
    await page.click('[data-testid="login-submit"]')

    // Verify successful login
    await expect(page.locator('[data-testid="dashboard"]')).toBeVisible()
  })

  test('real-time messaging and presence indicators', async ({ browser }) => {
    // Create two browser contexts for simulating multiple users
    const context1 = await browser.newContext()
    const context2 = await browser.newContext()
    const page1 = await context1.newPage()
    const page2 = await context2.newPage()

    try {
      // User 1: Login and join chat
      await loginUser(page1, TEST_USERS.alice)
      await page1.click('[data-testid="chat-room-general"]')

      // User 2: Login and join same chat
      await loginUser(page2, TEST_USERS.bob)
      await page2.click('[data-testid="chat-room-general"]')

      // Verify presence indicators
      await expect(page1.locator('[data-testid="presence-bob"]')).toBeVisible()
      await expect(page2.locator('[data-testid="presence-alice"]')).toBeVisible()

      // Test real-time messaging
      const testMessage = `Hello from Alice at ${Date.now()}!`
      await sendTestMessage(page1, testMessage)

      // Verify message appears for both users
      await expect(page1.locator(`[data-testid="message-content"]:has-text("${testMessage}")`)).toBeVisible()
      await expect(page2.locator(`[data-testid="message-content"]:has-text("${testMessage}")`)).toBeVisible()

      // Test typing indicators
      await page2.fill('[data-testid="message-input"]', 'Bob is typing...')
      await expect(page1.locator('[data-testid="typing-indicator"]')).toContainText('Bob is typing')

    } finally {
      await context1.close()
      await context2.close()
    }
  })

  test('document collaboration and synchronization', async ({ browser }) => {
    const context1 = await browser.newContext()
    const context2 = await browser.newContext()
    const page1 = await context1.newPage()
    const page2 = await context2.newPage()

    try {
      // Both users login
      await loginUser(page1, TEST_USERS.alice)
      await loginUser(page2, TEST_USERS.bob)

      // User 1 creates a document
      await createTestDocument(page1, TEST_DATA.document)

      // User 2 navigates to the same document (assuming shared workspace)
      await page2.click('[data-testid="shared-documents"]')
      await page2.click('[data-testid="document-test-document"]')

      // Verify document content is synchronized
      await expect(page2.locator('[data-testid="document-editor"]')).toHaveValue(TEST_DATA.document)

      // User 2 makes an edit
      const additionalContent = ' - Edited by Bob'
      await page2.fill('[data-testid="document-editor"]', TEST_DATA.document + additionalContent)
      await page2.click('[data-testid="save-document"]')

      // Verify change appears for User 1
      await page1.reload()
      await expect(page1.locator('[data-testid="document-editor"]')).toHaveValue(TEST_DATA.document + additionalContent)

    } finally {
      await context1.close()
      await context2.close()
    }
  })

  test('file sharing and storage integration', async ({ page }) => {
    await loginUser(page, TEST_USERS.alice)

    // Navigate to file sharing area
    await page.click('[data-testid="file-sharing-tab"]')

    // Create a test file
    const testContent = 'Test file content for sharing'
    const fileContent = Buffer.from(testContent)

    // Upload file
    const fileInput = page.locator('[data-testid="file-upload-input"]')
    await fileInput.setInputFiles({
      name: TEST_DATA.fileName,
      mimeType: 'text/plain',
      buffer: fileContent
    })

    // Verify file upload
    await expect(page.locator(`[data-testid="uploaded-file-${TEST_DATA.fileName}"]`)).toBeVisible()

    // Share file with another user
    await page.click(`[data-testid="share-file-${TEST_DATA.fileName}"]`)
    await page.fill('[data-testid="share-with-input"]', TEST_USERS.bob.username)
    await page.click('[data-testid="confirm-share"]')

    // Verify sharing notification
    await expect(page.locator('[data-testid="share-success-notification"]')).toBeVisible()
  })

  test('security features and input validation', async ({ page }) => {
    await page.goto('/')

    // Test XSS prevention
    await page.fill('[data-testid="message-input"]', '<script>alert("XSS")</script>')
    await page.click('[data-testid="send-message"]')

    // Verify script is sanitized
    await expect(page.locator('[data-testid="message-content"]').last()).not.toContainText('<script>')

    // Test SQL injection prevention
    await page.fill('[data-testid="search-input"]', "'; DROP TABLE users; --")
    await page.click('[data-testid="search-submit"]')

    // Verify no error and safe handling
    await expect(page.locator('[data-testid="error-message"]')).not.toBeVisible()

    // Test file upload security
    const fileInput = page.locator('[data-testid="file-upload-input"]')
    await fileInput.setInputFiles({
      name: '../../../etc/passwd',
      mimeType: 'text/plain',
      buffer: Buffer.from('malicious content')
    })

    // Verify path traversal is blocked
    await expect(page.locator('[data-testid="upload-error"]')).toContainText('Invalid file path')
  })

  test('responsive design and mobile experience', async ({ browser }) => {
    // Test mobile viewport
    const context = await browser.newContext({
      viewport: { width: 375, height: 667 },
      userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15'
    })
    const mobilePage = await context.newPage()

    try {
      await loginUser(mobilePage, TEST_USERS.alice)

      // Verify mobile layout
      await expect(mobilePage.locator('[data-testid="mobile-navigation"]')).toBeVisible()
      await expect(mobilePage.locator('[data-testid="chat-interface"]')).toHaveCSS('width', '375px')

      // Test touch interactions
      await mobilePage.click('[data-testid="mobile-menu-toggle"]')
      await expect(mobilePage.locator('[data-testid="mobile-menu"]')).toBeVisible()

      // Test mobile messaging
      await sendTestMessage(mobilePage, 'Mobile test message')
      await expect(mobilePage.locator('[data-testid="message-content"]').last()).toContainText('Mobile test message')

    } finally {
      await context.close()
    }
  })

  test('performance and accessibility compliance', async ({ page }) => {
    await loginUser(page, TEST_USERS.alice)

    // Measure page load performance
    const startTime = Date.now()
    await page.click('[data-testid="chat-room-general"]')
    const loadTime = Date.now() - startTime

    // Verify sub-100ms interaction (allowing some margin for test environment)
    expect(loadTime).toBeLessThan(500)

    // Test accessibility
    const accessibilitySnapshot = await page.accessibility.snapshot()
    expect(accessibilitySnapshot).toBeDefined()

    // Verify ARIA labels
    await expect(page.locator('[aria-label="Send message"]')).toBeVisible()
    await expect(page.locator('[aria-label="File upload"]')).toBeVisible()

    // Test keyboard navigation
    await page.keyboard.press('Tab')
    await expect(page.locator(':focus')).toHaveAttribute('data-testid', 'message-input')

    await page.keyboard.press('Tab')
    await expect(page.locator(':focus')).toHaveAttribute('data-testid', 'send-button')
  })

  test('error handling and recovery', async ({ page }) => {
    await loginUser(page, TEST_USERS.alice)

    // Simulate network disconnection
    await page.context().setOffline(true)

    // Try to send message
    await page.fill('[data-testid="message-input"]', 'Test message during offline')
    await page.click('[data-testid="send-message"]')

    // Verify offline handling
    await expect(page.locator('[data-testid="offline-indicator"]')).toBeVisible()
    await expect(page.locator('[data-testid="message-status"]').last()).toHaveAttribute('data-status', 'pending')

    // Reconnect
    await page.context().setOffline(false)

    // Verify message sync
    await expect(page.locator('[data-testid="message-status"]').last()).toHaveAttribute('data-status', 'sent')

    // Test error recovery for failed operations
    await expect(page.locator('[data-testid="retry-button"]')).not.toBeVisible()
  })

  test('data persistence and synchronization', async ({ browser }) => {
    const context1 = await browser.newContext()
    const context2 = await browser.newContext()
    const page1 = await context1.newPage()
    const page2 = await context2.newPage()

    try {
      await loginUser(page1, TEST_USERS.alice)

      // Create content
      await sendTestMessage(page1, 'Persistent test message')

      // Verify message appears
      await expect(page1.locator('[data-testid="message-content"]').last()).toContainText('Persistent test message')

      // Simulate page refresh
      await page1.reload()
      await page1.waitForLoadState('networkidle')

      // Verify data persistence
      await expect(page1.locator('[data-testid="message-content"]').last()).toContainText('Persistent test message')

      // Test cross-device synchronization
      await loginUser(page2, TEST_USERS.alice)

      // Verify same data appears on second device
      await expect(page2.locator('[data-testid="message-content"]').last()).toContainText('Persistent test message')

    } finally {
      await context1.close()
      await context2.close()
    }
  })

  test('PQC cryptographic operations end-to-end', async ({ page }) => {
    await loginUser(page, TEST_USERS.alice)

    // Navigate to secure messaging
    await page.click('[data-testid="secure-messaging"]')

    // Send a PQC-encrypted message
    const secretMessage = 'This is a secret message protected by ML-DSA and Kyber'
    await page.fill('[data-testid="secure-message-input"]', secretMessage)
    await page.click('[data-testid="send-encrypted"]')

    // Verify PQC encryption indicator
    await expect(page.locator('[data-testid="encryption-indicator"]')).toHaveAttribute('data-status', 'pqc-encrypted')

    // Verify message content is not visible in plain text
    const messageElement = page.locator('[data-testid="message-content"]').last()
    const messageText = await messageElement.textContent()
    expect(messageText).not.toContain(secretMessage) // Should be PQC-encrypted

    // Test PQC decryption (assuming auto-decryption for own messages)
    await expect(page.locator('[data-testid="decrypted-content"]')).toContainText(secretMessage)

    // Verify PQC signature
    await expect(page.locator('[data-testid="signature-status"]')).toHaveAttribute('data-valid', 'true')
  })

  test('load testing and performance validation', async ({ page }) => {
    await loginUser(page, TEST_USERS.alice)

    // Test multiple rapid operations
    const startTime = Date.now()

    for (let i = 0; i < 10; i++) {
      await sendTestMessage(page, `Load test message ${i}`)
    }

    const endTime = Date.now()
    const totalTime = endTime - startTime

    // Verify all messages were sent
    for (let i = 0; i < 10; i++) {
      await expect(page.locator(`[data-testid="message-content"]:has-text("Load test message ${i}")`)).toBeVisible()
    }

    // Verify reasonable performance (should complete within reasonable time)
    expect(totalTime).toBeLessThan(10000) // 10 seconds for 10 messages

    // Test memory usage (basic check)
    const performance = await page.evaluate(() => ({
      memory: (performance as any).memory
    }))

    if (performance.memory) {
      // Ensure memory usage is reasonable
      expect(performance.memory.usedJSHeapSize).toBeLessThan(100 * 1024 * 1024) // 100MB
    }
  })
})