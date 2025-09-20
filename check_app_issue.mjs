import puppeteer from 'puppeteer';

const browser = await puppeteer.launch({ headless: false });
const page = await browser.newPage();

// Add polyfill for TextDecoder if needed
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
});

// Capture console errors
const errors = [];
page.on('console', msg => {
  if (msg.type() === 'error') {
    errors.push(msg.text());
  }
});

page.on('pageerror', err => {
  errors.push(err.message);
});

console.log('Loading http://localhost:1422...');
await page.goto('http://localhost:1422', { waitUntil: 'networkidle0' });

await new Promise(resolve => setTimeout(resolve, 2000));

const status = await page.evaluate(() => {
  return {
    text: document.body.innerText || 'empty',
    hasError: document.body.innerText.includes('Error')
  };
});

console.log('\nPage content:');
console.log(status.text || '(blank)');

if (errors.length > 0) {
  console.log('\nErrors detected:');
  errors.forEach(err => console.log('-', err));
}

if (status.hasError || status.text === 'empty') {
  console.log('\n❌ App not rendering. The main App component has issues.');
  console.log('\nLet me switch back to SimpleApp which was working...');
} else {
  console.log('\n✅ App is working!');
}

await new Promise(resolve => setTimeout(resolve, 3000));
await browser.close();
