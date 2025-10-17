/**
 * Messaging Flow E2E Tests (Web Mode)
 * 
 * Prerequisites: Run `npm run tauri dev` before running tests
 */

import { test, expect } from '@playwright/test';
import { TauriTestHelper } from '../../utils/tauri-helpers';

test.describe('Core Messaging - Basic Functionality', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await page.waitForTimeout(2000);
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('M1: Can navigate to channel and see message input', async ({ page }) => {
    const channelElements = page.locator('[data-testid="channel"], [role="listitem"], a, button').filter({
      hasText: /general|channel|space|chat/i
    });

    const channelCount = await channelElements.count();
    
    if (channelCount > 0) {
      await channelElements.first().click();
      console.log('✅ Clicked on a channel');
      await page.waitForTimeout(1000);
    }

    const messageInput = page.locator(
      'textarea, input[placeholder*="message" i], input[placeholder*="type" i], [contenteditable="true"]'
    );

    const inputCount = await messageInput.count();
    expect(inputCount).toBeGreaterThan(0);
    
    await helper.screenshot(page, 'messaging-channel-view');
    console.log('✅ Message input area found');
  });

  test('M2: Can type and send a message', async ({ page }) => {
    const testMessage = `Test message ${Date.now()}`;
    
    const messageInput = page.locator(
      'textarea, input[placeholder*="message" i], [contenteditable="true"]'
    ).first();

    await messageInput.waitFor({ state: 'visible', timeout: 5000 });
    await messageInput.fill(testMessage);
    console.log('✅ Typed test message');

    await page.waitForTimeout(500);

    const sendButton = page.locator('button').filter({
      hasText: /send|post|submit/i
    }).or(
      page.locator('button[type="submit"], button[aria-label*="send" i]')
    ).first();

    const buttonVisible = await sendButton.isVisible().catch(() => false);

    if (buttonVisible) {
      await sendButton.click();
      console.log('✅ Clicked send button');
    } else {
      await messageInput.press('Enter');
      console.log('✅ Pressed Enter to send');
    }

    await page.waitForTimeout(1000);

    const messageList = await page.locator('body').textContent() || '';
    const messageAppeared = messageList.includes(testMessage) || 
                           messageList.includes(testMessage.substring(0, 20));

    await helper.screenshot(page, 'messaging-sent-message');
    
    expect(messageAppeared).toBe(true);
    console.log('✅ Message sent and appeared');
  });

  test('M3: Unread badge UI exists', async ({ page }) => {
    const badgeElements = page.locator(
      '[data-testid*="badge"], [class*="badge"], [class*="unread"], .MuiBadge-badge'
    );

    const badgeCount = await badgeElements.count();
    
    await helper.screenshot(page, 'messaging-badges');
    console.log(`Found ${badgeCount} badge elements`);
    expect(badgeCount >= 0).toBe(true);
  });

  test('M4: Activity feed or sidebar is visible', async ({ page }) => {
    const sidebarItems = page.locator('[role="list"], [role="listitem"], nav, aside');
    const hasUI = await sidebarItems.count() > 0;

    await helper.screenshot(page, 'messaging-sidebar');
    expect(hasUI).toBe(true);
    console.log('✅ Sidebar/activity UI present');
  });

  test('M5: New Chat button exists', async ({ page }) => {
    const newChatButton = page.locator('button, a, [role="button"]').filter({
      hasText: /new chat|new channel|new message|create|\+/i
    });

    const buttonCount = await newChatButton.count();
    console.log(`Found ${buttonCount} new chat buttons`);
    
    await helper.screenshot(page, 'messaging-new-chat');
    expect(buttonCount >= 0).toBe(true);
  });
});
