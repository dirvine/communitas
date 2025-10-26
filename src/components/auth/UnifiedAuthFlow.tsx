import {
    ArrowForward as ArrowForwardIcon, AutoAwesome as AutoAwesomeIcon, CheckCircle as CheckCircleIcon, ContentCopy as ContentCopyIcon, Error as ErrorIcon, Fingerprint as FingerprintIcon, Info as InfoIcon, Key as KeyIcon, Lock as LockIcon, Person as PersonIcon, SaveAlt as SaveAltIcon, Visibility as VisibilityIcon,
    VisibilityOff as VisibilityOffIcon
} from '@mui/icons-material';
import {
    Alert, alpha, Box, Button, Card,
    CardContent, Chip, CircularProgress, Dialog, DialogActions, DialogContent, DialogTitle, Divider, IconButton, InputAdornment, LinearProgress, Paper, Stack, TextField, Typography, useTheme
} from '@mui/material';
import { AnimatePresence, motion } from 'framer-motion';
import React, { useEffect, useState } from 'react';
import { useAuth } from '../../contexts/AuthContext';
import { generateFourWordIdentity } from '../../utils/identity';
import { PasskeyRegistration } from './PasskeyRegistration';

interface UnifiedAuthFlowProps {
  initialMode?: 'login' | 'register';
  onSuccess?: () => void;
  onCancel?: () => void;
}

interface PasswordStrength {
  score: number;
  label: string;
  color: 'error' | 'warning' | 'success';
}

const calculatePasswordStrength = (password: string): PasswordStrength => {
  let score = 0;

  if (password.length >= 8) score++;
  if (password.length >= 12) score++;
  if (/[a-z]/.test(password)) score++;
  if (/[A-Z]/.test(password)) score++;
  if (/[0-9]/.test(password)) score++;
  if (/[^a-zA-Z0-9]/.test(password)) score++;

  if (score <= 2) return { score: 33, label: 'Weak', color: 'error' };
  if (score <= 4) return { score: 66, label: 'Medium', color: 'warning' };
  return { score: 100, label: 'Strong', color: 'success' };
};

export const UnifiedAuthFlow: React.FC<UnifiedAuthFlowProps> = ({
  initialMode = 'login',
  onSuccess,
  onCancel,
}) => {
  const theme = useTheme();
  const { login, createIdentity, signInWithPasskey, registerPasskey, authState } = useAuth();

  const [mode, setMode] = useState<'login' | 'register'>(initialMode);
  const [loginMode, setLoginMode] = useState<'quick' | 'full'>('quick');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [passwordStrength, setPasswordStrength] = useState<PasswordStrength>({ score: 0, label: '', color: 'error' });
  const [_identitySaved, _setIdentitySaved] = useState(true);
  const [showIdentityModal, setShowIdentityModal] = useState(false);
  const [showPasskeyModal, setShowPasskeyModal] = useState(false);
  const [_copiedToClipboard, setCopiedToClipboard] = useState(false);

  const [formData, setFormData] = useState({
    name: '',
    password: '',
    confirmPassword: '',
    fourWordAddress: '',
    rememberMe: false,
  });

  const [generatedFourWords, setGeneratedFourWords] = useState('');
  const [registrationPassword, setRegistrationPassword] = useState(''); // Store password for passkey enrollment

  // Persistent debug logging that survives page reloads
  const logDebug = (message: string, data?: any) => {
    const timestamp = new Date().toISOString();
    const logEntry = `[${timestamp}] ${message} ${data ? JSON.stringify(data) : ''}`;
    console.log(message, data || '');

    // Persist to localStorage
    try {
      const logs = JSON.parse(localStorage.getItem('debug_logs') || '[]');
      logs.push(logEntry);
      // Keep last 100 logs
      if (logs.length > 100) logs.shift();
      localStorage.setItem('debug_logs', JSON.stringify(logs));
    } catch (e) {
      // Ignore storage errors
    }
  };

  // Expose debug helper to window for console access
  useEffect(() => {
    (window as any).getDebugLogs = () => {
      const logs = JSON.parse(localStorage.getItem('debug_logs') || '[]');
      console.log('=== Debug Logs (last 100) ===');
      logs.forEach((log: string) => console.log(log));
      return logs;
    };
    (window as any).clearDebugLogs = () => {
      localStorage.removeItem('debug_logs');
      console.log('Debug logs cleared');
    };
  }, []);

  // Component lifecycle tracking for debugging
  useEffect(() => {
    logDebug('🟢 UnifiedAuthFlow MOUNTED', { mode, showIdentityModal, showPasskeyModal });
    return () => {
      logDebug('🔴 UnifiedAuthFlow UNMOUNTING', { mode, showIdentityModal, showPasskeyModal });
    };
  }, []);

  // Track modal state changes
  useEffect(() => {
    logDebug('📊 Modal state changed:', { showIdentityModal, showPasskeyModal });
  }, [showIdentityModal, showPasskeyModal]);

  useEffect(() => {
    const generateIdentity = async () => {
      if (mode === 'register' && !generatedFourWords) {
        const fourWords = await generateFourWordIdentity();
        setGeneratedFourWords(fourWords);
        setFormData(prev => ({ ...prev, fourWordAddress: fourWords }));
      }
    };
    generateIdentity();
  }, [mode, generatedFourWords]);

  useEffect(() => {
    if (formData.password) {
      setPasswordStrength(calculatePasswordStrength(formData.password));
    }
  }, [formData.password]);

  const handleInputChange = (field: string) => (event: React.ChangeEvent<HTMLInputElement>) => {
    setFormData(prev => ({ ...prev, [field]: event.target.value }));
    setError(null);
  };

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(generatedFourWords);
      setCopiedToClipboard(true);
      setTimeout(() => setCopiedToClipboard(false), 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  };

  const downloadIdentity = () => {
    try {
      console.log('📥 Starting identity backup download...');
      const data = {
        fourWordAddress: generatedFourWords,
        name: formData.name,
        createdAt: new Date().toISOString(),
        warning: 'Keep this file secure - it contains your identity information'
      };
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `communitas-identity-${generatedFourWords}.json`;
      document.body.appendChild(a); // Append to DOM for compatibility
      a.click();
      document.body.removeChild(a); // Clean up
      URL.revokeObjectURL(url);
      console.log('✅ Identity backup download initiated');
    } catch (err) {
      console.error('Failed to download identity:', err);
      alert('Failed to download backup. Please copy your four-word identity manually.');
    }
  };

  const validateForm = (): boolean => {
    if (mode === 'register') {
      if (!formData.name.trim()) {
        setError('Name is required');
        return false;
      }
      if (!formData.password || formData.password.length < 8) {
        setError('Password must be at least 8 characters');
        return false;
      }
      if (formData.password !== formData.confirmPassword) {
        setError('Passwords do not match');
        return false;
      }
      // Note: identitySaved check removed - that's for the success modal, not pre-registration
    } else {
      // In quick login mode, only password is required
      if (loginMode === 'quick') {
        if (!formData.password.trim()) {
          setError('Password is required');
          return false;
        }
      } else {
        // In full login mode, both fields are required
        if (!formData.fourWordAddress.trim()) {
          setError('Four-word address is required');
          return false;
        }
        if (!formData.password.trim()) {
          setError('Password is required');
          return false;
        }
      }
    }
    return true;
  };

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();

    if (!validateForm()) return;

    setLoading(true);
    setError(null);

    try {
      if (mode === 'register') {
        // Store password for passkey enrollment (cleared after modal closes)
        setRegistrationPassword(formData.password);

        const identity = await createIdentity(
          formData.name,
          { password: formData.password, fourWords: generatedFourWords }
        );
        if (identity) {
          setSuccess(true);
          setShowIdentityModal(true);
        }
      } else {
        // In quick mode, try password-only login
        const fourWords = loginMode === 'quick' ? '' : formData.fourWordAddress;
        const success = await login(
          fourWords,
          formData.password
        );
        if (success) {
          if (formData.rememberMe) {
            localStorage.setItem('communitas-remember-me', 'true');
          }
          setSuccess(true);
          setTimeout(() => {
            onSuccess?.();
          }, 1000);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Authentication failed');
    } finally {
      setLoading(false);
    }
  };

  const handlePasskeyAuth = async () => {
    logDebug('🔵 handlePasskeyAuth called', {
      mode,
      isAuthenticated: authState.isAuthenticated,
      showIdentityModal,
      showPasskeyModal,
      hasRegistrationPassword: !!registrationPassword,
      hasFormPassword: !!formData.password
    });
    setLoading(true);
    setError(null);

    try {
      if (mode === 'register' && authState.isAuthenticated) {
        logDebug('🔵 Registering passkey...');

        // Use stored registration password (available after successful registration)
        const passwordToUse = registrationPassword || formData.password;
        logDebug('🔵 Password to use:', {
          source: registrationPassword ? 'registrationPassword' : 'formData.password',
          length: passwordToUse?.length
        });

        // Validate password is available
        if (!passwordToUse || passwordToUse.length < 8) {
          logDebug('❌ Password validation failed:', { length: passwordToUse?.length });
          setError('Password must be at least 8 characters to register passkey');
          setLoading(false);
          return;
        }

        logDebug('🔵 Calling registerPasskey...');
        // Register passkey with password (stores in OS keyring)
        const result = await registerPasskey(passwordToUse);
        logDebug('🔵 registerPasskey result:', result);
        if (result) {
          console.log('✅ Passkey enrolled successfully');
          setError('✅ Touch ID / Face ID enrolled successfully!');
          // Clear stored password after successful enrollment
          setRegistrationPassword('');
          // Close modal and complete registration after showing success message
          setTimeout(() => {
            setShowIdentityModal(false);
            onSuccess?.();
          }, 2000); // Give user time to see success message
        }
      } else {
        // Login mode - authenticate with Touch ID
        if (!formData.fourWordAddress.trim()) {
          setError('Please enter your four-word address first');
          setLoading(false);
          return;
        }

        const success = await signInWithPasskey(formData.fourWordAddress.trim());
        if (success) {
          setSuccess(true);
          setTimeout(() => {
            onSuccess?.();
          }, 1000);
        }
      }
    } catch (err) {
      console.error('Passkey error:', err);
      const errorMsg = err instanceof Error ? err.message : 'Passkey operation failed';
      setError(errorMsg);
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      {/* Success Modal for Registration */}
      <Dialog
        open={showIdentityModal}
        onClose={(_event, reason) => {
          logDebug('🔴 Modal onClose triggered!', { reason, showIdentityModal });
          // Don't call onSuccess here - let user explicitly choose to skip or complete
          // This prevents premature unmounting when modal closes accidentally
          setShowIdentityModal(false);
          // Don't clear password here - need it for passkey registration
          // Password will be cleared when passkey modal closes or is skipped
        }}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          <Stack direction="row" alignItems="center" spacing={1}>
            <CheckCircleIcon color="success" />
            <Typography variant="h6">Identity Created Successfully!</Typography>
          </Stack>
        </DialogTitle>
        <DialogContent>
          <Alert severity="warning" sx={{ mb: 2 }}>
            <Typography variant="subtitle2" gutterBottom>
              <strong>Save Your Four-Word Identity Now!</strong>
            </Typography>
            <Typography variant="body2">
              This is the ONLY way to access your account from other devices.
              It cannot be recovered if lost.
            </Typography>
          </Alert>

          <Paper
            sx={{
              p: 2,
              backgroundColor: alpha(theme.palette.primary.main, 0.05),
              borderRadius: 1,
              mb: 2,
            }}
          >
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              Your Four-Word Identity:
            </Typography>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Typography variant="h5" fontWeight={600} color="primary">
                {generatedFourWords}
              </Typography>
              <IconButton onClick={copyToClipboard} color="primary">
                <ContentCopyIcon />
              </IconButton>
            </Stack>
          </Paper>

          <Typography variant="body2" color="text.secondary" paragraph>
            You can now log in on this device with just your password.
            To log in on a new device, you'll need both your four-word identity and password.
          </Typography>

          <Stack spacing={2}>
            <Button
              variant="outlined"
              fullWidth
              startIcon={<SaveAltIcon />}
              onClick={downloadIdentity}
            >
              Download Identity Backup
            </Button>

            <Button
              variant="outlined"
              fullWidth
              startIcon={<FingerprintIcon />}
              onClick={handlePasskeyAuth}
              disabled={loading}
              color="primary"
            >
              {loading ? 'Enrolling...' : 'Enroll Touch ID / Face ID'}
            </Button>
          </Stack>

          {error && (
            <Alert severity="error" sx={{ mt: 2 }}>
              {error}
            </Alert>
          )}
        </DialogContent>
        <DialogActions sx={{ flexDirection: 'column', gap: 1, alignItems: 'stretch', p: 3 }}>
          <Button
            variant="contained"
            startIcon={<FingerprintIcon />}
            onClick={() => {
              setShowIdentityModal(false);
              setShowPasskeyModal(true);
            }}
            fullWidth
          >
            Set Up Biometric Login
          </Button>
          <Button
            variant="outlined"
            onClick={() => {
              setShowIdentityModal(false);
              setRegistrationPassword(''); // Clear stored password for security
              onSuccess?.();
            }}
            fullWidth
          >
            Skip for Now
          </Button>
        </DialogActions>
      </Dialog>

      {/* Passkey Registration Modal */}
      <PasskeyRegistration
        open={showPasskeyModal}
        fourWords={generatedFourWords || authState.user?.fourWordAddress || ''}
        displayName={formData.name || authState.user?.name || ''}
        password={registrationPassword} // Pass password for keyring storage
        onClose={() => {
          setShowPasskeyModal(false);
          setRegistrationPassword(''); // Clear password for security
        }}
        onSuccess={() => {
          setShowPasskeyModal(false);
          setRegistrationPassword(''); // Clear password for security
          onSuccess?.();
        }}
      />

      <Box
        sx={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.1)} 0%, ${alpha(theme.palette.secondary.main, 0.1)} 100%)`,
          p: 2,
        }}
      >
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <Card
          sx={{
            maxWidth: 450,
            width: '100%',
            backdropFilter: 'blur(10px)',
            backgroundColor: alpha(theme.palette.background.paper, 0.95),
            boxShadow: theme.shadows[20],
            borderRadius: 3,
            overflow: 'visible',
          }}
        >
          <CardContent sx={{ p: 4 }}>
            {/* Header */}
            <Stack direction="row" alignItems="center" spacing={2} mb={3}>
              <Box
                sx={{
                  width: 48,
                  height: 48,
                  borderRadius: '50%',
                  background: `linear-gradient(135deg, ${theme.palette.primary.main}, ${theme.palette.secondary.main})`,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
              >
                <AutoAwesomeIcon sx={{ color: 'white' }} />
              </Box>
              <Box>
                <Typography variant="h5" fontWeight={700}>
                  {mode === 'register' ? 'Create Identity' : 'Welcome Back'}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {mode === 'register'
                    ? 'Join the decentralized network'
                    : 'Sign in to your secure identity'}
                </Typography>
              </Box>
            </Stack>

            <Divider sx={{ mb: 3 }} />

            {/* Success State */}
            <AnimatePresence>
              {success && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.8 }}
                >
                  <Alert
                    severity="success"
                    icon={<CheckCircleIcon />}
                    sx={{ mb: 3 }}
                  >
                    {mode === 'register'
                      ? 'Identity created successfully!'
                      : 'Signed in successfully!'}
                  </Alert>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Error State */}
            <AnimatePresence>
              {error && (
                <motion.div
                  initial={{ opacity: 0, x: -20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                >
                  <Alert
                    severity="error"
                    icon={<ErrorIcon />}
                    onClose={() => setError(null)}
                    sx={{ mb: 3 }}
                  >
                    {error}
                  </Alert>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Form */}
            <form onSubmit={handleSubmit}>
              <Stack spacing={2.5}>
                {mode === 'register' ? (
                  <>
                    {/* Register Fields */}
                    <TextField
                      fullWidth
                      label="Your Name"
                      value={formData.name}
                      onChange={handleInputChange('name')}
                      disabled={loading || success}
                      InputProps={{
                        startAdornment: (
                          <InputAdornment position="start">
                            <PersonIcon fontSize="small" />
                          </InputAdornment>
                        ),
                      }}
                    />

                    {/* Note: Four-word identity will be shown in success modal after creation */}

                    <TextField
                      fullWidth
                      label="Password"
                      type={showPassword ? 'text' : 'password'}
                      value={formData.password}
                      onChange={handleInputChange('password')}
                      disabled={loading || success}
                      InputProps={{
                        startAdornment: (
                          <InputAdornment position="start">
                            <LockIcon fontSize="small" />
                          </InputAdornment>
                        ),
                        endAdornment: (
                          <InputAdornment position="end">
                            <IconButton
                              size="small"
                              onClick={() => setShowPassword(!showPassword)}
                              edge="end"
                            >
                              {showPassword ? <VisibilityOffIcon /> : <VisibilityIcon />}
                            </IconButton>
                          </InputAdornment>
                        ),
                      }}
                    />

                    {formData.password && (
                      <Box>
                        <LinearProgress
                          variant="determinate"
                          value={passwordStrength.score}
                          color={passwordStrength.color}
                          sx={{ height: 6, borderRadius: 3 }}
                        />
                        <Typography variant="caption" color={`${passwordStrength.color}.main`} sx={{ mt: 0.5 }}>
                          Password strength: {passwordStrength.label}
                        </Typography>
                      </Box>
                    )}

                    <TextField
                      fullWidth
                      label="Confirm Password"
                      type={showConfirmPassword ? 'text' : 'password'}
                      value={formData.confirmPassword}
                      onChange={handleInputChange('confirmPassword')}
                      disabled={loading || success}
                      InputProps={{
                        startAdornment: (
                          <InputAdornment position="start">
                            <LockIcon fontSize="small" />
                          </InputAdornment>
                        ),
                        endAdornment: (
                          <InputAdornment position="end">
                            <IconButton
                              size="small"
                              onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                              edge="end"
                            >
                              {showConfirmPassword ? <VisibilityOffIcon /> : <VisibilityIcon />}
                            </IconButton>
                          </InputAdornment>
                        ),
                      }}
                    />
                  </>
                ) : (
                  <>
                    {/* Login Mode Selector */}
                    <Paper
                      elevation={0}
                      sx={{
                        p: 1.5,
                        backgroundColor: alpha(theme.palette.info.main, 0.05),
                        borderRadius: 1,
                      }}
                    >
                      <Stack direction="row" alignItems="center" spacing={1} mb={1}>
                        <InfoIcon color="info" fontSize="small" />
                        <Typography variant="body2" color="info.dark">
                          {loginMode === 'quick'
                            ? 'Quick login with password only (works on devices you\'ve used before)'
                            : 'Full login with four-word identity (required for new devices)'}
                        </Typography>
                      </Stack>
                      <Button
                        size="small"
                        onClick={() => setLoginMode(loginMode === 'quick' ? 'full' : 'quick')}
                        color="info"
                      >
                        Switch to {loginMode === 'quick' ? 'Full Login' : 'Quick Login'}
                      </Button>
                    </Paper>

                    {/* Login Fields */}
                    {loginMode === 'full' && (
                      <TextField
                        fullWidth
                        label="Four-Word Address"
                        value={formData.fourWordAddress}
                        onChange={handleInputChange('fourWordAddress')}
                        disabled={loading || success}
                        placeholder="ocean-forest-moon-star"
                        helperText="Your universal identity across all devices"
                        InputProps={{
                          startAdornment: (
                            <InputAdornment position="start">
                              <KeyIcon fontSize="small" />
                            </InputAdornment>
                          ),
                        }}
                      />
                    )}

                    <TextField
                      fullWidth
                      label="Password"
                      type={showPassword ? 'text' : 'password'}
                      value={formData.password}
                      onChange={handleInputChange('password')}
                      disabled={loading || success}
                      InputProps={{
                        startAdornment: (
                          <InputAdornment position="start">
                            <LockIcon fontSize="small" />
                          </InputAdornment>
                        ),
                        endAdornment: (
                          <InputAdornment position="end">
                            <IconButton
                              size="small"
                              onClick={() => setShowPassword(!showPassword)}
                              edge="end"
                            >
                              {showPassword ? <VisibilityOffIcon /> : <VisibilityIcon />}
                            </IconButton>
                          </InputAdornment>
                        ),
                      }}
                    />
                  </>
                )}

                {/* Action Buttons */}
                <Stack direction="row" spacing={2} width="100%">
                  {onCancel && (
                    <Button
                      variant="outlined"
                      size="large"
                      fullWidth
                      onClick={onCancel}
                      disabled={loading}
                      sx={{ py: 1.5 }}
                    >
                      Cancel
                    </Button>
                  )}
                  <Button
                    type="submit"
                    variant="contained"
                    size="large"
                    fullWidth
                    disabled={loading || success}
                    endIcon={loading ? null : <ArrowForwardIcon />}
                    sx={{
                      py: 1.5,
                      background: loading ? undefined : `linear-gradient(135deg, ${theme.palette.primary.main}, ${theme.palette.secondary.main})`,
                    }}
                  >
                    {loading ? (
                      <CircularProgress size={24} color="inherit" />
                    ) : (
                      mode === 'register' ? 'Create Identity' : 'Sign In'
                    )}
                  </Button>
                </Stack>

                {/* Passkey Option - Only show in login mode */}
                {mode === 'login' && (
                  <>
                    <Divider>
                      <Chip label="OR" size="small" />
                    </Divider>

                    <Button
                      variant="outlined"
                      size="large"
                      fullWidth
                      onClick={handlePasskeyAuth}
                      disabled={loading || success}
                      startIcon={<FingerprintIcon />}
                      sx={{ py: 1.5 }}
                    >
                      Sign in with Passkey
                    </Button>
                  </>
                )}

                {/* Mode Switch */}
                <Box textAlign="center" mt={2}>
                  <Typography variant="body2" color="text.secondary">
                    {mode === 'register'
                      ? 'Already have an identity?'
                      : "Don't have an identity?"}
                    <Button
                      onClick={() => {
                        setMode(mode === 'register' ? 'login' : 'register');
                        setError(null);
                        setFormData({
                          name: '',
                          password: '',
                          confirmPassword: '',
                          fourWordAddress: '',
                          rememberMe: false,
                        });
                      }}
                      disabled={loading}
                      sx={{ ml: 1 }}
                    >
                      {mode === 'register' ? 'Sign In' : 'Create Identity'}
                    </Button>
                  </Typography>
                </Box>
              </Stack>
            </form>
          </CardContent>
        </Card>
      </motion.div>
    </Box>
    </>
  );
};

export default UnifiedAuthFlow;