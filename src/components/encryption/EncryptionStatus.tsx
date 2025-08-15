import React, { useState } from 'react';
import {
  Box,
  Chip,
  IconButton,
  Tooltip,
  Menu,
  MenuItem,
  Typography,
  Switch,
  FormControlLabel,
  Alert,
  Divider,
  ListItemIcon,
  ListItemText,
  Stack,
  CircularProgress,
} from '@mui/material';
import {
  Security as SecurityIcon,
  SecurityOutlined as SecurityOutlinedIcon,
  Key as KeyIcon,
  VpnKey as VpnKeyIcon,
  Shield as ShieldIcon,
  Warning as WarningIcon,
  Info as InfoIcon,
  Settings as SettingsIcon,
  Lock as LockIcon,
  LockOpen as LockOpenIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useEncryption } from '../../contexts/EncryptionContext';
import { useAuth } from '../../contexts/AuthContext';

export interface EncryptionStatusProps {
  showDetails?: boolean;
  compact?: boolean;
}

export const EncryptionStatus: React.FC<EncryptionStatusProps> = ({
  showDetails = false,
  compact = false,
}) => {
  const { state, toggleEncryption, clearEncryption } = useEncryption();
  const { authState } = useAuth();
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const getStatusColor = (): "success" | "warning" | "error" | "default" => {
    if (!authState.isAuthenticated) return 'default';
    if (!state.isInitialized) return 'warning';
    if (!state.encryptionEnabled) return 'warning';
    return 'success';
  };

  const getStatusIcon = () => {
    if (state.loading) return <CircularProgress size={16} />;
    if (!authState.isAuthenticated) return <SecurityOutlinedIcon />;
    if (state.error) return <WarningIcon />;
    if (!state.isInitialized || !state.encryptionEnabled) return <LockOpenIcon />;
    return <LockIcon />;
  };

  const getStatusText = (): string => {
    if (!authState.isAuthenticated) return 'Not signed in';
    if (state.loading) return 'Initializing...';
    if (state.error) return 'Encryption error';
    if (!state.isInitialized) return 'Not initialized';
    if (!state.encryptionEnabled) return 'Encryption disabled';
    return 'Encrypted';
  };

  const handleToggleEncryption = () => {
    toggleEncryption(!state.encryptionEnabled);
    handleClose();
  };

  const handleClearEncryption = () => {
    clearEncryption();
    handleClose();
  };

  if (compact) {
    return (
      <Tooltip title={`Encryption: ${getStatusText()}`}>
        <IconButton
          size="small"
          onClick={handleClick}
          sx={{
            color: getStatusColor() === 'success' ? 'success.main' : 
                   getStatusColor() === 'warning' ? 'warning.main' : 
                   getStatusColor() === 'error' ? 'error.main' : 'text.secondary'
          }}
        >
          {getStatusIcon()}
        </IconButton>
      </Tooltip>
    );
  }

  return (
    <Box>
      <motion.div
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
      >
        <Chip
          icon={getStatusIcon()}
          label={getStatusText()}
          color={getStatusColor()}
          variant="outlined"
          onClick={handleClick}
          sx={{
            cursor: 'pointer',
            '& .MuiChip-icon': {
              fontSize: '1rem',
            },
          }}
        />
      </motion.div>

      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={handleClose}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'right',
        }}
        PaperProps={{
          sx: { minWidth: 280, mt: 1 },
        }}
      >
        {/* Status Header */}
        <Box sx={{ p: 2, pb: 1 }}>
          <Stack direction="row" alignItems="center" spacing={1}>
            <SecurityIcon color="primary" />
            <Typography variant="h6" fontWeight={600}>
              Encryption Status
            </Typography>
          </Stack>
        </Box>

        {/* Current Status */}
        <Box sx={{ px: 2, pb: 1 }}>
          {state.error && (
            <Alert severity="error" size="small" sx={{ mb: 1 }}>
              {state.error}
            </Alert>
          )}

          {!authState.isAuthenticated ? (
            <Alert severity="info" size="small" icon={<InfoIcon />}>
              Sign in to enable end-to-end encryption
            </Alert>
          ) : state.isInitialized ? (
            <Stack spacing={1}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <KeyIcon fontSize="small" color="success" />
                <Typography variant="body2" color="success.main">
                  Master key: Active
                </Typography>
              </Box>
              
              {state.userKeyPair && (
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <VpnKeyIcon fontSize="small" color="success" />
                  <Typography variant="body2" color="success.main">
                    Key pair: Ready
                  </Typography>
                </Box>
              )}
              
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <ShieldIcon fontSize="small" color={state.encryptionEnabled ? 'success' : 'warning'} />
                <Typography variant="body2" color={state.encryptionEnabled ? 'success.main' : 'warning.main'}>
                  Encryption: {state.encryptionEnabled ? 'Enabled' : 'Disabled'}
                </Typography>
              </Box>
            </Stack>
          ) : (
            <Alert severity="warning" size="small">
              Encryption not initialized
            </Alert>
          )}
        </Box>

        <Divider />

        {/* Settings */}
        {authState.isAuthenticated && state.isInitialized && (
          <>
            <MenuItem onClick={handleToggleEncryption}>
              <ListItemIcon>
                {state.encryptionEnabled ? <LockOpenIcon /> : <LockIcon />}
              </ListItemIcon>
              <ListItemText>
                {state.encryptionEnabled ? 'Disable' : 'Enable'} Encryption
              </ListItemText>
            </MenuItem>

            <MenuItem onClick={handleClearEncryption} sx={{ color: 'warning.main' }}>
              <ListItemIcon sx={{ color: 'inherit' }}>
                <WarningIcon />
              </ListItemIcon>
              <ListItemText>
                Clear All Keys
              </ListItemText>
            </MenuItem>
          </>
        )}

        {/* Additional Details */}
        {showDetails && state.isInitialized && (
          <>
            <Divider />
            <Box sx={{ p: 2 }}>
              <Typography variant="caption" color="text.secondary" gutterBottom>
                Session Keys: {state.sessionKeys.size}
              </Typography>
              <br />
              <Typography variant="caption" color="text.secondary">
                Shared Keys: {state.sharedKeys.size}
              </Typography>
            </Box>
          </>
        )}
      </Menu>
    </Box>
  );
};

export default EncryptionStatus;