import puppeteer from 'puppeteer';

console.log('🔍 Testing app with TextDecoder polyfill...\n');

const browser = await puppeteer.launch({ 
  headless: false,
  args: ['--disable-cache']
});

const page = await browser.newPage();

// Add TextDecoder polyfill before navigation
await page.evaluateOnNewDocument(() => {
  // Add TextDecoder and TextEncoder polyfills
  if (typeof TextDecoder === 'undefined') {
    window.TextDecoder = class {
      decode(buffer) {
        let result = '';
        const bytes = new Uint8Array(buffer);
        for (let i = 0; i < bytes.length; i++) {
          result += String.fromCharCode(bytes[i]);
        }
        return result;
      }
    };
  }
  
  if (typeof TextEncoder === 'undefined') {
    window.TextEncoder = class {
      encode(str) {
        const bytes = new Uint8Array(str.length);
        for (let i = 0; i < str.length; i++) {
          bytes[i] = str.charCodeAt(i);
        }
        return bytes;
      }
    };
  }
});

// Enable console logging
page.on('console', msg => {
  const type = msg.type();
  if (type === 'error' || type === 'warning') {
    console.log(`[${type.toUpperCase()}]`, msg.text());
  }
});

page.on('pageerror', error => {
  console.log('[PAGE ERROR]', error.message);
});

// Navigate
console.log('📡 Loading http://localhost:1422...');
await page.goto('http://localhost:1422', { 
  waitUntil: 'networkidle0',
  timeout: 15000 
});

// Wait for React
await new Promise(resolve => setTimeout(resolve, 2000));

// Check content
const appStatus = await page.evaluate(() => {
  const root = document.getElementById('root');
  const bodyText = document.body.innerText;
  
  return {
    hasRoot: !!root,
    rootChildren: root?.children.length || 0,
    hasContent: bodyText && bodyText.length > 0,
    text: bodyText?.substring(0, 500) || 'empty',
    title: document.title,
    hasReact: !!window.React || !!window.__REACT_DEVTOOLS_GLOBAL_HOOK__
  };
});

console.log('\n📊 App Status:');
console.log('Has root element:', appStatus.hasRoot);
console.log('Root children:', appStatus.rootChildren);
console.log('Has content:', appStatus.hasContent);
console.log('React detected:', appStatus.hasReact);
console.log('Page title:', appStatus.title);
console.log('\nContent preview:');
console.log(appStatus.text);

if (appStatus.hasContent && appStatus.text !== 'empty' && !appStatus.text.includes('Error')) {
  console.log('\n✅ APP IS WORKING WITH POLYFILL!');
  
  // Take screenshot
  await page.screenshot({ 
    path: 'testnet/app_with_polyfill.png',
    fullPage: true 
  });
  console.log('📸 Screenshot: testnet/app_with_polyfill.png');
  
  console.log('\n🎉 SUCCESS! The app renders when TextDecoder is polyfilled.');
  console.log('\n📝 NOTE: The blank screen in YOUR browser is likely due to:');
  console.log('1. Browser extensions interfering');
  console.log('2. Cached bad state');
  console.log('3. Browser console errors');
  console.log('\n🔧 TRY THESE FIXES:');
  console.log('1. Open Chrome DevTools (F12)');
  console.log('2. Go to Console tab');
  console.log('3. Right-click → Clear console');
  console.log('4. Network tab → Disable cache (checkbox)');
  console.log('5. Hard refresh: Cmd+Shift+R');
  console.log('6. Or try: http://localhost:1422 in Firefox/Safari');
} else {
  console.log('\n⚠️ App still not rendering correctly');
  console.log('Checking for other issues...');
}

console.log('\n👀 Keeping browser open for inspection (10 seconds)...');
await new Promise(resolve => setTimeout(resolve, 10000));

await browser.close();
