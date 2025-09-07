#!/usr/bin/env node

import puppeteer from 'puppeteer';

async function testP2PConnection() {
  console.log('Testing P2P connection in Communitas app...\n');
  
  const browser = await puppeteer.launch({
    headless: true,
    defaultViewport: { width: 1400, height: 900 }
  });
  
  const page = await browser.newPage();
  
  // Enable console logging
  page.on('console', msg => {
    if (msg.type() === 'log' || msg.type() === 'info') {
      console.log('Browser log:', msg.text());
    } else if (msg.type() === 'error') {
      console.log('Browser error:', msg.text());
    }
  });
  
  console.log('Navigating to http://localhost:1422...');
  await page.goto('http://localhost:1422', { waitUntil: 'networkidle2' });
  
  // Wait for the app to load - look for any content
  await page.waitForSelector('body', { timeout: 5000 });

  // Get the page title
  const title = await page.title();
  console.log('✓ Page loaded:', title);

  // Check if we're in browser fallback mode
  const fallbackText = await page.evaluate(() => {
    const h1 = document.querySelector('h1');
    return h1 ? h1.textContent : '';
  });

  if (fallbackText && fallbackText.includes('Communitas Desktop App Required')) {
    console.log('ℹ️  App is running in browser fallback mode');
    console.log('💡 To test P2P features, run the Tauri desktop app with: npm run tauri dev');
    console.log('📱 The desktop app provides full P2P networking capabilities');
    return;
  }
  
  // Look for the connect button using different selectors
  try {
    // Try to find the button with text
    const connectButton = await page.evaluateHandle(() => {
      const buttons = Array.from(document.querySelectorAll('button'));
      return buttons.find(btn => btn.textContent?.includes('Connect to P2P Network'));
    });

    if (connectButton && await connectButton.evaluate(el => el !== null)) {
      console.log('✓ Found "Connect to P2P Network" button');

      // Click the button
      await page.evaluate(() => {
        const buttons = Array.from(document.querySelectorAll('button'));
        const btn = buttons.find(btn => btn.textContent?.includes('Connect to P2P Network'));
        if (btn) btn.click();
      });
      console.log('✓ Clicked connect button');

      // Wait for connection attempt
      await page.waitForTimeout(5000);

      // Check for any alerts or status changes
      const alerts = await page.$$eval('div[role="alert"]', elements =>
        elements.map(el => el.textContent)
      );

      if (alerts.length > 0) {
        console.log('\n📡 Connection status:');
        alerts.forEach(alert => console.log('  -', alert));
      }

      // Check if peer count changed
      const peerCount = await page.evaluate(() => {
        const h3s = document.querySelectorAll('h3');
        for (const h3 of h3s) {
          const text = h3.textContent || '';
          if (/^\d+$/.test(text)) {
            return parseInt(text);
          }
        }
        return 0;
      });

      if (peerCount > 0) {
        console.log(`\n✅ Successfully connected! Peer count: ${peerCount}`);
      } else {
        console.log('\n⚠️  Connection attempted but no peers detected yet');
        console.log('💡 This is expected in a local test environment without other nodes');
      }
    } else {
      console.log('ℹ️  App may already be connected or button not found');
    }
  } catch (e) {
    console.log('Error during connection test:', e.message);
    console.log('💡 This may be expected if the P2P features require the Tauri backend');
  }
  
  // Take a final screenshot
  await page.screenshot({ path: 'p2p-connection-test.png' });
  console.log('\n📸 Screenshot saved as p2p-connection-test.png');
  
  await browser.close();
  console.log('\n✅ Test completed!');
}

testP2PConnection().catch(console.error);