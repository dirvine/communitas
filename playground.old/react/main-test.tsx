import React from 'react'
import ReactDOM from 'react-dom/client'

// Very simple test component
const TestComponent: React.FC = () => {
  return (
    <div style={{ padding: '20px', backgroundColor: '#f0f0f0' }}>
      <h1>🚀 React is Working!</h1>
      <p>This confirms that React is loading and rendering correctly.</p>
      <button onClick={() => alert('Button works!')}>
        Test Button
      </button>
      <p>Current time: {new Date().toLocaleTimeString()}</p>
    </div>
  );
};

// Test React mounting
const rootElement = document.getElementById('root');
if (rootElement) {
  console.log('✅ Root element found');
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <React.StrictMode>
      <TestComponent />
    </React.StrictMode>
  );
  console.log('✅ React component rendered');
} else {
  console.error('❌ Root element not found!');
}