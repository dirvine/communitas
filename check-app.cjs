const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  const page = await browser.newPage();

  // Listen for console messages
  page.on('console', msg => {
    console.log('Browser console:', msg.type(), '-', msg.text());
  });

  // Listen for errors
  page.on('error', err => {
    console.error('Browser error:', err);
  });

  page.on('pageerror', err => {
    console.error('Page error:', err.message);
    if (err.stack) {
      console.error('Stack trace:', err.stack);
    }
  });

  await page.goto('http://localhost:1422/', { waitUntil: 'networkidle0' });

  // Check if React rendered anything
  const rootContent = await page.evaluate(() => {
    const root = document.getElementById('root');
    return {
      hasRoot: !!root,
      innerHTML: root ? root.innerHTML : null,
      childCount: root ? root.children.length : 0
    };
  });

  console.log('\nRoot element status:', rootContent);

  // Check for any error messages
  const errorElement = await page.evaluate(() => {
    const errors = Array.from(document.querySelectorAll('[class*="error"]'));
    return errors.map(e => ({ text: e.textContent, className: e.className }));
  });

  if (errorElement.length > 0) {
    console.log('\nError elements found:', errorElement);
  }

  await browser.close();
})();