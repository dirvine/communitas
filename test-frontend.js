#!/usr/bin/env node

import puppeteer from 'puppeteer';

async function testCommunitas() {
  console.log('Launching browser to test Communitas frontend...');
  
  const browser = await puppeteer.launch({
    headless: false, // Show the browser
    defaultViewport: { width: 1400, height: 900 }
  });
  
  const page = await browser.newPage();
  
  console.log('Navigating to http://localhost:1420...');
  await page.goto('http://localhost:1420', { waitUntil: 'networkidle2' });
  
  // Wait for the main app to load
  await page.waitForSelector('h6', { timeout: 5000 });
  
  // Get the page title
  const title = await page.title();
  console.log('Page title:', title);
  
  // Check if the main heading is present
  const heading = await page.$eval('h6', el => el.textContent);
  console.log('App heading:', heading);
  
  // Check for P2P connect button
  try {
    const connectButton = await page.waitForSelector('button:has-text("Connect to P2P Network")', { timeout: 3000 });
    if (connectButton) {
      console.log('✓ Found "Connect to P2P Network" button');
      console.log('Clicking connect button to join the 100-node network...');
      await connectButton.click();
      
      // Wait for connection
      await page.waitForTimeout(3000);
      
      // Check for success message
      const successAlert = await page.$('div[role="alert"]');
      if (successAlert) {
        const alertText = await successAlert.evaluate(el => el.textContent);
        console.log('Connection status:', alertText);
      }
    }
  } catch (e) {
    console.log('P2P connect button not found or already connected');
  }
  
  // Check for network status
  try {
    await page.waitForSelector('[data-testid="network-status"]', { timeout: 2000 });
    const networkStatus = await page.$eval('[data-testid="network-status"]', el => el.textContent);
    console.log('Network status:', networkStatus);
  } catch (e) {
    console.log('Network status element not found');
  }
  
  // Click through tabs to verify they're working
  const tabs = [
    { index: 1, name: 'Organization' },
    { index: 2, name: 'Messages' },
    { index: 3, name: 'Files' },
    { index: 4, name: 'Documents' },
    { index: 5, name: 'Network' },
    { index: 6, name: 'Storage' },
    { index: 7, name: 'Diagnostics' },
    { index: 8, name: 'Calling' },
    { index: 9, name: 'Website' },
    { index: 10, name: 'Identity' }
  ];
  
  console.log('\nTesting navigation tabs...');
  for (const tab of tabs) {
    try {
      // Try to find and click the tab button
      const tabButton = await page.$(`button:has-text("${tab.name}")`);
      if (tabButton) {
        await tabButton.click();
        console.log(`✓ ${tab.name} tab clicked`);
        await page.waitForTimeout(500);
      } else {
        // Fallback: try by index
        const buttons = await page.$$('button');
        if (buttons[tab.index]) {
          await buttons[tab.index].click();
          console.log(`✓ ${tab.name} tab clicked (by index)`);
          await page.waitForTimeout(500);
        }
      }
    } catch (e) {
      console.log(`✗ Could not click ${tab.name} tab`);
    }
  }
  
  // Take a screenshot
  console.log('\nTaking screenshot...');
  await page.screenshot({ path: 'communitas-screenshot.png' });
  console.log('Screenshot saved as communitas-screenshot.png');
  
  // Check console for errors
  page.on('console', msg => {
    if (msg.type() === 'error') {
      console.log('Console error:', msg.text());
    }
  });
  
  console.log('\n✅ Frontend test completed!');
  console.log('The browser will remain open for manual inspection.');
  console.log('Press Ctrl+C to close.');
  
  // Keep browser open for manual inspection
  await new Promise(() => {});
}

testCommunitas().catch(console.error);