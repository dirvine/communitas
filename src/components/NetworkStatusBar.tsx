/**
 * Network Status Bar Component
 * Displays network connectivity status and Four-Word identities
 * Shows user identity, endpoint identity, and peer count
 * Allows quick access to bootstrap connection dialog
 */

import {
    CloudSync as CloudSyncIcon, Computer as ComputerIcon, ContentCopy as CopyIcon, Error as ErrorIcon, Link as LinkIcon, Person as PersonIcon, Settings as SettingsIcon, Storage as StorageIcon, Sync as SyncIcon, Wifi as WifiIcon,
    WifiOff as WifiOffIcon
} from '@mui/icons-material';
import {
    Badge, Chip, Divider, IconButton, ListItemIcon,
    ListItemText, Menu,
    MenuItem, Stack, Typography
} from '@mui/material';
import React, { useEffect, useState } from 'react';
import { networkService, NetworkStatus } from '../services/network/NetworkConnectionService';
import { BootstrapConnectionDialog } from './dialogs/BootstrapConnectionDialog';

interface NetworkStatusBarProps {
  onSyncClick?: () => void;
}

export const NetworkStatusBar: React.FC<NetworkStatusBarProps> = ({ onSyncClick }) => {
  const [networkState, setNetworkState] = useState(networkService.getState());
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [bootstrapDialogOpen, setBootstrapDialogOpen] = useState(false);
  const [copiedField, setCopiedField] = useState<string | null>(null);

  useEffect(() => {
    const unsubscribe = networkService.subscribe(setNetworkState);
    return unsubscribe;
  }, []);

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleMenuClose = () => {
    setAnchorEl(null);
  };

  const handleConnect = async () => {
    handleMenuClose();
    await networkService.connect();
  };

  const handleDisconnect = async () => {
    handleMenuClose();
    await networkService.disconnect();
  };

  const handleBootstrapConnect = () => {
    handleMenuClose();
    setBootstrapDialogOpen(true);
  };

  const handleSyncNow = () => {
    handleMenuClose();
    if (onSyncClick) {
      onSyncClick();
    }
  };

  const copyToClipboard = async (text: string, field: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedField(field);
      setTimeout(() => setCopiedField(null), 2000);
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  const getStatusColor = (status: NetworkStatus): 'success' | 'warning' | 'error' | 'default' => {
    switch (status) {
      case 'connected':
        return 'success';
      case 'connecting':
        return 'warning';
      case 'error':
        return 'error';
      default:
        return 'default';
    }
  };

  const getStatusIcon = (status: NetworkStatus) => {
    switch (status) {
      case 'connected':
        return <WifiIcon />;
      case 'connecting':
        return <SyncIcon className="animate-spin" />;
      case 'error':
        return <ErrorIcon />;
      default:
        return <WifiOffIcon />;
    }
  };

  const getStatusLabel = (status: NetworkStatus): string => {
    switch (status) {
      case 'connected':
        return `Connected (${networkState.peers} peers)`;
      case 'connecting':
        return 'Connecting...';
      case 'offline':
        return 'Offline';
      case 'local':
        return 'Local Mode';
      case 'error':
        return 'Connection Error';
      default:
        return 'Unknown';
    }
  };

  const formatFourWords = (fourWords: string | null) => {
    if (!fourWords) return 'Not available';
    return fourWords.split('-').map(word =>
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ');
  };

  return (
    <>
      <Badge
        badgeContent={networkState.peers > 0 ? networkState.peers : undefined}
        color="primary"
        overlap="rectangular"
        invisible={networkState.status !== 'connected'}
      >
        <Chip
          icon={getStatusIcon(networkState.status)}
          label={getStatusLabel(networkState.status)}
          color={getStatusColor(networkState.status)}
          size="small"
          onClick={handleMenuOpen}
          sx={{ cursor: 'pointer' }}
        />
      </Badge>

      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={handleMenuClose}
        PaperProps={{
          sx: { width: 320, maxWidth: '100%' }
        }}
      >
        {/* Network Status Section */}
        <MenuItem disabled>
          <ListItemIcon>
            {getStatusIcon(networkState.status)}
          </ListItemIcon>
          <ListItemText
            primary="Network Status"
            secondary={getStatusLabel(networkState.status)}
          />
        </MenuItem>

        {networkState.error && (
          <MenuItem disabled>
            <ListItemText
              secondary={
                <Typography variant="caption" color="error">
                  {networkState.error}
                </Typography>
              }
            />
          </MenuItem>
        )}

        <Divider />

        {/* User Four-Words */}
        {networkState.userFourWords && (
          <MenuItem>
            <ListItemIcon>
              <PersonIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText
              primary="Your Identity"
              secondary={
                <Stack direction="row" alignItems="center" spacing={0.5}>
                  <Typography variant="caption" sx={{ fontFamily: 'monospace' }}>
                    {formatFourWords(networkState.userFourWords)}
                  </Typography>
                  <IconButton
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation();
                      copyToClipboard(networkState.userFourWords!, 'user');
                    }}
                  >
                    <CopyIcon fontSize="small" />
                  </IconButton>
                </Stack>
              }
            />
          </MenuItem>
        )}

        {/* Endpoint Four-Words */}
        {networkState.endpointFourWords && (
          <MenuItem>
            <ListItemIcon>
              <ComputerIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText
              primary="Connection Endpoint"
              secondary={
                <Stack direction="row" alignItems="center" spacing={0.5}>
                  <Typography
                    variant="caption"
                    sx={{
                      fontFamily: 'monospace',
                      color: 'success.main',
                      fontWeight: 'bold'
                    }}
                  >
                    {formatFourWords(networkState.endpointFourWords)}
                  </Typography>
                  <IconButton
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation();
                      copyToClipboard(networkState.endpointFourWords!, 'endpoint');
                    }}
                  >
                    <CopyIcon fontSize="small" />
                  </IconButton>
                </Stack>
              }
              secondaryTypographyProps={{
                component: 'div'
              }}
            />
          </MenuItem>
        )}

        <Divider />

        {/* Connection Actions */}
        {networkState.status === 'connected' ? (
          <>
            <MenuItem onClick={handleSyncNow}>
              <ListItemIcon>
                <CloudSyncIcon fontSize="small" />
              </ListItemIcon>
              <ListItemText primary="Sync Now" />
            </MenuItem>
            <MenuItem onClick={handleDisconnect}>
              <ListItemIcon>
                <WifiOffIcon fontSize="small" />
              </ListItemIcon>
              <ListItemText primary="Go Offline" />
            </MenuItem>
          </>
        ) : (
          <>
            <MenuItem onClick={handleConnect}>
              <ListItemIcon>
                <WifiIcon fontSize="small" />
              </ListItemIcon>
              <ListItemText primary="Connect to Network" />
            </MenuItem>
            <MenuItem onClick={handleBootstrapConnect}>
              <ListItemIcon>
                <LinkIcon fontSize="small" />
              </ListItemIcon>
              <ListItemText primary="Connect via Four-Words..." />
            </MenuItem>
          </>
        )}

        <Divider />

        {/* Local Storage Info */}
        <MenuItem onClick={handleMenuClose}>
          <ListItemIcon>
            <StorageIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText
            primary="Local Storage"
            secondary={networkState.status === 'local' || networkState.status === 'offline'
              ? "Working in offline mode"
              : "Syncing with network"
            }
          />
        </MenuItem>

        {/* Bootstrap Nodes */}
        {networkState.bootstrapNodes.length > 0 && (
          <MenuItem disabled>
            <ListItemIcon>
              <SettingsIcon fontSize="small" />
            </ListItemIcon>
            <ListItemText
              primary="Bootstrap Nodes"
              secondary={`${networkState.bootstrapNodes.length} configured`}
            />
          </MenuItem>
        )}
      </Menu>

      {/* Bootstrap Connection Dialog */}
      <BootstrapConnectionDialog
        open={bootstrapDialogOpen}
        onClose={() => setBootstrapDialogOpen(false)}
        onConnected={(fourWords) => {
          console.log(`Connected via Four-Words: ${fourWords}`);
        }}
      />
    </>
  );
};

export default NetworkStatusBar;