import { expect } from 'chai';

describe('Navigation + Auth shell', () => {
  it('renders the login screen with sign-in controls', async () => {
    await browser.url('tauri://localhost');
    const heading = await $('h1=Welcome back');
    await heading.waitForExist({ timeout: 10000 });
    const signInButton = await $('button=Sign in');
    expect(await signInButton.isExisting()).to.equal(true);
  });
});
