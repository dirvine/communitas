import React, { useState, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  Chip,
  Stack,
  Alert,
  IconButton,
  Tooltip,
  CircularProgress,
  Collapse,
} from '@mui/material';
import {
  Security as SecurityIcon,
  CheckCircle as CheckIcon,
  Error as ErrorIcon,
  Info as InfoIcon,
  ExpandMore as ExpandIcon,
  ExpandLess as CollapseIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

interface SecureStorageInfo {
  available: boolean;
  backend: string;
  platform: string;
}

interface SecureStorageStatusProps {
  showDetails?: boolean;
  compact?: boolean;
}

export const SecureStorageStatus: React.FC<SecureStorageStatusProps> = ({
  showDetails = false,
  compact = false,
}) => {
  const [storageInfo, setStorageInfo] = useState<SecureStorageInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expanded, setExpanded] = useState(showDetails);

  useEffect(() => {
    fetchStorageInfo();
  }, []);

  const fetchStorageInfo = async () => {
    try {
      setLoading(true);
      setError(null);
      const info = await invoke<SecureStorageInfo>('get_secure_storage_info');
      setStorageInfo(info);
    } catch (err) {
      console.error('Failed to get secure storage info:', err);
      setError(err instanceof Error ? err.message : 'Failed to get storage info');
    } finally {
      setLoading(false);
    }
  };

  const getStatusColor = (): 'success' | 'error' | 'warning' => {
    if (!storageInfo) return 'warning';
    return storageInfo.available ? 'success' : 'error';
  };

  const getStatusIcon = () => {
    if (loading) return <CircularProgress size={16} />;
    if (error) return <ErrorIcon />;
    if (!storageInfo) return <InfoIcon />;
    return storageInfo.available ? <CheckIcon /> : <ErrorIcon />;
  };

  const getStatusText = (): string => {
    if (loading) return 'Checking...';
    if (error) return 'Error';
    if (!storageInfo) return 'Unknown';
    return storageInfo.available ? 'Available' : 'Unavailable';
  };

  const getPlatformDisplayName = (platform: string): string => {
    switch (platform) {
      case 'macos': return 'macOS';
      case 'windows': return 'Windows';
      case 'linux': return 'Linux';
      default: return platform;
    }
  };

  if (compact) {
    return (
      <Tooltip 
        title={
          storageInfo 
            ? `Secure storage: ${storageInfo.available ? 'Available' : 'Unavailable'} (${storageInfo.backend})`
            : 'Secure storage status'
        }
      >
        <Chip
          icon={getStatusIcon()}
          label={loading ? 'Storage' : getStatusText()}
          color={getStatusColor()}
          size="small"
          onClick={() => setExpanded(!expanded)}
        />
      </Tooltip>
    );
  }

  return (
    <Paper sx={{ p: 2 }}>
      <Stack spacing={2}>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Stack direction="row" alignItems="center" spacing={2}>
            <SecurityIcon color="primary" />
            <Typography variant="h6">Secure Storage</Typography>
            <Chip
              icon={getStatusIcon()}
              label={getStatusText()}
              color={getStatusColor()}
              size="small"
            />
          </Stack>

          {!showDetails && (
            <IconButton
              size="small"
              onClick={() => setExpanded(!expanded)}
            >
              {expanded ? <CollapseIcon /> : <ExpandIcon />}
            </IconButton>
          )}
        </Stack>

        {error && (
          <Alert severity="error">
            <Typography variant="body2">{error}</Typography>
          </Alert>
        )}

        <Collapse in={expanded || showDetails}>
          <Stack spacing={2}>
            {storageInfo && (
              <>
                <Box>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    Platform
                  </Typography>
                  <Typography variant="body1">
                    {getPlatformDisplayName(storageInfo.platform)}
                  </Typography>
                </Box>

                <Box>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    Storage Backend
                  </Typography>
                  <Typography variant="body1">
                    {storageInfo.backend}
                  </Typography>
                </Box>

                {storageInfo.available ? (
                  <Alert severity="success">
                    <Typography variant="body2">
                      Encryption keys are stored securely using your system's built-in secure storage.
                      Keys are encrypted and protected by your operating system.
                    </Typography>
                  </Alert>
                ) : (
                  <Alert severity="error">
                    <Typography variant="body2">
                      Secure storage is not available on your system. Encryption keys cannot be stored securely.
                      Please ensure your system supports secure storage or contact support for assistance.
                    </Typography>
                  </Alert>
                )}

                <Box>
                  <Typography variant="caption" color="text.secondary">
                    Secure storage provides hardware-backed encryption where available and integrates
                    with your operating system's credential management system.
                  </Typography>
                </Box>
              </>
            )}
          </Stack>
        </Collapse>
      </Stack>
    </Paper>
  );
};

export default SecureStorageStatus;