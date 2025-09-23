import React from 'react'
import ReactDOM from 'react-dom/client'

// Minimal app with NO external dependencies
const MinimalApp = () => {
  const [count, setCount] = React.useState(0)
  const [identity, setIdentity] = React.useState<any>(null)
  const hasTauri = React.useMemo(() => typeof (window as any).__TAURI__ !== 'undefined', [])
  
  const createIdentity = () => {
    const fourWords = ['brave', 'new', 'world', 'user']
    setIdentity({
      words: fourWords,
      address: fourWords.join('-'),
      created: new Date().toISOString()
    })
  }
  
  return (
    <div style={{
      fontFamily: '-apple-system, system-ui, sans-serif',
      maxWidth: '800px',
      margin: '0 auto',
      padding: '40px 20px'
    }}>
      <h1 style={{ color: '#2563eb' }}>🎉 Communitas is Working!</h1>
      
      <div style={{
        background: hasTauri ? '#10b981' : '#f59e0b',
        color: 'white',
        padding: '10px 20px',
        borderRadius: '6px',
        marginBottom: '20px'
      }}>
        Tauri Status: {hasTauri ? '✅ Connected' : '⚠️ Browser Mode'}
      </div>
      
      <div style={{
        background: '#f3f4f6',
        padding: '20px',
        borderRadius: '8px',
        marginBottom: '20px'
      }}>
        <h2>Test Counter</h2>
        <p>Count: {count}</p>
        <button 
          onClick={() => setCount(c => c + 1)}
          style={{
            background: '#3b82f6',
            color: 'white',
            border: 'none',
            padding: '8px 16px',
            borderRadius: '4px',
            cursor: 'pointer'
          }}
        >
          Increment
        </button>
      </div>
      
      <div style={{
        background: '#f3f4f6',
        padding: '20px',
        borderRadius: '8px'
      }}>
        <h2>Identity Management</h2>
        {!identity ? (
          <>
            <p>Create your local P2P identity</p>
            <button 
              onClick={createIdentity}
              style={{
                background: '#10b981',
                color: 'white',
                border: 'none',
                padding: '10px 20px',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '16px'
              }}
            >
              Create Identity
            </button>
          </>
        ) : (
          <div style={{ 
            background: 'white', 
            padding: '15px', 
            borderRadius: '6px',
            border: '2px solid #10b981'
          }}>
            <h3>✅ Identity Created!</h3>
            <p><strong>Four-word address:</strong> {identity.address}</p>
            <p><strong>Created:</strong> {identity.created}</p>
            <button 
              onClick={() => setIdentity(null)}
              style={{
                background: '#ef4444',
                color: 'white',
                border: 'none',
                padding: '8px 16px',
                borderRadius: '4px',
                cursor: 'pointer',
                marginTop: '10px'
              }}
            >
              Reset
            </button>
          </div>
        )}
      </div>
      
      <div style={{
        marginTop: '40px',
        padding: '20px',
        background: '#fef3c7',
        borderRadius: '8px',
        fontSize: '14px'
      }}>
        <h3>Debug Info</h3>
        <ul>
          <li>React Version: {React.version}</li>
          <li>URL: {window.location.href}</li>
          <li>Time: {new Date().toLocaleTimeString()}</li>
          <li>Tauri Available: {String(hasTauri)}</li>
        </ul>
      </div>
    </div>
  )
}

// Mount the app
const rootEl = document.getElementById('root')
if (rootEl) {
  console.log('Mounting minimal app...')
  const root = ReactDOM.createRoot(rootEl)
  root.render(<MinimalApp />)
  console.log('App mounted successfully!')
} else {
  console.error('Root element not found!')
  document.body.innerHTML = '<h1 style="color: red;">Root element not found!</h1>'
}