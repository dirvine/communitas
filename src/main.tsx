import React from 'react'
import ReactDOM from 'react-dom/client'
import './index.css'

// Very simple test component
const TestComponent: React.FC = () => {
  console.log('✅ React component function executed');
  return React.createElement('div', {
    style: { padding: '20px', backgroundColor: '#f0f0f0' }
  }, [
    React.createElement('h1', { key: 'title' }, '🚀 React is Working!'),
    React.createElement('p', { key: 'desc' }, 'This confirms that React is loading and rendering correctly.'),
    React.createElement('button', {
      key: 'button',
      onClick: () => alert('Button works!')
    }, 'Test Button'),
    React.createElement('p', { key: 'time' }, `Current time: ${new Date().toLocaleTimeString()}`)
  ]);
};

// Test React mounting with error handling
console.log('🔍 Starting React mounting test...');

const rootElement = document.getElementById('root');
if (rootElement) {
  console.log('✅ Root element found');
  try {
    const root = ReactDOM.createRoot(rootElement);
    console.log('✅ ReactDOM.createRoot succeeded');

    root.render(
      React.createElement(React.StrictMode, null,
        React.createElement(TestComponent, null)
      )
    );
    console.log('✅ React component rendered successfully');
  } catch (error) {
    console.error('❌ React mounting failed:', error);
    // Fallback: render error message
    rootElement.innerHTML = `
      <div style="padding: 20px; background: #ffebee; border: 1px solid #f44336; border-radius: 4px;">
        <h2>❌ React Error</h2>
        <p>React failed to mount. Check console for details.</p>
        <pre style="background: #f5f5f5; padding: 10px; border-radius: 4px;">${error}</pre>
      </div>
    `;
  }
} else {
  console.error('❌ Root element not found!');
  document.body.innerHTML = `
    <div style="padding: 20px; background: #ffebee; border: 1px solid #f44336; border-radius: 4px;">
      <h2>❌ Root Element Missing</h2>
      <p>The root element (#root) was not found in the DOM.</p>
    </div>
  `;
}
