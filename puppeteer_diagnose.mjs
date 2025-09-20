import puppeteer from 'puppeteer';
import fs from 'fs';

console.log('🔍 Comprehensive Browser Diagnosis');
console.log('===================================\n');

const browser = await puppeteer.launch({ 
  headless: false,  // Show browser window
  args: ['--disable-cache', '--disable-application-cache', '--incognito']
});

const page = await browser.newPage();

// Clear all caches
await page.setCacheEnabled(false);

// Set viewport
await page.setViewport({ width: 1280, height: 720 });

// Navigate with network monitoring
console.log('📡 Navigating to http://localhost:1422...');
await page.goto('http://localhost:1422', { 
  waitUntil: 'networkidle0',
  timeout: 30000 
});

// Wait a bit for React
await new Promise(resolve => setTimeout(resolve, 2000));

// Take screenshot
await page.screenshot({ 
  path: 'testnet/app_screenshot.png',
  fullPage: true 
});
console.log('📸 Screenshot saved to testnet/app_screenshot.png');

// Check what's visible
const content = await page.evaluate(() => {
  const root = document.getElementById('root');
  const body = document.body;
  
  // Check for any opacity or visibility issues
  function checkVisibility(el) {
    if (!el) return 'element null';
    const style = getComputedStyle(el);
    return {
      display: style.display,
      visibility: style.visibility,
      opacity: style.opacity,
      position: style.position,
      zIndex: style.zIndex,
      overflow: style.overflow,
      width: el.offsetWidth,
      height: el.offsetHeight
    };
  }
  
  return {
    rootExists: !!root,
    rootHTML: root ? root.innerHTML.substring(0, 500) : 'No root element',
    rootChildren: root ? root.children.length : 0,
    bodyHTML: body.innerHTML.substring(0, 500),
    documentTitle: document.title,
    reactDevTools: !!(window.React || window.__REACT_DEVTOOLS_GLOBAL_HOOK__),
    rootStyles: checkVisibility(root),
    bodyStyles: checkVisibility(body),
    allText: document.body.innerText
  };
});

console.log('\n📋 Page Analysis:');
console.log('Root element exists:', content.rootExists);
console.log('Root children count:', content.rootChildren);
console.log('Document title:', content.documentTitle);
console.log('React detected:', content.reactDevTools);
console.log('\nRoot Styles:', JSON.stringify(content.rootStyles, null, 2));
console.log('\nBody Styles:', JSON.stringify(content.bodyStyles, null, 2));
console.log('\nVisible Text:', content.allText ? content.allText.substring(0, 200) : 'No text');

// Check for console errors
page.on('console', msg => {
  if (msg.type() === 'error') {
    console.log('❌ Console Error:', msg.text());
  }
});

// Try injecting CSS to force visibility
await page.evaluate(() => {
  const style = document.createElement('style');
  style.textContent = `
    * {
      opacity: 1 !important;
      visibility: visible !important;
    }
    #root {
      display: block !important;
      min-height: 100vh !important;
      background: white !important;
    }
  `;
  document.head.appendChild(style);
});

console.log('\n💉 Injected visibility CSS...');

// Wait and take another screenshot
await new Promise(resolve => setTimeout(resolve, 1000));
await page.screenshot({ 
  path: 'testnet/app_forced_visible.png',
  fullPage: true 
});
console.log('📸 Screenshot with forced visibility: testnet/app_forced_visible.png');

// Create a simple HTML file that definitely works
const testHTML = `<!DOCTYPE html>
<html>
<head>
  <title>Test Page</title>
  <style>
    body { 
      margin: 0; 
      padding: 20px; 
      font-family: system-ui; 
      background: white;
    }
    .container {
      max-width: 800px;
      margin: 0 auto;
    }
    .status { 
      background: #e8f5e9; 
      padding: 15px; 
      border-radius: 8px;
      margin: 20px 0;
    }
    iframe {
      width: 100%;
      height: 600px;
      border: 2px solid #333;
      margin-top: 20px;
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>Communitas App Debug View</h1>
    <div class="status">
      <h2>✅ This page confirms your browser is working!</h2>
      <p>If you can see this, the issue is with the React app at localhost:1422</p>
    </div>
    
    <h3>App Preview (iframe):</h3>
    <iframe src="http://localhost:1422" id="appFrame"></iframe>
    
    <div style="margin-top: 20px">
      <button onclick="checkApp()">Check App in iFrame</button>
      <button onclick="window.open('http://localhost:1422', '_blank')">Open App in New Tab</button>
      <button onclick="location.reload(true)">Hard Refresh</button>
    </div>
    
    <div id="results"></div>
  </div>
  
  <script>
    function checkApp() {
      const iframe = document.getElementById('appFrame');
      const results = document.getElementById('results');
      
      try {
        const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
        const root = iframeDoc.getElementById('root');
        
        if (root && root.children.length > 0) {
          results.innerHTML = '<h3 style="color: green">✅ App is rendering in iframe!</h3>';
        } else {
          results.innerHTML = '<h3 style="color: red">❌ App not rendering in iframe</h3>';
        }
      } catch(e) {
        results.innerHTML = '<h3 style="color: orange">⚠️ Cannot access iframe (CORS)</h3>';
      }
    }
    
    // Auto-check after load
    setTimeout(checkApp, 2000);
  </script>
</body>
</html>`;

fs.writeFileSync('testnet/debug_view.html', testHTML);
console.log('\n📝 Created testnet/debug_view.html - Open this in your browser!');

// Open the debug page
const debugPage = await browser.newPage();
await debugPage.goto(`file://${process.cwd()}/testnet/debug_view.html`);
await new Promise(resolve => setTimeout(resolve, 2000));

console.log('\n✅ Diagnosis complete!');
console.log('\n🔧 IMMEDIATE FIXES TO TRY:');
console.log('=====================================');
console.log('1. 🔄 Hard refresh the page:');
console.log('   Mac: Cmd + Shift + R');
console.log('   Windows/Linux: Ctrl + Shift + F5');
console.log('\n2. 🆕 Open in new incognito window:');
console.log('   Mac: Cmd + Shift + N');
console.log('   Windows/Linux: Ctrl + Shift + N');
console.log('   Then go to: http://localhost:1422');
console.log('\n3. 📂 Open the debug view:');
console.log('   file://' + process.cwd() + '/testnet/debug_view.html');
console.log('\n4. 🧹 Clear all browser data:');
console.log('   Chrome: Settings → Privacy → Clear browsing data');
console.log('   Check all boxes, choose "All time"');
console.log('\n5. 🔥 Try Firefox or Safari instead');

// Keep browser open for 10 seconds
console.log('\n👀 Browser window will stay open for 10 seconds...');
await new Promise(resolve => setTimeout(resolve, 10000));

await browser.close();
