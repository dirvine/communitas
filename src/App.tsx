import React, { useState, useEffect, Suspense } from 'react'
import { BrowserRouter, Routes, Route, Navigate, useNavigate, useLocation } from 'react-router-dom'
import {
  AppBar,
  Toolbar,
  Typography,
  Box,
  IconButton,
  Button,
  Tooltip,
  Switch,
  FormControlLabel,
  Chip,
  Stack,
} from '@mui/material'
import {
  Menu as MenuIcon,
  Person,
  ChevronLeft,
  ChevronRight,
  Lan as LanIcon,
  Home as HomeIcon,
} from '@mui/icons-material'
import useMediaQuery from '@mui/material/useMediaQuery'
import { SnackbarProvider } from 'notistack'
import { NetworkHealth } from './types'

// Feature Flags
import { featureFlags, useFeatureFlag } from './services/featureFlags'

// Theme System
import { ThemeProvider, ThemeSwitcher } from './components/theme'

// Authentication System
import { AuthProvider, AuthStatus } from './components/auth'

// Encryption System
import { EncryptionProvider, EncryptionStatus } from './components/encryption'

  // Responsive Layout  
  import { useSidebarBehavior } from './components/responsive'

// Navigation - both old and new
import { WhatsAppStyleNavigation } from './components/navigation/WhatsAppStyleNavigation'
import { NavigationProvider } from './contexts/NavigationContext'
import BreadcrumbNavigation from './components/navigation/BreadcrumbNavigation'
import ContextAwareSidebar from './components/navigation/ContextAwareSidebar'

// Mock data for testing
import {
  mockOrganizations,
  mockPersonalGroups,
  mockPersonalUsers,
} from './data/mockCollaborationData'

// Tauri Context
import { TauriProvider } from './contexts/TauriContext'
import { BrowserFallback } from './components/BrowserFallback'
import { isTauriApp } from './utils/tauri'
import { ensureIdentity } from './utils/identity'

// WebRTC Communication
import { SimpleCommunicationHub } from './components/webrtc'
import { LoginDialog } from './components/auth/LoginDialog'

// Communication
import { EntitySelector } from './components/communication/EntitySelector'

// Error handling
import ErrorBoundary from './components/ErrorBoundary'

// Real-time Sync
import { GlobalSyncBar } from './components/sync/GlobalSyncBar'

// Network Status
import { NetworkStatusIndicator } from './components/network/NetworkStatusIndicator'
import { EndpointStatusDisplay, CompactEndpointStatus } from './components/network/EndpointStatusDisplay'
import { networkService } from './services/network/NetworkConnectionService'

import OverviewDashboard from './components/OverviewDashboard'
// import FirstRunWizard from './components/onboarding/FirstRunWizard'
import QuickActionsBar from './components/QuickActionsBar'
import StorageWorkspaceDialog from './components/storage/StorageWorkspaceDialog'

const IdentityTab = React.lazy(() => import('./components/tabs/IdentityTab'))
const WebsitePublishPanel = React.lazy(() => import('./components/dev/WebsitePublishPanel'))
const UnifiedDashboard = React.lazy(() => import('./components/unified/UnifiedDashboard').then(m => ({ default: m.UnifiedDashboard })))
const CollaborativeEditingTest = React.lazy(() => import('./components/testing/CollaborativeEditingTest').then(m => ({ default: m.CollaborativeEditingTest })))
const SimpleCollaborationTest = React.lazy(() => import('./components/testing/SimpleCollaborationTest').then(m => ({ default: m.SimpleCollaborationTest })))
const TestPage = React.lazy(() => import('./components/testing/TestPage').then(m => ({ default: m.TestPage })))
const SimpleTest = React.lazy(() => import('./components/testing/SimpleTest').then(m => ({ default: m.SimpleTest })))
const MessageConsole = React.lazy(() => import('./components/dev/MessageConsole').then(m => ({ default: m.MessageConsole })))
const GroupPage = React.lazy(() => import('./components/pages/GroupPage').then(m => ({ default: m.GroupPage })))
const UserPage = React.lazy(() => import('./components/pages/UserPage').then(m => ({ default: m.UserPage })))

// Test button component that uses React Router navigation
const TestButton: React.FC = () => {
  const navigate = useNavigate();

  return (
    <Button
      variant="contained"
      onClick={() => navigate('/test/collaboration')}
      sx={{ mt: 2 }}
    >
      🧪 Test Collaborative Editing
    </Button>
  );
};


function App() {
  console.log('App component rendering...')
  
  // Experimental mode is now the default
  // Enable all features
  React.useEffect(() => {
    featureFlags.enable('unified-design-system')
    featureFlags.enable('context-aware-navigation')
    featureFlags.enable('four-word-identity')
    featureFlags.enable('unified-storage-ui')
  }, [])
  
  // Check which features are enabled
  const useContextNav = useFeatureFlag('context-aware-navigation', 'user_owner_123')
  
  // Navigation context for unified navigation
  const [navigationContext, setNavigationContext] = useState<{
    mode: 'personal' | 'organization' | 'project'
    organizationId?: string
    organizationName?: string
    projectId?: string
    projectName?: string
    fourWords?: string
  }>({
    mode: 'personal',
    fourWords: undefined,
  })

  const [showOverview, setShowOverview] = useState(false)
  const [authDialogOpen, setAuthDialogOpen] = useState(false)
  const [_selectedEntity, _setSelectedEntity] = useState<any>(null)
  const [showStorageWorkspace, setShowStorageWorkspace] = useState(false)
  const [showEntitySelector, setShowEntitySelector] = useState(false)
  const [pendingAction, setPendingAction] = useState<'call' | 'video' | 'screen' | 'storage' | null>(null)
  const [networkHealth, setNetworkHealth] = useState<NetworkHealth>({
    status: 'Disconnected',
    peer_count: 0,
    nat_type: 'Unknown',
    bandwidth_kbps: 0,
    avg_latency_ms: 0,
  })
  
  // Use responsive sidebar behavior
  const { defaultOpen } = useSidebarBehavior()
  const [sidebarOpen, setSidebarOpen] = useState(defaultOpen)
  const handleToggleSidebar = () => setSidebarOpen(o => !o)
  const isSmall = useMediaQuery('(max-width:900px)')

  useEffect(() => {
    if (isSmall) setSidebarOpen(false)
  }, [isSmall])

  useEffect(() => {
    // Load or generate identity
    ensureIdentity().then(four => {
      setNavigationContext(prev => ({ ...prev, fourWords: four }))
    }).catch(() => {
      // leave undefined; UI can handle missing identity
    })

    // Initialize network connection service (auto-connects on startup)
    console.log('🚀 Initializing network connection service...')
    // Network service will auto-connect in its constructor
  }, [])

  // Listen for global storage workspace open requests (from dashboards, etc.)
  useEffect(() => {
    const handler = (e: any) => {
      setShowStorageWorkspace(true)
    }
    window.addEventListener('open-storage-workspace' as any, handler)
    return () => window.removeEventListener('open-storage-workspace' as any, handler)
  }, [])

  useEffect(() => {
    let mounted = true
    const fetchHealth = async () => {
      try {
        const res = await (await import('@tauri-apps/api/core')).invoke<any>('get_network_health')
        if (!mounted) return
        setNetworkHealth({
          status: res.status === 'connected' ? 'Connected' : 'Disconnected',
          peer_count: res.peer_count ?? 0,
          nat_type: res.nat_type ?? 'Unknown',
          bandwidth_kbps: res.bandwidth_kbps ?? 0,
          avg_latency_ms: res.avg_latency_ms ?? 0,
        })
      } catch {
        // keep default
      }
    }
    fetchHealth()
    const id = setInterval(fetchHealth, 2000)
    return () => { mounted = false; clearInterval(id) }
  }, [])



  // handleToggleSidebar defined above near sidebarOpen declaration

  // Removed unused _setCurrentTab; keep modal toggles where needed
  const _handleTabChange = (_newValue: number) => {
    if (_newValue === 2) {
      setShowOverview(true)
    } else if (_newValue === 3) {
      setAuthDialogOpen(true)
    }
  }

  // Collaboration feature handlers
  const handleVideoCall = (entityId?: string, entityType?: string) => {
    if (!entityId || !entityType) {
      // Show entity selector for video call
      setPendingAction('video')
      setShowEntitySelector(true)
      return
    }
    console.log('Starting video call for', entityType, entityId)
    // TODO: Integrate with WebRTC implementation
  }

  const handleAudioCall = (entityId?: string, entityType?: string) => {
    if (!entityId || !entityType) {
      // Show entity selector for audio call
      setPendingAction('call')
      setShowEntitySelector(true)
      return
    }
    console.log('Starting audio call for', entityType, entityId)
    // TODO: Integrate with WebRTC implementation
  }

  const handleScreenShare = (entityId?: string, entityType?: string) => {
    if (!entityId || !entityType) {
      // Show entity selector for screen share
      setPendingAction('screen')
      setShowEntitySelector(true)
      return
    }
    console.log('Starting screen share for', entityType, entityId)
    // TODO: Integrate with WebRTC implementation
  }

  const handleOpenFiles = (entityId?: string, entityType?: string) => {
    if (!entityId || !entityType) {
      // Show entity selector for storage
      setPendingAction('storage')
      setShowEntitySelector(true)
      return
    }
    console.log('Opening files for', entityType, entityId)
    _setSelectedEntity({ id: entityId, type: entityType })
    // Use the unified storage workspace dialog instead of the basic file sharing dialog
    setShowStorageWorkspace(true)
  }

  const handleEntitySelected = (entity: any, type: 'person' | 'group' | 'organization') => {
    // Execute the pending action with the selected entity
    const entityId = entity.id
    const entityType = type
    
    switch (pendingAction) {
      case 'video':
        handleVideoCall(entityId, entityType)
        break
      case 'call':
        handleAudioCall(entityId, entityType)
        break
      case 'screen':
        handleScreenShare(entityId, entityType)
        break
      case 'storage':
        handleOpenFiles(entityId, entityType)
        break
    }
    
    setPendingAction(null)
  }

  const handleQuickAction = (action: string) => {
    // Check if we have a selected entity or context
    const hasContext = navigationContext.organizationId || navigationContext.projectId || _selectedEntity
    
    switch (action) {
      case 'start_voice_call':
        if (hasContext) {
          const currentType = navigationContext.mode
          const currentId = navigationContext.organizationId || navigationContext.projectId || _selectedEntity?.id
          if (currentId) {
            handleAudioCall(currentId, currentType)
          } else {
            handleAudioCall() // Will show selector
          }
        } else {
          handleAudioCall() // Will show selector
        }
        break
      case 'start_video_call':
        if (hasContext) {
          const currentType = navigationContext.mode
          const currentId = navigationContext.organizationId || navigationContext.projectId || _selectedEntity?.id
          if (currentId) {
            handleVideoCall(currentId, currentType)
          } else {
            handleVideoCall() // Will show selector
          }
        } else {
          handleVideoCall() // Will show selector
        }
        break
      case 'upload_documents':
      case 'storage_settings':
      case 'upload_files':
      case 'open_chat':
      default:
        if (hasContext) {
          const currentType = navigationContext.mode
          const currentId = navigationContext.organizationId || navigationContext.projectId || _selectedEntity?.id
          if (currentId) {
            handleOpenFiles(currentId, currentType)
          } else {
            handleOpenFiles() // Will show selector
          }
        } else {
          handleOpenFiles() // Will show selector
        }
        break
    }
  }

  const handleWhatsAppNavigate = (path: string, entity: any) => {
    console.log('WhatsApp Navigation:', path, entity)
    _setSelectedEntity(entity)

    // Update navigation context based on path
    if (path.startsWith('/org/')) {
      const parts = path.split('/')
      const orgId = parts[2]
      const org = mockOrganizations.find(o => o.id === orgId)
      setNavigationContext({
        mode: 'organization',
        organizationId: orgId,
        organizationName: org?.name || 'Organization',
        fourWords: org?.networkIdentity.fourWords || 'unknown-org',
      })
    } else if (path === '/') {
      setNavigationContext({
        mode: 'personal',
        fourWords: navigationContext.fourWords,
      })
    }
  }

  // Responsive header component
  const HeaderComponent = ({ onMenuClick, showMenuButton }: { 
    onMenuClick?: () => void; 
    showMenuButton?: boolean;
  }) => {
    const navigate = useNavigate();
    return (
    <Toolbar sx={{ gap: 1 }}>
      {showMenuButton && (
        <IconButton
          color="inherit"
          edge="start"
          onClick={onMenuClick}
          sx={{ mr: 2, flexShrink: 0 }}
        >
          <MenuIcon />
        </IconButton>
      )}
      <IconButton color="inherit" onClick={() => navigate('/') } sx={{ mr: 1 }} aria-label="Home">
        <HomeIcon />
      </IconButton>
      <Typography 
        variant="h6" 
        component="div" 
        sx={{ 
          flexGrow: 1,
          background: (theme) => theme.gradients?.primary,
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          backgroundClip: 'text',
          fontWeight: 600,
          fontSize: { xs: '0.9rem', sm: '1.1rem', md: '1.25rem' },
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          minWidth: 0,
        }}
      >
        Communitas
      </Typography>
      <Box sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        gap: { xs: 0.5, sm: 1 },
        flexShrink: 0,
        ml: 'auto',
      }}>
        {/* Compact Endpoint Status showing four-words or offline */}
        <CompactEndpointStatus />
        <Button
          variant="outlined"
          size="small"
          startIcon={<LanIcon />}
          onClick={async () => {
            const text = navigationContext.fourWords || 'local'
            try { await navigator.clipboard.writeText(text) } catch {}
          }}
          sx={{ 
            borderColor: (theme) => theme.palette.divider,
            color: 'primary.main',
            minWidth: 'auto',
            px: { xs: 1, sm: 2 },
            '&:hover': {
              borderColor: (theme) => theme.palette.primary.light,
              backgroundColor: (theme) => theme.palette.action.hover,
            },
            '& .MuiButton-startIcon': { mr: { xs: 0.5, sm: 1 } },
          }}
          title="Click to copy your local four-word address"
        >
          <Box component="span" sx={{ display: { xs: 'none', sm: 'inline' } }}>
            {navigationContext.fourWords || 'local'}
          </Box>
        </Button>
        <Box sx={{ display: { xs: 'none', md: 'flex' }, alignItems: 'center', gap: 1 }}>
          <EncryptionStatus compact={true} />
          <ThemeSwitcher compact showPresets />
        </Box>
        <AuthStatus compact={true} showLabel={false} />
      </Box>
    </Toolbar>
  )}

  // Check if running in Tauri or browser
  // Show full UI in development mode or when in Tauri
  // In development, always show full UI to enable testing
  const isDevelopment = import.meta.env.DEV
  const showFullUI = isDevelopment || isTauriApp()
  
  console.log('App render check:', { isDevelopment, isTauriApp: isTauriApp(), showFullUI })
  
  if (!showFullUI) {
    console.log('Showing BrowserFallback')
    return (
      <ThemeProvider>
        <BrowserFallback />
      </ThemeProvider>
    );
  }
  
  console.log('Showing full UI')

  // Handle navigation from unified navigation
  const _handleUnifiedNavigate = (_path: string) => {
    console.log('Navigate to:', _path)

    // Parse the path to update context
    if (_path.startsWith('/org/')) {
      const parts = _path.split('/')
      const orgId = parts[2]
      setNavigationContext({
        mode: 'organization',
        organizationId: orgId,
        organizationName: 'Acme Corp', // TODO: Fetch from store
        fourWords: 'acme-global-secure-network',
      })
      window.dispatchEvent(new CustomEvent('app:navigate', { detail: _path }))
    } else if (_path.startsWith('/project/')) {
      const parts = _path.split('/')
      const projectId = parts[2]
      setNavigationContext({
        mode: 'project',
        projectId: projectId,
        projectName: 'Project Alpha', // TODO: Fetch from store
        fourWords: 'alpha-mission-space-explore',
      })
      window.dispatchEvent(new CustomEvent('app:navigate', { detail: _path }))
    } else if (_path.startsWith('/group/')) {
      const parts = _path.split('/')
      const groupId = parts[2]
      setNavigationContext(prev => ({ ...prev, mode: 'personal' }))
      window.dispatchEvent(new CustomEvent('app:navigate', { detail: _path }))
    } else if (_path.startsWith('/user/')) {
      const parts = _path.split('/')
      const userId = parts[2]
      setNavigationContext(prev => ({ ...prev, mode: 'personal' }))
      window.dispatchEvent(new CustomEvent('app:navigate', { detail: _path }))
    } else {
      setNavigationContext({
        mode: 'personal',
        fourWords: 'ocean-forest-moon-star',
      })
      window.dispatchEvent(new CustomEvent('app:navigate', { detail: '/' }))
    }

    // TODO: Implement actual routing
  }

  // Render theme provider conditionally to keep prop types correct

  // First-run wizard removed in browser mode; use AuthStatus/Login dialog instead

  const ThemedApp = (
    <TauriProvider>
      <AuthProvider>
        <EncryptionProvider>
          <NavigationProvider>
          <BrowserRouter>
          {/** Bridge navigation events to React Router */}
          {(() => {
            const NavBridge: React.FC = () => {
              const nav = useNavigate();
              const location = useLocation();
              // bridge custom events -> router navigate
              useEffect(() => {
                const handler = (e: any) => { if (e?.detail) nav(e.detail) }
                window.addEventListener('app:navigate', handler)
                return () => window.removeEventListener('app:navigate', handler)
              }, [nav])
              // route-level navigation context
              useEffect(() => {
                const path = location.pathname
                if (path.startsWith('/org/')) {
                  const parts = path.split('/');
                  const orgId = parts[2]
                  setNavigationContext(prev => ({ ...prev, mode: 'organization', organizationId: orgId }))
                } else if (path.startsWith('/project/')) {
                  const parts = path.split('/');
                  const projectId = parts[2]
                  setNavigationContext(prev => ({ ...prev, mode: 'project', projectId }))
                } else if (path.startsWith('/group/')) {
                  const parts = path.split('/');
                  const groupId = parts[2]
                  setNavigationContext(prev => ({ ...prev, mode: 'personal', projectId: undefined, organizationId: undefined }))
                  // could attach group selection state if needed
                } else if (path.startsWith('/user/')) {
                  const parts = path.split('/');
                  const userId = parts[2]
                  setNavigationContext(prev => ({ ...prev, mode: 'personal', projectId: undefined, organizationId: undefined }))
                } else {
                  setNavigationContext(prev => ({ ...prev, mode: 'personal', projectId: undefined, organizationId: undefined }))
                }
              }, [location.pathname])
              
              // bridge action events -> handlers
              useEffect(() => {
                const handler = (e: any) => {
                  const d = e?.detail; if (!d) return;
                  switch (d.action) {
                    case 'video': handleVideoCall(d.entityId, d.entityType); break;
                    case 'call': handleAudioCall(d.entityId, d.entityType); break;
                    case 'screen': handleScreenShare(d.entityId, d.entityType); break;
                    case 'storage': handleOpenFiles(d.entityId, d.entityType); break;
                  }
                }
                window.addEventListener('app:action', handler)
                return () => window.removeEventListener('app:action', handler)
              }, [])
              return null
            }
            return <NavBridge />
          })()}
          {/* Global Sync Status Bar */}
          <GlobalSyncBar 
            userId="user_owner_123" // TODO: Use actual authenticated user ID
            position="top"
            autoHide={true}
            autoHideDelay={5000}
          />
          
          {/* Conditionally render breadcrumb navigation */}
          {useContextNav && <BreadcrumbNavigation />}
          
          {/* Using experimental UI as default */}
          {/* New WhatsApp-style UI */}
            <Box sx={{ display: 'flex', height: '100vh', position: 'relative' }}>
              {/* Responsive Sidebar */}
              {isSmall ? (
                <>
                  {!sidebarOpen && (
                    <Box sx={{ position: 'absolute', top: 8, left: 8, zIndex: 2000 }}>
                      <IconButton size="small" onClick={handleToggleSidebar} aria-label="Open sidebar">
                        <ChevronRight />
                      </IconButton>
                    </Box>
                  )}
                  {sidebarOpen && (
                    <>
                      <Box onClick={handleToggleSidebar} sx={{ position: 'absolute', inset: 0, bgcolor: 'rgba(0,0,0,0.35)', zIndex: 1199 }} />
                      <Box sx={{ position: 'absolute', top: 0, left: 0, bottom: 0, width: '85vw', maxWidth: 360, bgcolor: 'background.paper', borderRight: theme => `1px solid ${theme.palette.divider}`, zIndex: 1200, overflow: 'hidden' }}>
                        <WhatsAppStyleNavigation
                          organizations={mockOrganizations}
                          personalGroups={mockPersonalGroups}
                          personalUsers={mockPersonalUsers}
                          currentUserId="user_owner_123"
                          onNavigate={handleWhatsAppNavigate}
                          onVideoCall={handleVideoCall}
                          onAudioCall={handleAudioCall}
                          onScreenShare={handleScreenShare}
                          onOpenFiles={handleOpenFiles}
                        />
                        <Box sx={{ position: 'absolute', top: 8, right: 8 }}>
                          <IconButton size="small" onClick={handleToggleSidebar}>
                            <ChevronLeft />
                          </IconButton>
                        </Box>
                      </Box>
                    </>
                  )}
                </>
              ) : (
                <>
                  <Box
                    sx={{
                      width: sidebarOpen ? 320 : 0,
                      transition: 'width 0.2s ease',
                      borderRight: theme => (sidebarOpen ? `1px solid ${theme.palette.divider}` : 'none'),
                      overflow: 'hidden',
                      position: 'relative',
                      minWidth: 0,
                    }}
                  >
                    <WhatsAppStyleNavigation
                      organizations={mockOrganizations}
                      personalGroups={mockPersonalGroups}
                      personalUsers={mockPersonalUsers}
                      currentUserId="user_owner_123"
                      onNavigate={handleWhatsAppNavigate}
                      onVideoCall={handleVideoCall}
                      onAudioCall={handleAudioCall}
                      onScreenShare={handleScreenShare}
                      onOpenFiles={handleOpenFiles}
                    />
                    {sidebarOpen && (
                      <Box sx={{ position: 'absolute', top: 8, right: 8 }}>
                        <IconButton size="small" onClick={handleToggleSidebar}>
                          <ChevronLeft />
                        </IconButton>
                      </Box>
                    )}
                  </Box>
                  {!sidebarOpen && (
                    <Box sx={{ position: 'absolute', top: 8, left: 8, zIndex: 2000 }}>
                      <IconButton size="small" onClick={handleToggleSidebar} aria-label="Open sidebar">
                        <ChevronRight />
                      </IconButton>
                    </Box>
                  )}
                </>
              )}
              <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                <AppBar position="sticky" elevation={1} sx={{ 
                  backgroundColor: theme => theme.palette.background.paper,
                  color: theme => theme.palette.text.primary,
                  borderBottom: theme => `1px solid ${theme.palette.divider}`
                }}>
                  <HeaderComponent 
                    onMenuClick={handleToggleSidebar}
                    showMenuButton={false}
                  />
                </AppBar>
                <Box sx={{ flex: 1, overflow: 'auto', bgcolor: 'grey.50' }}>
                  <Suspense fallback={<Box sx={{ p: 3 }}><Typography>Loading…</Typography></Box>}>
                  <Routes>
                    <Route path="/" element={
                      <Box sx={{
                        display: 'flex', alignItems: 'center', justifyContent: 'center',
                        height: '100%', flexDirection: 'column', p: 3 }}>
                        {/* Prominent Endpoint Status Display */}
                        <Box sx={{ mb: 4, width: '100%', maxWidth: 500 }}>
                          <EndpointStatusDisplay />
                        </Box>
                        
                        <Typography variant="h5" color="text.secondary" gutterBottom>
                          Welcome to Communitas
                        </Typography>
                        <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
                          Select a contact, group, or organization to start collaborating
                        </Typography>
                        
                        <Stack spacing={2} alignItems="center">
                          <TestButton />
                          
                        </Stack>
                      </Box>
                    } />
                    <Route path="/group/:groupId" element={<GroupPage />} />
                    <Route path="/user/:userId" element={<UserPage />} />
                    <Route path="/test" element={<SimpleTest />} />
                    <Route path="/test/page" element={<TestPage />} />
                    <Route path="/test/collaboration" element={<CollaborativeEditingTest />} />
                    <Route path="/test/simple" element={<SimpleCollaborationTest />} />
                    <Route path="/dev/console" element={<MessageConsole />} />
                    <Route path="/dev/website" element={<WebsitePublishPanel />} />
                    <Route path="/org/:orgId/*" element={<UnifiedDashboard userId="user_owner_123" userName="Owner" />} />
                    <Route path="/project/:projectId/*" element={<UnifiedDashboard userId="user_owner_123" userName="Owner" />} />
                    <Route path="*" element={<Navigate to="/" replace />} />
                  </Routes>
                  </Suspense>
                </Box>
              </Box>
              
              {/* WebRTC Communication Hub removed to avoid duplicate FABs */}
              {/* Quick Actions in experimental UI */}
              <QuickActionsBar
                context={{ type: navigationContext.mode as any, entity: navigationContext }}
                onAction={handleQuickAction}
                position="bottom-right"
                notifications={0}
              />
            </Box>
          </BrowserRouter>
  
  {/* Overview Modal */}
  {showOverview && (
    <>
      <Box
        sx={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.5)',
          zIndex: 1299,
        }}
        onClick={() => setShowOverview(false)}
      />
      <OverviewDashboard 
        networkHealth={networkHealth} 
        onClose={() => setShowOverview(false)}
      />
    </>
  )}
  
  <LoginDialog open={authDialogOpen} onClose={() => setAuthDialogOpen(false)} />
  
  {/* Unified Storage Workspace Dialog */}
  <StorageWorkspaceDialog
    open={showStorageWorkspace}
    onClose={() => setShowStorageWorkspace(false)}
    entity={{
      entityId: navigationContext.organizationId || navigationContext.projectId || 'current-user',
      entityType: navigationContext.mode,
      entityName: navigationContext.organizationName || navigationContext.projectName || 'Your Storage',
      fourWords: navigationContext.fourWords,
    }}
  />
  
  {/* Entity Selector Dialog */}
  <EntitySelector
    open={showEntitySelector}
    onClose={() => {
      setShowEntitySelector(false)
      setPendingAction(null)
    }}
    onSelect={handleEntitySelected}
    actionType={pendingAction || 'call'}
  />
          </NavigationProvider>
        </EncryptionProvider>
      </AuthProvider>
    </TauriProvider>
  );

  return (
    <ThemeProvider>
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
            {ThemedApp}
          </Box>
        </SnackbarProvider>
      </ErrorBoundary>
    </ThemeProvider>
  )
}

export default App
