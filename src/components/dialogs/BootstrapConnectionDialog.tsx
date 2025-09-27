/**
 * Bootstrap Connection Dialog
 * Allows users to connect to the network via Four-Word identities
 * - Enter friend's computer Four-Word connection identity
 * - Display our connection Four-Words
 * - Save trusted bootstrap nodes
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  Typography,
  Box,
  Alert,
  Stack,
  Chip,
  IconButton,
  Tooltip,
  CircularProgress,
  Paper,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  Divider,
  InputAdornment,
} from '@mui/material';
import {
  ContentCopy as CopyIcon,
  Close as CloseIcon,
  Check as CheckIcon,
  Wifi as WifiIcon,
  WifiOff as WifiOffIcon,
  Computer as ComputerIcon,
  Person as PersonIcon,
  Delete as DeleteIcon,
  Add as AddIcon,
  QrCode2 as QrCodeIcon,
  Share as ShareIcon,
} from '@mui/icons-material';
import { networkService } from '../../services/network/NetworkConnectionService';
import { offlineStorage } from '../../services/storage/OfflineStorageService';
import { validateFourWordIdentity } from '../../utils/identity';
import { invoke } from '@tauri-apps/api/core';

interface BootstrapConnectionDialogProps {
  open: boolean;
  onClose: () => void;
  onConnected?: (fourWords: string) => void;
}

interface BootstrapNode {
  fourWords: string;
  label?: string;
  lastUsed?: Date;
  trusted: boolean;
}

export const BootstrapConnectionDialog: React.FC<BootstrapConnectionDialogProps> = ({
  open,
  onClose,
  onConnected,
}) => {
  const [inputFourWords, setInputFourWords] = useState('');
  const [validationError, setValidationError] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [savedNodes, setSavedNodes] = useState<BootstrapNode[]>([]);
  const [nodeLabel, setNodeLabel] = useState('');

  // Network state
  const [networkState, setNetworkState] = useState(networkService.getState());

  useEffect(() => {
    // Subscribe to network state changes
    const unsubscribe = networkService.subscribe(setNetworkState);

    // Load saved bootstrap nodes
    loadSavedNodes();

    return unsubscribe;
  }, []);

  const loadSavedNodes = async () => {
    const nodes = await offlineStorage.get<BootstrapNode[]>('bootstrap_nodes');
    if (nodes) {
      setSavedNodes(nodes);
    }
  };

  const saveBootstrapNode = async (fourWords: string, label?: string) => {
    const newNode: BootstrapNode = {
      fourWords,
      label: label || fourWords.slice(0, 20),
      lastUsed: new Date(),
      trusted: true,
    };

    const updatedNodes = [...savedNodes.filter(n => n.fourWords !== fourWords), newNode];
    setSavedNodes(updatedNodes);
    await offlineStorage.store('bootstrap_nodes', updatedNodes, { encrypt: true });
  };

  const removeBootstrapNode = async (fourWords: string) => {
    const updatedNodes = savedNodes.filter(n => n.fourWords !== fourWords);
    setSavedNodes(updatedNodes);
    await offlineStorage.store('bootstrap_nodes', updatedNodes, { encrypt: true });
  };

  const handleInputChange = async (value: string) => {
    setInputFourWords(value);
    setValidationError(null);

    if (value.trim()) {
      setIsValidating(true);
      try {
        const isValid = await validateFourWordIdentity(value);

        if (!isValid) {
          setValidationError('Invalid Four-Word format. Should be like: ocean-forest-moon-star');
        } else {
          // Try backend validation if available
          try {
            const backendCheck = await invoke<boolean>('validate_four_words', {
              fourWords: value.trim().toLowerCase().replace(/\s+/g, '-')
            });
            if (!backendCheck) {
              setValidationError('Four-Words not recognized by network');
            }
          } catch {
            // Backend not available, local validation is enough
          }
        }
      } catch (error) {
        setValidationError('Validation failed');
      } finally {
        setIsValidating(false);
      }
    }
  };

  const handleConnect = async () => {
    const fourWords = inputFourWords.trim().toLowerCase().replace(/\s+/g, '-');

    if (!fourWords) {
      setValidationError('Please enter Four-Words');
      return;
    }

    if (validationError) {
      return;
    }

    setIsConnecting(true);
    try {
      // Save as bootstrap node if requested
      if (nodeLabel) {
        await saveBootstrapNode(fourWords, nodeLabel);
      }

      // Attempt connection via Four-Words
      const result = await invoke<boolean>('connect_via_four_words', { fourWords });

      if (result) {
        // Update network service
        await networkService.connect();

        if (onConnected) {
          onConnected(fourWords);
        }

        // Close dialog after successful connection
        setTimeout(() => {
          onClose();
        }, 1000);
      } else {
        setValidationError('Failed to connect to this bootstrap node');
      }
    } catch (error) {
      console.error('Connection failed:', error);
      setValidationError(error instanceof Error ? error.message : 'Connection failed');
    } finally {
      setIsConnecting(false);
    }
  };

  const handleConnectToSaved = async (node: BootstrapNode) => {
    setInputFourWords(node.fourWords);
    setNodeLabel(node.label || '');
    await handleConnect();
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

  const formatFourWords = (fourWords: string | null) => {
    if (!fourWords) return 'Not available';
    return fourWords.split('-').map(word =>
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ');
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="sm"
      fullWidth
      PaperProps={{
        sx: { minHeight: 500 }
      }}
    >
      <DialogTitle>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Stack direction="row" alignItems="center" spacing={1}>
            <WifiIcon color="primary" />
            <Typography variant="h6">Bootstrap Connection</Typography>
          </Stack>
          <IconButton onClick={onClose} size="small">
            <CloseIcon />
          </IconButton>
        </Stack>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={3}>
          {/* Our Connection Identity */}
          <Paper elevation={2} sx={{ p: 2, bgcolor: 'background.default' }}>
            <Stack spacing={2}>
              <Stack direction="row" alignItems="center" spacing={1}>
                <PersonIcon fontSize="small" color="primary" />
                <Typography variant="subtitle2" color="primary">
                  Your Connection Identity
                </Typography>
              </Stack>

              {/* User Four-Words */}
              <Box>
                <Typography variant="caption" color="text.secondary" gutterBottom>
                  User Identity:
                </Typography>
                <Stack direction="row" alignItems="center" spacing={1}>
                  <Typography variant="body2" fontFamily="monospace">
                    {formatFourWords(networkState.userFourWords)}
                  </Typography>
                  {networkState.userFourWords && (
                    <Tooltip title={copiedField === 'user' ? 'Copied!' : 'Copy'}>
                      <IconButton
                        size="small"
                        onClick={() => copyToClipboard(networkState.userFourWords!, 'user')}
                      >
                        {copiedField === 'user' ? <CheckIcon fontSize="small" /> : <CopyIcon fontSize="small" />}
                      </IconButton>
                    </Tooltip>
                  )}
                </Stack>
              </Box>

              {/* Endpoint Four-Words */}
              <Box>
                <Typography variant="caption" color="text.secondary" gutterBottom>
                  Connection Endpoint (Share this):
                </Typography>
                <Stack direction="row" alignItems="center" spacing={1}>
                  <Typography
                    variant="body1"
                    fontFamily="monospace"
                    sx={{
                      fontWeight: 'bold',
                      color: networkState.endpointFourWords ? 'success.main' : 'text.disabled'
                    }}
                  >
                    {formatFourWords(networkState.endpointFourWords)}
                  </Typography>
                  {networkState.endpointFourWords && (
                    <>
                      <Tooltip title={copiedField === 'endpoint' ? 'Copied!' : 'Copy'}>
                        <IconButton
                          size="small"
                          onClick={() => copyToClipboard(networkState.endpointFourWords!, 'endpoint')}
                        >
                          {copiedField === 'endpoint' ? <CheckIcon fontSize="small" /> : <CopyIcon fontSize="small" />}
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Share">
                        <IconButton size="small">
                          <ShareIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Show QR Code">
                        <IconButton size="small">
                          <QrCodeIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                    </>
                  )}
                </Stack>
              </Box>

              {/* Connection Status */}
              <Stack direction="row" alignItems="center" spacing={1}>
                {networkState.status === 'connected' ? (
                  <>
                    <Chip
                      icon={<WifiIcon />}
                      label={`Connected (${networkState.peers} peers)`}
                      color="success"
                      size="small"
                    />
                  </>
                ) : networkState.status === 'connecting' ? (
                  <Chip
                    icon={<CircularProgress size={16} />}
                    label="Connecting..."
                    color="warning"
                    size="small"
                  />
                ) : (
                  <Chip
                    icon={<WifiOffIcon />}
                    label="Not connected"
                    color="default"
                    size="small"
                  />
                )}
              </Stack>
            </Stack>
          </Paper>

          <Divider />

          {/* Connect via Four-Words */}
          <Stack spacing={2}>
            <Typography variant="subtitle2">
              Connect via Friend's Four-Words
            </Typography>

            <TextField
              fullWidth
              placeholder="e.g., ocean-forest-moon-star"
              value={inputFourWords}
              onChange={(e) => handleInputChange(e.target.value)}
              error={!!validationError}
              helperText={validationError}
              disabled={isConnecting}
              InputProps={{
                startAdornment: (
                  <InputAdornment position="start">
                    <ComputerIcon fontSize="small" />
                  </InputAdornment>
                ),
                endAdornment: isValidating && (
                  <InputAdornment position="end">
                    <CircularProgress size={20} />
                  </InputAdornment>
                ),
              }}
            />

            <TextField
              fullWidth
              placeholder="Label (optional)"
              value={nodeLabel}
              onChange={(e) => setNodeLabel(e.target.value)}
              disabled={isConnecting}
              size="small"
              helperText="Save this as a trusted bootstrap node"
            />
          </Stack>

          {/* Saved Bootstrap Nodes */}
          {savedNodes.length > 0 && (
            <>
              <Divider />
              <Stack spacing={1}>
                <Typography variant="subtitle2">
                  Saved Bootstrap Nodes
                </Typography>
                <List dense>
                  {savedNodes.map((node) => (
                    <ListItem key={node.fourWords} disablePadding>
                      <ListItemText
                        primary={node.label}
                        secondary={formatFourWords(node.fourWords)}
                        secondaryTypographyProps={{ fontFamily: 'monospace', fontSize: '0.75rem' }}
                      />
                      <ListItemSecondaryAction>
                        <Stack direction="row" spacing={1}>
                          <Tooltip title="Connect">
                            <IconButton
                              size="small"
                              onClick={() => handleConnectToSaved(node)}
                              disabled={isConnecting}
                            >
                              <WifiIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                          <Tooltip title="Remove">
                            <IconButton
                              size="small"
                              onClick={() => removeBootstrapNode(node.fourWords)}
                            >
                              <DeleteIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                        </Stack>
                      </ListItemSecondaryAction>
                    </ListItem>
                  ))}
                </List>
              </Stack>
            </>
          )}

          {/* Connection Tips */}
          <Alert severity="info" variant="outlined">
            <Typography variant="caption">
              <strong>Tip:</strong> Ask your friend for their Connection Endpoint Four-Words.
              They can find it in this same dialog when they're online.
            </Typography>
          </Alert>
        </Stack>
      </DialogContent>

      <DialogActions>
        <Button onClick={onClose}>
          Cancel
        </Button>
        <Button
          variant="contained"
          onClick={handleConnect}
          disabled={isConnecting || !!validationError || !inputFourWords.trim()}
          startIcon={isConnecting ? <CircularProgress size={16} /> : <WifiIcon />}
        >
          {isConnecting ? 'Connecting...' : 'Connect'}
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default BootstrapConnectionDialog;