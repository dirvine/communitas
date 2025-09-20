import puppeteer from 'puppeteer';

console.log('🔍 Detailed app check...\n');

const browser = await puppeteer.launch({ 
  headless: false,
  args: ['--disable-cache', '--disable-application-cache']
});

const page = await browser.newPage();

// Enable console logging
page.on('console', msg => {
  console.log('Console:', msg.type(), '-', msg.text());
});

page.on('pageerror', error => {
  console.log('Page Error:', error.message);
});

// Navigate
console.log('📡 Loading http://localhost:1422...');
await page.goto('http://localhost:1422', { 
  waitUntil: 'domcontentloaded',
  timeout: 10000 
});

// Wait a bit
await new Promise(resolve => setTimeout(resolve, 2000));

// Detailed check
const details = await page.evaluate(() => {
  const root = document.getElementById('root');
  
  // Get all the info we can
  return {
    url: window.location.href,
    title: document.title,
    rootExists: !!root,
    rootId: root?.id,
    rootClassName: root?.className,
    rootChildren: root?.children.length || 0,
    rootInnerHTML: root?.innerHTML?.substring(0, 1000) || 'no innerHTML',
    bodyText: document.body.innerText || 'no text',
    reactRoot: !!document.querySelector('[data-reactroot]'),
    reactFiber: !!window._reactRootContainer || !!window.__REACT_DEVTOOLS_GLOBAL_HOOK__
  };
});

console.log('\n📊 Page Details:');
console.log('URL:', details.url);
console.log('Title:', details.title);
console.log('Root exists:', details.rootExists);
console.log('Root children:', details.rootChildren);
console.log('React detected:', details.reactFiber);
console.log('\nBody text:');
console.log(details.bodyText);
console.log('\nRoot HTML:');
console.log(details.rootInnerHTML);

// Take screenshot
await page.screenshot({ 
  path: 'testnet/current_state.png',
  fullPage: true 
});
console.log('\n📸 Screenshot: testnet/current_state.png');

// Try forcing a reload
console.log('\n🔄 Forcing reload...');
await page.reload({ waitUntil: 'networkidle0' });
await new Promise(resolve => setTimeout(resolve, 2000));

const afterReload = await page.evaluate(() => {
  return document.body.innerText || 'empty';
});

console.log('\nAfter reload:');
console.log(afterReload);

console.log('\n✅ Check complete. Browser will stay open for 5 seconds...');
await new Promise(resolve => setTimeout(resolve, 5000));

await browser.close();
