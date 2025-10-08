import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogContent,
  DialogActions,
  Box,
  Typography,
  Button,
  Card,
  CardContent,
  Chip,
  Alert,
  CircularProgress,
  alpha,
} from '@mui/material';
import { CheckCircle as CheckCircleIcon, Settings as SettingsIcon } from '@mui/icons-material';
import { useAuth } from '../../contexts/AuthContext';
import { invoke } from '@tauri-apps/api/core';

interface FirstLaunchWelcomeProps {
  open: boolean;
  onClose: () => void;
}

export const FirstLaunchWelcome: React.FC<FirstLaunchWelcomeProps> = ({ open, onClose }) => {
  const { createIdentity, getOsUsername, enableAutoLogin } = useAuth();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [fourWords, setFourWords] = useState<string>('');
  const [displayName, setDisplayName] = useState<string>('');
  const [step, setStep] = useState<'creating' | 'created' | 'complete'>('creating');

  useEffect(() => {
    if (open) {
      performFirstLaunchSetup();
    }
  }, [open]);

  const performFirstLaunchSetup = async () => {
    try {
      setLoading(true);
      setError(null);
      setStep('creating');

      // Step 1: Generate four-word identity
      console.log('🎲 Generating four-word identity...');
      const generatedFourWords = await invoke('generate_four_word_identity') as string;
      setFourWords(generatedFourWords);
      console.log('✅ Generated:', generatedFourWords);

      // Step 2: Get OS username
      console.log('👤 Getting OS username...');
      const osUsername = await getOsUsername();
      setDisplayName(osUsername);
      console.log('✅ OS Username:', osUsername);

      // Step 3: Create vault (using four-words as password)
      console.log('🔐 Creating encrypted vault...');
      await createIdentity(osUsername, {
        fourWords: generatedFourWords,
        password: generatedFourWords, // Use four-words as password
      });
      console.log('✅ Vault created');
      setStep('created');

      // Step 4: Enable auto-login (store in keyring)
      console.log('🔑 Enabling auto-login...');
      await enableAutoLogin(generatedFourWords, generatedFourWords);
      console.log('✅ Auto-login enabled');

      setStep('complete');
      setLoading(false);
    } catch (err) {
      console.error('First launch setup failed:', err);
      setError(err instanceof Error ? err.message : 'Setup failed. Please try again.');
      setLoading(false);
    }
  };

  const handleContinue = () => {
    console.log('🚀 First launch setup complete, continuing to app');
    onClose();
  };

  const handleCancel = () => {
    console.log('❌ New identity creation cancelled');
    onClose();
  };

  return (
    <Dialog open={open} maxWidth="md" fullWidth onClose={handleCancel}>
      <DialogContent sx={{ p: 4 }}>
        {loading ? (
          <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 3, py: 4 }}>
            <CircularProgress size={64} />
            <Typography variant="h6" color="text.secondary">
              {step === 'creating' && 'Setting up your identity...'}
              {step === 'created' && 'Configuring auto-login...'}
            </Typography>
            <Typography variant="body2" color="text.secondary" align="center">
              This will only take a moment
            </Typography>
            <Button
              variant="outlined"
              onClick={handleCancel}
              sx={{ mt: 2 }}
            >
              Cancel
            </Button>
          </Box>
        ) : error ? (
          <Box sx={{ textAlign: 'center' }}>
            <Alert severity="error" sx={{ mb: 3 }}>
              {error}
            </Alert>
            <Box sx={{ display: 'flex', justifyContent: 'center', gap: 2 }}>
              <Button variant="outlined" onClick={handleCancel}>
                Cancel
              </Button>
              <Button variant="contained" onClick={performFirstLaunchSetup}>
                Try Again
              </Button>
            </Box>
          </Box>
        ) : (
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
            {/* Success Icon */}
            <Box sx={{ textAlign: 'center' }}>
              <CheckCircleIcon sx={{ fontSize: 64, color: 'success.main', mb: 2 }} />
              <Typography variant="h4" fontWeight={600} gutterBottom>
                Welcome to Communitas! 🎉
              </Typography>
              <Typography variant="body1" color="text.secondary">
                Your secure identity has been created
              </Typography>
            </Box>

            {/* Identity Card */}
            <Card
              sx={{
                bgcolor: (theme) =>
                  theme.palette.mode === 'dark'
                    ? alpha(theme.palette.primary.main, 0.1)
                    : alpha(theme.palette.primary.main, 0.05),
                border: 1,
                borderColor: 'primary.main',
              }}
            >
              <CardContent sx={{ textAlign: 'center', py: 3 }}>
                <Typography variant="overline" color="text.secondary" gutterBottom>
                  Your Identity
                </Typography>
                <Typography
                  variant="h5"
                  fontWeight={600}
                  sx={{
                    fontFamily: 'monospace',
                    color: 'primary.main',
                    my: 2,
                  }}
                >
                  {fourWords}
                </Typography>
                <Box sx={{ display: 'flex', justifyContent: 'center', gap: 1, flexWrap: 'wrap' }}>
                  <Chip label={displayName} color="default" size="small" />
                  <Chip label="Auto-login Enabled" color="success" size="small" icon={<CheckCircleIcon />} />
                </Box>
              </CardContent>
            </Card>

            {/* Info Alerts */}
            <Alert severity="success" icon={<CheckCircleIcon />}>
              <Typography variant="body2" fontWeight={600} gutterBottom>
                You're all set!
              </Typography>
              <Typography variant="caption">
                This device will automatically sign you in. You can export a QR code from Settings
                to link other devices.
              </Typography>
            </Alert>

            <Alert severity="info" icon={<SettingsIcon />}>
              <Typography variant="caption">
                <strong>Pro tip:</strong> Your four-word identity is like your username on the network.
                You can share it with others, but your vault password stays secure on this device.
              </Typography>
            </Alert>
          </Box>
        )}
      </DialogContent>

      {!loading && !error && (
        <DialogActions sx={{ px: 4, pb: 3, justifyContent: 'center', gap: 2 }}>
          <Button
            variant="outlined"
            size="large"
            onClick={handleCancel}
            sx={{ minWidth: 150, py: 1.5 }}
          >
            Cancel
          </Button>
          <Button
            variant="contained"
            size="large"
            onClick={handleContinue}
            sx={{ minWidth: 150, py: 1.5 }}
          >
            Get Started
          </Button>
        </DialogActions>
      )}
    </Dialog>
  );
};
