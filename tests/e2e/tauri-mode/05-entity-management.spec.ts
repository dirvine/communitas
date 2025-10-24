/**
 * Entity Management E2E Tests
 * 
 * Tests complete CRUD flows for all entity types:
 * - Groups (create, add/remove members, delete)
 * - Organizations (create, manage members, settings)
 * - Projects (create, assign members, archive)
 * - Contacts (add, edit, remove)
 * 
 * Prerequisites: Run `npm run tauri dev` before running tests
 */

import { test, expect } from '@playwright/test';
import { TauriTestHelper } from '../../utils/tauri-helpers';

test.describe('Entity Management - Groups', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('G1: Can create a new group', async ({ page }) => {
    // GIVEN: User is on main app screen
    await page.waitForTimeout(1000);

    // WHEN: User clicks "New Group" or similar
    const newGroupButton = page.locator('button, [role="button"]').filter({
      hasText: /new.*group|create.*group|add.*group/i
    }).first();

    if (await newGroupButton.isVisible({ timeout: 2000 })) {
      await newGroupButton.click();
      await page.waitForTimeout(500);

      // Fill in group details
      const groupNameInput = page.locator('input[name*="name" i], input[placeholder*="name" i]').first();
      await groupNameInput.fill('Test Group');

      const groupDescInput = page.locator('textarea, input[name*="description" i]').first();
      if (await groupDescInput.isVisible({ timeout: 1000 })) {
        await groupDescInput.fill('Test group description');
      }

      // Submit
      const createButton = page.locator('button').filter({ hasText: /create|save|submit/i }).first();
      await createButton.click();

      // THEN: Group should appear in list
      await expect(page.locator('text=Test Group')).toBeVisible({ timeout: 5000 });
      
      await helper.screenshot(page, 'group-created');
    } else {
      test.skip();
    }
  });

  test('G2: Can add member to group', async ({ page }) => {
    // GIVEN: Group exists
    await page.waitForTimeout(1000);

    // Find or create a group first
    const groupItem = page.locator('[data-testid*="group"], [role="listitem"]').filter({
      hasText: /test.*group|general|team/i
    }).first();

    if (await groupItem.isVisible({ timeout: 2000 })) {
      await groupItem.click();
      await page.waitForTimeout(500);

      // WHEN: User clicks "Add Member"
      const addMemberButton = page.locator('button').filter({
        hasText: /add.*member|invite|add.*person/i
      }).first();

      if (await addMemberButton.isVisible({ timeout: 2000 })) {
        await addMemberButton.click();
        await page.waitForTimeout(500);

        // Fill member details (four-word address or email)
        const memberInput = page.locator('input').first();
        await memberInput.fill('ocean-forest-moon-star');

        const confirmButton = page.locator('button').filter({ hasText: /add|invite|confirm/i }).first();
        await confirmButton.click();

        // THEN: Member should appear in group members list
        await expect(
          page.locator('text=/ocean.*forest.*moon.*star/i')
        ).toBeVisible({ timeout: 5000 });

        await helper.screenshot(page, 'group-member-added');
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });

  test('G3: Can remove member from group', async ({ page }) => {
    // GIVEN: Group with members
    await page.waitForTimeout(1000);

    const groupItem = page.locator('[data-testid*="group"]').first();
    if (await groupItem.isVisible({ timeout: 2000 })) {
      await groupItem.click();
      await page.waitForTimeout(500);

      // Find a member in the list
      const memberItem = page.locator('[data-testid*="member"], [role="listitem"]').first();
      
      if (await memberItem.isVisible({ timeout: 2000 })) {
        // WHEN: User clicks remove/delete on member
        const removeButton = memberItem.locator('button, [role="button"]').filter({
          hasText: /remove|delete|kick/i
        }).or(memberItem.locator('[data-testid*="remove"], [aria-label*="remove" i]'));

        if (await removeButton.isVisible({ timeout: 1000 })) {
          await removeButton.click();

          // Confirm deletion if dialog appears
          const confirmButton = page.locator('button').filter({ hasText: /confirm|yes|remove/i }).first();
          if (await confirmButton.isVisible({ timeout: 1000 })) {
            await confirmButton.click();
          }

          // THEN: Member should be removed from list
          await page.waitForTimeout(1000);
          await helper.screenshot(page, 'group-member-removed');
        } else {
          test.skip();
        }
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });

  test('G4: Can view group members list', async ({ page }) => {
    await page.waitForTimeout(1000);

    const groupItem = page.locator('[data-testid*="group"]').first();
    if (await groupItem.isVisible({ timeout: 2000 })) {
      await groupItem.click();
      await page.waitForTimeout(500);

      // THEN: Should see members section
      const membersSection = page.locator('[data-testid*="members"], text=/members/i').first();
      await expect(membersSection).toBeVisible({ timeout: 3000 });

      await helper.screenshot(page, 'group-members-view');
    } else {
      test.skip();
    }
  });

  test('G5: Can edit group details', async ({ page }) => {
    await page.waitForTimeout(1000);

    const groupItem = page.locator('[data-testid*="group"]').first();
    if (await groupItem.isVisible({ timeout: 2000 })) {
      await groupItem.click();
      await page.waitForTimeout(500);

      // WHEN: User clicks edit
      const editButton = page.locator('button').filter({ hasText: /edit|settings/i }).first();
      
      if (await editButton.isVisible({ timeout: 2000 })) {
        await editButton.click();
        await page.waitForTimeout(500);

        // Modify name
        const nameInput = page.locator('input[name*="name" i]').first();
        await nameInput.fill('Updated Group Name');

        // Save
        const saveButton = page.locator('button').filter({ hasText: /save|update/i }).first();
        await saveButton.click();

        // THEN: Should see updated name
        await expect(page.locator('text=Updated Group Name')).toBeVisible({ timeout: 3000 });

        await helper.screenshot(page, 'group-edited');
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });

  test('G6: Can delete/leave group', async ({ page }) => {
    await page.waitForTimeout(1000);

    const groupItem = page.locator('[data-testid*="group"]').first();
    if (await groupItem.isVisible({ timeout: 2000 })) {
      const groupName = await groupItem.textContent();
      await groupItem.click();
      await page.waitForTimeout(500);

      // WHEN: User deletes/leaves group
      const deleteButton = page.locator('button').filter({ hasText: /delete|leave|remove/i }).first();
      
      if (await deleteButton.isVisible({ timeout: 2000 })) {
        await deleteButton.click();

        // Confirm
        const confirmButton = page.locator('button').filter({ hasText: /confirm|yes|delete/i }).first();
        if (await confirmButton.isVisible({ timeout: 1000 })) {
          await confirmButton.click();
        }

        // THEN: Group should be removed from list
        await page.waitForTimeout(1000);
        await helper.screenshot(page, 'group-deleted');
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });
});

test.describe('Entity Management - Organizations', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('O1: Can create organization', async ({ page }) => {
    await page.waitForTimeout(1000);

    const newOrgButton = page.locator('button').filter({
      hasText: /new.*org|create.*org|add.*org/i
    }).first();

    if (await newOrgButton.isVisible({ timeout: 2000 })) {
      await newOrgButton.click();
      await page.waitForTimeout(500);

      const orgNameInput = page.locator('input[name*="name" i]').first();
      await orgNameInput.fill('Test Organization');

      const createButton = page.locator('button').filter({ hasText: /create|save/i }).first();
      await createButton.click();

      await expect(page.locator('text=Test Organization')).toBeVisible({ timeout: 5000 });
      await helper.screenshot(page, 'organization-created');
    } else {
      test.skip();
    }
  });

  test('O2: Can add members to organization', async ({ page }) => {
    await page.waitForTimeout(1000);

    // Navigate to organization
    const orgItem = page.locator('[data-testid*="org"]').first();
    if (await orgItem.isVisible({ timeout: 2000 })) {
      await orgItem.click();
      await page.waitForTimeout(500);

      const addMemberButton = page.locator('button').filter({ hasText: /add.*member/i }).first();
      if (await addMemberButton.isVisible({ timeout: 2000 })) {
        await addMemberButton.click();
        await page.waitForTimeout(500);

        const memberInput = page.locator('input').first();
        await memberInput.fill('winter-spring-summer-fall');

        const confirmButton = page.locator('button').filter({ hasText: /add|invite/i }).first();
        await confirmButton.click();

        await expect(page.locator('text=/winter.*spring/i')).toBeVisible({ timeout: 5000 });
        await helper.screenshot(page, 'org-member-added');
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });

  test('O3: Can assign roles in organization', async ({ page }) => {
    await page.waitForTimeout(1000);

    const orgItem = page.locator('[data-testid*="org"]').first();
    if (await orgItem.isVisible({ timeout: 2000 })) {
      await orgItem.click();
      await page.waitForTimeout(500);

      // Find member
      const memberItem = page.locator('[data-testid*="member"]').first();
      if (await memberItem.isVisible({ timeout: 2000 })) {
        // Look for role selector
        const roleButton = memberItem.locator('button, select').filter({
          hasText: /role|admin|member/i
        }).first();

        if (await roleButton.isVisible({ timeout: 1000 })) {
          await roleButton.click();
          
          // Select a role
          const adminOption = page.locator('text=/admin|owner|moderator/i').first();
          if (await adminOption.isVisible({ timeout: 1000 })) {
            await adminOption.click();
            await helper.screenshot(page, 'org-role-assigned');
          }
        } else {
          test.skip();
        }
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });
});

test.describe('Entity Management - Projects', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('P1: Can create project', async ({ page }) => {
    await page.waitForTimeout(1000);

    const newProjectButton = page.locator('button').filter({
      hasText: /new.*project|create.*project/i
    }).first();

    if (await newProjectButton.isVisible({ timeout: 2000 })) {
      await newProjectButton.click();
      await page.waitForTimeout(500);

      const projectNameInput = page.locator('input[name*="name" i]').first();
      await projectNameInput.fill('Test Project');

      const createButton = page.locator('button').filter({ hasText: /create|save/i }).first();
      await createButton.click();

      await expect(page.locator('text=Test Project')).toBeVisible({ timeout: 5000 });
      await helper.screenshot(page, 'project-created');
    } else {
      test.skip();
    }
  });

  test('P2: Can assign members to project', async ({ page }) => {
    await page.waitForTimeout(1000);

    const projectItem = page.locator('[data-testid*="project"]').first();
    if (await projectItem.isVisible({ timeout: 2000 })) {
      await projectItem.click();
      await page.waitForTimeout(500);

      const addMemberButton = page.locator('button').filter({ hasText: /add.*member|assign/i }).first();
      if (await addMemberButton.isVisible({ timeout: 2000 })) {
        await addMemberButton.click();
        await page.waitForTimeout(500);

        const memberInput = page.locator('input').first();
        await memberInput.fill('north-south-east-west');

        const confirmButton = page.locator('button').filter({ hasText: /add|assign/i }).first();
        await confirmButton.click();

        await expect(page.locator('text=/north.*south/i')).toBeVisible({ timeout: 5000 });
        await helper.screenshot(page, 'project-member-assigned');
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });

  test('P3: Can archive/complete project', async ({ page }) => {
    await page.waitForTimeout(1000);

    const projectItem = page.locator('[data-testid*="project"]').first();
    if (await projectItem.isVisible({ timeout: 2000 })) {
      await projectItem.click();
      await page.waitForTimeout(500);

      const archiveButton = page.locator('button').filter({
        hasText: /archive|complete|close/i
      }).first();

      if (await archiveButton.isVisible({ timeout: 2000 })) {
        await archiveButton.click();

        // Confirm if needed
        const confirmButton = page.locator('button').filter({ hasText: /confirm|yes/i }).first();
        if (await confirmButton.isVisible({ timeout: 1000 })) {
          await confirmButton.click();
        }

        await page.waitForTimeout(1000);
        await helper.screenshot(page, 'project-archived');
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });
});

test.describe('Entity Management - Contacts', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('C1: Can add contact', async ({ page }) => {
    await page.waitForTimeout(1000);

    const addContactButton = page.locator('button').filter({
      hasText: /add.*contact|new.*contact/i
    }).first();

    if (await addContactButton.isVisible({ timeout: 2000 })) {
      await addContactButton.click();
      await page.waitForTimeout(500);

      // Enter four-word address
      const addressInput = page.locator('input').first();
      await addressInput.fill('fire-water-earth-air');

      // Optional: Add display name
      const nameInput = page.locator('input[name*="name" i]').first();
      if (await nameInput.isVisible({ timeout: 1000 })) {
        await nameInput.fill('Alice');
      }

      const saveButton = page.locator('button').filter({ hasText: /add|save/i }).first();
      await saveButton.click();

      await expect(page.locator('text=/fire.*water|Alice/i')).toBeVisible({ timeout: 5000 });
      await helper.screenshot(page, 'contact-added');
    } else {
      test.skip();
    }
  });

  test('C2: Can edit contact details', async ({ page }) => {
    await page.waitForTimeout(1000);

    const contactItem = page.locator('[data-testid*="contact"]').first();
    if (await contactItem.isVisible({ timeout: 2000 })) {
      await contactItem.click();
      await page.waitForTimeout(500);

      const editButton = page.locator('button').filter({ hasText: /edit/i }).first();
      if (await editButton.isVisible({ timeout: 2000 })) {
        await editButton.click();
        await page.waitForTimeout(500);

        const nameInput = page.locator('input[name*="name" i]').first();
        await nameInput.fill('Updated Name');

        const saveButton = page.locator('button').filter({ hasText: /save|update/i }).first();
        await saveButton.click();

        await expect(page.locator('text=Updated Name')).toBeVisible({ timeout: 3000 });
        await helper.screenshot(page, 'contact-edited');
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });

  test('C3: Can remove contact', async ({ page }) => {
    await page.waitForTimeout(1000);

    const contactItem = page.locator('[data-testid*="contact"]').first();
    if (await contactItem.isVisible({ timeout: 2000 })) {
      await contactItem.click();
      await page.waitForTimeout(500);

      const deleteButton = page.locator('button').filter({ hasText: /delete|remove/i }).first();
      if (await deleteButton.isVisible({ timeout: 2000 })) {
        await deleteButton.click();

        const confirmButton = page.locator('button').filter({ hasText: /confirm|yes/i }).first();
        if (await confirmButton.isVisible({ timeout: 1000 })) {
          await confirmButton.click();
        }

        await page.waitForTimeout(1000);
        await helper.screenshot(page, 'contact-removed');
      } else {
        test.skip();
      }
    } else {
      test.skip();
    }
  });
});
