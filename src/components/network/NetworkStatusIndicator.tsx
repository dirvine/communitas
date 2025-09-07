/**
 * Network Status Indicator Component
 * 
 * Shows current network status with visual feedback
 * Click to retry connection when offline
 */

import React, { useState, useEffect } from 'react';
import {
  Button,
  Chip,
  CircularProgress,
  Tooltip,
  Box,
  Typography,
  Popover,
  Stack,
  IconButton,
  Alert,
  Divider,
} from '@mui/material';
import {
  WifiOff as OfflineIcon,
  Wifi as OnlineIcon,
  CloudOff as LocalIcon,
  Sync as ConnectingIcon,
  Error as ErrorIcon,
  Refresh as RefreshIcon,
  Group as PeersIcon,
  AccessTime as TimeIcon,
  Router as NodeIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { networkService, NetworkStatus } from '../../services/network/NetworkConnectionService';

export const NetworkStatusIndicator: React.FC = () => {
  const [networkState, setNetworkState] = useState(networkService.getState());
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
  const [isRetrying, setIsRetrying] = useState(false);

  useEffect(() => {
    // Subscribe to network status changes
    const unsubscribe = networkService.subscribe((state) => {
      setNetworkState(state);
      setIsRetrying(state.status === 'connecting');
    });

    return unsubscribe;
  }, []);

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    if (networkState.status === 'connected') {
      // Show details popover
      setAnchorEl(event.currentTarget);
    } else if (!isRetrying) {
      // Attempt to connect
      handleConnect();
    }
  };

  const handleConnect = async () => {
    setIsRetrying(true);
    try {
      await networkService.connect();
    } finally {
      setIsRetrying(false);
    }
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const getStatusIcon = () => {
    switch (networkState.status) {
      case 'connected':
        return <OnlineIcon />;
      case 'connecting':
        return <CircularProgress size={16} sx={{ color: 'inherit' }} />;
      case 'offline':
        return <OfflineIcon />;
      case 'local':
        return <LocalIcon />;
      case 'error':
        return <ErrorIcon />;
      default:
        return <OfflineIcon />;
    }
  };

  const getStatusColor = (): 'success' | 'warning' | 'error' | 'default' => {
    switch (networkState.status) {
      case 'connected':
        return 'success';
      case 'connecting':
        return 'warning';
      case 'offline':
      case 'local':
        return 'warning';
      case 'error':
        return 'error';
      default:
        return 'default';
    }
  };

  const getStatusLabel = () => {
    switch (networkState.status) {
      case 'connected':
        return `Connected (${networkState.peers} peers)`;
      case 'connecting':
        return 'Connecting...';
      case 'offline':
        return 'Offline Mode';
      case 'local':
        return 'Local Mode';
      case 'error':
        return 'Connection Error';
      default:
        return 'Unknown';
    }
  };

  const getTooltipText = () => {
    if (networkState.status === 'connected') {
      return 'Click for network details';
    } else if (isRetrying) {
      return 'Connecting to network...';
    } else {
      return 'Click to connect to network';
    }
  };

  const formatTime = (date: Date | null) => {
    if (!date) return 'Never';
    const now = new Date();
    const diff = now.getTime() - new Date(date).getTime();
    const minutes = Math.floor(diff / 60000);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  };

  const open = Boolean(anchorEl);

  return (
    <>
      <Tooltip title={getTooltipText()} arrow>
        <Chip
          icon={getStatusIcon()}
          label={getStatusLabel()}
          color={getStatusColor()}
          onClick={handleClick}
          clickable={!isRetrying}
          variant={networkState.status === 'connected' ? 'filled' : 'outlined'}
          component={motion.div}
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
          sx={{
            cursor: isRetrying ? 'wait' : 'pointer',
            fontWeight: 500,
            animation: networkState.status === 'connecting' ? 'pulse 2s infinite' : 'none',
            '@keyframes pulse': {
              '0%': { opacity: 1 },
              '50%': { opacity: 0.7 },
              '100%': { opacity: 1 },
            },
          }}
        />
      </Tooltip>

      {/* Details Popover */}
      <Popover
        open={open}
        anchorEl={anchorEl}
        onClose={handleClose}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'right',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
      >
        <Box sx={{ p: 2, minWidth: 300 }}>
          <Stack spacing={2}>
            {/* Header */}
            <Box display="flex" alignItems="center" justifyContent="space-between">
              <Typography variant="h6" sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                {getStatusIcon()}
                Network Status
              </Typography>
              <IconButton size="small" onClick={handleConnect} disabled={isRetrying}>
                <RefreshIcon />
              </IconButton>
            </Box>

            <Divider />

            {/* Status Details */}
            <Stack spacing={1.5}>
              {/* Connection Status */}
              <Box display="flex" alignItems="center" gap={1}>
                <OnlineIcon fontSize="small" color={networkState.status === 'connected' ? 'success' : 'disabled'} />
                <Typography variant="body2">
                  Status: <strong>{getStatusLabel()}</strong>
                </Typography>
              </Box>

              {/* Peer Count */}
              {networkState.status === 'connected' && (
                <Box display="flex" alignItems="center" gap={1}>
                  <PeersIcon fontSize="small" />
                  <Typography variant="body2">
                    Connected Peers: <strong>{networkState.peers}</strong>
                  </Typography>
                </Box>
              )}

              {/* Bootstrap Nodes */}
              {networkState.bootstrapNodes.length > 0 && (
                <Box display="flex" alignItems="center" gap={1}>
                  <NodeIcon fontSize="small" />
                  <Typography variant="body2">
                    Bootstrap Nodes: <strong>{networkState.bootstrapNodes.length}</strong>
                  </Typography>
                </Box>
              )}

              {/* Last Connection */}
              <Box display="flex" alignItems="center" gap={1}>
                <TimeIcon fontSize="small" />
                <Typography variant="body2">
                  Last Connected: <strong>{formatTime(networkState.lastSuccessfulConnection)}</strong>
                </Typography>
              </Box>

              {/* Last Attempt */}
              {networkState.lastConnectionAttempt && (
                <Box display="flex" alignItems="center" gap={1}>
                  <TimeIcon fontSize="small" />
                  <Typography variant="body2">
                    Last Attempt: <strong>{formatTime(networkState.lastConnectionAttempt)}</strong>
                  </Typography>
                </Box>
              )}

              {/* Error Message */}
              {networkState.error && (
                <Alert severity="error" sx={{ py: 0.5 }}>
                  {networkState.error}
                </Alert>
              )}

              {/* Retry Count */}
              {networkState.retryCount > 0 && (
                <Typography variant="caption" color="text.secondary">
                  Retry attempts: {networkState.retryCount}/{3}
                </Typography>
              )}
            </Stack>

            <Divider />

            {/* Actions */}
            <Stack direction="row" spacing={1}>
              {networkState.status === 'connected' ? (
                <Button
                  fullWidth
                  variant="outlined"
                  size="small"
                  onClick={() => {
                    networkService.disconnect();
                    handleClose();
                  }}
                  startIcon={<LocalIcon />}
                >
                  Go Local
                </Button>
              ) : (
                <Button
                  fullWidth
                  variant="contained"
                  size="small"
                  onClick={handleConnect}
                  disabled={isRetrying}
                  startIcon={isRetrying ? <CircularProgress size={16} /> : <OnlineIcon />}
                >
                  {isRetrying ? 'Connecting...' : 'Connect to Network'}
                </Button>
              )}
            </Stack>

            {/* Info Message */}
            <Alert severity="info" sx={{ py: 0.5 }}>
              <Typography variant="caption">
                {networkState.status === 'connected' 
                  ? 'You are connected to the P2P network. All features available.'
                  : networkState.status === 'local' || networkState.status === 'offline'
                  ? 'Working in local mode. All data is saved locally and will sync when online.'
                  : 'Attempting to connect to the network...'}
              </Typography>
            </Alert>
          </Stack>
        </Box>
      </Popover>
    </>
  );
};

/**
 * Simplified network status badge for mobile/compact views
 */
export const NetworkStatusBadge: React.FC = () => {
  const [networkState, setNetworkState] = useState(networkService.getState());
  const [isRetrying, setIsRetrying] = useState(false);

  useEffect(() => {
    const unsubscribe = networkService.subscribe((state) => {
      setNetworkState(state);
      setIsRetrying(state.status === 'connecting');
    });

    return unsubscribe;
  }, []);

  const handleClick = async () => {
    if (networkState.status !== 'connected' && !isRetrying) {
      setIsRetrying(true);
      try {
        await networkService.connect();
      } finally {
        setIsRetrying(false);
      }
    }
  };

  const getIcon = () => {
    switch (networkState.status) {
      case 'connected':
        return (
          <Box
            sx={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              backgroundColor: 'success.main',
              animation: 'pulse 2s infinite',
              '@keyframes pulse': {
                '0%': { boxShadow: '0 0 0 0 rgba(76, 175, 80, 0.7)' },
                '70%': { boxShadow: '0 0 0 10px rgba(76, 175, 80, 0)' },
                '100%': { boxShadow: '0 0 0 0 rgba(76, 175, 80, 0)' },
              },
            }}
          />
        );
      case 'connecting':
        return <CircularProgress size={8} />;
      default:
        return (
          <Box
            sx={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              backgroundColor: 'warning.main',
            }}
          />
        );
    }
  };

  return (
    <AnimatePresence>
      <Tooltip 
        title={networkState.status === 'connected' ? 'Online' : 'Offline - Click to connect'}
        arrow
      >
        <IconButton
          size="small"
          onClick={handleClick}
          disabled={isRetrying}
          component={motion.button}
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
        >
          {getIcon()}
        </IconButton>
      </Tooltip>
    </AnimatePresence>
  );
};