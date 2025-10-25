// Copyright (c) 2025 Saorsa Labs Limited
//
// Update Notification Component (Enhanced)
//
// Displays update notifications with comprehensive progress tracking
// Design: Matches STORYBOARD.md design system

import {
    Close as CloseIcon,
    Download as DownloadIcon, InfoOutlined, NewReleases as UpdateIcon
} from '@mui/icons-material';
import {
    Alert,
    AlertTitle,
    Box,
    Button, Chip, Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    IconButton,
    Snackbar, Stack, Typography
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import React, { useCallback, useEffect, useState } from 'react';
import { UpdateProgress } from './update/UpdateProgress';

// STORYBOARD.md Design Tokens
const TOKENS = {
  bgPrimary: '#161C20',
  bgSecondary: '#1a1f24',
  bgTertiary: '#101518',
  borderColor: '#2a3038',
  accentGreen: '#2EB67D',
  accentBlue: '#1E88E5',
  textPrimary: '#F4F6F8',
  textSecondary: '#9AA2AB',
};

interface UpdateStatus {
  available: boolean;
  current_version: string;
  latest_version?: string;
  download_url?: string;
  release_notes?: string;
  checking: boolean;
  error?: string;
}

interface UpdateProgressState {
  status: 'idle' | 'checking' | 'downloading' | 'installing' | 'completed' | 'error';
  progress: number;
  downloadSpeed?: number;
  timeRemaining?: number;
  totalSize?: number;
  downloadedBytes?: number;
  errorMessage?: string;
}

export const UpdateNotification: React.FC = () => {
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const [progressState, setProgressState] = useState<UpdateProgressState>({
    status: 'idle',
    progress: 0,
  });

  useEffect(() => {
    // Check for updates on mount
    checkForUpdates();

    // Check every 6 hours
    const interval = setInterval(checkForUpdates, 6 * 60 * 60 * 1000);

    // Listen for update progress events from backend
    const unlistenPromises: Promise<() => void>[] = [];

    // Download progress
    unlistenPromises.push(
      listen<{ progress: number; speed: number; total: number; downloaded: number }>(
        'update-download-progress',
        (event) => {
          setProgressState(prev => ({
            ...prev,
            status: 'downloading',
            progress: event.payload.progress,
            downloadSpeed: event.payload.speed,
            totalSize: event.payload.total,
            downloadedBytes: event.payload.downloaded,
            timeRemaining: event.payload.speed > 0
              ? (event.payload.total - event.payload.downloaded) / event.payload.speed
              : undefined,
          }));
        }
      )
    );

    // Install progress
    unlistenPromises.push(
      listen<{ progress: number }>('update-install-progress', (event) => {
        setProgressState(prev => ({
          ...prev,
          status: 'installing',
          progress: event.payload.progress,
        }));
      })
    );

    // Update completed
    unlistenPromises.push(
      listen('update-completed', () => {
        setProgressState(prev => ({
          ...prev,
          status: 'completed',
          progress: 100,
        }));
      })
    );

    // Update error
    unlistenPromises.push(
      listen<{ message: string }>('update-error', (event) => {
        setProgressState(prev => ({
          ...prev,
          status: 'error',
          errorMessage: event.payload.message,
        }));
        setInstalling(false);
      })
    );

    // Cleanup
    return () => {
      clearInterval(interval);
      Promise.all(unlistenPromises).then(unlisteners => {
        unlisteners.forEach(unlisten => unlisten());
      });
    };
  }, []);

  const checkForUpdates = async () => {
    try {
      const status = await invoke<UpdateStatus>('check_for_updates');
      setUpdateStatus(status);

      if (status.available) {
        setDialogOpen(true);
      }
    } catch (error) {
      console.error('Failed to check for updates:', error);
      setUpdateStatus({
        available: false,
        current_version: '0.0.0',
        checking: false,
        error: String(error),
      });
    }
  };

  const handleInstall = async () => {
    setInstalling(true);
    setProgressState({
      status: 'downloading',
      progress: 0,
    });

    try {
      await invoke('install_update');
      setSnackbarOpen(true);

      // App will restart automatically after update
    } catch (error) {
      console.error('Failed to install update:', error);
      setProgressState({
        status: 'error',
        progress: 0,
        errorMessage: String(error),
      });
      setInstalling(false);
    }
  };

  const handleCancelUpdate = useCallback(() => {
    // Reset progress state
    setProgressState({
      status: 'idle',
      progress: 0,
    });
    setInstalling(false);
  }, []);

  const handleRetryUpdate = useCallback(() => {
    handleInstall();
  }, []);

  const handleClose = () => {
    setDialogOpen(false);
  };

  if (!updateStatus?.available) {
    return null;
  }

  return (
    <>
      {/* Update Available Dialog - STORYBOARD.md styled */}
      <Dialog
        open={dialogOpen}
        onClose={installing ? undefined : handleClose}
        maxWidth="sm"
        fullWidth
        PaperProps={{
          sx: {
            bgcolor: TOKENS.bgSecondary,
            border: `1px solid ${TOKENS.borderColor}`,
            borderRadius: 2,
          },
        }}
      >
        <DialogTitle sx={{ pb: 2 }}>
          <Stack direction="row" alignItems="center" spacing={1.5}>
            <Box
              sx={{
                width: 40,
                height: 40,
                borderRadius: '50%',
                background: `linear-gradient(135deg, ${TOKENS.accentGreen}, ${TOKENS.accentBlue})`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: 'white',
              }}
            >
              <UpdateIcon />
            </Box>
            <Box flex={1}>
              <Typography variant="h6" sx={{ color: TOKENS.textPrimary, fontSize: 16, fontWeight: 600 }}>
                Update Available
              </Typography>
              <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
                A new version is ready to install
              </Typography>
            </Box>
            {updateStatus?.latest_version && (
              <Chip
                label={`v${updateStatus.latest_version}`}
                size="small"
                sx={{
                  bgcolor: `${TOKENS.accentGreen}20`,
                  color: TOKENS.accentGreen,
                  fontFamily: 'monospace',
                }}
              />
            )}
          </Stack>
          {!installing && (
            <IconButton
              aria-label="close"
              onClick={handleClose}
              sx={{
                position: 'absolute',
                right: 8,
                top: 8,
                color: TOKENS.textSecondary,
                '&:hover': {
                  color: TOKENS.textPrimary,
                  bgcolor: `${TOKENS.borderColor}80`,
                },
              }}
            >
              <CloseIcon />
            </IconButton>
          )}
        </DialogTitle>

        <DialogContent sx={{ pt: 0 }}>
          <Stack spacing={2}>
            {/* Version Info */}
            <Box
              sx={{
                p: 1.5,
                bgcolor: TOKENS.bgTertiary,
                borderRadius: 1,
                border: `1px solid ${TOKENS.borderColor}`,
              }}
            >
              <Stack direction="row" justifyContent="space-between" spacing={2}>
                <Box>
                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
                    Current
                  </Typography>
                  <Typography variant="body2" sx={{ color: TOKENS.textPrimary, fontFamily: 'monospace' }}>
                    v{updateStatus?.current_version}
                  </Typography>
                </Box>
                <Box sx={{ textAlign: 'right' }}>
                  <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
                    New
                  </Typography>
                  <Typography
                    variant="body2"
                    sx={{
                      color: TOKENS.accentGreen,
                      fontFamily: 'monospace',
                      fontWeight: 600,
                    }}
                  >
                    v{updateStatus?.latest_version}
                  </Typography>
                </Box>
              </Stack>
            </Box>

            {/* Progress Display */}
            {installing && progressState.status !== 'idle' && (
              <UpdateProgress
                status={progressState.status}
                progress={progressState.progress}
                version={updateStatus?.latest_version}
                downloadSpeed={progressState.downloadSpeed}
                timeRemaining={progressState.timeRemaining}
                totalSize={progressState.totalSize}
                downloadedBytes={progressState.downloadedBytes}
                errorMessage={progressState.errorMessage}
                onCancel={progressState.status === 'downloading' ? handleCancelUpdate : undefined}
                onRetry={progressState.status === 'error' ? handleRetryUpdate : undefined}
              />
            )}

            {/* Release Notes */}
            {updateStatus?.release_notes && !installing && (
              <Box>
                <Stack direction="row" alignItems="center" spacing={1} mb={1}>
                  <InfoOutlined sx={{ fontSize: 16, color: TOKENS.textSecondary }} />
                  <Typography variant="body2" fontWeight={600} sx={{ color: TOKENS.textPrimary }}>
                    What's New
                  </Typography>
                </Stack>
                <Box
                  sx={{
                    maxHeight: 200,
                    overflow: 'auto',
                    p: 1.5,
                    bgcolor: TOKENS.bgTertiary,
                    borderRadius: 1,
                    border: `1px solid ${TOKENS.borderColor}`,
                  }}
                >
                  <Typography
                    variant="body2"
                    component="pre"
                    sx={{
                      whiteSpace: 'pre-wrap',
                      fontFamily: 'monospace',
                      color: TOKENS.textSecondary,
                      fontSize: 12,
                      lineHeight: 1.6,
                    }}
                  >
                    {updateStatus.release_notes}
                  </Typography>
                </Box>
              </Box>
            )}
          </Stack>
        </DialogContent>

        {!installing && (
          <DialogActions sx={{ p: 2, pt: 0 }}>
            <Button
              onClick={handleClose}
              sx={{
                color: TOKENS.textSecondary,
                '&:hover': {
                  bgcolor: `${TOKENS.borderColor}80`,
                },
              }}
            >
              Later
            </Button>
            <Button
              onClick={handleInstall}
              variant="contained"
              startIcon={<DownloadIcon />}
              sx={{
                bgcolor: TOKENS.accentGreen,
                color: TOKENS.bgTertiary,
                fontWeight: 600,
                '&:hover': {
                  bgcolor: TOKENS.accentGreen,
                  filter: 'brightness(1.1)',
                  transform: 'translateY(-1px)',
                },
              }}
            >
              Install Update
            </Button>
          </DialogActions>
        )}
      </Dialog>

      {/* Update Success Snackbar - STORYBOARD.md styled */}
      <Snackbar
        open={snackbarOpen}
        autoHideDuration={6000}
        onClose={() => setSnackbarOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          onClose={() => setSnackbarOpen(false)}
          severity="success"
          variant="filled"
          sx={{
            bgcolor: TOKENS.accentGreen,
            color: TOKENS.bgTertiary,
            border: `1px solid ${TOKENS.accentGreen}`,
            boxShadow: `0 4px 12px ${TOKENS.accentGreen}40`,
          }}
        >
          <AlertTitle sx={{ fontWeight: 600 }}>Update Installed</AlertTitle>
          Communitas will restart to complete the update.
        </Alert>
      </Snackbar>
    </>
  );
};

export default UpdateNotification;
