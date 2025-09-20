import React, { useState, useEffect, Suspense } from 'react'
import { BrowserRouter, Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom'
import {
  AppBar,
  Toolbar,
  Typography,
  Box,
  IconButton,
  Button,
  Stack,
  Avatar,
  Menu,
  MenuItem,
  Chip,
} from '@mui/material'
import {
  Menu as MenuIcon,
  Home as HomeIcon,
  Lan as LanIcon,
  AccountCircle as AccountIcon,
  Login as LoginIcon,
  Logout as LogoutIcon,
  Person as PersonIcon,
} from '@mui/icons-material'
import { SnackbarProvider } from 'notistack'

// Core providers - keep these minimal
import { ThemeProvider } from './components/theme'

// Navigation
import { NavigationProvider } from './contexts/NavigationContext'

// Authentication
import { AuthProvider, useAuth } from './contexts/AuthContext'
import LoginDialog from './components/auth/LoginDialog'

// Tauri integration
import { TauriProvider } from './contexts/TauriContext'
import { BrowserFallback } from './components/BrowserFallback'
import { isTauriApp } from './utils/tauri'

// Error handling
import ErrorBoundary from './components/ErrorBoundary'

// Main components
import OverviewDashboard from './components/OverviewDashboard'
import QuickActionsBar from './components/QuickActionsBar'

// Lazy loaded components for better performance
const IdentityTab = React.lazy(() => import('./components/tabs/IdentityTab'))
const UnifiedDashboard = React.lazy(() => import('./components/unified/UnifiedDashboard').then(m => ({ default: m.UnifiedDashboard })))
const TestPage = React.lazy(() => import('./components/testing/TestPage').then(m => ({ default: m.TestPage })))

// Loading component
const LoadingSpinner = () => (
  <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '200px' }}>
    <Typography>Loading...</Typography>
  </Box>
)

// Main App Component
const AppContent: React.FC = () => {
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [currentView, setCurrentView] = useState('dashboard')
  const [loginDialogOpen, setLoginDialogOpen] = useState(false)
  const [userMenuAnchor, setUserMenuAnchor] = useState<null | HTMLElement>(null)

  // Authentication
  const { authState, logout } = useAuth()

  // Mock data for required props
  const mockNetworkHealth = {
    status: 'connected',
    peer_count: 5,
    nat_type: 'unknown',
    bandwidth_kbps: 1000,
    avg_latency_ms: 45,
  }

  const mockContext = {
    type: 'personal' as const,
    entity: null,
  }

  const handleAction = (action: string, data?: any) => {
    console.log('Action:', action, data)
  }

  const toggleSidebar = () => setSidebarOpen(!sidebarOpen)

  const handleLoginClick = () => {
    setLoginDialogOpen(true)
  }

  const handleLoginSuccess = () => {
    setLoginDialogOpen(false)
    setCurrentView('dashboard')
  }

  const handleUserMenuClick = (event: React.MouseEvent<HTMLElement>) => {
    setUserMenuAnchor(event.currentTarget)
  }

  const handleUserMenuClose = () => {
    setUserMenuAnchor(null)
  }

  const handleLogout = async () => {
    await logout()
    setUserMenuAnchor(null)
    setCurrentView('dashboard')
  }

  return (
    <NavigationProvider>
      <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
        {/* App Bar */}
        <AppBar position="static" sx={{ zIndex: 1100 }}>
          <Toolbar>
            <IconButton
              color="inherit"
              edge="start"
              onClick={toggleSidebar}
              sx={{ mr: 2 }}
            >
              <MenuIcon />
            </IconButton>
            <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
              Communitas - P2P Collaboration
            </Typography>
            <Stack direction="row" spacing={1} alignItems="center">
              <Button
                color="inherit"
                startIcon={<HomeIcon />}
                onClick={() => setCurrentView('dashboard')}
              >
                Dashboard
              </Button>
              <Button
                color="inherit"
                startIcon={<LanIcon />}
                onClick={() => setCurrentView('network')}
              >
                Network
              </Button>

              {/* Authentication Section */}
              {authState.isAuthenticated ? (
                <>
                  {/* User Info */}
                  <Chip
                    avatar={
                      <Avatar sx={{ width: 24, height: 24 }}>
                        <PersonIcon fontSize="small" />
                      </Avatar>
                    }
                    label={authState.user?.fourWordAddress || 'User'}
                    variant="outlined"
                    sx={{ color: 'white', borderColor: 'white' }}
                    onClick={handleUserMenuClick}
                  />

                  {/* User Menu */}
                  <Menu
                    anchorEl={userMenuAnchor}
                    open={Boolean(userMenuAnchor)}
                    onClose={handleUserMenuClose}
                  >
                    <MenuItem onClick={() => { setCurrentView('identity'); handleUserMenuClose(); }}>
                      <PersonIcon sx={{ mr: 1 }} />
                      Identity
                    </MenuItem>
                    <MenuItem onClick={handleLogout}>
                      <LogoutIcon sx={{ mr: 1 }} />
                      Logout
                    </MenuItem>
                  </Menu>
                </>
              ) : (
                <Button
                  color="inherit"
                  startIcon={<LoginIcon />}
                  onClick={handleLoginClick}
                  variant="outlined"
                  sx={{ borderColor: 'white', color: 'white' }}
                >
                  Sign In
                </Button>
              )}
            </Stack>
          </Toolbar>
        </AppBar>

        {/* Main Content */}
        <Box sx={{ display: 'flex', flexGrow: 1 }}>
          {/* Sidebar */}
          <Box
            sx={{
              width: sidebarOpen ? 240 : 0,
              transition: 'width 0.3s',
              overflow: 'hidden',
              bgcolor: 'background.paper',
              borderRight: 1,
              borderColor: 'divider',
            }}
          >
            {sidebarOpen && (
              <Box sx={{ p: 2 }}>
                <Typography variant="h6" gutterBottom>
                  Navigation
                </Typography>
                <Stack spacing={1}>
                  <Button
                    fullWidth
                    variant={currentView === 'dashboard' ? 'contained' : 'outlined'}
                    onClick={() => setCurrentView('dashboard')}
                  >
                    Dashboard
                  </Button>
                  <Button
                    fullWidth
                    variant={currentView === 'identity' ? 'contained' : 'outlined'}
                    onClick={() => setCurrentView('identity')}
                  >
                    Identity
                  </Button>
                  <Button
                    fullWidth
                    variant={currentView === 'network' ? 'contained' : 'outlined'}
                    onClick={() => setCurrentView('network')}
                  >
                    Network
                  </Button>
                  <Button
                    fullWidth
                    variant={currentView === 'test' ? 'contained' : 'outlined'}
                    onClick={() => setCurrentView('test')}
                  >
                    Test Page
                  </Button>
                </Stack>
              </Box>
            )}
          </Box>

          {/* Main Content Area */}
          <Box sx={{ flexGrow: 1, p: 3 }}>
            <Suspense fallback={<LoadingSpinner />}>
              {currentView === 'dashboard' && (
                <OverviewDashboard networkHealth={mockNetworkHealth} />
              )}
              {currentView === 'identity' && <IdentityTab />}
              {currentView === 'network' && (
                <UnifiedDashboard
                  userId="test-user"
                  userName="Test User"
                  fourWords="test-user-words"
                />
              )}
              {currentView === 'test' && <TestPage />}
            </Suspense>
          </Box>
        </Box>

        {/* Quick Actions Bar */}
        <QuickActionsBar
          context={mockContext}
          onAction={handleAction}
        />

        {/* Login Dialog */}
        <LoginDialog
          open={loginDialogOpen}
          onClose={() => setLoginDialogOpen(false)}
          onSuccess={handleLoginSuccess}
        />
      </Box>
    </NavigationProvider>
  )
}

// Main App with all providers
const App: React.FC = () => {
  const isTauri = isTauriApp()

  return (
    <ErrorBoundary>
      <ThemeProvider>
        <SnackbarProvider maxSnack={3}>
          <AuthProvider>
            <TauriProvider>
              {isTauri ? (
                <AppContent />
              ) : (
                <BrowserFallback>
                  <AppContent />
                </BrowserFallback>
              )}
            </TauriProvider>
          </AuthProvider>
        </SnackbarProvider>
      </ThemeProvider>
    </ErrorBoundary>
  )
}

export default App