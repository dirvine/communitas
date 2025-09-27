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
} from '@mui/material';
import {
  Person as PersonIcon,
  Email as EmailIcon,
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
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
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
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [passwordStrength, setPasswordStrength] = useState<PasswordStrength>({ score: 0, label: '', color: 'error' });

  const [formData, setFormData] = useState({
    name: '',
    email: '',
    password: '',
    confirmPassword: '',
    fourWordAddress: '',
    rememberMe: false,
  });

  const [generatedFourWords, setGeneratedFourWords] = useState('');

  useEffect(() => {
    if (mode === 'register' && !generatedFourWords) {
      const fourWords = generateFourWordIdentity();
      setGeneratedFourWords(fourWords);
      setFormData(prev => ({ ...prev, fourWordAddress: fourWords }));
    }
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

  const validateForm = (): boolean => {
    if (mode === 'register') {
      if (!formData.name.trim()) {
        setError('Name is required');
        return false;
      }
      if (!formData.email.trim() || !validator.isEmail(formData.email)) {
        setError('Valid email is required');
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
    } else {
      if (!formData.fourWordAddress.trim()) {
        setError('Four-word address is required');
        return false;
      }
      if (!formData.password.trim()) {
        setError('Password is required');
        return false;
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
        const success = await createIdentity(
          formData.name,
          formData.email,
          formData.password
        );
        if (success) {
          setSuccess(true);
          setTimeout(() => {
            onSuccess?.();
          }, 1500);
        }
      } else {
        const success = await login(
          formData.fourWordAddress,
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
    setLoading(true);
    setError(null);

    try {
      if (mode === 'register' && authState.isAuthenticated) {
        await registerPasskey();
        setSuccess(true);
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
      setError(err instanceof Error ? err.message : 'Passkey authentication failed');
    } finally {
      setLoading(false);
    }
  };

  return (
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

                    <TextField
                      fullWidth
                      label="Email Address"
                      type="email"
                      value={formData.email}
                      onChange={handleInputChange('email')}
                      disabled={loading || success}
                      InputProps={{
                        startAdornment: (
                          <InputAdornment position="start">
                            <EmailIcon fontSize="small" />
                          </InputAdornment>
                        ),
                      }}
                    />

                    <Paper
                      variant="outlined"
                      sx={{
                        p: 2,
                        backgroundColor: alpha(theme.palette.primary.main, 0.05),
                        borderColor: theme.palette.primary.main,
                      }}
                    >
                      <Typography variant="caption" color="text.secondary" gutterBottom>
                        Your Four-Word Identity
                      </Typography>
                      <Typography variant="body1" fontWeight={600} color="primary">
                        {generatedFourWords}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        Save this securely - it's your unique network address
                      </Typography>
                    </Paper>

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
                    {/* Login Fields */}
                    <TextField
                      fullWidth
                      label="Four-Word Address"
                      value={formData.fourWordAddress}
                      onChange={handleInputChange('fourWordAddress')}
                      disabled={loading || success}
                      placeholder="ocean-forest-moon-star"
                      InputProps={{
                        startAdornment: (
                          <InputAdornment position="start">
                            <KeyIcon fontSize="small" />
                          </InputAdornment>
                        ),
                      }}
                    />

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

                {/* Submit Button */}
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

                {/* Passkey Option */}
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
                  {mode === 'register' && authState.isAuthenticated
                    ? 'Add Passkey'
                    : 'Sign in with Passkey'}
                </Button>

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
                          email: '',
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
  );
};

export default UnifiedAuthFlow;