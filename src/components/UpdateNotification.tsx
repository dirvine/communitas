// Copyright (c) 2025 Saorsa Labs Limited
//
// Update Notification Component
//
// Displays update notifications and allows users to install updates

import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Alert,
  AlertTitle,
  Box,
  Button,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  IconButton,
  Snackbar,
  Typography,
} from '@mui/material';
import {
  Close as CloseIcon,
  Download as DownloadIcon,
  NewReleases as UpdateIcon,
} from '@mui/icons-material';

interface UpdateStatus {
  available: boolean;
  current_version: string;
  latest_version?: string;
  download_url?: string;
  release_notes?: string;
  checking: boolean;
  error?: string;
}

export const UpdateNotification: React.FC = () => {
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [snackbarOpen, setSnackbarOpen] = useState(false);

  useEffect(() => {
    // Check for updates on mount
    checkForUpdates();

    // Check every 6 hours
    const interval = setInterval(checkForUpdates, 6 * 60 * 60 * 1000);

    return () => clearInterval(interval);
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
    try {
      await invoke('install_update');
      setSnackbarOpen(true);
      setDialogOpen(false);

      // App will restart automatically after update
    } catch (error) {
      console.error('Failed to install update:', error);
      alert(`Update failed: ${error}`);
    } finally {
      setInstalling(false);
    }
  };

  const handleClose = () => {
    setDialogOpen(false);
  };

  if (!updateStatus?.available) {
    return null;
  }

  return (
    <>
      {/* Update Available Dialog */}
      <Dialog
        open={dialogOpen}
        onClose={handleClose}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          <Box display="flex" alignItems="center" gap={1}>
            <UpdateIcon color="primary" />
            <Typography variant="h6">Update Available</Typography>
          </Box>
          <IconButton
            aria-label="close"
            onClick={handleClose}
            sx={{
              position: 'absolute',
              right: 8,
              top: 8,
            }}
          >
            <CloseIcon />
          </IconButton>
        </DialogTitle>

        <DialogContent>
          <DialogContentText>
            <Typography variant="body1" gutterBottom>
              A new version of Communitas is available!
            </Typography>
            <Box mt={2} mb={2}>
              <Typography variant="body2" color="text.secondary">
                Current version: <strong>{updateStatus.current_version}</strong>
              </Typography>
              <Typography variant="body2" color="text.secondary">
                New version: <strong>{updateStatus.latest_version}</strong>
              </Typography>
            </Box>

            {updateStatus.release_notes && (
              <Box mt={2}>
                <Typography variant="subtitle2" gutterBottom>
                  What's New:
                </Typography>
                <Box
                  sx={{
                    maxHeight: 200,
                    overflow: 'auto',
                    p: 1,
                    bgcolor: 'background.default',
                    borderRadius: 1,
                  }}
                >
                  <Typography
                    variant="body2"
                    component="pre"
                    sx={{ whiteSpace: 'pre-wrap', fontFamily: 'monospace' }}
                  >
                    {updateStatus.release_notes}
                  </Typography>
                </Box>
              </Box>
            )}
          </DialogContentText>
        </DialogContent>

        <DialogActions>
          <Button onClick={handleClose} disabled={installing}>
            Later
          </Button>
          <Button
            onClick={handleInstall}
            variant="contained"
            startIcon={installing ? <CircularProgress size={20} /> : <DownloadIcon />}
            disabled={installing}
          >
            {installing ? 'Installing...' : 'Install Update'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Update Success Snackbar */}
      <Snackbar
        open={snackbarOpen}
        autoHideDuration={6000}
        onClose={() => setSnackbarOpen(false)}
      >
        <Alert
          onClose={() => setSnackbarOpen(false)}
          severity="success"
          variant="filled"
        >
          <AlertTitle>Update Installed</AlertTitle>
          Communitas will restart to complete the update.
        </Alert>
      </Snackbar>
    </>
  );
};

export default UpdateNotification;
