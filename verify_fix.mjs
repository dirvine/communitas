import puppeteer from 'puppeteer';

console.log('🔍 Verifying the fix...\n');

const browser = await puppeteer.launch({ 
  headless: true,
  args: ['--disable-cache']
});

const page = await browser.newPage();

// Navigate
console.log('📡 Loading http://localhost:1422...');
await page.goto('http://localhost:1422', { 
  waitUntil: 'networkidle0',
  timeout: 10000 
});

// Wait for React
await new Promise(resolve => setTimeout(resolve, 1000));

// Check content
const appStatus = await page.evaluate(() => {
  const root = document.getElementById('root');
  const hasError = document.body.innerText.includes('Failed to render') || 
                   document.body.innerText.includes('ReferenceError');
  const hasContent = root && root.children.length > 0 && !hasError;
  const text = document.body.innerText.substring(0, 200);
  
  return {
    hasContent,
    hasError,
    text,
    title: document.title
  };
});

if (appStatus.hasError) {
  console.log('❌ App still has errors!');
  console.log('Error text:', appStatus.text);
} else if (appStatus.hasContent) {
  console.log('✅ APP IS WORKING!');
  console.log('Title:', appStatus.title);
  console.log('Content preview:', appStatus.text);
  
  // Take success screenshot
  await page.screenshot({ 
    path: 'testnet/app_working.png',
    fullPage: true 
  });
  console.log('\n📸 Screenshot saved: testnet/app_working.png');
  
  console.log('\n🎉 SUCCESS! The app is now rendering correctly!');
  console.log('\n📱 To see it in your browser:');
  console.log('1. Hard refresh: Cmd+Shift+R (Mac) or Ctrl+F5 (Windows)');
  console.log('2. Or open: http://localhost:1422 in a new incognito window');
} else {
  console.log('⚠️ App loaded but no content detected');
}

await browser.close();
