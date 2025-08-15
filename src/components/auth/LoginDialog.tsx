import React, { useState } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
  Stack,
  Typography,
  IconButton,
  Alert,
  Box,
  Chip,
  Divider,
  Switch,
  FormControlLabel,
  CircularProgress,
} from '@mui/material';
import {
  Close as CloseIcon,
  Visibility as VisibilityIcon,
  VisibilityOff as VisibilityOffIcon,
  Key as KeyIcon,
  Person as PersonIcon,
  NetworkCheck as NetworkIcon,
  Security as SecurityIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useAuth } from '../../contexts/AuthContext';
import { useResponsive } from '../responsive';

interface LoginDialogProps {
  open: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

export const LoginDialog: React.FC<LoginDialogProps> = ({
  open,
  onClose,
  onSuccess,
}) => {
  const { login, createIdentity, authState } = useAuth();
  const { isMobile } = useResponsive();
  
  const [mode, setMode] = useState<'login' | 'create'>('login');
  const [formData, setFormData] = useState({
    fourWordAddress: '',
    privateKey: '',
    name: '',
    email: '',
  });
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [advancedMode, setAdvancedMode] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleLogin = async () => {
    if (!formData.fourWordAddress.trim()) {
      setError('Four-word address is required');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const success = await login(
        formData.fourWordAddress.trim(),
        advancedMode && formData.privateKey ? formData.privateKey : undefined
      );

      if (success) {
        onSuccess?.();
        onClose();
        // Reset form
        setFormData({
          fourWordAddress: '',
          privateKey: '',
          name: '',
          email: '',
        });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateIdentity = async () => {
    if (!formData.name.trim()) {
      setError('Name is required');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await createIdentity(formData.name.trim(), formData.email.trim() || undefined);
      onSuccess?.();
      onClose();
      // Reset form
      setFormData({
        fourWordAddress: '',
        privateKey: '',
        name: '',
        email: '',
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Identity creation failed');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (mode === 'login') {
      handleLogin();
    } else {
      handleCreateIdentity();
    }
  };

  const resetForm = () => {
    setFormData({
      fourWordAddress: '',
      privateKey: '',
      name: '',
      email: '',
    });
    setError(null);
    setAdvancedMode(false);
    setShowPrivateKey(false);
  };

  const switchMode = (newMode: 'login' | 'create') => {
    setMode(newMode);
    resetForm();
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="sm"
      fullWidth
      fullScreen={isMobile}
      PaperProps={{
        component: motion.div,
        initial: { opacity: 0, scale: 0.9 },
        animate: { opacity: 1, scale: 1 },
        transition: { duration: 0.2 },
      }}
    >
      <DialogTitle
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          pb: 1,
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <SecurityIcon color="primary" />
          <Typography variant="h6" fontWeight={600}>
            {mode === 'login' ? 'Sign In to Communitas' : 'Create New Identity'}
          </Typography>
        </Box>
        <IconButton onClick={onClose} size="small">
          <CloseIcon />
        </IconButton>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={3}>
          {/* Mode Switcher */}
          <Stack direction="row" spacing={1} justifyContent="center">
            <Chip
              label="Sign In"
              variant={mode === 'login' ? 'filled' : 'outlined'}
              color={mode === 'login' ? 'primary' : 'default'}
              onClick={() => switchMode('login')}
              sx={{ cursor: 'pointer', minWidth: 80 }}
            />
            <Chip
              label="Create Identity"
              variant={mode === 'create' ? 'filled' : 'outlined'}
              color={mode === 'create' ? 'primary' : 'default'}
              onClick={() => switchMode('create')}
              sx={{ cursor: 'pointer', minWidth: 120 }}
            />
          </Stack>

          <Divider />

          {/* Error Alert */}
          <AnimatePresence>
            {error && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: 'auto' }}
                exit={{ opacity: 0, height: 0 }}
              >
                <Alert severity="error" onClose={() => setError(null)}>
                  {error}
                </Alert>
              </motion.div>
            )}
          </AnimatePresence>

          {/* Auth State Error */}
          {authState.error && (
            <Alert severity="error">
              {authState.error}
            </Alert>
          )}

          <Box component="form" onSubmit={handleSubmit}>
            <Stack spacing={3}>
              {/* Login Mode */}
              {mode === 'login' && (
                <>
                  <TextField
                    fullWidth
                    label="Four-Word Address"
                    placeholder="brave-ocean-gentle-mountain"
                    value={formData.fourWordAddress}
                    onChange={(e) =>
                      setFormData(prev => ({ ...prev, fourWordAddress: e.target.value }))
                    }
                    InputProps={{
                      startAdornment: <PersonIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                    }}
                    helperText="Your unique four-word network address"
                    disabled={loading}
                  />

                  {/* Advanced Login Options */}
                  <FormControlLabel
                    control={
                      <Switch
                        checked={advancedMode}
                        onChange={(e) => setAdvancedMode(e.target.checked)}
                        disabled={loading}
                      />
                    }
                    label="Advanced Options"
                    sx={{ mb: advancedMode ? 0 : -1 }}
                  />

                  <AnimatePresence>
                    {advancedMode && (
                      <motion.div
                        initial={{ opacity: 0, height: 0 }}
                        animate={{ opacity: 1, height: 'auto' }}
                        exit={{ opacity: 0, height: 0 }}
                      >
                        <TextField
                          fullWidth
                          label="Private Key (Optional)"
                          type={showPrivateKey ? 'text' : 'password'}
                          value={formData.privateKey}
                          onChange={(e) =>
                            setFormData(prev => ({ ...prev, privateKey: e.target.value }))
                          }
                          InputProps={{
                            startAdornment: <KeyIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                            endAdornment: (
                              <IconButton
                                onClick={() => setShowPrivateKey(!showPrivateKey)}
                                edge="end"
                                size="small"
                              >
                                {showPrivateKey ? <VisibilityOffIcon /> : <VisibilityIcon />}
                              </IconButton>
                            ),
                          }}
                          helperText="Leave empty to use stored credentials or hardware wallet"
                          disabled={loading}
                        />
                      </motion.div>
                    )}
                  </AnimatePresence>
                </>
              )}

              {/* Create Identity Mode */}
              {mode === 'create' && (
                <>
                  <TextField
                    fullWidth
                    label="Display Name"
                    placeholder="Your Name"
                    value={formData.name}
                    onChange={(e) =>
                      setFormData(prev => ({ ...prev, name: e.target.value }))
                    }
                    InputProps={{
                      startAdornment: <PersonIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                    }}
                    helperText="How others will see you on the network"
                    disabled={loading}
                    required
                  />

                  <TextField
                    fullWidth
                    label="Email (Optional)"
                    type="email"
                    placeholder="you@example.com"
                    value={formData.email}
                    onChange={(e) =>
                      setFormData(prev => ({ ...prev, email: e.target.value }))
                    }
                    helperText="Used for notifications and recovery"
                    disabled={loading}
                  />

                  <Alert severity="info" sx={{ mt: 2 }}>
                    <Typography variant="body2">
                      Creating a new identity will generate a unique four-word address and 
                      cryptographic keys for secure communication on the P2P network.
                    </Typography>
                  </Alert>
                </>
              )}
            </Stack>
          </Box>

          {/* Network Status */}
          <Box
            sx={{
              p: 2,
              backgroundColor: 'background.paper',
              borderRadius: 1,
              border: '1px solid',
              borderColor: 'divider',
            }}
          >
            <Stack direction="row" alignItems="center" spacing={1}>
              <NetworkIcon color="primary" fontSize="small" />
              <Typography variant="body2" color="text.secondary">
                Network Status: Ready to connect
              </Typography>
            </Stack>
          </Box>
        </Stack>
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 3 }}>
        <Button onClick={onClose} disabled={loading}>
          Cancel
        </Button>
        <Button
          variant="contained"
          onClick={handleSubmit}
          disabled={loading || !formData.fourWordAddress.trim() && mode === 'login' || !formData.name.trim() && mode === 'create'}
          startIcon={
            loading ? (
              <CircularProgress size={20} />
            ) : mode === 'login' ? (
              <PersonIcon />
            ) : (
              <SecurityIcon />
            )
          }
          sx={{ minWidth: 120 }}
        >
          {loading
            ? 'Please wait...'
            : mode === 'login'
            ? 'Sign In'
            : 'Create Identity'
          }
        </Button>
      </DialogActions>
    </Dialog>
  );
};

export default LoginDialog;