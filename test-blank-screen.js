const puppeteer = require('puppeteer');

async function analyzeBlankScreen() {
  console.log('🔍 Starting Communitas app analysis...\n');

  const browser = await puppeteer.launch({
    headless: false,
    devtools: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });

  const page = await browser.newPage();

  // Enable console logging
  page.on('console', msg => {
    const type = msg.type();
    const text = msg.text();
    console.log(`[Browser ${type}]: ${text}`);
  });

  // Catch errors
  page.on('pageerror', error => {
    console.error('❌ Page error:', error.message);
  });

  // Monitor failed requests
  page.on('requestfailed', request => {
    console.error('❌ Request failed:', request.url(), '-', request.failure().errorText);
  });

  try {
    console.log('📱 Navigating to http://localhost:1422/');
    await page.goto('http://localhost:1422/', {
      waitUntil: 'networkidle0',
      timeout: 30000
    });

    // Wait a bit for React to mount
    await page.waitForTimeout(3000);

    // Check page title
    const title = await page.title();
    console.log(`\n📄 Page title: "${title}"`);

    // Check for React root
    const hasRoot = await page.evaluate(() => {
      const root = document.getElementById('root');
      return {
        exists: !!root,
        innerHTML: root ? root.innerHTML.substring(0, 200) : null,
        childCount: root ? root.children.length : 0
      };
    });
    console.log('\n🌳 React root element:', hasRoot);

    // Check for any visible text
    const bodyText = await page.evaluate(() => document.body.innerText);
    console.log('\n📝 Visible text on page:', bodyText ? bodyText.substring(0, 500) : 'No text found');

    // Check for React DevTools
    const hasReact = await page.evaluate(() => {
      return !!(window.React || window.__REACT_DEVTOOLS_GLOBAL_HOOK__);
    });
    console.log('\n⚛️ React detected:', hasReact);

    // Check for Tauri API
    const hasTauri = await page.evaluate(() => {
      return typeof window.__TAURI__ !== 'undefined';
    });
    console.log('🦀 Tauri API available:', hasTauri);

    // Get all network errors
    const errors = await page.evaluate(() => {
      const errorElements = Array.from(document.querySelectorAll('.error, [class*="error"]'));
      return errorElements.map(el => el.textContent).filter(text => text);
    });
    if (errors.length > 0) {
      console.log('\n❌ Error messages found:', errors);
    }

    // Check localStorage for any app state
    const localStorage = await page.evaluate(() => {
      const items = {};
      for (let i = 0; i < window.localStorage.length; i++) {
        const key = window.localStorage.key(i);
        items[key] = window.localStorage.getItem(key);
      }
      return items;
    });
    console.log('\n💾 LocalStorage:', Object.keys(localStorage).length > 0 ? localStorage : 'Empty');

    // Check for any modals or dialogs
    const hasDialog = await page.evaluate(() => {
      const selectors = [
        '[role="dialog"]',
        '.MuiDialog-root',
        '[class*="modal"]',
        '[class*="Modal"]',
        '[class*="wizard"]',
        '[class*="Wizard"]'
      ];
      for (const selector of selectors) {
        const element = document.querySelector(selector);
        if (element) {
          return {
            found: true,
            selector: selector,
            visible: window.getComputedStyle(element).display !== 'none',
            text: element.textContent?.substring(0, 100)
          };
        }
      }
      return { found: false };
    });
    console.log('\n🪟 Dialog/Modal status:', hasDialog);

    // Get computed styles of body
    const bodyStyles = await page.evaluate(() => {
      const body = document.body;
      const computed = window.getComputedStyle(body);
      return {
        backgroundColor: computed.backgroundColor,
        color: computed.color,
        display: computed.display,
        visibility: computed.visibility,
        opacity: computed.opacity
      };
    });
    console.log('\n🎨 Body styles:', bodyStyles);

    // Check for any loading indicators
    const hasLoader = await page.evaluate(() => {
      const loaderSelectors = [
        '.loading',
        '[class*="loading"]',
        '[class*="spinner"]',
        '.MuiCircularProgress-root',
        '[class*="progress"]'
      ];
      for (const selector of loaderSelectors) {
        const element = document.querySelector(selector);
        if (element) {
          return {
            found: true,
            selector: selector,
            visible: window.getComputedStyle(element).display !== 'none'
          };
        }
      }
      return { found: false };
    });
    console.log('\n⏳ Loading indicator:', hasLoader);

    // Take a screenshot for visual inspection
    await page.screenshot({ path: 'blank-screen-debug.png', fullPage: true });
    console.log('\n📸 Screenshot saved as blank-screen-debug.png');

    // Get all script tags
    const scripts = await page.evaluate(() => {
      return Array.from(document.querySelectorAll('script')).map(script => ({
        src: script.src || 'inline',
        type: script.type || 'text/javascript',
        hasContent: script.innerHTML.length > 0
      }));
    });
    console.log('\n📜 Script tags:', scripts);

    // Wait for user to inspect
    console.log('\n✅ Analysis complete. Browser will stay open for inspection.');
    console.log('Press Ctrl+C to close.');

    // Keep browser open
    await new Promise(() => {});

  } catch (error) {
    console.error('❌ Error during analysis:', error);
  }
}

analyzeBlankScreen().catch(console.error);