// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

/**
 * Connection Status Component
 *
 * Displays P2P network connection status in sidebar:
 * - User's four-word identity
 * - Online/offline indicator
 * - Peer count
 * - Bootstrap peer management
 */

import React, { useEffect, useState } from 'react';
import {
  Box,
  Typography,
  Chip,
  IconButton,
  TextField,
  Button,
  List,
  ListItem,
  ListItemText,
  Collapse,
  Tooltip,
  CircularProgress,
  Alert,
} from '@mui/material';
import {
  Wifi as WifiIcon,
  WifiOff as WifiOffIcon,
  Refresh as RefreshIcon,
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
  PersonAdd as PersonAddIcon,
} from '@mui/icons-material';
import {
  connectionService,
  ConnectionStatus as ConnectionStatusType,
  BootstrapPeer,
  ConnectionService,
} from '../services/ConnectionService';

interface ConnectionStatusProps {
  /** Auto-refresh interval in milliseconds (default: 15000 = 15s) */
  refreshInterval?: number;
  /** Compact mode for sidebar (default: true) */
  compact?: boolean;
}

export const ConnectionStatus: React.FC<ConnectionStatusProps> = ({
  refreshInterval = 15000,
  compact = true,
}) => {
  const [status, setStatus] = useState<ConnectionStatusType | null>(null);
  const [peers, setPeers] = useState<BootstrapPeer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [expanded, setExpanded] = useState(false);
  const [addingPeer, setAddingPeer] = useState(false);
  const [newPeerFourWords, setNewPeerFourWords] = useState('');
  const [addPeerError, setAddPeerError] = useState('');

  // Fetch connection status
  const fetchStatus = async () => {
    try {
      const newStatus = await connectionService.getStatus();
      setStatus(newStatus);
      setError('');
    } catch (err) {
      setError(`Failed to get status: ${err}`);
    }
  };

  // Fetch cached peers
  const fetchPeers = async () => {
    try {
      const newPeers = await connectionService.getCachedPeers();
      setPeers(newPeers);
    } catch (err) {
      console.error('Failed to get peers:', err);
    }
  };

  // Initial load
  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      await Promise.all([fetchStatus(), fetchPeers()]);
      setLoading(false);
    };

    loadData();
  }, []);

  // Auto-refresh
  useEffect(() => {
    const interval = setInterval(() => {
      fetchStatus();
      if (expanded) {
        fetchPeers();
      }
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [refreshInterval, expanded]);

  // Handle manual refresh
  const handleRefresh = async () => {
    setLoading(true);
    await Promise.all([fetchStatus(), fetchPeers()]);
    setLoading(false);
  };

  // Handle add bootstrap peer
  const handleAddPeer = async () => {
    if (!newPeerFourWords.trim()) {
      setAddPeerError('Please enter a four-word address');
      return;
    }

    try {
      setLoading(true);
      setAddPeerError('');
      await connectionService.addBootstrapPeer(newPeerFourWords.trim());
      setNewPeerFourWords('');
      setAddingPeer(false);
      await fetchPeers();
      setLoading(false);
    } catch (err) {
      setAddPeerError(`Failed to add peer: ${err}`);
      setLoading(false);
    }
  };

  // Get status color
  const statusColor = status
    ? ConnectionService.getStatusColor(status.online, status.peer_count)
    : 'error';

  // Get status text
  const statusText = status
    ? status.online
      ? `Online (${status.peer_count} peer${status.peer_count === 1 ? '' : 's'})`
      : 'Offline'
    : 'Unknown';

  if (loading && !status) {
    return (
      <Box sx={{ p: 2, textAlign: 'center' }}>
        <CircularProgress size={24} />
      </Box>
    );
  }

  return (
    <Box sx={{ width: '100%' }}>
      {/* Status Header */}
      <Box
        sx={{
          p: compact ? 1.5 : 2,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          cursor: 'pointer',
          '&:hover': { bgcolor: 'action.hover' },
        }}
        onClick={() => setExpanded(!expanded)}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flex: 1, minWidth: 0 }}>
          {status?.online ? (
            <WifiIcon color={statusColor} fontSize="small" />
          ) : (
            <WifiOffIcon color={statusColor} fontSize="small" />
          )}
          <Box sx={{ flex: 1, minWidth: 0 }}>
            <Typography
              variant={compact ? 'caption' : 'body2'}
              sx={{
                fontWeight: 500,
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}
            >
              {status?.four_words || 'Loading...'}
            </Typography>
            <Typography
              variant="caption"
              color="text.secondary"
              sx={{
                display: 'block',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}
            >
              {statusText}
            </Typography>
          </Box>
        </Box>

        <Box sx={{ display: 'flex', gap: 0.5 }}>
          <Tooltip title="Refresh">
            <IconButton
              size="small"
              onClick={(e) => {
                e.stopPropagation();
                handleRefresh();
              }}
              disabled={loading}
            >
              <RefreshIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          {expanded ? <ExpandLessIcon fontSize="small" /> : <ExpandMoreIcon fontSize="small" />}
        </Box>
      </Box>

      {/* Expanded Details */}
      <Collapse in={expanded}>
        <Box sx={{ px: 2, pb: 2 }}>
          {error && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {error}
            </Alert>
          )}

          {/* Connection Quality */}
          {status && (
            <Box sx={{ mb: 2 }}>
              <Typography variant="caption" color="text.secondary" gutterBottom>
                Connection Quality
              </Typography>
              <Chip
                label={`${ConnectionService.getConnectionQuality(status.peer_count)}%`}
                color={statusColor}
                size="small"
                sx={{ width: '100%' }}
              />
            </Box>
          )}

          {/* Add Bootstrap Peer */}
          <Box sx={{ mb: 2 }}>
            {!addingPeer ? (
              <Button
                fullWidth
                size="small"
                variant="outlined"
                startIcon={<PersonAddIcon />}
                onClick={() => setAddingPeer(true)}
              >
                Add Bootstrap Peer
              </Button>
            ) : (
              <Box>
                <TextField
                  fullWidth
                  size="small"
                  label="Four-word address"
                  placeholder="alpha-bravo-charlie-delta"
                  value={newPeerFourWords}
                  onChange={(e) => setNewPeerFourWords(e.target.value)}
                  onKeyPress={(e) => {
                    if (e.key === 'Enter') {
                      handleAddPeer();
                    }
                  }}
                  error={!!addPeerError}
                  helperText={addPeerError}
                  sx={{ mb: 1 }}
                />
                <Box sx={{ display: 'flex', gap: 1 }}>
                  <Button
                    size="small"
                    variant="contained"
                    onClick={handleAddPeer}
                    disabled={loading}
                    fullWidth
                  >
                    Add
                  </Button>
                  <Button
                    size="small"
                    variant="outlined"
                    onClick={() => {
                      setAddingPeer(false);
                      setNewPeerFourWords('');
                      setAddPeerError('');
                    }}
                    disabled={loading}
                    fullWidth
                  >
                    Cancel
                  </Button>
                </Box>
              </Box>
            )}
          </Box>

          {/* Cached Peers List */}
          {peers.length > 0 && (
            <Box>
              <Typography variant="caption" color="text.secondary" gutterBottom>
                Known Peers ({peers.length})
              </Typography>
              <List dense disablePadding>
                {peers.slice(0, 5).map((peer, idx) => (
                  <ListItem
                    key={peer.peer_id}
                    disablePadding
                    sx={{
                      py: 0.5,
                      borderBottom:
                        idx < Math.min(4, peers.length - 1) ? '1px solid' : 'none',
                      borderColor: 'divider',
                    }}
                  >
                    <ListItemText
                      primary={
                        <Typography
                          variant="caption"
                          sx={{
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                            display: 'block',
                          }}
                        >
                          {peer.four_words}
                        </Typography>
                      }
                      secondary={
                        <Typography variant="caption" color="text.secondary">
                          {(peer.success_rate * 100).toFixed(0)}% success •{' '}
                          {ConnectionService.formatLastSeen(peer.last_seen)}
                        </Typography>
                      }
                    />
                  </ListItem>
                ))}
              </List>
              {peers.length > 5 && (
                <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5 }}>
                  +{peers.length - 5} more
                </Typography>
              )}
            </Box>
          )}

          {peers.length === 0 && (
            <Typography variant="caption" color="text.secondary">
              No known peers yet. Add a friend's address to bootstrap.
            </Typography>
          )}
        </Box>
      </Collapse>
    </Box>
  );
};
