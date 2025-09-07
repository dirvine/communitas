/**
 * Endpoint Status Display Component
 * 
 * Prominently displays the current endpoint four-word address or offline status
 */

import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Chip,
  Stack,
  IconButton,
  Tooltip,
  Paper,
  Fade,
  Alert,
} from '@mui/material';
import {
  LocationOn as LocationIcon,
  WifiOff as OfflineIcon,
  Wifi as OnlineIcon,
  CloudOff as LocalIcon,
  ContentCopy as CopyIcon,
  Refresh as RefreshIcon,
  Sync as ConnectingIcon,
} from '@mui/icons-material';
import { motion } from 'framer-motion';
import { networkService, NetworkStatus } from '../../services/network/NetworkConnectionService';

export const EndpointStatusDisplay: React.FC = () => {
  const [networkState, setNetworkState] = useState(networkService.getState());
  const [copied, setCopied] = useState(false);
  const [isRetrying, setIsRetrying] = useState(false);

  useEffect(() => {
    const unsubscribe = networkService.subscribe((state) => {
      setNetworkState(state);
      setIsRetrying(state.status === 'connecting');
    });

    return unsubscribe;
  }, []);

  const handleCopyEndpoint = () => {
    if (networkState.endpointFourWords) {
      navigator.clipboard.writeText(networkState.endpointFourWords);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleConnect = async () => {
    if (!isRetrying && networkState.status !== 'connected') {
      setIsRetrying(true);
      try {
        await networkService.connect();
      } finally {
        setIsRetrying(false);
      }
    }
  };

  const getStatusIcon = () => {
    switch (networkState.status) {
      case 'connected':
        return <OnlineIcon sx={{ fontSize: 20 }} />;
      case 'connecting':
        return <ConnectingIcon sx={{ fontSize: 20, animation: 'spin 2s linear infinite' }} />;
      case 'offline':
        return <OfflineIcon sx={{ fontSize: 20 }} />;
      case 'local':
        return <LocalIcon sx={{ fontSize: 20 }} />;
      default:
        return <OfflineIcon sx={{ fontSize: 20 }} />;
    }
  };

  const getStatusColor = () => {
    switch (networkState.status) {
      case 'connected':
        return 'success.main';
      case 'connecting':
        return 'warning.main';
      case 'offline':
      case 'local':
        return 'text.secondary';
      case 'error':
        return 'error.main';
      default:
        return 'text.secondary';
    }
  };

  const getEndpointDisplay = () => {
    if (networkState.status === 'connected' && networkState.endpointFourWords) {
      return networkState.endpointFourWords;
    } else if (networkState.status === 'connecting') {
      return 'Connecting...';
    } else if (networkState.status === 'offline') {
      return 'Offline';
    } else if (networkState.status === 'local') {
      return 'Local Mode';
    } else {
      return 'Not Connected';
    }
  };

  const isClickable = networkState.status !== 'connected' && !isRetrying;

  return (
    <Paper
      elevation={0}
      component={motion.div}
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      sx={{
        p: 1.5,
        backgroundColor: 'background.paper',
        border: '1px solid',
        borderColor: 'divider',
        borderRadius: 2,
        cursor: isClickable ? 'pointer' : 'default',
        transition: 'all 0.3s ease',
        '&:hover': isClickable ? {
          backgroundColor: 'action.hover',
          borderColor: 'primary.main',
        } : {},
      }}
      onClick={isClickable ? handleConnect : undefined}
    >
      <Stack spacing={1}>
        {/* Label */}
        <Typography 
          variant="caption" 
          color="text.secondary"
          sx={{ 
            textTransform: 'uppercase', 
            letterSpacing: 1,
            fontSize: '0.7rem',
            fontWeight: 600
          }}
        >
          Current Network Location
        </Typography>

        {/* Endpoint Display */}
        <Stack direction="row" alignItems="center" spacing={1.5}>
          {/* Status Icon */}
          <Box sx={{ color: getStatusColor(), display: 'flex', alignItems: 'center' }}>
            {getStatusIcon()}
          </Box>

          {/* Four-Word Address or Status */}
          <Typography
            variant="h6"
            sx={{
              fontFamily: 'monospace',
              fontWeight: networkState.status === 'connected' ? 600 : 400,
              color: networkState.status === 'connected' ? 'text.primary' : 'text.secondary',
              letterSpacing: networkState.status === 'connected' ? 1 : 0,
              flexGrow: 1,
            }}
          >
            {getEndpointDisplay()}
          </Typography>

          {/* Action Buttons */}
          {networkState.status === 'connected' && networkState.endpointFourWords && (
            <Fade in>
              <Stack direction="row" spacing={0.5}>
                <Tooltip title={copied ? 'Copied!' : 'Copy endpoint address'}>
                  <IconButton 
                    size="small" 
                    onClick={(e) => {
                      e.stopPropagation();
                      handleCopyEndpoint();
                    }}
                    sx={{ 
                      color: copied ? 'success.main' : 'text.secondary',
                      transition: 'color 0.3s ease'
                    }}
                  >
                    <CopyIcon fontSize="small" />
                  </IconButton>
                </Tooltip>
              </Stack>
            </Fade>
          )}

          {/* Retry Button for Error State */}
          {(networkState.status === 'error' || networkState.status === 'local') && !isRetrying && (
            <Tooltip title="Retry connection">
              <IconButton 
                size="small" 
                onClick={(e) => {
                  e.stopPropagation();
                  handleConnect();
                }}
                color="primary"
              >
                <RefreshIcon fontSize="small" />
              </IconButton>
            </Tooltip>
          )}
        </Stack>

        {/* Additional Info */}
        {networkState.status === 'connected' && (
          <Stack direction="row" spacing={2}>
            <Chip
              size="small"
              label={`${networkState.peers} peers`}
              variant="outlined"
              sx={{ height: 20, fontSize: '0.75rem' }}
            />
            {networkState.userFourWords && (
              <Chip
                size="small"
                label={`You: ${networkState.userFourWords}`}
                variant="outlined"
                sx={{ height: 20, fontSize: '0.75rem' }}
              />
            )}
          </Stack>
        )}

        {/* Click to Connect Hint */}
        {isClickable && (
          <Fade in>
            <Alert 
              severity="info" 
              sx={{ 
                py: 0.5, 
                mt: 1,
                '& .MuiAlert-message': { fontSize: '0.75rem' }
              }}
            >
              Click to connect to the network
            </Alert>
          </Fade>
        )}

        {/* Error Message */}
        {networkState.status === 'error' && networkState.error && (
          <Alert 
            severity="error" 
            sx={{ 
              py: 0.5, 
              mt: 1,
              '& .MuiAlert-message': { fontSize: '0.75rem' }
            }}
          >
            {networkState.error}
          </Alert>
        )}
      </Stack>
    </Paper>
  );
};

/**
 * Compact version for header/toolbar display
 */
export const CompactEndpointStatus: React.FC = () => {
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

  const getDisplay = () => {
    if (networkState.status === 'connected' && networkState.endpointFourWords) {
      return (
        <>
          <LocationIcon sx={{ fontSize: 16, mr: 0.5 }} />
          {networkState.endpointFourWords}
        </>
      );
    } else if (networkState.status === 'offline') {
      return (
        <>
          <OfflineIcon sx={{ fontSize: 16, mr: 0.5 }} />
          Offline
        </>
      );
    } else {
      return (
        <>
          <LocalIcon sx={{ fontSize: 16, mr: 0.5 }} />
          Local
        </>
      );
    }
  };

  return (
    <Chip
      label={getDisplay()}
      size="small"
      color={networkState.status === 'connected' ? 'success' : 'default'}
      variant={networkState.status === 'connected' ? 'filled' : 'outlined'}
      onClick={networkState.status !== 'connected' ? handleClick : undefined}
      clickable={networkState.status !== 'connected'}
      sx={{
        fontFamily: 'monospace',
        fontSize: '0.8rem',
        fontWeight: 500,
        cursor: networkState.status !== 'connected' ? 'pointer' : 'default',
      }}
    />
  );
};

export default EndpointStatusDisplay;
