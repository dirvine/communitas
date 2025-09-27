import './polyfills'  // Import polyfills first
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
// import AppSimple from './AppSimple'  // Use simple app for testing
// import AppMinimal from './AppMinimal'  // Use minimal app for testing
import ErrorBoundary from './components/ErrorBoundary'
import { LocalStorageProvider } from './contexts/LocalStorageProvider'
import './index.css'

// Test harnesses disabled for production stability
// async function loadHarnesses() {
//   if (!import.meta.env.DEV) {
//     return
//   }
//
//   const harnesses: Array<() => Promise<unknown>> = [
//     () => import('./test-identity'),
//     () => import('./test-offline-capabilities'),
//     () => import('./setup-test-workspace'),
//     () => import('./test-tauri-groups'),
//     () => import('./test-network-connection'),
//   ]
//
//   await Promise.all(
//     harnesses.map(async (load) => {
//       try {
//         await load()
//       } catch (error) {
//         console.warn('[Communitas] Failed to load dev harness module:', error)
//       }
//     }),
//   )
// }

async function bootstrap() {
  console.info('[Communitas] Bootstrap starting...')
  const rootElement = document.getElementById('root')
  if (!rootElement) {
    throw new Error('Root element not found')
  }

  console.info('[Communitas] Bootstrapping app', {
    mode: import.meta.env.MODE,
    dev: import.meta.env.DEV,
    tauri: typeof window !== 'undefined' && !!(window as any).__TAURI__,
    hasTextDecoder: typeof TextDecoder !== 'undefined',
  })

  if (typeof window !== 'undefined' && (window as any).__TAURI__) {
    try {
      // Import the MCP plugin from local copy
      const mcpModule = await import('./services/mcp-plugin.js')
      console.info('[Communitas] MCP module loaded:', Object.keys(mcpModule))
      
      if (mcpModule.setupPluginListeners && typeof mcpModule.setupPluginListeners === 'function') {
        await mcpModule.setupPluginListeners()
        console.info('[Communitas] MCP plugin listeners initialised successfully')
        
        // Make it globally available for testing
        (window as any).__MCP__ = mcpModule
        console.info('[Communitas] MCP module available at window.__MCP__')
      } else {
        console.warn('[Communitas] setupPluginListeners not found in MCP module')
      }
    } catch (error) {
      console.error('[Communitas] Failed to setup MCP plugin listeners:', error)
    }
  }

  // Test harnesses disabled for production stability
  // await loadHarnesses()

  console.info('[Communitas] Creating root...')
  const root = ReactDOM.createRoot(rootElement)

  console.info('[Communitas] Rendering app...')
  try {
    root.render(
      <React.StrictMode>
        <ErrorBoundary>
          <LocalStorageProvider>
            <App />
          </LocalStorageProvider>
        </ErrorBoundary>
      </React.StrictMode>,
    )
    console.info('[Communitas] App rendered successfully')
  } catch (error) {
    console.error('[Communitas] Failed to render app:', error)
    throw error
  }
}

bootstrap().catch((error) => {
  console.error('[Communitas] Failed to bootstrap', error)
  const rootElement = document.getElementById('root')
  if (rootElement) {
    rootElement.innerHTML = `
      <div style="padding: 24px; font-family: system-ui, sans-serif; color: #c00;">
        <h1 style="margin-bottom: 12px;">Something went wrong</h1>
        <p style="margin: 0 0 16px 0;">${(error as Error).message}</p>
        <p style="margin: 0; font-size: 0.9rem; color: #555;">Check the developer console for details.</p>
      </div>
    `
  }
})
