import React from 'react'
import { BrowserRouter, Routes, Route, Navigate, useLocation } from 'react-router-dom'
import { Box } from '@mui/material'
import { SnackbarProvider } from 'notistack'

// Theme System
import { ThemeProvider } from './components/theme'
import { darkTheme as modernDarkTheme } from './styles/theme'
import { ThemeProvider as MuiThemeProvider } from '@mui/material/styles'

// Modern Shell - The only UI we need
import { ModernShellPrototypeScreen } from './components/prototype/ModernShellPrototype'
import { SitesDemo } from './components/SitesDemo'

// Contexts needed for ModernShell
import { TauriProvider } from './contexts/TauriContext'
import { AuthProvider } from './components/auth'
import { EncryptionProvider } from './components/encryption'
import { EntityDirectoryProvider } from './contexts/EntityDirectoryContext'

// Error handling
import ErrorBoundary from './components/ErrorBoundary'

// Browser fallback
import { BrowserFallback } from './components/BrowserFallback'
import { isTauriApp } from './utils/tauri'

// Inner component that has access to useLocation() from BrowserRouter
function AppContent() {
  const location = useLocation();

  // Check if running in Tauri or browser
  // Show full UI in development mode or when in Tauri
  const isDevelopment = import.meta.env.DEV || true  // Force true for browser testing
  const showFullUI = isDevelopment || isTauriApp()

  console.log('App render check:', { isDevelopment, isTauriApp: isTauriApp(), showFullUI, envDEV: import.meta.env.DEV })

  if (!showFullUI) {
    console.log('Showing BrowserFallback')
    return (
      <ThemeProvider>
        <BrowserFallback />
      </ThemeProvider>
    );
  }

  // Always show ModernShellPrototype - it's our complete UI
  return (
    <ThemeProvider>
      <MuiThemeProvider theme={modernDarkTheme}>
        <TauriProvider>
          <AuthProvider>
            <EncryptionProvider>
              <EntityDirectoryProvider>
                <Routes>
                  <Route path="/" element={<ModernShellPrototypeScreen />} />
                  <Route path="/prototype/modern-shell" element={<ModernShellPrototypeScreen />} />
                  <Route path="/sites-demo" element={<SitesDemo />} />
                  <Route path="*" element={<Navigate to="/" replace />} />
                </Routes>
              </EntityDirectoryProvider>
            </EncryptionProvider>
          </AuthProvider>
        </TauriProvider>
      </MuiThemeProvider>
    </ThemeProvider>
  )
}

// Main App wrapper that provides BrowserRouter context
function App() {
  return (
    <ErrorBoundary>
      <SnackbarProvider maxSnack={3} anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}>
        <Box
          component="main"
          role="main"
          aria-label="Communitas P2P Collaboration Platform"
          sx={{
            '&:focus-visible': {
              outline: 'none',
            },
          }}
          tabIndex={-1}
        >
          <BrowserRouter>
            <AppContent />
          </BrowserRouter>
        </Box>
      </SnackbarProvider>
    </ErrorBoundary>
  )
}

export default App
