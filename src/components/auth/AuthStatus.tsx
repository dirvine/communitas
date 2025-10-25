import {
    AccountCircle as AccountCircleIcon,
    ArrowDropDown as ArrowDropDownIcon, Key as KeyIcon, Login as LoginIcon,
    Logout as LogoutIcon, NetworkCheck as NetworkIcon, Person as PersonIcon, Security as SecurityIcon, Settings as SettingsIcon
} from '@mui/icons-material';
import {
    Alert, Avatar, Box, Button, Chip, CircularProgress,
    Dialog, DialogActions, DialogContent, DialogTitle, Divider, List,
    ListItem, ListItemIcon,
    ListItemText, Menu,
    MenuItem, Stack, Tooltip, Typography
} from '@mui/material';
import React, { useState } from 'react';
import { useAuth } from '../../contexts/AuthContext';
import { useNavigation } from '../../contexts/NavigationContext';
import { UnifiedAuthFlow } from './UnifiedAuthFlow';
// Removed: ProfileManager - using ModernShellPrototype instead
import SettingsInterface from '../settings/SettingsInterface';

interface AuthStatusProps {
  compact?: boolean;
  showLabel?: boolean;
}

export const AuthStatus: React.FC<AuthStatusProps> = ({
  compact = false,
  showLabel = true,
}) => {
  const { authState, logout, getNetworkStatus } = useAuth();
  const { switchToPersonal, selectEntity } = useNavigation();
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [loginDialogOpen, setLoginDialogOpen] = useState(false);
  const [loginInitialMode, setLoginInitialMode] = useState<'login' | 'register'>('login');
  const [profileDialogOpen, setProfileDialogOpen] = useState(false);
  const [settingsDialogOpen, setSettingsDialogOpen] = useState(false);
  const [securityKeysDialogOpen, setSecurityKeysDialogOpen] = useState(false);
  const [networkStatus, setNetworkStatus] = useState<{ connected: boolean; peers: number } | null>(null);
  const [loggingOut, setLoggingOut] = useState(false);

  console.log('🟡 AuthStatus render - loginDialogOpen:', loginDialogOpen);


  const open = Boolean(anchorEl);

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    console.log('🔵 AuthStatus avatar clicked!', authState.isAuthenticated);
    if (authState.isAuthenticated) {
      // Open menu directly when avatar is clicked for better UX
      setAnchorEl(event.currentTarget);
      // Update network status when menu opens
      updateNetworkStatus();
    } else {
      setLoginInitialMode('login');
      setLoginDialogOpen(true);
    }
  };


  const handleClose = () => {
    setAnchorEl(null);
  };

  const updateNetworkStatus = async () => {
    try {
      const status = await getNetworkStatus();
      setNetworkStatus(status);
    } catch (error) {
      console.error('Failed to get network status:', error);
    }
  };

  const handleLogout = async () => {
    setLoggingOut(true);
    try {
      await logout();
    } catch (error) {
      console.error('Logout failed:', error);
    } finally {
      setLoggingOut(false);
      handleClose();
    }
  };

  const handleOpenProfile = () => {
    setProfileDialogOpen(true);
    handleClose();
  };

  const handleOpenSettings = () => {
    setSettingsDialogOpen(true);
    handleClose();
  };

  const handleOpenSecurityKeys = () => {
    setSecurityKeysDialogOpen(true);
    handleClose();
  };

  if (authState.loading) {
    return (
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <CircularProgress size={24} />
        {showLabel && !compact && (
          <Typography variant="body2" color="text.secondary">
            Authenticating...
          </Typography>
        )}
      </Box>
    );
  }

  if (!authState.isAuthenticated) {
    return (
      <>
        <Stack direction="row" spacing={1} alignItems="center">
          <Button
            variant={compact ? 'text' : 'outlined'}
            startIcon={<LoginIcon />}
            onClick={() => {
              console.log('🔴 Sign In button clicked!');
              setLoginInitialMode('login');
              setLoginDialogOpen(true);
              console.log('🔴 LoginDialog should open, state set to true');
            }}
            size={compact ? 'small' : 'medium'}
          >
            Sign In
          </Button>
          {!compact && (
            <Button
              variant="contained"
              startIcon={<SecurityIcon />}
              onClick={() => { setLoginInitialMode('register'); setLoginDialogOpen(true); }}
              size="medium"
              color="primary"
            >
              Create Identity
            </Button>
          )}
        </Stack>

        {loginDialogOpen && (
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
              initialMode={loginInitialMode}
              onSuccess={() => setLoginDialogOpen(false)}
              onCancel={() => setLoginDialogOpen(false)}
            />
          </Box>
        )}
      </>
    );
  }

  const user = authState.user!;

  return (
    <>
      <Stack direction="row" spacing={1} alignItems="center">
        <Tooltip
          title={`Click to open menu • ${user.fourWordAddress}`}
          arrow
          placement="bottom"
        >
          <Button
            onClick={handleClick}
            variant="outlined"
            size={compact ? 'small' : 'medium'}
            sx={{
              p: 0.5,
              minWidth: 'auto',
              border: '1px solid',
              borderColor: 'divider',
              borderRadius: 2,
              '&:hover': {
                backgroundColor: 'action.hover',
                borderColor: 'primary.main',
              },
            }}
            endIcon={<ArrowDropDownIcon />}
          >
            <Avatar
              sx={{
                width: compact ? 28 : 36,
                height: compact ? 28 : 36,
                bgcolor: 'primary.main',
                fontSize: compact ? '0.875rem' : '1rem',
                fontWeight: 600,
                mr: 0.5,
              }}
            >
              {user.name.charAt(0).toUpperCase()}
            </Avatar>
          </Button>
        </Tooltip>

        {showLabel && !compact && (
          <Box sx={{ minWidth: 0, cursor: 'pointer' }} onClick={handleClick}>
            <Typography
              variant="body2"
              fontWeight={600}
              sx={{
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
                maxWidth: 120,
                '&:hover': {
                  color: 'primary.main',
                },
              }}
            >
              {user.name}
            </Typography>
            <Typography
              variant="caption"
              color="text.secondary"
              sx={{
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
                maxWidth: 120,
                display: 'block',
                '&:hover': {
                  color: 'primary.main',
                },
              }}
            >
              {user.fourWordAddress}
            </Typography>
          </Box>
        )}
      </Stack>

      {/* User Menu */}
      <Menu
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        onClick={handleClose}
        PaperProps={{
          elevation: 4,
          sx: {
            overflow: 'visible',
            filter: 'drop-shadow(0px 2px 8px rgba(0,0,0,0.32))',
            mt: 1.5,
            minWidth: 280,
            '&:before': {
              content: '""',
              display: 'block',
              position: 'absolute',
              top: 0,
              right: 14,
              width: 10,
              height: 10,
              bgcolor: 'background.paper',
              transform: 'translateY(-50%) rotate(45deg)',
              zIndex: 0,
            },
          },
        }}
        transformOrigin={{ horizontal: 'right', vertical: 'top' }}
        anchorOrigin={{ horizontal: 'right', vertical: 'bottom' }}
      >
        {/* User Info Header */}
        <Box sx={{ px: 2, py: 1.5, borderBottom: '1px solid', borderColor: 'divider' }}>
          <Stack direction="row" alignItems="center" spacing={2}>
            <Avatar
              sx={{
                width: 48,
                height: 48,
                bgcolor: 'primary.main',
                fontSize: '1.25rem',
                fontWeight: 600,
              }}
            >
              {user.name.charAt(0).toUpperCase()}
            </Avatar>
            <Box sx={{ minWidth: 0, flex: 1 }}>
              <Typography variant="subtitle1" fontWeight={600} noWrap>
                {user.name}
              </Typography>
              <Typography variant="body2" color="text.secondary" noWrap>
                {user.fourWordAddress}
              </Typography>
              <Stack direction="row" spacing={0.5} sx={{ mt: 0.5 }}>
                <Chip
                  size="small"
                  variant="outlined"
                  color="success"
                  label="Authenticated"
                  icon={<SecurityIcon />}
                  sx={{ fontSize: '0.7rem', height: 20 }}
                />
                {networkStatus && (
                  <Chip
                    size="small"
                    variant="outlined"
                    color={networkStatus.connected ? 'success' : 'warning'}
                    label={networkStatus.connected ? 'Online' : 'Offline'}
                    icon={<NetworkIcon />}
                    sx={{ fontSize: '0.7rem', height: 20 }}
                  />
                )}
              </Stack>
            </Box>
          </Stack>
        </Box>

        {/* Menu Items */}
        <MenuItem onClick={handleOpenProfile}>
          <ListItemIcon>
            <PersonIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>
            <Typography variant="body2">Manage Profile</Typography>
          </ListItemText>
        </MenuItem>

        <MenuItem onClick={handleOpenSettings}>
          <ListItemIcon>
            <SettingsIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>
            <Typography variant="body2">Account Settings</Typography>
          </ListItemText>
        </MenuItem>

        <MenuItem onClick={handleOpenSecurityKeys}>
          <ListItemIcon>
            <KeyIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>
            <Typography variant="body2">Security & Keys</Typography>
          </ListItemText>
        </MenuItem>

        <MenuItem onClick={() => {
          handleClose();
          switchToPersonal();
          setTimeout(() => {
            selectEntity('overview');
          }, 100);
        }}>
          <ListItemIcon>
            <AccountCircleIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>
            <Typography variant="body2">My Storage Disks</Typography>
            <Typography variant="caption" color="text.secondary">Website & Data Storage</Typography>
          </ListItemText>
        </MenuItem>

        <Divider />

        {/* Network Status */}
        {networkStatus && (
          <>
            <Box sx={{ px: 2, py: 1 }}>
              <Typography variant="caption" color="text.secondary">
                Network Status
              </Typography>
              <Stack direction="row" alignItems="center" spacing={1} sx={{ mt: 0.5 }}>
                <NetworkIcon
                  fontSize="small"
                  color={networkStatus.connected ? 'success' : 'warning'}
                />
                <Typography variant="body2">
                  {networkStatus.connected ? 'Connected' : 'Disconnected'} • {networkStatus.peers} peers
                </Typography>
              </Stack>
            </Box>
            <Divider />
          </>
        )}

        {/* Logout */}
        <MenuItem onClick={handleLogout} disabled={loggingOut}>
          <ListItemIcon>
            {loggingOut ? (
              <CircularProgress size={20} />
            ) : (
              <LogoutIcon fontSize="small" />
            )}
          </ListItemIcon>
          <ListItemText>
            <Typography variant="body2" color={loggingOut ? 'text.disabled' : 'inherit'}>
              {loggingOut ? 'Signing out...' : 'Sign Out'}
            </Typography>
          </ListItemText>
        </MenuItem>
      </Menu>

      {/* Profile Dialog */}
      <Dialog
        open={profileDialogOpen}
        onClose={() => setProfileDialogOpen(false)}
        maxWidth="lg"
        fullWidth
        PaperProps={{
          sx: {
            minHeight: '60vh',
          },
        }}
      >
        {/* Removed: ProfileManager - using ModernShellPrototype instead */}
        <Box p={3}>
          <Typography>Profile management available in main interface</Typography>
        </Box>
      </Dialog>

      {/* Settings Dialog */}
      <Dialog
        open={settingsDialogOpen}
        onClose={() => setSettingsDialogOpen(false)}
        maxWidth="lg"
        fullWidth
        PaperProps={{
          sx: {
            minHeight: '60vh',
          },
        }}
      >
        <SettingsInterface />
      </Dialog>

      {/* Security & Keys Dialog */}
      <Dialog
        open={securityKeysDialogOpen}
        onClose={() => setSecurityKeysDialogOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>Security & Keys Management</DialogTitle>
        <DialogContent>
          <Alert severity="info" sx={{ mb: 2 }}>
            <Typography variant="body2">
              Advanced security and key management features. Manage your cryptographic keys and security settings.
            </Typography>
          </Alert>
          
          <Box sx={{ mt: 2 }}>
            <Typography variant="h6" gutterBottom>
              Post-Quantum Cryptography Keys
            </Typography>
            <List>
              <ListItem>
                <ListItemIcon>
                  <KeyIcon />
                </ListItemIcon>
                <ListItemText
                  primary="ML-DSA Signing Key"
                  secondary="Used for message and document signing"
                />
                <Button variant="outlined" size="small">
                  View Details
                </Button>
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <SecurityIcon />
                </ListItemIcon>
                <ListItemText
                  primary="ML-KEM Encryption Key"
                  secondary="Used for secure communications"
                />
                <Button variant="outlined" size="small">
                  View Details
                </Button>
              </ListItem>
            </List>
            
            <Typography variant="h6" gutterBottom sx={{ mt: 3 }}>
              Key Management Actions
            </Typography>
            <Stack direction="row" spacing={2} sx={{ mt: 2 }}>
              <Button variant="outlined" startIcon={<KeyIcon />}>
                Export Keys
              </Button>
              <Button variant="outlined" startIcon={<SecurityIcon />}>
                Backup Keys
              </Button>
              <Button variant="outlined" color="warning">
                Regenerate Keys
              </Button>
            </Stack>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setSecurityKeysDialogOpen(false)}>
            Close
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
};

export default AuthStatus;
