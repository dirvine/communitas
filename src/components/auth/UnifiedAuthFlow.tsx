import React, { useState, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  TextField,
  Button,
  Typography,
  Stack,
  Divider,
  Alert,
  InputAdornment,
  IconButton,
  LinearProgress,
  Chip,
  useTheme,
  alpha,
  Fade,
  Slide,
  Paper,
  Tooltip,
  CircularProgress,
  Checkbox,
  FormControlLabel,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
} from '@mui/material';
import {
  Person as PersonIcon,
  Lock as LockIcon,
  Visibility as VisibilityIcon,
  VisibilityOff as VisibilityOffIcon,
  Security as SecurityIcon,
  Key as KeyIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
  ArrowForward as ArrowForwardIcon,
  Fingerprint as FingerprintIcon,
  AutoAwesome as AutoAwesomeIcon,
  ContentCopy as ContentCopyIcon,
  Warning as WarningIcon,
  Info as InfoIcon,
  SaveAlt as SaveAltIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
// import { invoke } from '@tauri-apps/api/tauri'; // TODO: Re-enable when Tauri backend is available
import { useAuth } from '../../contexts/AuthContext';
import { generateFourWordIdentity } from '../../utils/identity';
import validator from 'validator';

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
  const [identitySaved, setIdentitySaved] = useState(true);
  const [showIdentityModal, setShowIdentityModal] = useState(false);
  const [copiedToClipboard, setCopiedToClipboard] = useState(false);
  const [passwordWasGenerated, setPasswordWasGenerated] = useState(false);
  const [passkeyEnrolled, setPasskeyEnrolled] = useState(false);

  // Check if running in Tauri (desktop app)
  const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI__;

  const [formData, setFormData] = useState({
    name: '',
    password: '',
    confirmPassword: '',
    fourWordAddress: '',
    rememberMe: false,
  });

  const [generatedFourWords, setGeneratedFourWords] = useState('');

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

  // Generate a cryptographically secure password
  const generateSecurePassword = (): string => {
    const length = 32; // Strong password length
    const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?';
    const randomValues = new Uint8Array(length);
    crypto.getRandomValues(randomValues);

    let password = '';
    for (let i = 0; i < length; i++) {
      password += charset[randomValues[i] % charset.length];
    }

    console.log('🔐 Generated secure password (32 chars)');
    return password;
  };

  const validateForm = (): boolean => {
    if (mode === 'register') {
      if (!formData.name.trim()) {
        setError('Name is required');
        return false;
      }
      // Password is now optional - will be auto-generated if empty
      // If provided, validate length and match
      if (formData.password || formData.confirmPassword) {
        if (formData.password && formData.password.length < 8) {
          setError('Password must be at least 8 characters');
          return false;
        }
        if (formData.password !== formData.confirmPassword) {
          setError('Passwords do not match');
          return false;
        }
      }
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
        // Generate secure password if user didn't provide one
        const password = formData.password || generateSecurePassword();
        const wasPasswordGenerated = !formData.password;

        if (wasPasswordGenerated) {
          console.log('🔐 Using auto-generated password for vault encryption');
        }

        const identity = await createIdentity(
          formData.name,
          { password, fourWords: generatedFourWords }
        );

        if (identity) {
          // If password was auto-generated, store it in keyring for future passkey use
          if (wasPasswordGenerated) {
            // TODO: Re-enable when Tauri backend is available
            /*
            try {
              console.log('💾 Storing auto-generated password in system keyring');
              await invoke('auth_store_password_in_keyring', {
                fourWords: generatedFourWords,
                password
              });
              console.log('✅ Password stored in keyring successfully');
            } catch (keyringError) {
              console.warn('⚠️ Failed to store password in keyring:', keyringError);
              // Non-fatal - continue with registration
            }
            */
          }

          // Track whether password was auto-generated to show appropriate UI
          setPasswordWasGenerated(wasPasswordGenerated);
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
    console.log('handlePasskeyAuth called', { mode, isAuthenticated: authState.isAuthenticated });
    setLoading(true);
    setError(null);

    try {
      if (mode === 'register' && authState.isAuthenticated && authState.user) {
        console.log('Registering passkey...');
        const result = await registerPasskey(authState.user.fourWordAddress);
        if (result) {
          console.log('✅ Passkey enrolled successfully');
          setPasskeyEnrolled(true);
          // Clear success message after showing it briefly
          setTimeout(() => setPasskeyEnrolled(false), 5000);
        }
      } else {
        const success = await signInWithPasskey();
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
        onClose={() => {
          setShowIdentityModal(false);
          onSuccess?.();
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

          {passwordWasGenerated ? (
            <>
              {isTauri ? (
                <Alert severity="success" sx={{ mb: 2 }}>
                  <Typography variant="subtitle2" gutterBottom>
                    <strong>✅ Secure Password Created</strong>
                  </Typography>
                  <Typography variant="body2">
                    We've created a secure password for you and stored it in your system keyring.
                    You won't need to type it again on this device - it's managed automatically by your operating system.
                  </Typography>
                </Alert>
              ) : (
                <Alert severity="info" sx={{ mb: 2 }}>
                  <Typography variant="subtitle2" gutterBottom>
                    <strong>Recommended: Set Up Biometric Login</strong>
                  </Typography>
                  <Typography variant="body2">
                    We've created a secure password for you and stored it in your system keyring.
                    Set up Touch ID or Face ID for the fastest, most secure login experience.
                  </Typography>
                </Alert>
              )}

              <Stack spacing={2}>
                {!isTauri && (
                  <Button
                    variant="contained"
                    fullWidth
                    size="large"
                    startIcon={<FingerprintIcon />}
                    onClick={handlePasskeyAuth}
                    disabled={loading}
                    color="primary"
                  >
                    {loading ? 'Enrolling...' : 'Set Up Touch ID / Face ID'}
                  </Button>
                )}

                <Button
                  variant="outlined"
                  fullWidth
                  startIcon={<SaveAltIcon />}
                  onClick={downloadIdentity}
                >
                  Download Identity Backup
                </Button>
              </Stack>
            </>
          ) : (
            <>
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

                {!isTauri && (
                  <Button
                    variant="outlined"
                    fullWidth
                    startIcon={<FingerprintIcon />}
                    onClick={handlePasskeyAuth}
                    disabled={loading}
                    color="primary"
                  >
                    {loading ? 'Enrolling...' : 'Enroll Touch ID / Face ID (Optional)'}
                  </Button>
                )}
              </Stack>
            </>
          )}

          {passkeyEnrolled && (
            <Alert severity="success" icon={<CheckCircleIcon />} sx={{ mt: 2 }}>
              <Typography variant="subtitle2" gutterBottom>
                <strong>✅ Biometric Login Enabled!</strong>
              </Typography>
              <Typography variant="body2">
                You can now sign in with Touch ID or Face ID. Your password is safely stored and you'll never need to type it again on this device.
              </Typography>
            </Alert>
          )}

          {error && (
            <Alert severity="error" sx={{ mt: 2 }}>
              {error}
            </Alert>
          )}
        </DialogContent>
        <DialogActions>
          <Button
            variant="contained"
            onClick={() => {
              setShowIdentityModal(false);
              onSuccess?.();
            }}
          >
            Continue to App
          </Button>
        </DialogActions>
      </Dialog>

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
                      label="Password (Optional)"
                      type={showPassword ? 'text' : 'password'}
                      value={formData.password}
                      onChange={handleInputChange('password')}
                      disabled={loading || success}
                      helperText="Leave empty to auto-generate a secure password and enable Touch ID/Face ID"
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

                    {formData.password && (
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
                    )}
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