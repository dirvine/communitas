// Ultra-simple React test - no imports, just vanilla JS
const root = document.getElementById('root');
if (root) {
  root.innerHTML = `
    <div style="padding: 20px; background: #f0f0f0; font-family: Arial;">
      <h1>🚀 Vanilla JS Test</h1>
      <p>This tests if JavaScript can modify the DOM without React.</p>
      <button onclick="alert('Vanilla JS works!')">Test Button</button>
      <p>Time: <span id="time"></span></p>
    </div>
  `;

  // Update time every second
  setInterval(() => {
    const timeEl = document.getElementById('time');
    if (timeEl) {
      timeEl.textContent = new Date().toLocaleTimeString();
    }
  }, 1000);

  console.log('✅ Vanilla JS test loaded successfully');
} else {
  console.error('❌ Root element not found');
}