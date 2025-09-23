import React, { useState, useEffect } from 'react'

const TauriTest: React.FC = () => {
  const [tauriInfo, setTauriInfo] = useState<any>({})
  const [testResult, setTestResult] = useState<string>('')
  const [displayName, setDisplayName] = useState('')
  const [identity, setIdentity] = useState<any>(null)

  useEffect(() => {
    // Check for Tauri
    const checkTauri = () => {
      const info = {
        hasTauri: !!(window as any).__TAURI__,
        hasTauriIPC: !!(window as any).__TAURI_IPC__,
        hasTauriCore: !!(window as any).__TAURI__?.core,
        protocol: window.location.protocol,
        userAgent: navigator.userAgent,
        tauriKeys: (window as any).__TAURI__ ? Object.keys((window as any).__TAURI__) : []
      }
      setTauriInfo(info)
      console.log('Tauri info:', info)
    }

    // Check immediately and after a delay
    checkTauri()
    setTimeout(checkTauri, 500)
  }, [])

  const testTauriCommand = async () => {
    setTestResult('Testing...')
    try {
      if ((window as any).__TAURI__?.core) {
        const { invoke } = (window as any).__TAURI__.core
        
        // Try to call a simple command
        try {
          const result = await invoke('core_initialize')
          setTestResult('Success! Core initialized: ' + JSON.stringify(result))
        } catch (err: any) {
          setTestResult('Command failed: ' + err.toString())
        }
      } else {
        setTestResult('Tauri core not available')
      }
    } catch (error: any) {
      setTestResult('Error: ' + error.toString())
    }
  }

  const createIdentity = async () => {
    try {
      if ((window as any).__TAURI__?.core) {
        const { invoke } = (window as any).__TAURI__.core
        
        // Generate four random words
        const words = ['happy', 'sunny', 'blue', 'green', 'swift', 'bright', 'cool', 'warm']
        const fourWords = []
        for (let i = 0; i < 4; i++) {
          fourWords.push(words[Math.floor(Math.random() * words.length)])
        }
        
        try {
          // Try to claim identity with four words
          const result = await invoke('core_claim', { words: fourWords })
          setIdentity({ fourWords: fourWords.join('-'), result })
          setTestResult('Identity created: ' + fourWords.join('-'))
        } catch (err: any) {
          // Fall back to mock
          setIdentity({ 
            fourWords: fourWords.join('-'),
            displayName: displayName || 'Test User',
            mock: true 
          })
          setTestResult('Mock identity: ' + fourWords.join('-'))
        }
      } else {
        // Create mock identity
        const mockId = {
          fourWords: 'test-user-mock-identity',
          displayName: displayName || 'Test User',
          mock: true
        }
        setIdentity(mockId)
        setTestResult('Mock identity created (no Tauri)')
      }
    } catch (error: any) {
      setTestResult('Error creating identity: ' + error.toString())
    }
  }

  const styles = {
    container: {
      padding: '20px',
      fontFamily: 'system-ui, -apple-system, sans-serif',
      backgroundColor: '#1a1a1a',
      color: '#ffffff',
      minHeight: '100vh'
    },
    section: {
      marginBottom: '20px',
      padding: '15px',
      backgroundColor: '#2a2a2a',
      borderRadius: '8px'
    },
    title: {
      color: '#4a9eff',
      marginBottom: '10px'
    },
    info: {
      backgroundColor: '#333',
      padding: '10px',
      borderRadius: '4px',
      marginBottom: '10px',
      fontFamily: 'monospace',
      fontSize: '12px',
      overflowX: 'auto' as const
    },
    button: {
      backgroundColor: '#4a9eff',
      color: 'white',
      border: 'none',
      padding: '10px 20px',
      borderRadius: '4px',
      cursor: 'pointer',
      marginRight: '10px',
      marginBottom: '10px'
    },
    input: {
      padding: '8px',
      borderRadius: '4px',
      border: '1px solid #444',
      backgroundColor: '#333',
      color: 'white',
      marginRight: '10px',
      marginBottom: '10px'
    },
    success: {
      color: '#4ade80'
    },
    error: {
      color: '#f87171'
    }
  }

  return (
    <div style={styles.container}>
      <h1>Communitas - Tauri Test</h1>
      
      <div style={styles.section}>
        <h2 style={styles.title}>Tauri Detection</h2>
        <div style={styles.info}>
          <div>✓ Tauri Available: {tauriInfo.hasTauri ? '✅ YES' : '❌ NO'}</div>
          <div>✓ Tauri IPC: {tauriInfo.hasTauriIPC ? '✅ YES' : '❌ NO'}</div>
          <div>✓ Tauri Core: {tauriInfo.hasTauriCore ? '✅ YES' : '❌ NO'}</div>
          <div>✓ Protocol: {tauriInfo.protocol}</div>
          <div>✓ Tauri APIs: {tauriInfo.tauriKeys?.join(', ') || 'None'}</div>
        </div>
      </div>

      <div style={styles.section}>
        <h2 style={styles.title}>Test Tauri Commands</h2>
        <button style={styles.button} onClick={testTauriCommand}>
          Test Core Initialize
        </button>
        {testResult && (
          <div style={{
            ...styles.info,
            color: testResult.includes('Success') ? styles.success.color : 
                   testResult.includes('Error') ? styles.error.color : 'white'
          }}>
            {testResult}
          </div>
        )}
      </div>

      <div style={styles.section}>
        <h2 style={styles.title}>Identity Management</h2>
        <div>
          <input
            style={styles.input}
            type="text"
            placeholder="Enter display name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
          />
          <button style={styles.button} onClick={createIdentity}>
            Create Identity
          </button>
        </div>
        {identity && (
          <div style={styles.info}>
            <div style={styles.success}>
              ✅ Identity Created {identity.mock ? '(Mock)' : '(Real)'}
            </div>
            <div>Four Words: {identity.fourWords}</div>
            <div>Display Name: {identity.displayName}</div>
            {identity.result && (
              <details>
                <summary>Raw Result</summary>
                <pre>{JSON.stringify(identity.result, null, 2)}</pre>
              </details>
            )}
          </div>
        )}
      </div>

      <div style={styles.section}>
        <h2 style={styles.title}>MCP Plugin Status</h2>
        <div style={styles.info}>
          <div>Socket Path: /tmp/tauri-mcp-communitas-*.sock</div>
          <div>Check console logs for MCP connection details</div>
        </div>
      </div>

      <div style={styles.section}>
        <h2 style={styles.title}>Environment</h2>
        <div style={styles.info}>
          <div>Development Mode: {import.meta.env.DEV ? 'YES' : 'NO'}</div>
          <div>Mode: {import.meta.env.MODE}</div>
          <div>Base URL: {import.meta.env.BASE_URL}</div>
        </div>
      </div>
    </div>
  )
}

export default TauriTest