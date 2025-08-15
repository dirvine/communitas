import React, { useState, useEffect, useMemo } from 'react'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import {
  AppBar,
  Toolbar,
  Typography,
  Box,
  IconButton,
  Button,
  Fab,
  Tooltip,
  Badge,
  Switch,
  FormControlLabel,
  Paper,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
} from '@mui/material'
import { 
  Menu as MenuIcon, 
  Person,
  Science as ExperimentalIcon,
  Notifications,
  Search,
} from '@mui/icons-material'
import { SnackbarProvider } from 'notistack'
import { NetworkHealth } from './types'

// Feature Flags
import { featureFlags, useFeatureFlag } from './services/featureFlags'

// Theme System - both old and new
import { ThemeProvider, ThemeSwitcher } from './components/theme'
import { ThemeProvider as UnifiedThemeProvider } from '@mui/material/styles'
import unifiedTheme from './theme/unified'

// Authentication System
import { AuthProvider, AuthStatus } from './components/auth'

// Encryption System
import { EncryptionProvider, EncryptionStatus } from './components/encryption'

// Responsive Layout
import { ResponsiveLayout, useSidebarBehavior } from './components/responsive'

// Navigation - both old and new
import EnhancedNavigation from './components/navigation/EnhancedNavigation'
import UnifiedNavigation from './components/navigation/UnifiedNavigation'
import { WhatsAppStyleNavigation } from './components/navigation/WhatsAppStyleNavigation'
import { NavigationProvider } from './contexts/NavigationContext'
import BreadcrumbNavigation from './components/navigation/BreadcrumbNavigation'
import ContextAwareSidebar from './components/navigation/ContextAwareSidebar'

// Collaboration components
import { UnifiedFileSharing } from './components/collaboration/UnifiedFileSharing'

// Mock data for testing
import { 
  mockOrganizations, 
  mockPersonalGroups, 
  mockPersonalUsers,
  mockSharedFiles,
  mockPublishedWebsite 
} from './data/mockCollaborationData'

// Tauri Context
import { TauriProvider } from './contexts/TauriContext'
import { BrowserFallback } from './components/BrowserFallback'
import { isTauriApp } from './utils/tauri'
import { ensureIdentity } from './utils/identity'

// WebRTC Communication
import { SimpleCommunicationHub } from './components/webrtc'

// Real-time Sync
import { GlobalSyncBar } from './components/sync/GlobalSyncBar'

// Tab panels
import OrganizationTab from './components/tabs/OrganizationTab'
import GroupsAndPeopleTab from './components/tabs/GroupsAndPeopleTab'
import OverviewDashboard from './components/OverviewDashboard'
import IdentityTab from './components/tabs/IdentityTab'

// Unified components
import UnifiedHome from './components/unified/UnifiedHome'
import { UnifiedDashboard } from './components/unified/UnifiedDashboard'
import FirstRunWizard from './components/onboarding/FirstRunWizard'
import QuickActionsBar from './components/QuickActionsBar'
import StorageWorkspaceDialog from './components/storage/StorageWorkspaceDialog'

interface TabPanelProps {
  children?: React.ReactNode
  index: number
  value: number
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      {...other}
    >
      {value === index && (
        <Box 
          sx={{ 
            height: { xs: 'calc(100vh - 56px)', sm: 'calc(100vh - 64px)' }, 
            overflow: 'auto',
            p: { xs: 0, sm: 1 },
          }}
        >
          {children}
        </Box>
      )}
    </div>
  )
}

function App() {
  // Feature flags for progressive migration
  const [experimentalMode, setExperimentalMode] = useState(() => {
    return localStorage.getItem('communitas-experimental-mode') === 'true'
  })
  
  // Check which features are enabled
  const useUnifiedUI = useFeatureFlag('unified-design-system', 'user_owner_123')
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

  const [currentTab, setCurrentTab] = useState(0)
  const [showOverview, setShowOverview] = useState(false)
  const [showIdentity, setShowIdentity] = useState(false)
  const [selectedEntity, setSelectedEntity] = useState<any>(null)
  const [showFileSharing, setShowFileSharing] = useState(false)
  const [showStorageWorkspace, setShowStorageWorkspace] = useState(false)
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

  useEffect(() => {
    // Load or generate identity
    ensureIdentity().then(four => {
      setNavigationContext(prev => ({ ...prev, fourWords: four }))
    }).catch(() => {
      // leave undefined; UI can handle missing identity
    })

    // Initialize feature flags based on experimental mode
    if (experimentalMode) {
      featureFlags.enable('unified-design-system')
      featureFlags.enable('context-aware-navigation')
      featureFlags.enable('four-word-identity')
      featureFlags.enable('unified-storage-ui')
    }
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

  const handleTabChange = (newValue: number) => {
    // Handle special cases for Overview and Identity modals
    if (newValue === 2) {
      setShowOverview(true)
    } else if (newValue === 3) {
      setShowIdentity(true)
    } else {
      setCurrentTab(newValue)
    }
  }

  const handleToggleSidebar = () => {
    setSidebarOpen(!sidebarOpen)
  }

  // Collaboration feature handlers
  const handleVideoCall = (entityId: string, entityType: string) => {
    console.log('Starting video call for', entityType, entityId)
    // TODO: Integrate with WebRTC implementation
  }

  const handleAudioCall = (entityId: string, entityType: string) => {
    console.log('Starting audio call for', entityType, entityId)
    // TODO: Integrate with WebRTC implementation
  }

  const handleScreenShare = (entityId: string, entityType: string) => {
    console.log('Starting screen share for', entityType, entityId)
    // TODO: Integrate with WebRTC implementation
  }

  const handleOpenFiles = (entityId: string, entityType: string) => {
    console.log('Opening files for', entityType, entityId)
    setSelectedEntity({ id: entityId, type: entityType })
    // Use the unified storage workspace dialog instead of the basic file sharing dialog
    setShowStorageWorkspace(true)
  }

  const handleQuickAction = (action: string) => {
    const currentType = navigationContext.mode
    const currentId = navigationContext.organizationId || navigationContext.projectId || 'current-user'
    switch (action) {
      case 'start_voice_call':
        handleAudioCall(currentId, currentType)
        break
      case 'start_video_call':
        handleVideoCall(currentId, currentType)
        break
      case 'upload_documents':
      case 'storage_settings':
      case 'upload_files':
      case 'open_chat':
      default:
        handleOpenFiles(currentId, currentType)
        break
    }
  }

  const handleWhatsAppNavigate = (path: string, entity: any) => {
    console.log('WhatsApp Navigation:', path, entity)
    setSelectedEntity(entity)
    
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
  }) => (
    <Toolbar>
      {showMenuButton && (
        <IconButton
          color="inherit"
          edge="start"
          onClick={onMenuClick}
          sx={{ mr: 2 }}
        >
          <MenuIcon />
        </IconButton>
      )}
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
          fontSize: { xs: '1rem', sm: '1.25rem' },
        }}
      >
        Communitas - P2P Collaboration Platform
      </Typography>
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        {/* Experimental Mode Toggle */}
        <Tooltip title="Enable experimental unified UI">
          <FormControlLabel
            control={
              <Switch
                checked={experimentalMode}
                onChange={(e) => {
                  const enabled = e.target.checked
                  setExperimentalMode(enabled)
                  localStorage.setItem('communitas-experimental-mode', enabled.toString())
                  
                  // Enable/disable Phase 1 features
                  if (enabled) {
                    featureFlags.enable('unified-design-system')
                    featureFlags.enable('context-aware-navigation')
                    featureFlags.enable('four-word-identity')
                    featureFlags.enable('unified-storage-ui')
                  } else {
                    featureFlags.disable('unified-design-system')
                    featureFlags.disable('context-aware-navigation')
                    featureFlags.disable('four-word-identity')
                    featureFlags.disable('unified-storage-ui')
                  }
                  
                  // Reload to apply changes
                  window.location.reload()
                }}
                icon={<ExperimentalIcon />}
                checkedIcon={<ExperimentalIcon />}
                size="small"
              />
            }
            label=""
            sx={{ m: 0 }}
          />
        </Tooltip>
        <Button
          variant="outlined"
          size="small"
          startIcon={<Person />}
          onClick={() => setShowIdentity(true)}
          sx={{ 
            borderColor: 'rgba(255,255,255,0.3)',
            color: 'inherit',
            '&:hover': {
              borderColor: 'rgba(255,255,255,0.5)',
              backgroundColor: 'rgba(255,255,255,0.1)',
            }
          }}
        >
          Identity
        </Button>
        <EncryptionStatus compact={true} />
        {!experimentalMode && <ThemeSwitcher compact showPresets />}
        <AuthStatus compact={true} showLabel={false} />
      </Box>
    </Toolbar>
  )

  // Check if running in Tauri or browser
  // Show full UI in development browser mode, otherwise show fallback
  const showFullUI = isTauriApp()
  
  if (!showFullUI) {
    return (
      <ThemeProvider>
        <BrowserFallback />
      </ThemeProvider>
    );
  }

  // Handle navigation from unified navigation
  const handleUnifiedNavigate = (path: string) => {
    console.log('Navigate to:', path)
    
    // Parse the path to update context
    if (path.startsWith('/org/')) {
      const parts = path.split('/')
      const orgId = parts[2]
      setNavigationContext({
        mode: 'organization',
        organizationId: orgId,
        organizationName: 'Acme Corp', // TODO: Fetch from store
        fourWords: 'acme-global-secure-network',
      })
    } else if (path.startsWith('/project/')) {
      const parts = path.split('/')
      const projectId = parts[2]
      setNavigationContext({
        mode: 'project',
        projectId: projectId,
        projectName: 'Project Alpha', // TODO: Fetch from store
        fourWords: 'alpha-mission-space-explore',
      })
    } else {
      setNavigationContext({
        mode: 'personal',
        fourWords: 'ocean-forest-moon-star',
      })
    }
    
    // TODO: Implement actual routing
  }

  // Render theme provider conditionally to keep prop types correct

  // First-run gate stored in localStorage for now
  const [firstRunOpen, setFirstRunOpen] = useState(() => {
    return localStorage.getItem('communitas-onboarded') !== 'true'
  })

  const handleWizardClose = () => {
    localStorage.setItem('communitas-onboarded', 'true')
    setFirstRunOpen(false)
  }

  const ThemedApp = (
    <TauriProvider>
      <AuthProvider>
        <EncryptionProvider>
          <NavigationProvider>
          <FirstRunWizard open={firstRunOpen} onClose={handleWizardClose} />
          <BrowserRouter>
          {/* Global Sync Status Bar */}
          <GlobalSyncBar 
            userId="user_owner_123" // TODO: Use actual authenticated user ID
            position="top"
            autoHide={true}
            autoHideDelay={5000}
          />
          
          {/* Conditionally render breadcrumb navigation */}
          {useContextNav && <BreadcrumbNavigation />}
          
          {/* Conditional UI rendering based on feature flags */}
          {(experimentalMode || useContextNav) ? (
            // New WhatsApp-style UI
            <Box sx={{ display: 'flex', height: '100vh' }}>
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
              <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
                <AppBar position="sticky" elevation={0}>
                  <HeaderComponent 
                    onMenuClick={handleToggleSidebar}
                    showMenuButton={false}
                  />
                </AppBar>
                <Box sx={{ flex: 1, overflow: 'auto', bgcolor: 'grey.50' }}>
                  <Routes>
                    <Route path="/" element={
                      <Box sx={{ 
                        display: 'flex', alignItems: 'center', justifyContent: 'center',
                        height: '100%', flexDirection: 'column' }}>
                        <Typography variant="h5" color="text.secondary" gutterBottom>
                          Welcome to Communitas
                        </Typography>
                        <Typography variant="body1" color="text.secondary">
                          Select a contact, group, or organization to start collaborating
                        </Typography>
                      </Box>
                    } />
                    <Route path="/org/:orgId/*" element={<UnifiedDashboard userId="user_owner_123" userName="Owner" />} />
                    <Route path="/project/:projectId/*" element={<UnifiedDashboard userId="user_owner_123" userName="Owner" />} />
                    <Route path="*" element={<Navigate to="/" replace />} />
                  </Routes>
                </Box>
                {/* Floating Quick Actions */}
                <QuickActionsBar
                  context={{ type: navigationContext.mode as any, entity: navigationContext }}
                  onAction={handleQuickAction}
                  position="bottom-right"
                  notifications={0}
                />
              </Box>
              
              {/* WebRTC Communication Hub - Global overlay */}
              <SimpleCommunicationHub />
            </Box>
          ) : (
            // Legacy UI
            <ResponsiveLayout
              header={<HeaderComponent />}
              sidebar={
                <ContextAwareSidebar
                  currentUserId="user_owner_123"
                />
              }
              sidebarOpen={sidebarOpen}
              onSidebarToggle={handleToggleSidebar}
              maxWidth="xl"
            >
              {/* Tab Panels */}
              <TabPanel value={currentTab} index={0}>
                <OrganizationTab />
              </TabPanel>
              <TabPanel value={currentTab} index={1}>
                <GroupsAndPeopleTab />
              </TabPanel>
              
              {/* WebRTC Communication Hub - Global overlay */}
              <SimpleCommunicationHub />
              {/* Quick Actions in legacy UI */}
              <QuickActionsBar
                context={{ type: navigationContext.mode as any, entity: navigationContext }}
                onAction={handleQuickAction}
                position="bottom-right"
                notifications={0}
              />
            </ResponsiveLayout>
          )}
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
  
  {/* Identity Modal */}
  {showIdentity && (
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
        onClick={() => setShowIdentity(false)}
      />
      <Box
        sx={{
          position: 'fixed',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          width: '90%',
          maxWidth: 1200,
          height: '80vh',
          backgroundColor: 'background.paper',
          borderRadius: 2,
          overflow: 'auto',
          zIndex: 1300,
          p: 3,
        }}
      >
        <IdentityTab onClose={() => setShowIdentity(false)} />
      </Box>
    </>
  )}
  
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
          </NavigationProvider>
        </EncryptionProvider>
      </AuthProvider>
    </TauriProvider>
  );

  return (
    <SnackbarProvider maxSnack={3} anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}>
      {experimentalMode ? (
        <UnifiedThemeProvider theme={unifiedTheme.light}>{ThemedApp}</UnifiedThemeProvider>
      ) : (
        <ThemeProvider>{ThemedApp}</ThemeProvider>
      )}
    </SnackbarProvider>
  )
}

export default App
