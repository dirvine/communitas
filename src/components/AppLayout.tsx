import React, { useState, useEffect, useCallback, useMemo } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import {
  AppBar,
  Toolbar,
  Typography,
  Box,
  IconButton,
  Button,
} from '@mui/material'
import {
  Menu as MenuIcon,
  ChevronLeft,
  ChevronRight,
  Lan as LanIcon,
  Home as HomeIcon,
} from '@mui/icons-material'
import useMediaQuery from '@mui/material/useMediaQuery'
import { NetworkHealth } from '../types'

// Feature Flags
import { featureFlags, useFeatureFlag } from '../services/featureFlags'

// Theme System
import { ThemeProvider, ThemeSwitcher } from './theme'

// Authentication System
import { AuthProvider, AuthStatus } from './auth'

// Encryption System
import { EncryptionProvider, EncryptionStatus } from './encryption'

// Responsive Layout
import { useSidebarBehavior } from './responsive'

// Navigation - both old and new
import { ModernNavigation } from './navigation/ModernNavigation'
import { NavigationProvider } from '../contexts/NavigationContext'
import BreadcrumbNavigation from './navigation/BreadcrumbNavigation'

// Mock data for testing
import { EntityDirectoryProvider } from '../contexts/EntityDirectoryContext'

// Tauri Context
import { TauriProvider } from '../contexts/TauriContext'
import { BrowserFallback } from './BrowserFallback'
import { isTauriApp } from '../utils/tauri'
import { ensureIdentity } from '../utils/identity'

// Components
import { LoginDialog } from './auth/LoginDialog'
import { EntitySelector } from './communication/EntitySelector'
import { GlobalSyncBar } from './sync/GlobalSyncBar'
import { CompactEndpointStatus } from './network/EndpointStatusDisplay'
import { NetworkStatusBar } from './NetworkStatusBar'

import OverviewDashboard from './OverviewDashboard'
import QuickActionsBar from './QuickActionsBar'
import StorageWorkspaceDialog from './storage/StorageWorkspaceDialog'

// Missing imports
import { ThemeProvider as MuiThemeProvider } from '@mui/material/styles'

interface AppLayoutProps {
  children: React.ReactNode
}

export const AppLayout: React.FC<AppLayoutProps> = ({ children }) => {
  const navigate = useNavigate()

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
  const [selectedEntity, setSelectedEntity] = useState<any>(null)
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

  // Check which features are enabled
  const useContextNav = useFeatureFlag('context-aware-navigation', 'user_owner_123')

  // Use responsive sidebar behavior
  const { defaultOpen } = useSidebarBehavior()
  const [sidebarOpen, setSidebarOpen] = useState(defaultOpen)
  const isSmall = useMediaQuery('(max-width:900px)')

  // Memoized handlers
  const handleToggleSidebar = useCallback(() => setSidebarOpen(o => !o), [])

  // Responsive sidebar behavior
  useEffect(() => {
    if (isSmall) setSidebarOpen(false)
  }, [isSmall])

  // Initialize features and identity
  useEffect(() => {
    // Enable all features
    featureFlags.enable('unified-design-system')
    featureFlags.enable('context-aware-navigation')
    featureFlags.enable('four-word-identity')
    featureFlags.enable('unified-storage-ui')

    // Load or generate identity
    ensureIdentity().then(four => {
      setNavigationContext(prev => ({ ...prev, fourWords: four }))
    }).catch(() => {
      // leave undefined; UI can handle missing identity
    })
  }, [])

  // Listen for global storage workspace open requests (from dashboards, etc.)
  useEffect(() => {
    const handler = (e: any) => {
      setShowStorageWorkspace(true)
    }
    window.addEventListener('open-storage-workspace' as any, handler)
    return () => window.removeEventListener('open-storage-workspace' as any, handler)
  }, [])

  // Network health monitoring
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

  // Memoized header component for better performance
  const HeaderComponent = React.memo(({ onMenuClick, showMenuButton }: {
    onMenuClick?: () => void;
    showMenuButton?: boolean;
  }) => {
    const handleCopyAddress = useCallback(async () => {
      const text = navigationContext.fourWords || 'local'
      try { await navigator.clipboard.writeText(text) } catch {}
    }, [navigationContext.fourWords])

    return (
      <Toolbar sx={{ gap: 1 }}>
        {showMenuButton && (
          <IconButton
            color="inherit"
            edge="start"
            onClick={onMenuClick}
            sx={{ mr: 2, flexShrink: 0 }}
            aria-label="Toggle sidebar"
          >
            <MenuIcon />
          </IconButton>
        )}
        <IconButton
          color="inherit"
          onClick={() => navigate('/')}
          sx={{ mr: 1 }}
          aria-label="Home"
        >
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
          {/* Network Status Bar with Four-Word identities and bootstrap connection */}
          <NetworkStatusBar onSyncClick={() => {
            // Trigger sync - will be implemented with backend sync operations
            console.log('Sync triggered from network status bar');
          }} />
          <Button
            variant="outlined"
            size="small"
            startIcon={<LanIcon />}
            onClick={handleCopyAddress}
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
    )
  })

  // Memoized environment check for better performance
  const environmentCheck = useMemo(() => {
    const isDevelopment = import.meta.env.DEV
    const showFullUI = isDevelopment || isTauriApp()
    return { isDevelopment, showFullUI }
  }, [])

  // Early return for browser fallback
  if (!environmentCheck.showFullUI) {
    return <BrowserFallback />;
  }

  return (
    <TauriProvider>
      <AuthProvider>
        <EncryptionProvider>
          <EntityDirectoryProvider>
            <NavigationProvider>
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
                        <ModernNavigation
                          currentUserId="user_owner_123"
                          onNavigate={(path, entity) => {
                            console.log('WhatsApp Navigation:', path, entity)
                            setSelectedEntity(entity)
                            navigate(path)
                          }}
                          onVideoCall={(entityId, entityType) => console.log('Video call:', entityId, entityType)}
                          onAudioCall={(entityId, entityType) => console.log('Audio call:', entityId, entityType)}
                          onScreenShare={(entityId, entityType) => console.log('Screen share:', entityId, entityType)}
                          onOpenFiles={(entityId, entityType) => console.log('Open files:', entityId, entityType)}
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
                    <ModernNavigation
                      currentUserId="user_owner_123"
                      onNavigate={(path, entity) => {
                        console.log('WhatsApp Navigation:', path, entity)
                        setSelectedEntity(entity)
                        navigate(path)
                      }}
                      onVideoCall={(entityId, entityType) => console.log('Video call:', entityId, entityType)}
                      onAudioCall={(entityId, entityType) => console.log('Audio call:', entityId, entityType)}
                      onScreenShare={(entityId, entityType) => console.log('Screen share:', entityId, entityType)}
                      onOpenFiles={(entityId, entityType) => console.log('Open files:', entityId, entityType)}
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
                  {children}
                </Box>
              </Box>

              {/* Quick Actions in experimental UI */}
              <QuickActionsBar
                context={{ type: navigationContext.mode as any, entity: navigationContext }}
                onAction={(action) => console.log('Quick action:', action)}
                position="bottom-right"
                notifications={0}
              />
            </Box>

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
              onSelect={(entity, type) => console.log('Entity selected:', entity, type)}
              actionType={pendingAction || 'call'}
            />
            </NavigationProvider>
          </EntityDirectoryProvider>
        </EncryptionProvider>
      </AuthProvider>
    </TauriProvider>
  )
}
