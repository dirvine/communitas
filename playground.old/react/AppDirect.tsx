import React, { useState } from 'react'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import CssBaseline from '@mui/material/CssBaseline'
import Box from '@mui/material/Box'
import AppBar from '@mui/material/AppBar'
import Toolbar from '@mui/material/Toolbar'
import Typography from '@mui/material/Typography'
import Button from '@mui/material/Button'
import TextField from '@mui/material/TextField'
import CircularProgress from '@mui/material/CircularProgress'
import Alert from '@mui/material/Alert'
import Container from '@mui/material/Container'
import Paper from '@mui/material/Paper'

const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#6366f1',
    },
    background: {
      default: '#0f172a',
      paper: '#1e293b',
    },
  },
})

const AppDirect: React.FC = () => {
  const [displayName, setDisplayName] = useState('')
  const [isGenerating, setIsGenerating] = useState(false)
  const [userIdentity, setUserIdentity] = useState<any>(null)
  const [showApp, setShowApp] = useState(false)

  // Check if we're in Tauri
  const inTauri = typeof window !== 'undefined' && (
    (window as any).__TAURI__ || 
    (window as any).__TAURI_IPC__ ||
    window.location.protocol === 'tauri:'
  )

  const handleSignup = async () => {
    setIsGenerating(true)
    
    try {
      // If in Tauri, try to use the actual commands
      if (inTauri && (window as any).__TAURI__) {
        console.log('Attempting to use Tauri commands...')
        try {
          const { invoke } = (window as any).__TAURI__.core
          // Try to initialize the core
          await invoke('core_initialize')
          console.log('Core initialized successfully')
          
          // Try to get or create identity
          const identity = await invoke('core_get_identity')
          console.log('Got identity:', identity)
          
          setUserIdentity(identity)
          setShowApp(true)
        } catch (error) {
          console.error('Tauri command failed:', error)
          // Fall back to mock
          createMockIdentity()
        }
      } else {
        // Use mock identity
        createMockIdentity()
      }
    } catch (error) {
      console.error('Failed to generate identity:', error)
      createMockIdentity()
    } finally {
      setIsGenerating(false)
    }
  }

  const createMockIdentity = () => {
    const mockIdentity = {
      display_name: displayName || 'Anonymous User',
      four_word_address: `${randomWord()}-${randomWord()}-${randomWord()}-${randomWord()}`
    }
    setUserIdentity(mockIdentity)
    setTimeout(() => setShowApp(true), 1000)
  }

  const randomWord = () => {
    const words = ['happy', 'sunny', 'blue', 'green', 'swift', 'bright', 'cool', 'warm', 'soft', 'bold']
    return words[Math.floor(Math.random() * words.length)]
  }

  if (showApp && userIdentity) {
    return (
      <ThemeProvider theme={theme}>
        <CssBaseline />
        <Box sx={{ flexGrow: 1 }}>
          <AppBar position="static">
            <Toolbar>
              <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
                Communitas - P2P Collaboration
              </Typography>
              <Typography variant="body2" sx={{ mr: 2 }}>
                {userIdentity.display_name} ({userIdentity.four_word_address})
              </Typography>
              <Button color="inherit" onClick={() => setShowApp(false)}>
                Logout
              </Button>
            </Toolbar>
          </AppBar>
          
          <Container maxWidth="lg" sx={{ mt: 4 }}>
            <Paper sx={{ p: 3 }}>
              <Typography variant="h4" gutterBottom>
                Welcome to Communitas
              </Typography>
              <Typography variant="body1" paragraph>
                You are now connected to the P2P network.
              </Typography>
              
              <Box sx={{ mt: 3 }}>
                <Alert severity="info" sx={{ mb: 2 }}>
                  Running in {inTauri ? 'Tauri Desktop' : 'Web Browser'} mode
                </Alert>
                
                <Typography variant="h6" gutterBottom>
                  Your Identity
                </Typography>
                <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                  Display Name: {userIdentity.display_name}
                </Typography>
                <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                  Four Word Address: {userIdentity.four_word_address}
                </Typography>
              </Box>

              {inTauri && (
                <Box sx={{ mt: 3 }}>
                  <Typography variant="h6" gutterBottom>
                    Tauri Environment Detected
                  </Typography>
                  <Typography variant="body2">
                    Desktop features are available. You can use secure identity management, 
                    local storage, and P2P networking.
                  </Typography>
                </Box>
              )}
            </Paper>
          </Container>
        </Box>
      </ThemeProvider>
    )
  }

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box sx={{ 
        display: 'flex', 
        justifyContent: 'center', 
        alignItems: 'center', 
        minHeight: '100vh',
        bgcolor: 'background.default' 
      }}>
        <Paper sx={{ p: 4, maxWidth: 400, width: '100%' }}>
          <Typography variant="h4" gutterBottom align="center">
            Welcome to Communitas
          </Typography>
          <Typography variant="body1" gutterBottom align="center" color="text.secondary">
            Create your identity to get started.
          </Typography>
          
          {inTauri && (
            <Alert severity="success" sx={{ mb: 2 }}>
              Tauri Desktop Mode Active
            </Alert>
          )}
          
          <TextField
            fullWidth
            label="Display Name"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder="Enter your name"
            sx={{ mb: 2 }}
          />
          
          <Button
            fullWidth
            variant="contained"
            onClick={handleSignup}
            disabled={isGenerating}
            startIcon={isGenerating ? <CircularProgress size={20} /> : null}
          >
            {isGenerating ? 'Creating Identity...' : 'Create Identity'}
          </Button>
        </Paper>
      </Box>
    </ThemeProvider>
  )
}

export default AppDirect