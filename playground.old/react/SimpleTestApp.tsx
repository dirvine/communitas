import React from 'react'

export default function SimpleTestApp() {
  return (
    <div style={{ 
      padding: '40px', 
      backgroundColor: '#f5f5f5', 
      minHeight: '100vh',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      <h1 style={{ color: '#333', marginBottom: '20px' }}>
        🚀 Communitas P2P Chat & Diagnostics
      </h1>
      
      <div style={{ 
        backgroundColor: 'white', 
        padding: '20px', 
        borderRadius: '8px',
        boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        marginBottom: '20px'
      }}>
        <h2 style={{ color: '#666', fontSize: '18px', marginBottom: '10px' }}>
          System Status
        </h2>
        <p style={{ margin: '5px 0' }}>✅ React: Working</p>
        <p style={{ margin: '5px 0' }}>✅ Tauri: {typeof (window as any).__TAURI__ !== 'undefined' ? 'Connected' : 'Not detected (dev mode)'}</p>
        <p style={{ margin: '5px 0' }}>✅ Development Mode: {import.meta.env.DEV ? 'Yes' : 'No'}</p>
      </div>

      <div style={{ 
        backgroundColor: 'white', 
        padding: '20px', 
        borderRadius: '8px',
        boxShadow: '0 2px 4px rgba(0,0,0,0.1)'
      }}>
        <h2 style={{ color: '#666', fontSize: '18px', marginBottom: '10px' }}>
          Quick Actions
        </h2>
        <button 
          onClick={() => alert('Opening chat interface...')}
          style={{
            padding: '10px 20px',
            marginRight: '10px',
            backgroundColor: '#007bff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer'
          }}
        >
          💬 Open Chat
        </button>
        <button 
          onClick={() => alert('Starting diagnostics...')}
          style={{
            padding: '10px 20px',
            backgroundColor: '#28a745',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer'
          }}
        >
          🔍 Run Diagnostics
        </button>
      </div>
    </div>
  )
}