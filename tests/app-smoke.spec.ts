import { test, expect } from '@playwright/test'

test.describe('Communitas app smoke (offline-first)', () => {
  test('identity, org create, tab sweep, snapshots', async ({ page }) => {
    // 1) Navigate
    await page.goto('/')
    await expect(page.locator('body')).toBeVisible()

    // 2) Identity: try Sign In -> Create Identity path (best-effort)
    const signIn = page.getByRole('button', { name: /Sign In/i })
    if (await signIn.isVisible().catch(() => false)) {
      await signIn.click()
      // Try to switch to Create Identity within the dialog if visible
      const createIdentityTab = page.getByRole('button', { name: /Create Identity/i })
      if (await createIdentityTab.isVisible().catch(() => false)) {
        await createIdentityTab.click()
      }
      // Fill Display Name if the field exists
      const displayName = page.getByLabel(/Display Name/i)
      if (await displayName.isVisible().catch(() => false)) {
        await displayName.fill(`Test User ${new Date().toISOString().slice(11,19)}`)
      }
      // Submit Create Identity if the button exists
      const createIdentityBtn = page.getByRole('button', { name: /^Create Identity$/ })
      if (await createIdentityBtn.isVisible().catch(() => false)) {
        await createIdentityBtn.click()
      }
    }

    // 3) Open Identity tab and snapshot
    const identityTab = page.getByRole('button', { name: /Identity/i })
    if (await identityTab.isVisible().catch(() => false)) {
      await identityTab.click()
    }
    await page.screenshot({ path: 'playwright-artifacts/identity.png', fullPage: false })

    // 4) Try Create Organization via visible entry point
    const createOrg = page.getByRole('button', { name: /Create Organization/i })
    if (await createOrg.isVisible().catch(() => false)) {
      await createOrg.click()

      // Expect either a dialog or a panel with Organization Name
      const orgNameByLabel = page.getByLabel(/Organization Name/i)
      if (await orgNameByLabel.isVisible().catch(() => false)) {
        await orgNameByLabel.fill(`Org ${Date.now().toString().slice(-6)}`)
      }
      const descByLabel = page.getByLabel(/Description/i)
      if (await descByLabel.isVisible().catch(() => false)) {
        await descByLabel.fill('Automated test organization')
      }
      const submitOrg = page.getByRole('button', { name: /^Create Organization$/ })
      if (await submitOrg.isVisible().catch(() => false)) {
        await submitOrg.click()
      }
      await page.waitForTimeout(500)
      await page.screenshot({ path: 'playwright-artifacts/org.png', fullPage: false })
    }

    // 5) Tab sweep (best-effort)
    const tabs = ['Organization','Files','Diagnostics','Identity']
    for (const tab of tabs) {
      const btn = page.getByRole('button', { name: new RegExp(`^${tab}$`, 'i') })
      if (await btn.isVisible().catch(() => false)) {
        await btn.click()
        await page.waitForTimeout(200)
        await page.screenshot({ path: `playwright-artifacts/tab-${tab.toLowerCase()}.png`, fullPage: false })
      }
    }
  })
})

