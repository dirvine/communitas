import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  Alert,
  CircularProgress,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Divider,
  alpha,
  useTheme,
} from '@mui/material';
import {
  Fingerprint as FingerprintIcon,
  Security as SecurityIcon,
  Verified as VerifiedIcon,
  Warning as WarningIcon,
  CheckCircle as CheckCircleIcon,
  DevicesOther as DevicesIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

interface PasskeyInfo {
  four_words: string;
  registered_at: number;
  last_used: number | null;
  device_name: string;
}

interface PasskeyRegistrationProps {
  open: boolean;
  fourWords: string;
  displayName: string;
  onClose: () => void;
  onSuccess: () => void;
}

export const PasskeyRegistration: React.FC<PasskeyRegistrationProps> = ({
  open,
  fourWords,
  displayName,
  onClose,
  onSuccess,
}) => {
  const theme = useTheme();
  const [step, setStep] = useState<'intro' | 'registering' | 'success' | 'error'>('intro');
  const [error, setError] = useState<string | null>(null);
  const [passkeyInfo, setPasskeyInfo] = useState<PasskeyInfo | null>(null);
  const [deviceName, setDeviceName] = useState('');

  // Detect device/platform on mount
  useEffect(() => {
    if (open) {
      detectDevice();
      checkExistingPasskey();
    }
  }, [open, fourWords]);

  const detectDevice = () => {
    const userAgent = navigator.userAgent;
    let device = 'This Device';

    if (userAgent.includes('Mac')) {
      device = 'MacBook';
    } else if (userAgent.includes('Windows')) {
      device = 'Windows PC';
    } else if (userAgent.includes('Linux')) {
      device = 'Linux Machine';
    }

    setDeviceName(device);
  };

  const checkExistingPasskey = async () => {
    try {
      const hasPasskey = await invoke<boolean>('auth_passkey_has_passkey', {
        fourWords,
      });

      if (hasPasskey) {
        const info = await invoke<PasskeyInfo>('auth_passkey_get_info', {
          fourWords,
        });
        setPasskeyInfo(info);
      }
    } catch (err) {
      console.error('Failed to check existing passkey:', err);
    }
  };

  const handleRegister = async () => {
    try {
      setStep('registering');
      setError(null);

      // Register passkey with backend
      const info = await invoke<PasskeyInfo>('auth_passkey_register', {
        fourWords,
        deviceName,
      });

      setPasskeyInfo(info);
      setStep('success');

      // Call success callback after a short delay
      setTimeout(() => {
        onSuccess();
        handleClose();
      }, 2000);
    } catch (err: any) {
      console.error('Passkey registration failed:', err);
      setError(err?.message || 'Failed to register passkey. Please try again.');
      setStep('error');
    }
  };

  const handleClose = () => {
    // Reset state
    setStep('intro');
    setError(null);
    onClose();
  };

  const formatDate = (timestamp: number): string => {
    return new Date(timestamp * 1000).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  return (
    <Dialog
      open={open}
      onClose={step === 'registering' ? undefined : handleClose}
      maxWidth="sm"
      fullWidth
      PaperProps={{
        sx: {
          borderRadius: 3,
        },
      }}
    >
      <DialogTitle sx={{ pb: 1 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}>
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: 48,
              height: 48,
              borderRadius: 2,
              bgcolor: (theme) => alpha(theme.palette.primary.main, 0.1),
            }}
          >
            <FingerprintIcon color="primary" fontSize="large" />
          </Box>
          <Box>
            <Typography variant="h6" fontWeight={600}>
              Biometric Authentication
            </Typography>
            <Typography variant="caption" color="text.secondary">
              {displayName} • {fourWords}
            </Typography>
          </Box>
        </Box>
      </DialogTitle>

      <DialogContent>
        {/* Introduction Step */}
        {step === 'intro' && !passkeyInfo && (
          <>
            <Alert severity="info" sx={{ mb: 3 }}>
              Enable quick sign-in with Touch ID, Face ID, or Windows Hello
            </Alert>

            <Typography variant="body2" paragraph>
              With biometric authentication, you can sign in securely without entering your password
              every time. Your biometric data never leaves your device.
            </Typography>

            <List sx={{ bgcolor: (theme) => alpha(theme.palette.primary.main, 0.05), borderRadius: 2, py: 1 }}>
              <ListItem>
                <ListItemIcon>
                  <SecurityIcon color="primary" />
                </ListItemIcon>
                <ListItemText
                  primary="Secure & Private"
                  secondary="Your biometric data stays on your device"
                />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <VerifiedIcon color="primary" />
                </ListItemIcon>
                <ListItemText
                  primary="Fast & Convenient"
                  secondary="Sign in with a touch or glance"
                />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <DevicesIcon color="primary" />
                </ListItemIcon>
                <ListItemText
                  primary="Device-Specific"
                  secondary={`Only works on ${deviceName}`}
                />
              </ListItem>
            </List>
          </>
        )}

        {/* Already Registered */}
        {step === 'intro' && passkeyInfo && (
          <>
            <Alert severity="success" icon={<CheckCircleIcon />} sx={{ mb: 3 }}>
              Passkey already registered for this identity
            </Alert>

            <Box
              sx={{
                bgcolor: (theme) => alpha(theme.palette.success.main, 0.05),
                borderRadius: 2,
                p: 2,
              }}
            >
              <Typography variant="body2" fontWeight={600} gutterBottom>
                Current Passkey
              </Typography>
              <Typography variant="body2" color="text.secondary" paragraph>
                Device: {passkeyInfo.device_name}
              </Typography>
              <Typography variant="body2" color="text.secondary" paragraph>
                Registered: {formatDate(passkeyInfo.registered_at)}
              </Typography>
              {passkeyInfo.last_used && (
                <Typography variant="body2" color="text.secondary">
                  Last used: {formatDate(passkeyInfo.last_used)}
                </Typography>
              )}
            </Box>

            <Alert severity="info" sx={{ mt: 2 }}>
              You can continue using your existing passkey or re-register on this device.
            </Alert>
          </>
        )}

        {/* Registering Step */}
        {step === 'registering' && (
          <Box
            sx={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              py: 4,
              gap: 3,
            }}
          >
            <CircularProgress size={64} />
            <Box sx={{ textAlign: 'center' }}>
              <Typography variant="h6" fontWeight={600} gutterBottom>
                Setting up biometric authentication
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Please complete the biometric verification on your device
              </Typography>
            </Box>
          </Box>
        )}

        {/* Success Step */}
        {step === 'success' && (
          <Box
            sx={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              py: 4,
              gap: 2,
            }}
          >
            <Box
              sx={{
                width: 80,
                height: 80,
                borderRadius: '50%',
                bgcolor: (theme) => alpha(theme.palette.success.main, 0.1),
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <CheckCircleIcon sx={{ fontSize: 48, color: 'success.main' }} />
            </Box>
            <Box sx={{ textAlign: 'center' }}>
              <Typography variant="h6" fontWeight={600} gutterBottom>
                Passkey Registered Successfully!
              </Typography>
              <Typography variant="body2" color="text.secondary">
                You can now use biometric authentication to sign in
              </Typography>
            </Box>
          </Box>
        )}

        {/* Error Step */}
        {step === 'error' && (
          <>
            <Alert severity="error" icon={<WarningIcon />} sx={{ mb: 3 }}>
              {error || 'Failed to register passkey'}
            </Alert>

            <Typography variant="body2" paragraph>
              Passkey registration failed. This could be due to:
            </Typography>

            <List dense>
              <ListItem>
                <ListItemText
                  primary="• Your device doesn't support biometric authentication"
                />
              </ListItem>
              <ListItem>
                <ListItemText
                  primary="• Biometric authentication is disabled in your system settings"
                />
              </ListItem>
              <ListItem>
                <ListItemText
                  primary="• The authentication was cancelled or timed out"
                />
              </ListItem>
            </List>

            <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
              You can still use your password to sign in.
            </Typography>
          </>
        )}
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 3 }}>
        {/* Intro Actions */}
        {step === 'intro' && (
          <>
            <Button onClick={handleClose} variant="outlined">
              Cancel
            </Button>
            <Button
              onClick={handleRegister}
              variant="contained"
              startIcon={<FingerprintIcon />}
            >
              {passkeyInfo ? 'Re-register Passkey' : 'Register Passkey'}
            </Button>
          </>
        )}

        {/* Registering Actions */}
        {step === 'registering' && (
          <Typography variant="caption" color="text.secondary" sx={{ flex: 1, textAlign: 'center' }}>
            Follow the prompt on your device
          </Typography>
        )}

        {/* Error Actions */}
        {step === 'error' && (
          <>
            <Button onClick={handleClose} variant="outlined">
              Close
            </Button>
            <Button onClick={handleRegister} variant="contained">
              Try Again
            </Button>
          </>
        )}
      </DialogActions>
    </Dialog>
  );
};
