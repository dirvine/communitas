import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
// import SimpleTestApp from './SimpleTestApp'
import './index.css'
import './test-identity' // Load test utilities for console testing
import './test-offline-capabilities' // Load offline test suite
import './setup-test-workspace' // Load workspace setup utilities
import './test-tauri-groups' // Load Tauri group testing utilities
import './test-network-connection' // Load network connection test utilities

// Error boundary to catch runtime errors
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error?: Error }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ padding: '20px', backgroundColor: '#fee' }}>
          <h1>Runtime Error</h1>
          <pre>{this.state.error?.toString()}</pre>
          <details>
            <summary>Stack Trace</summary>
            <pre>{this.state.error?.stack}</pre>
          </details>
        </div>
      )
    }
    return this.props.children
  }
}

// Mount the app
const rootElement = document.getElementById('root')
console.log('Root element:', rootElement)
console.log('Environment:', { 
  isDev: import.meta.env.DEV,
  mode: import.meta.env.MODE,
  tauriAvailable: typeof (window as any).__TAURI__ !== 'undefined'
})

if (rootElement) {
  const root = ReactDOM.createRoot(rootElement)
  console.log('Creating React root...')
  
  try {
    root.render(
      <React.StrictMode>
        <ErrorBoundary>
          <App />
        </ErrorBoundary>
      </React.StrictMode>
    )
    console.log('React app mounted successfully')
  } catch (error) {
    console.error('Failed to render app:', error)
    rootElement.innerHTML = `
      <div style="padding: 20px; background: #fee; color: #c00;">
        <h1>Failed to render app</h1>
        <pre>${error}</pre>
      </div>
    `
  }
} else {
  console.error('Root element not found!')
  document.body.innerHTML = '<h1>Root element not found!</h1>'
}