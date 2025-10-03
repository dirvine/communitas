import React, { useState, useEffect } from 'react';
import {
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Avatar,
  Box,
  Typography,
  Divider,
  IconButton,
  Badge,
  Chip,
  alpha,
  CircularProgress,
} from '@mui/material';
import {
  Person as PersonIcon,
  SwapHoriz as SwapIcon,
  Add as AddIcon,
  Logout as LogoutIcon,
  Settings as SettingsIcon,
  Security as SecurityIcon,
  Fingerprint as FingerprintIcon,
  Check as CheckIcon,
  ExpandMore as ExpandMoreIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

// Recent identity from backend
interface RecentIdentity {
  four_words: string;
  display_name: string;
  last_used: number;
  has_passkey: boolean;
}

// Current session info
interface SessionInfo {
  session_id: string;
  four_words: string;
  display_name: string;
}

interface IdentitySwitchMenuProps {
  currentSession: SessionInfo | null;
  onSwitch: (fourWords: string) => Promise<void>;
  onAddNew: () => void;
  onSettings: () => void;
  onLogout: () => Promise<void>;
}

export const IdentitySwitchMenu: React.FC<IdentitySwitchMenuProps> = ({
  currentSession,
  onSwitch,
  onAddNew,
  onSettings,
  onLogout,
}) => {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [identities, setIdentities] = useState<RecentIdentity[]>([]);
  const [loading, setLoading] = useState(false);
  const [switching, setSwitching] = useState<string | null>(null);

  const open = Boolean(anchorEl);

  // Load recent identities when menu opens
  useEffect(() => {
    if (open) {
      loadRecentIdentities();
    }
  }, [open]);

  const loadRecentIdentities = async () => {
    try {
      setLoading(true);
      const recent = await invoke<RecentIdentity[]>('auth_get_recent_identities');
      setIdentities(recent);
    } catch (err) {
      console.error('Failed to load identities:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const handleSwitchIdentity = async (fourWords: string) => {
    try {
      setSwitching(fourWords);
      await onSwitch(fourWords);
      handleClose();
    } catch (err) {
      console.error('Failed to switch identity:', err);
    } finally {
      setSwitching(null);
    }
  };

  const handleAddNew = () => {
    handleClose();
    onAddNew();
  };

  const handleSettings = () => {
    handleClose();
    onSettings();
  };

  const handleLogout = async () => {
    try {
      await onLogout();
      handleClose();
    } catch (err) {
      console.error('Failed to logout:', err);
    }
  };

  const getAvatarColor = (fourWords: string): string => {
    let hash = 0;
    for (let i = 0; i < fourWords.length; i++) {
      hash = fourWords.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = Math.abs(hash % 360);
    return `hsl(${hue}, 65%, 55%)`;
  };

  const isCurrentIdentity = (fourWords: string): boolean => {
    return currentSession?.four_words === fourWords;
  };

  // If no session, show minimal button
  if (!currentSession) {
    return (
      <IconButton onClick={handleClick} size="small">
        <PersonIcon />
      </IconButton>
    );
  }

  return (
    <>
      {/* Trigger Button */}
      <Box
        onClick={handleClick}
        sx={{
          display: 'flex',
          alignItems: 'center',
          gap: 1,
          cursor: 'pointer',
          padding: '6px 12px',
          borderRadius: 2,
          transition: 'all 0.2s ease',
          '&:hover': {
            bgcolor: (theme) => alpha(theme.palette.primary.main, 0.1),
          },
        }}
      >
        <Badge
          overlap="circular"
          anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
          badgeContent={
            identities.find((id) => id.four_words === currentSession.four_words)?.has_passkey ? (
              <SecurityIcon
                sx={{
                  width: 16,
                  height: 16,
                  color: 'success.main',
                  bgcolor: 'background.paper',
                  borderRadius: '50%',
                  p: 0.25,
                }}
              />
            ) : null
          }
        >
          <Avatar
            sx={{
              width: 32,
              height: 32,
              bgcolor: getAvatarColor(currentSession.four_words),
              fontSize: '0.875rem',
              fontWeight: 600,
            }}
          >
            {currentSession.display_name.charAt(0).toUpperCase()}
          </Avatar>
        </Badge>
        <Box sx={{ display: { xs: 'none', sm: 'block' }, minWidth: 0 }}>
          <Typography variant="body2" fontWeight={600} noWrap>
            {currentSession.display_name}
          </Typography>
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{
              fontFamily: 'monospace',
              fontSize: '0.7rem',
              display: 'block',
            }}
            noWrap
          >
            {currentSession.four_words}
          </Typography>
        </Box>
        <ExpandMoreIcon
          sx={{
            fontSize: 20,
            color: 'text.secondary',
            transition: 'transform 0.2s ease',
            transform: open ? 'rotate(180deg)' : 'rotate(0deg)',
          }}
        />
      </Box>

      {/* Dropdown Menu */}
      <Menu
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        transformOrigin={{ horizontal: 'right', vertical: 'top' }}
        anchorOrigin={{ horizontal: 'right', vertical: 'bottom' }}
        PaperProps={{
          elevation: 8,
          sx: {
            minWidth: 320,
            mt: 1,
            overflow: 'visible',
            borderRadius: 2,
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
      >
        {/* Current Identity Header */}
        <Box sx={{ px: 2, py: 1.5 }}>
          <Typography variant="caption" color="text.secondary" fontWeight={600}>
            SIGNED IN AS
          </Typography>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5, mt: 1 }}>
            <Avatar
              sx={{
                width: 40,
                height: 40,
                bgcolor: getAvatarColor(currentSession.four_words),
                fontSize: '1rem',
                fontWeight: 600,
              }}
            >
              {currentSession.display_name.charAt(0).toUpperCase()}
            </Avatar>
            <Box sx={{ flex: 1, minWidth: 0 }}>
              <Typography variant="body1" fontWeight={600} noWrap>
                {currentSession.display_name}
              </Typography>
              <Typography
                variant="caption"
                color="text.secondary"
                sx={{
                  fontFamily: 'monospace',
                  fontSize: '0.75rem',
                  display: 'block',
                }}
                noWrap
              >
                {currentSession.four_words}
              </Typography>
            </Box>
          </Box>
        </Box>

        <Divider />

        {/* Switch Identity Section */}
        <Box sx={{ px: 2, py: 1 }}>
          <Typography
            variant="caption"
            color="text.secondary"
            fontWeight={600}
            sx={{ display: 'block', mb: 0.5 }}
          >
            SWITCH IDENTITY
          </Typography>
        </Box>

        {/* Loading State */}
        {loading && (
          <Box sx={{ display: 'flex', justifyContent: 'center', py: 2 }}>
            <CircularProgress size={24} />
          </Box>
        )}

        {/* Other Identities */}
        {!loading &&
          identities
            .filter((identity) => !isCurrentIdentity(identity.four_words))
            .map((identity) => (
              <MenuItem
                key={identity.four_words}
                onClick={() => handleSwitchIdentity(identity.four_words)}
                disabled={switching !== null}
                sx={{ px: 2, py: 1.5 }}
              >
                <ListItemIcon>
                  <Badge
                    overlap="circular"
                    anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                    badgeContent={
                      identity.has_passkey ? (
                        <FingerprintIcon
                          sx={{
                            width: 14,
                            height: 14,
                            color: 'primary.main',
                            bgcolor: 'background.paper',
                            borderRadius: '50%',
                            p: 0.25,
                          }}
                        />
                      ) : null
                    }
                  >
                    <Avatar
                      sx={{
                        width: 32,
                        height: 32,
                        bgcolor: getAvatarColor(identity.four_words),
                        fontSize: '0.875rem',
                      }}
                    >
                      {identity.display_name.charAt(0).toUpperCase()}
                    </Avatar>
                  </Badge>
                </ListItemIcon>
                <ListItemText
                  primary={
                    <Typography variant="body2" fontWeight={500}>
                      {identity.display_name}
                    </Typography>
                  }
                  secondary={
                    <Typography
                      variant="caption"
                      sx={{
                        fontFamily: 'monospace',
                        fontSize: '0.7rem',
                      }}
                    >
                      {identity.four_words}
                    </Typography>
                  }
                />
                {switching === identity.four_words && (
                  <CircularProgress size={20} sx={{ ml: 1 }} />
                )}
              </MenuItem>
            ))}

        {/* No other identities */}
        {!loading && identities.length <= 1 && (
          <Box sx={{ px: 2, py: 1.5 }}>
            <Typography variant="caption" color="text.secondary" fontStyle="italic">
              No other identities available
            </Typography>
          </Box>
        )}

        <Divider sx={{ my: 1 }} />

        {/* Actions */}
        <MenuItem onClick={handleAddNew} sx={{ px: 2, py: 1.5 }}>
          <ListItemIcon>
            <AddIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText primary="Add Identity" />
        </MenuItem>

        <MenuItem onClick={handleSettings} sx={{ px: 2, py: 1.5 }}>
          <ListItemIcon>
            <SettingsIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText primary="Settings" />
        </MenuItem>

        <Divider sx={{ my: 1 }} />

        <MenuItem onClick={handleLogout} sx={{ px: 2, py: 1.5, color: 'error.main' }}>
          <ListItemIcon>
            <LogoutIcon fontSize="small" color="error" />
          </ListItemIcon>
          <ListItemText primary="Sign Out" />
        </MenuItem>
      </Menu>
    </>
  );
};
