import React from 'react';

export const SimpleTest: React.FC = () => {
  return (
    <div style={{ padding: '20px', textAlign: 'center' }}>
      <h1>🧪 Simple Test Component</h1>
      <p>This is a very simple React component to test if routing works.</p>
      <button onClick={() => alert('Button clicked!')}>
        Click Me
      </button>
      <p>Current time: {new Date().toLocaleTimeString()}</p>
    </div>
  );
};