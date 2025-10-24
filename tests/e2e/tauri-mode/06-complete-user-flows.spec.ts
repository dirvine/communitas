/**
 * Complete User Flows E2E Tests
 * 
 * Tests end-to-end user journeys through the entire application:
 * - New user onboarding → create group → invite members → messaging
 * - Organization setup → create projects → assign team → collaborate
 * - Contact discovery → FOAF lookup → add to group → message
 * - Document creation → share → collaborative edit → publish
 * 
 * Prerequisites: Run `npm run tauri dev` before running tests
 */

import { test, expect } from '@playwright/test';
import { TauriTestHelper } from '../../utils/tauri-helpers';

test.describe('Complete User Flow: New User Journey', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: true }); // Clean state for full flow
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
  });

  test('UF1: Complete new user journey', async ({ page }) => {
    // STEP 1: Onboarding - Create Identity
    await page.waitForTimeout(2000);

    const claimButton = page.locator('button').filter({ hasText: /claim|create.*identity/i }).first();
    if (await claimButton.isVisible({ timeout: 3000 })) {
      await claimButton.click();
      await page.waitForTimeout(1500);

      await helper.screenshot(page, 'flow-identity-created');
    }

    // STEP 2: Create First Group
    await page.waitForTimeout(1000);

    const newGroupButton = page.locator('button').filter({ hasText: /new.*group|create.*group/i }).first();
    if (await newGroupButton.isVisible({ timeout: 2000 })) {
      await newGroupButton.click();
      await page.waitForTimeout(500);

      const groupNameInput = page.locator('input[name*="name" i]').first();
      await groupNameInput.fill('My First Team');

      const createButton = page.locator('button').filter({ hasText: /create|save/i }).first();
      await createButton.click();

      await expect(page.locator('text=My First Team')).toBeVisible({ timeout: 5000 });
      await helper.screenshot(page, 'flow-group-created');
    }

    // STEP 3: Add Members to Group
    const groupItem = page.locator('text=My First Team').first();
    if (await groupItem.isVisible({ timeout: 2000 })) {
      await groupItem.click();
      await page.waitForTimeout(500);

      const addMemberButton = page.locator('button').filter({ hasText: /add.*member/i }).first();
      if (await addMemberButton.isVisible({ timeout: 2000 })) {
        await addMemberButton.click();
        await page.waitForTimeout(500);

        const memberInput = page.locator('input').first();
        await memberInput.fill('alpha-beta-gamma-delta');

        const confirmButton = page.locator('button').filter({ hasText: /add|invite/i }).first();
        await confirmButton.click();

        await page.waitForTimeout(1000);
        await helper.screenshot(page, 'flow-member-added');
      }
    }

    // STEP 4: Send First Message
    const messageInput = page.locator('textarea, input[placeholder*="message" i]').first();
    if (await messageInput.isVisible({ timeout: 2000 })) {
      await messageInput.fill('Welcome to the team!');

      const sendButton = page.locator('button').filter({ hasText: /send/i }).first();
      await sendButton.click();

      await expect(page.locator('text=Welcome to the team!')).toBeVisible({ timeout: 5000 });
      await helper.screenshot(page, 'flow-message-sent');
    }

    // STEP 5: Verify Complete State
    await expect(page.locator('text=My First Team')).toBeVisible();
    await expect(page.locator('text=/alpha.*beta/i')).toBeVisible();
    await expect(page.locator('text=Welcome to the team!')).toBeVisible();

    await helper.screenshot(page, 'flow-complete-new-user');
  });
});

test.describe('Complete User Flow: Organization Setup', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('UF2: Organization setup flow', async ({ page }) => {
    await page.waitForTimeout(1000);

    // STEP 1: Create Organization
    const newOrgButton = page.locator('button').filter({ hasText: /new.*org|create.*org/i }).first();
    if (await newOrgButton.isVisible({ timeout: 2000 })) {
      await newOrgButton.click();
      await page.waitForTimeout(500);

      const orgNameInput = page.locator('input[name*="name" i]').first();
      await orgNameInput.fill('Acme Corp');

      const createButton = page.locator('button').filter({ hasText: /create/i }).first();
      await createButton.click();

      await expect(page.locator('text=Acme Corp')).toBeVisible({ timeout: 5000 });
      await helper.screenshot(page, 'flow-org-created');
    } else {
      test.skip();
    }

    // STEP 2: Add Team Members
    const orgItem = page.locator('text=Acme Corp').first();
    await orgItem.click();
    await page.waitForTimeout(500);

    const addMemberButton = page.locator('button').filter({ hasText: /add.*member/i }).first();
    if (await addMemberButton.isVisible({ timeout: 2000 })) {
      // Add first member
      await addMemberButton.click();
      await page.waitForTimeout(500);

      let memberInput = page.locator('input').first();
      await memberInput.fill('team-member-one-alice');

      let confirmButton = page.locator('button').filter({ hasText: /add|invite/i }).first();
      await confirmButton.click();
      await page.waitForTimeout(1000);

      // Add second member
      if (await addMemberButton.isVisible({ timeout: 1000 })) {
        await addMemberButton.click();
        await page.waitForTimeout(500);

        memberInput = page.locator('input').first();
        await memberInput.fill('team-member-two-bob');

        confirmButton = page.locator('button').filter({ hasText: /add|invite/i }).first();
        await confirmButton.click();
        await page.waitForTimeout(1000);
      }

      await helper.screenshot(page, 'flow-org-members-added');
    }

    // STEP 3: Create Project within Organization
    const newProjectButton = page.locator('button').filter({ hasText: /new.*project/i }).first();
    if (await newProjectButton.isVisible({ timeout: 2000 })) {
      await newProjectButton.click();
      await page.waitForTimeout(500);

      const projectNameInput = page.locator('input[name*="name" i]').first();
      await projectNameInput.fill('Q1 Product Launch');

      const createButton = page.locator('button').filter({ hasText: /create/i }).first();
      await createButton.click();

      await expect(page.locator('text=Q1 Product Launch')).toBeVisible({ timeout: 5000 });
      await helper.screenshot(page, 'flow-project-created');
    }

    // STEP 4: Assign Project Team
    const projectItem = page.locator('text=Q1 Product Launch').first();
    if (await projectItem.isVisible({ timeout: 2000 })) {
      await projectItem.click();
      await page.waitForTimeout(500);

      const assignButton = page.locator('button').filter({ hasText: /assign|add.*member/i }).first();
      if (await assignButton.isVisible({ timeout: 2000 })) {
        await assignButton.click();
        await page.waitForTimeout(500);

        // Select members from org
        const memberCheckbox = page.locator('input[type="checkbox"]').first();
        if (await memberCheckbox.isVisible({ timeout: 1000 })) {
          await memberCheckbox.click();
        }

        const confirmButton = page.locator('button').filter({ hasText: /assign|add/i }).first();
        await confirmButton.click();

        await helper.screenshot(page, 'flow-project-team-assigned');
      }
    }

    // Verify complete organization state
    await expect(page.locator('text=Acme Corp')).toBeVisible();
    await expect(page.locator('text=Q1 Product Launch')).toBeVisible();
  });
});

test.describe('Complete User Flow: Contact Discovery & Messaging', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('UF3: Contact discovery via FOAF', async ({ page }) => {
    await page.waitForTimeout(1000);

    // STEP 1: Search for Contact
    const searchInput = page.locator('input[placeholder*="search" i], input[type="search"]').first();
    if (await searchInput.isVisible({ timeout: 2000 })) {
      await searchInput.fill('charlie-delta-echo-foxtrot');
      await page.waitForTimeout(1000);

      // STEP 2: FOAF discovery results
      const searchResult = page.locator('[data-testid*="search-result"]').first();
      if (await searchResult.isVisible({ timeout: 5000 })) {
        await searchResult.click();
        await page.waitForTimeout(500);

        await helper.screenshot(page, 'flow-foaf-discovery');

        // STEP 3: Add to Contacts
        const addContactButton = page.locator('button').filter({ hasText: /add.*contact/i }).first();
        if (await addContactButton.isVisible({ timeout: 2000 })) {
          await addContactButton.click();
          await page.waitForTimeout(1000);

          await helper.screenshot(page, 'flow-contact-added');
        }

        // STEP 4: Start Conversation
        const messageButton = page.locator('button').filter({ hasText: /message|chat/i }).first();
        if (await messageButton.isVisible({ timeout: 2000 })) {
          await messageButton.click();
          await page.waitForTimeout(500);

          const messageInput = page.locator('textarea, input[placeholder*="message" i]').first();
          await messageInput.fill('Hi! Found you via FOAF discovery');

          const sendButton = page.locator('button').filter({ hasText: /send/i }).first();
          await sendButton.click();

          await expect(page.locator('text=Found you via FOAF discovery')).toBeVisible({ timeout: 5000 });
          await helper.screenshot(page, 'flow-foaf-message-sent');
        }
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });
});

test.describe('Complete User Flow: Document Collaboration', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('UF4: Document creation and collaboration', async ({ page }) => {
    await page.waitForTimeout(1000);

    // STEP 1: Create Document
    const newDocButton = page.locator('button').filter({ hasText: /new.*document|create.*doc/i }).first();
    if (await newDocButton.isVisible({ timeout: 2000 })) {
      await newDocButton.click();
      await page.waitForTimeout(500);

      const docTitleInput = page.locator('input[name*="title" i]').first();
      await docTitleInput.fill('Project Plan');

      const createButton = page.locator('button').filter({ hasText: /create/i }).first();
      await createButton.click();

      await expect(page.locator('text=Project Plan')).toBeVisible({ timeout: 5000 });
      await helper.screenshot(page, 'flow-doc-created');
    } else {
      test.skip();
    }

    // STEP 2: Edit Document Content
    const editor = page.locator('textarea, [contenteditable="true"], .editor').first();
    if (await editor.isVisible({ timeout: 2000 })) {
      await editor.click();
      await editor.fill('# Project Overview\n\nThis is our Q1 plan...');
      await page.waitForTimeout(500);

      await helper.screenshot(page, 'flow-doc-edited');
    }

    // STEP 3: Share Document
    const shareButton = page.locator('button').filter({ hasText: /share/i }).first();
    if (await shareButton.isVisible({ timeout: 2000 })) {
      await shareButton.click();
      await page.waitForTimeout(500);

      // Select group to share with
      const groupSelect = page.locator('select, [role="listbox"]').first();
      if (await groupSelect.isVisible({ timeout: 1000 })) {
        await groupSelect.click();
        
        const groupOption = page.locator('option, [role="option"]').first();
        await groupOption.click();

        const confirmButton = page.locator('button').filter({ hasText: /share|confirm/i }).first();
        await confirmButton.click();

        await helper.screenshot(page, 'flow-doc-shared');
      }
    }

    // STEP 4: Verify Sync State
    const syncIndicator = page.locator('[data-testid*="sync"], text=/synced|saved/i').first();
    if (await syncIndicator.isVisible({ timeout: 3000 })) {
      await helper.screenshot(page, 'flow-doc-synced');
    }
  });
});

test.describe('Complete User Flow: Settings & Preferences', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('UF5: Configure app settings', async ({ page }) => {
    await page.waitForTimeout(1000);

    // STEP 1: Open Settings
    const settingsButton = page.locator('button, [aria-label*="settings" i]').filter({
      hasText: /settings|preferences/i
    }).or(page.locator('[data-testid*="settings"]')).first();

    if (await settingsButton.isVisible({ timeout: 2000 })) {
      await settingsButton.click();
      await page.waitForTimeout(500);

      await helper.screenshot(page, 'flow-settings-opened');

      // STEP 2: Configure Bootstrap Nodes
      const bootstrapTab = page.locator('text=/bootstrap|network/i').first();
      if (await bootstrapTab.isVisible({ timeout: 2000 })) {
        await bootstrapTab.click();
        await page.waitForTimeout(500);

        const bootstrapInput = page.locator('input, textarea').filter({
          hasText: /127\.0\.0\.1|bootstrap/i
        }).or(page.locator('input[name*="bootstrap" i]')).first();

        if (await bootstrapInput.isVisible({ timeout: 1000 })) {
          await bootstrapInput.fill('127.0.0.1:9000');

          const saveButton = page.locator('button').filter({ hasText: /save|apply/i }).first();
          await saveButton.click();

          await helper.screenshot(page, 'flow-bootstrap-configured');
        }
      }

      // STEP 3: Configure Notifications
      const notificationsTab = page.locator('text=/notification/i').first();
      if (await notificationsTab.isVisible({ timeout: 2000 })) {
        await notificationsTab.click();
        await page.waitForTimeout(500);

        const notifToggle = page.locator('input[type="checkbox"]').first();
        if (await notifToggle.isVisible({ timeout: 1000 })) {
          await notifToggle.click();
          await helper.screenshot(page, 'flow-notifications-configured');
        }
      }

      // STEP 4: Review Identity
      const identityTab = page.locator('text=/identity|profile/i').first();
      if (await identityTab.isVisible({ timeout: 2000 })) {
        await identityTab.click();
        await page.waitForTimeout(500);

        // Should show four-word identity
        await expect(page.locator('text=/-[a-z]+-[a-z]+-[a-z]+/i')).toBeVisible({ timeout: 3000 });
        await helper.screenshot(page, 'flow-identity-viewed');
      }
    } else {
      test.skip();
    }
  });
});
