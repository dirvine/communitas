import React, { useState } from 'react';
import {
  Box,
  Avatar,
  IconButton,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Typography,
  Divider,
  Chip,
  Button,
  Stack,
  CircularProgress,
  Dialog,
  Tooltip,
} from '@mui/material';
import {
  Person as PersonIcon,
  Login as LoginIcon,
  Logout as LogoutIcon,
  Settings as SettingsIcon,
  Security as SecurityIcon,
  Key as KeyIcon,
  NetworkCheck as NetworkIcon,
  AccountCircle as AccountCircleIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useAuth } from '../../contexts/AuthContext';
import { LoginDialog } from './LoginDialog';
import { ProfileManager } from './ProfileManager';

interface AuthStatusProps {
  compact?: boolean;
  showLabel?: boolean;
}

export const AuthStatus: React.FC<AuthStatusProps> = ({
  compact = false,
  showLabel = true,
}) => {
  const { authState, logout, getNetworkStatus } = useAuth();
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [loginDialogOpen, setLoginDialogOpen] = useState(false);
  const [profileDialogOpen, setProfileDialogOpen] = useState(false);
  const [networkStatus, setNetworkStatus] = useState<{ connected: boolean; peers: number } | null>(null);
  const [loggingOut, setLoggingOut] = useState(false);

  const open = Boolean(anchorEl);

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    if (authState.isAuthenticated) {
      setAnchorEl(event.currentTarget);
      // Update network status when menu opens
      updateNetworkStatus();
    } else {
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
        <Button
          variant={compact ? 'text' : 'outlined'}
          startIcon={<LoginIcon />}
          onClick={() => setLoginDialogOpen(true)}
          size={compact ? 'small' : 'medium'}
        >
          {compact ? 'Sign In' : 'Sign In to Communitas'}
        </Button>

        <LoginDialog
          open={loginDialogOpen}
          onClose={() => setLoginDialogOpen(false)}
        />
      </>
    );
  }

  const user = authState.user!;

  return (
    <>
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.2 }}
      >
        <Tooltip
          title={`Signed in as ${user.name} (${user.fourWordAddress})`}
          arrow
          placement="bottom"
        >
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              gap: 1,
              cursor: 'pointer',
              p: compact ? 0.5 : 1,
              borderRadius: 1,
              '&:hover': {
                backgroundColor: 'action.hover',
              },
            }}
            onClick={handleClick}
          >
            <Avatar
              sx={{
                width: compact ? 32 : 40,
                height: compact ? 32 : 40,
                bgcolor: 'primary.main',
                fontSize: compact ? '0.875rem' : '1rem',
                fontWeight: 600,
              }}
            >
              {user.name.charAt(0).toUpperCase()}
            </Avatar>

            {showLabel && !compact && (
              <Box sx={{ minWidth: 0 }}>
                <Typography
                  variant="body2"
                  fontWeight={600}
                  sx={{
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                    maxWidth: 120,
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
                  }}
                >
                  {user.fourWordAddress}
                </Typography>
              </Box>
            )}
          </Box>
        </Tooltip>
      </motion.div>

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

        <MenuItem onClick={handleClose}>
          <ListItemIcon>
            <SettingsIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>
            <Typography variant="body2">Account Settings</Typography>
          </ListItemText>
        </MenuItem>

        <MenuItem onClick={handleClose}>
          <ListItemIcon>
            <KeyIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>
            <Typography variant="body2">Security & Keys</Typography>
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
        <ProfileManager onClose={() => setProfileDialogOpen(false)} />
      </Dialog>
    </>
  );
};

export default AuthStatus;