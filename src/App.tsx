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
  TextField,
  InputAdornment,
} from '@mui/material'
import {
  Menu as MenuIcon,
  Person,
  ChevronLeft,
  ChevronRight,
  Lan as LanIcon,
  Home as HomeIcon,
  Language as LanguageIcon,
  Search as SearchIcon,
  Person as PersonIcon,
  Router as RouterIcon,
} from '@mui/icons-material'
import useMediaQuery from '@mui/material/useMediaQuery'
import { SnackbarProvider } from 'notistack'
import { NetworkHealth } from './types'

// Feature Flags
import { featureFlags, useFeatureFlag } from './services/featureFlags'

// Theme System
import { ThemeProvider, ThemeSwitcher } from './components/theme'
import { theme as modernTheme, darkTheme as modernDarkTheme } from './styles/theme'
import { ThemeProvider as MuiThemeProvider } from '@mui/material/styles'

// Authentication System
import { AuthProvider, AuthStatus, useAuth } from './components/auth'

// Network Connection Service
import { NetworkConnectionService } from './services/network/NetworkConnectionService'

// Encryption System
import { EncryptionProvider, EncryptionStatus } from './components/encryption'

  // Responsive Layout  
  import { useSidebarBehavior } from './components/responsive'
  import ResponsiveLayout from './components/layout/ResponsiveLayout'

// Modern UI Components
import { GlassCard } from './components/ui/GlassCard'
import { ModernButton } from './components/ui/ModernButton'
import { ModernLoader } from './components/ui/ModernLoader'

// Navigation - both old and new
import { ModernNavigation } from './components/navigation/ModernNavigation'
import { UIShowcase } from './components/UIShowcase'
import { NavigationProvider } from './contexts/NavigationContext'
import BreadcrumbNavigation from './components/navigation/BreadcrumbNavigation'
import ContextAwareSidebar from './components/navigation/ContextAwareSidebar'
import { ModernShellPrototypeScreen } from './components/prototype/ModernShellPrototype'

// Mock data for testing
import { EntityDirectoryProvider, useEntityDirectory } from './contexts/EntityDirectoryContext'

// Tauri Context
import { TauriProvider } from './contexts/TauriContext'
import { BrowserFallback } from './components/BrowserFallback'
import { isTauriApp } from './utils/tauri'
import { ensureIdentity } from './utils/identity'

// WebRTC Communication
import { SimpleCommunicationHub } from './components/webrtc'
import { LoginDialog } from './components/auth/LoginDialog'
import { UnifiedAuthFlow } from './components/auth/UnifiedAuthFlow'
import { EnhancedEntityDialog } from './components/entity/EnhancedEntityDialog'

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

// import FirstRunWizard from './components/onboarding/FirstRunWizard'
import QuickActionsBar, { SettingsButton } from './components/QuickActionsBar'
import StorageWorkspaceDialog from './components/storage/StorageWorkspaceDialog'

const IdentityTab = React.lazy(() => import('./components/tabs/IdentityTab'))
const WebsitePublishPanel = React.lazy(() => import('./components/dev/WebsitePublishPanel'))
const UnifiedDashboard = React.lazy(() => import('./components/unified/UnifiedDashboard').then(m => ({ default: m.UnifiedDashboard })))
// Commented out - missing test components
// const CollaborativeEditingTest = React.lazy(() => import('./components/testing/CollaborativeEditingTest').then(m => ({ default: m.CollaborativeEditingTest })))
// const SimpleCollaborationTest = React.lazy(() => import('./components/testing/SimpleCollaborationTest').then(m => ({ default: m.SimpleCollaborationTest })))
// const TestPage = React.lazy(() => import('./components/testing/TestPage').then(m => ({ default: m.TestPage })))
// const SimpleTest = React.lazy(() => import('./components/testing/SimpleTest').then(m => ({ default: m.SimpleTest })))
const MessageConsole = React.lazy(() => import('./components/dev/MessageConsole').then(m => ({ default: m.MessageConsole })))
const GroupPage = React.lazy(() => import('./components/pages/GroupPage').then(m => ({ default: m.GroupPage })))
const UserPage = React.lazy(() => import('./components/pages/UserPage').then(m => ({ default: m.UserPage })))
const ChannelPage = React.lazy(() => import('./components/pages/ChannelPage').then(m => ({ default: m.ChannelPage })))
const ProjectPage = React.lazy(() => import('./components/pages/ProjectPage').then(m => ({ default: m.ProjectPage })))
const OrganizationViewWrapper = React.lazy(() => import('./components/views/OrganizationViewWrapper').then(m => ({ default: m.OrganizationViewWrapper })))

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


// Inner component that has access to useNavigate() from BrowserRouter
function AppContent() {
  const navigate = useNavigate(); // Now we can use this hook!
  const location = useLocation();

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

  const [authDialogOpen, setAuthDialogOpen] = useState(false)
  const [_selectedEntity, _setSelectedEntity] = useState<any>(null)
  const [showStorageWorkspace, setShowStorageWorkspace] = useState(false)
  const [showEntitySelector, setShowEntitySelector] = useState(false)
  const [pendingAction, setPendingAction] = useState<'call' | 'video' | 'screen' | 'storage' | null>(null)
  const [showContactDialog, setShowContactDialog] = useState(false)
  const [networkState, setNetworkState] = useState<any>(null)
  const [networkHealth, setNetworkHealth] = useState<NetworkHealth>({
    status: 'Disconnected',
    peer_count: 0,
    nat_type: 'Unknown',
    bandwidth_kbps: 0,
    avg_latency_ms: 0,
  })

  // Theme state
  const [isDarkMode, setIsDarkMode] = useState(() => {
    const saved = localStorage.getItem('theme-mode')
    return saved === 'dark'
  })
  const currentTheme = isDarkMode ? modernDarkTheme : modernTheme
  
  // Use responsive sidebar behavior with proper defaults
  const { defaultOpen } = useSidebarBehavior()
  const [sidebarOpen, setSidebarOpen] = useState(defaultOpen)
  const handleToggleSidebar = () => setSidebarOpen(o => !o)

  useEffect(() => {
    // Check if user already has an identity stored (don't auto-generate)
    const storedFourWords = localStorage.getItem('communitas-four-words')
    if (storedFourWords) {
      // User has previously logged in, set their four-words
      setNavigationContext(prev => ({ ...prev, fourWords: storedFourWords }))
    }
    // Don't auto-generate identity - wait for user to sign in

    // Initialize network connection service (auto-connects on startup)
    console.log('🚀 Initializing network connection service...')
    // Network service will auto-connect in its constructor

    // Subscribe to network state updates
    const updateNetworkState = (state: any) => {
      setNetworkState(state);
    };

    const unsubscribe = networkService.subscribe(updateNetworkState);

    return unsubscribe;
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
        const res = await (await import('@tauri-apps/api/core')).invoke<any>('health')
        if (!mounted) return
        setNetworkHealth({
          status: res.status === 'ok' ? 'Connected' : 'Disconnected',
          peer_count: 0, // health command doesn't return peer count
          nat_type: 'Unknown',
          bandwidth_kbps: 0,
          avg_latency_ms: 0,
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
      case 'add_contact':
        // Open the EnhancedEntityDialog for adding contacts
        setShowContactDialog(true)
        break
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

    // Construct full path with entity ID
    let fullPath = path;
    if (entity && entity.id) {
      // If path doesn't already include the ID, append it
      if (!path.includes(entity.id)) {
        fullPath = `${path}/${entity.id}`;
      }
    }

    // CRITICAL: Navigate to the path using React Router
    navigate(fullPath);

    // Update navigation context based on path
    if (path.startsWith('/org/') || path.startsWith('/organization')) {
      const orgId = entity?.id || fullPath.split('/').pop();
      const orgName = (entity && 'name' in entity) ? entity.name : 'Organization'
      const fourWords = (entity && entity.networkIdentity?.fourWords) || 'unknown-org'
      setNavigationContext({
        mode: 'organization',
        organizationId: orgId || '',
        organizationName: orgName,
        fourWords,
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
    const { authState } = useAuth();
    const { organizations } = useEntityDirectory();
    const [urlBarValue, setUrlBarValue] = React.useState('');
    const [urlBarFocused, setUrlBarFocused] = React.useState(false);
    const [urlBarError, setUrlBarError] = React.useState('');

    // Four-word address validation
    const validateFourWords = (input: string): boolean => {
      const pattern = /^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/;
      return pattern.test(input.trim().toLowerCase());
    };

    // Handle URL bar navigation
    const handleUrlNavigation = (fourWordAddress: string) => {
      const cleanAddress = fourWordAddress.trim().toLowerCase();
      if (validateFourWords(cleanAddress)) {
        console.log('Navigating to four-word address:', cleanAddress);
        
        // Check if this is a known entity from our mock data
        const matchingOrg = organizations.find(org => 
          org.networkIdentity.fourWords === cleanAddress
        );
        
        if (matchingOrg) {
          // Navigate to organization view
          const orgPath = `/org/${matchingOrg.id}`;
          navigate(orgPath);
          
          // Update navigation context for organization
          setNavigationContext({
            mode: 'organization',
            organizationId: matchingOrg.id,
            organizationName: matchingOrg.name,
            fourWords: cleanAddress,
          });
          
          console.log(`✅ Navigated to organization: ${matchingOrg.name}`);
        } else {
          // For unknown addresses, navigate to a generic network view
          // Update navigation context for network browsing
          setNavigationContext({
            mode: 'personal',
            fourWords: cleanAddress,
          });
          
          // Navigate to home with the four-word context
          navigate('/');
          
          console.log(`✅ Set network context for: ${cleanAddress}`);
        }
        
        setUrlBarValue(cleanAddress);
        setUrlBarError(''); // Clear any previous errors
      } else {
        console.warn('Invalid four-word address format:', cleanAddress);
        
        // Show visual error feedback
        setUrlBarError('Invalid format. Please use: word-word-word-word');
        
        console.error(`❌ Invalid address format. Please use: word-word-word-word`);
      }
    };

    // Handle Enter key press in URL bar
    const handleUrlBarKeyPress = (event: React.KeyboardEvent) => {
      if (event.key === 'Enter') {
        handleUrlNavigation(urlBarValue);
      }
    };

    // Update URL bar when navigation context changes
    React.useEffect(() => {
      if (navigationContext.fourWords && !urlBarFocused) {
        setUrlBarValue(navigationContext.fourWords);
      }
    }, [navigationContext.fourWords, urlBarFocused]);


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
          flexShrink: 0,
        }}
      >
        Communitas
      </Typography>

      {/* Identity Display - Your Identity and Connection Location */}
      <Box sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        gap: { xs: 0.5, md: 1 }, 
        mx: { xs: 0.5, md: 1 },
        flexShrink: 0
      }}>
        {/* User's Own Identity */}
        <Tooltip title="Your identity - share this for others to find you" arrow>
          <Chip
            icon={<PersonIcon sx={{ fontSize: { xs: '0.8rem', md: '1rem' } }} />}
            label={authState.user?.fourWordAddress || 'Not connected'}
            variant="outlined"
            size="small"
            sx={{
              fontFamily: 'monospace',
              fontSize: { xs: '0.65rem', md: '0.75rem' },
              backgroundColor: 'background.paper',
              borderColor: 'primary.main',
              color: 'primary.main',
              '& .MuiChip-icon': {
                color: 'primary.main'
              },
              cursor: 'pointer',
              maxWidth: { xs: '120px', sm: '180px', md: 'none' },
              '& .MuiChip-label': {
                overflow: 'hidden',
                textOverflow: 'ellipsis',
              }
            }}
            onClick={() => {
              if (authState.user?.fourWordAddress) {
                navigator.clipboard.writeText(authState.user.fourWordAddress);
                console.log('📋 Copied your identity to clipboard:', authState.user.fourWordAddress);
              }
            }}
          />
        </Tooltip>

        {/* Current Connection Location */}
        <Tooltip title="Your connection location - share this for others to bootstrap from you" arrow>
          <Chip
            icon={<RouterIcon sx={{ fontSize: { xs: '0.8rem', md: '1rem' } }} />}
            label={networkState?.endpointFourWords || 'local-mode'}
            variant="outlined"
            size="small"
            sx={{
              fontFamily: 'monospace',
              fontSize: { xs: '0.65rem', md: '0.75rem' },
              backgroundColor: 'background.paper',
              borderColor: 'secondary.main',
              color: 'secondary.main',
              '& .MuiChip-icon': {
                color: 'secondary.main'
              },
              cursor: 'pointer',
              maxWidth: { xs: '120px', sm: '180px', md: 'none' },
              '& .MuiChip-label': {
                overflow: 'hidden',
                textOverflow: 'ellipsis',
              }
            }}
            onClick={() => {
              const location = networkState?.endpointFourWords || 'local-mode';
              navigator.clipboard.writeText(location);
              console.log('📋 Copied connection location to clipboard:', location);
            }}
          />
        </Tooltip>
      </Box>
      
      {/* Four-Word Address URL Bar */}
      <Box sx={{ 
        display: { xs: 'none', sm: 'flex' }, 
        flexGrow: 1, 
        mx: 2, 
        maxWidth: 400 
      }}>
        <TextField
          value={urlBarValue}
          onChange={(e) => {
            setUrlBarValue(e.target.value);
            if (urlBarError) setUrlBarError(''); // Clear error on typing
          }}
          onKeyPress={handleUrlBarKeyPress}
          onFocus={() => setUrlBarFocused(true)}
          onBlur={() => setUrlBarFocused(false)}
          placeholder="Enter four-word address (e.g., ocean blue eagle star)"
          size="small"
          fullWidth
          error={!!urlBarError}
          helperText={urlBarError}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <LanguageIcon sx={{ color: 'text.secondary', fontSize: '1.1rem' }} />
              </InputAdornment>
            ),
            endAdornment: urlBarValue && (
              <InputAdornment position="end">
                <IconButton 
                  size="small" 
                  onClick={() => handleUrlNavigation(urlBarValue)}
                  sx={{ p: 0.5 }}
                >
                  <SearchIcon sx={{ fontSize: '1rem' }} />
                </IconButton>
              </InputAdornment>
            ),
            sx: {
              backgroundColor: 'background.paper',
              borderRadius: 3,
              '& .MuiOutlinedInput-notchedOutline': {
                borderColor: 'divider',
                borderWidth: 1,
              },
              '&:hover .MuiOutlinedInput-notchedOutline': {
                borderColor: 'primary.main',
              },
              '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
                borderColor: 'primary.main',
                borderWidth: 2,
              }
            }
          }}
          sx={{
            '& .MuiInputBase-input': {
              fontSize: '0.9rem',
              fontFamily: 'monospace',
              py: 1,
            }
          }}
        />
      </Box>
      <Box sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        gap: { xs: 0.5, sm: 1 },
        flexShrink: 0,
        ml: 'auto',
      }}>
        {/* Compact Endpoint Status showing connection state */}
        <CompactEndpointStatus />
        <Box sx={{ display: { xs: 'none', md: 'flex' }, alignItems: 'center', gap: 1 }}>
          <EncryptionStatus compact={true} />
          <ThemeSwitcher compact showPresets />
        </Box>
        <AuthStatus compact={true} showLabel={false} />
        <SettingsButton onAction={handleQuickAction} />
      </Box>
    </Toolbar>
  )}

  // Check if running in Tauri or browser
  // Show full UI in development mode or when in Tauri
  // In development, always show full UI to enable testing
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

  if (location.pathname.startsWith('/prototype/modern-shell')) {
    return (
      <ThemeProvider>
        <MuiThemeProvider theme={modernDarkTheme}>
          <ModernShellPrototypeScreen />
        </MuiThemeProvider>
      </ThemeProvider>
    );
  }

  const ThemedApp = (
    <TauriProvider>
      <AuthProvider>
        <EncryptionProvider>
          <EntityDirectoryProvider>
            <NavigationProvider>
          {/** Bridge navigation events to React Router */}
          {/* NavBridge temporarily disabled - causing infinite render loop
          {(() => {
            // NavBridge component moved outside to prevent re-creation
            return <NavBridge />
          })()} */}
          {/* Global Sync Status Bar */}
          <GlobalSyncBar 
            userId="user_owner_123" // TODO: Use actual authenticated user ID
            position="top"
            autoHide={true}
            autoHideDelay={5000}
          />
          
          {/* Conditionally render breadcrumb navigation */}
          {useContextNav && <BreadcrumbNavigation />}
          
          {/* Using ResponsiveLayout component for proper responsive behavior */}
          <ResponsiveLayout
            sidebarOpen={sidebarOpen}
            onSidebarToggle={handleToggleSidebar}
            header={
              <HeaderComponent 
                onMenuClick={handleToggleSidebar}
                showMenuButton={true}
              />
            }
            sidebar={
              <ModernNavigation
                currentUserId="user_owner_123"
                onNavigate={handleWhatsAppNavigate}
                onVideoCall={handleVideoCall}
                onAudioCall={handleAudioCall}
                onScreenShare={handleScreenShare}
                onOpenFiles={handleOpenFiles}
              />
            }
            maxWidth="xl"
          >
            <Box sx={{ height: '100%', bgcolor: 'grey.50' }}>
              <Suspense fallback={<Box sx={{ p: 3 }}><Typography>Loading…</Typography></Box>}>
                <Routes>
                  <Route path="/" element={<UnifiedDashboard userId="user_owner_123" userName="Owner" />} />
                  <Route path="/ui-showcase" element={<UIShowcase />} />
                  <Route path="/group/:groupId" element={<GroupPage />} />
                  <Route path="/user/:userId" element={<UserPage />} />
                  <Route path="/channel/:channelId" element={<ChannelPage />} />
                  <Route path="/project/:projectId" element={<ProjectPage />} />
                  <Route path="/prototype/modern-shell" element={<ModernShellPrototypeScreen />} />
                  <Route path="/org/:orgId/channel/:channelId" element={<ChannelPage />} />
                  <Route path="/org/:orgId/project/:projectId" element={<ProjectPage />} />
                  <Route path="/org/:orgId/group/:groupId" element={<GroupPage />} />
                  <Route path="/org/:orgId/user/:userId" element={<UserPage />} />
                  {/* Commented out - missing test components
                  <Route path="/test" element={<SimpleTest />} />
                  <Route path="/test/page" element={<TestPage />} />
                  <Route path="/test/collaboration" element={<CollaborativeEditingTest />} />
                  <Route path="/test/simple" element={<SimpleCollaborationTest />} /> */}
                  <Route path="/dev/console" element={<MessageConsole />} />
                  <Route path="/dev/website" element={<WebsitePublishPanel />} />
                  <Route path="/organization/:orgId" element={<OrganizationViewWrapper />} />
                  <Route path="/org/:orgId/*" element={<UnifiedDashboard userId="user_owner_123" userName="Owner" />} />
                  <Route path="*" element={<Navigate to="/" replace />} />
                </Routes>
              </Suspense>
            </Box>
            
            {/* Quick Actions in experimental UI */}
            <QuickActionsBar
              context={{ type: navigationContext.mode as any, entity: navigationContext }}
              onAction={handleQuickAction}
              position="bottom-right"
              notifications={0}
            />
          </ResponsiveLayout>

  {/* Replace LoginDialog with UnifiedAuthFlow */}
  {authDialogOpen && (
    <Box
      sx={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        zIndex: 9999,
      }}
    >
      <UnifiedAuthFlow
        initialMode="login"
        onSuccess={() => setAuthDialogOpen(false)}
        onCancel={() => setAuthDialogOpen(false)}
      />
    </Box>
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

  {/* Enhanced Contact Dialog */}
  <EnhancedEntityDialog
    open={showContactDialog}
    onClose={() => setShowContactDialog(false)}
    entityType="contact"
    isOnline={networkState?.status === 'connected'}
  />
            </NavigationProvider>
          </EntityDirectoryProvider>
        </EncryptionProvider>
      </AuthProvider>
    </TauriProvider>
  );

  return (
    <ThemeProvider>
      <MuiThemeProvider theme={currentTheme}>
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
      </MuiThemeProvider>
    </ThemeProvider>
  )
}

// Main App wrapper that provides BrowserRouter context
function App() {
  return (
    <BrowserRouter>
      <AppContent />
    </BrowserRouter>
  )
}

export default App
