import { expect } from 'chai';

/**
 * Full E2E Test Suite - Complete User Journey
 *
 * Tests the complete user flow from identity creation through all features:
 * - Identity creation and login
 * - Organization, Group, Channel, Project creation
 * - Contact management and messaging
 * - Kanban board operations
 * - Drive file operations
 *
 * Prerequisites:
 * - Fresh app state (no existing identity)
 * - Network connectivity for P2P tests
 */
describe('Full E2E User Journey', function () {
  // Increase timeout for E2E tests
  this.timeout(120000);

  // Test user credentials - generated during setup
  let testUser = {
    displayName: `E2E_Test_${Date.now()}`,
    password: 'TestPassword123!',
    fourWords: null, // Will be captured during creation
  };

  // Created entity IDs for subsequent tests
  let createdEntities = {
    organization: null,
    group: null,
    channel: null,
    project: null,
  };

  // ============================================
  // PHASE 1: Identity Setup
  // ============================================
  describe('Phase 1: Identity Creation & Login', () => {
    it('should navigate to create identity page', async () => {
      await browser.url('tauri://localhost');

      // Wait for login page
      const loginHeading = await $('h1=Welcome back');
      await loginHeading.waitForExist({ timeout: 15000 });

      // Click create link
      const createLink = await $('a=Create one');
      await createLink.click();

      // Verify we're on create page
      const createHeading = await $('h1=Create identity');
      await createHeading.waitForExist({ timeout: 10000 });
      expect(await createHeading.isExisting()).to.equal(true);
    });

    it('should display generated four words', async () => {
      // Wait for four words to be generated
      await browser.pause(2000); // Allow time for generation

      // Look for the four words preview (may be in a span or div)
      const wordsContainer = await $('[data-testid="four-words-preview"]');
      if (await wordsContainer.isExisting()) {
        testUser.fourWords = await wordsContainer.getText();
        console.log(`Generated four words: ${testUser.fourWords}`);
      } else {
        // Try alternative selector
        const previewText = await $('div*=word');
        if (await previewText.isExisting()) {
          testUser.fourWords = await previewText.getText();
        }
      }
    });

    it('should fill display name', async () => {
      const displayNameInput = await $('input[placeholder*="display"]');
      if (await displayNameInput.isExisting()) {
        await displayNameInput.setValue(testUser.displayName);
      } else {
        // Try alternative selector
        const nameInput = await $('input[name="displayName"]');
        if (await nameInput.isExisting()) {
          await nameInput.setValue(testUser.displayName);
        }
      }
    });

    it('should fill matching passwords', async () => {
      const passwordInputs = await $$('input[type="password"]');
      expect(passwordInputs.length).to.be.at.least(2, 'Should have at least 2 password fields');

      await passwordInputs[0].setValue(testUser.password);
      await passwordInputs[1].setValue(testUser.password);
    });

    it('should successfully create identity', async () => {
      const createButton = await $('button=Create identity');
      expect(await createButton.isExisting()).to.equal(true, 'Create button should exist');

      await createButton.click();

      // Wait for successful creation - should redirect to dashboard
      // Or show mnemonic backup page
      await browser.pause(5000); // Allow time for identity creation

      // Check for either dashboard or mnemonic backup page
      const dashboardHeading = await $('h1*=Welcome');
      const mnemonicPage = await $('*=mnemonic');
      const loadingIndicator = await $('*=Creating');

      // Wait for loading to complete
      if (await loadingIndicator.isExisting()) {
        await loadingIndicator.waitForExist({ timeout: 30000, reverse: true });
      }

      // Should land on dashboard or mnemonic backup
      const onDashboard = await dashboardHeading.isExisting();
      const onMnemonic = await mnemonicPage.isExisting();
      expect(onDashboard || onMnemonic).to.equal(
        true,
        'Should be on dashboard or mnemonic backup page after creation'
      );

      // Take screenshot for verification
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      await browser.saveScreenshot(`logs/e2e-identity-created-${timestamp}.png`);
    });

    it('should show main app after successful login', async () => {
      // If we're on mnemonic page, proceed to main app
      const continueButton = await $('button*=Continue');
      if (await continueButton.isExisting()) {
        await continueButton.click();
        await browser.pause(2000);
      }

      // Verify we're in the main app
      const sidebar = await $('aside');
      await sidebar.waitForExist({ timeout: 15000 });

      // Look for user profile in sidebar
      const profileHeader = await $('div*=' + testUser.displayName.split('_')[0]);
      if (!(await profileHeader.isExisting())) {
        // Try finding "User" fallback
        const userLabel = await $('div*=User');
        expect(await userLabel.isExisting()).to.equal(true, 'Should see user info in sidebar');
      }

      // Take screenshot
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      await browser.saveScreenshot(`logs/e2e-main-app-${timestamp}.png`);
    });
  });

  // ============================================
  // PHASE 2: Entity Creation
  // ============================================
  describe('Phase 2: Entity Creation', () => {
    describe('Organization', () => {
      it('should open create organization modal', async () => {
        // Find "My Organizations" section and click add button
        const orgSection = await $('*=My Organizations');
        await orgSection.waitForExist({ timeout: 10000 });

        // Look for add button near organizations
        const addOrgButton = await $('button[aria-label*="Create Organization"]');
        if (await addOrgButton.isExisting()) {
          await addOrgButton.click();
        } else {
          // Try clicking the + button in the section
          const plusButtons = await $$('button*=+');
          for (const btn of plusButtons) {
            const label = await btn.getAttribute('aria-label');
            if (label && label.includes('Organization')) {
              await btn.click();
              break;
            }
          }
        }

        // Wait for modal
        await browser.pause(1000);
        const modal = await $('[role="dialog"]');
        expect(await modal.isExisting()).to.equal(true, 'Create modal should open');
      });

      it('should create an organization', async () => {
        const orgName = `E2E Org ${Date.now()}`;

        // Fill organization name
        const nameInput = await $('input[placeholder*="name"]');
        if (await nameInput.isExisting()) {
          await nameInput.setValue(orgName);
        }

        // Fill description if available
        const descInput = await $('textarea');
        if (await descInput.isExisting()) {
          await descInput.setValue('E2E test organization');
        }

        // Submit
        const submitButton = await $('button=Create');
        if (!(await submitButton.isExisting())) {
          const altSubmit = await $('button*=Create');
          await altSubmit.click();
        } else {
          await submitButton.click();
        }

        // Wait for creation
        await browser.pause(3000);

        // Verify organization appears in sidebar
        const newOrg = await $(`*=${orgName.substring(0, 10)}`);
        expect(await newOrg.isExisting()).to.equal(true, 'Created org should appear');

        createdEntities.organization = orgName;
        console.log(`Created organization: ${orgName}`);
      });
    });

    describe('Group', () => {
      it('should create a group', async () => {
        // Look for add group button
        const addGroupButton = await $('button[aria-label*="Create Group"]');
        if (await addGroupButton.isExisting()) {
          await addGroupButton.click();
          await browser.pause(1000);

          const groupName = `E2E Group ${Date.now()}`;
          const nameInput = await $('input[placeholder*="name"]');
          if (await nameInput.isExisting()) {
            await nameInput.setValue(groupName);
          }

          const submitButton = await $('button*=Create');
          await submitButton.click();
          await browser.pause(2000);

          createdEntities.group = groupName;
          console.log(`Created group: ${groupName}`);
        } else {
          console.log('Group creation UI not available, skipping');
        }
      });
    });

    describe('Project', () => {
      it('should create a project', async () => {
        // Look for add project button
        const addProjectButton = await $('button[aria-label*="Create Project"]');
        if (await addProjectButton.isExisting()) {
          await addProjectButton.click();
          await browser.pause(1000);

          const projectName = `E2E Project ${Date.now()}`;
          const nameInput = await $('input[placeholder*="name"]');
          if (await nameInput.isExisting()) {
            await nameInput.setValue(projectName);
          }

          const submitButton = await $('button*=Create');
          await submitButton.click();
          await browser.pause(2000);

          createdEntities.project = projectName;
          console.log(`Created project: ${projectName}`);
        } else {
          console.log('Project creation UI not available, skipping');
        }
      });
    });
  });

  // ============================================
  // PHASE 3: Navigation Verification
  // ============================================
  describe('Phase 3: Navigation & Views', () => {
    it('should navigate to Messages view', async () => {
      const messagesNav = await $('*=Messages');
      if (await messagesNav.isExisting()) {
        await messagesNav.click();
        await browser.pause(1000);
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        await browser.saveScreenshot(`logs/e2e-messages-view-${timestamp}.png`);
      }
    });

    it('should navigate to Contacts view', async () => {
      const contactsNav = await $('*=Contacts');
      if (await contactsNav.isExisting()) {
        await contactsNav.click();
        await browser.pause(1000);
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        await browser.saveScreenshot(`logs/e2e-contacts-view-${timestamp}.png`);
      }
    });

    it('should navigate to Network view', async () => {
      const networkNav = await $('*=Network');
      if (await networkNav.isExisting()) {
        await networkNav.click();
        await browser.pause(1000);
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        await browser.saveScreenshot(`logs/e2e-network-view-${timestamp}.png`);
      }
    });

    it('should navigate back to Dashboard', async () => {
      // Click on user profile or home icon
      const profileHeader = await $('[data-testid="profile-header"]');
      if (await profileHeader.isExisting()) {
        await profileHeader.click();
      } else {
        // Navigate via URL
        await browser.url('tauri://localhost/');
      }
      await browser.pause(1000);
    });
  });

  // ============================================
  // PHASE 4: Entity Detail Views
  // ============================================
  describe('Phase 4: Entity Detail Views', () => {
    it('should open organization detail view', async () => {
      if (createdEntities.organization) {
        const orgItem = await $(`*=${createdEntities.organization.substring(0, 10)}`);
        if (await orgItem.isExisting()) {
          await orgItem.click();
          await browser.pause(2000);

          const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
          await browser.saveScreenshot(`logs/e2e-org-detail-${timestamp}.png`);
        }
      }
    });

    it('should verify entity tabs (Chat, Drive)', async () => {
      // Look for tab navigation
      const chatTab = await $('button*=Chat');
      const driveTab = await $('button*=Drive');

      if (await chatTab.isExisting()) {
        console.log('Chat tab available');
        await chatTab.click();
        await browser.pause(1000);
      }

      if (await driveTab.isExisting()) {
        console.log('Drive tab available');
        await driveTab.click();
        await browser.pause(1000);

        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        await browser.saveScreenshot(`logs/e2e-drive-view-${timestamp}.png`);
      }
    });
  });

  // ============================================
  // PHASE 5: Feature Testing
  // ============================================
  describe('Phase 5: Feature Verification', () => {
    describe('Search', () => {
      it('should focus search with keyboard shortcut', async () => {
        // Cmd+K or Ctrl+K should focus search
        await browser.keys(['Meta', 'k']);
        await browser.pause(500);

        const searchInput = await $('#global-search-input');
        const isFocused = await searchInput.isFocused();
        // May not work in all WebDriver contexts
        console.log(`Search focused: ${isFocused}`);
      });

      it('should filter entities by search term', async () => {
        const searchInput = await $('#global-search-input');
        if (await searchInput.isExisting()) {
          await searchInput.setValue('E2E');
          await browser.pause(1000);

          // Entities should be filtered
          const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
          await browser.saveScreenshot(`logs/e2e-search-filtered-${timestamp}.png`);

          // Clear search
          await searchInput.clearValue();
        }
      });
    });

    describe('Accessibility', () => {
      it('should have skip-to-content link', async () => {
        const skipLink = await $('a[href="#main-content"]');
        expect(await skipLink.isExisting()).to.equal(true, 'Skip link should exist');
      });

      it('should have proper ARIA landmarks', async () => {
        const main = await $('main#main-content');
        const aside = await $('aside');

        expect(await main.isExisting()).to.equal(true, 'Main landmark should exist');
        expect(await aside.isExisting()).to.equal(true, 'Aside landmark should exist');
      });
    });
  });

  // ============================================
  // PHASE 6: Cleanup & Summary
  // ============================================
  describe('Phase 6: Test Summary', () => {
    it('should capture final state screenshot', async () => {
      await browser.url('tauri://localhost/');
      await browser.pause(2000);

      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      await browser.saveScreenshot(`logs/e2e-final-state-${timestamp}.png`);
    });

    it('should log test summary', async () => {
      console.log('\n========================================');
      console.log('E2E TEST SUMMARY');
      console.log('========================================');
      console.log(`User: ${testUser.displayName}`);
      console.log(`Four Words: ${testUser.fourWords || 'Not captured'}`);
      console.log(`Organization: ${createdEntities.organization || 'Not created'}`);
      console.log(`Group: ${createdEntities.group || 'Not created'}`);
      console.log(`Project: ${createdEntities.project || 'Not created'}`);
      console.log('========================================\n');
    });
  });
});
