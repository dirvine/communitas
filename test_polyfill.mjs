import puppeteer from 'puppeteer';

console.log('Testing app with TextDecoder polyfill...\n');

const browser = await puppeteer.launch({
  headless: false,
  args: ['--disable-cache']
});

const page = await browser.newPage();

// Add polyfill
await page.evaluateOnNewDocument(() => {
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

// Navigate
console.log('Loading http://localhost:1422...');
await page.goto('http://localhost:1422', {
  waitUntil: 'networkidle0',
  timeout: 15000
});

// Wait
await new Promise(resolve => setTimeout(resolve, 2000));

// Check
const status = await page.evaluate(() => {
  const root = document.getElementById('root');
  const text = document.body.innerText || '';
  return {
    hasRoot: !!root,
    children: root?.children.length || 0,
    hasText: text.length > 0,
    text: text.substring(0, 500)
  };
});

console.log('\nApp Status:');
console.log('Has root:', status.hasRoot);
console.log('Children:', status.children);
console.log('Has text:', status.hasText);
console.log('\nContent:');
console.log(status.text);

if (status.hasText && !status.text.includes('Error')) {
  console.log('\n✅ APP IS WORKING!');
  await page.screenshot({ path: 'testnet/working_app.png' });
  console.log('Screenshot saved: testnet/working_app.png');
} else {
  console.log('\n❌ App not rendering correctly');
}

console.log('\nKeeping browser open for 10 seconds...');
await new Promise(resolve => setTimeout(resolve, 10000));

await browser.close();
