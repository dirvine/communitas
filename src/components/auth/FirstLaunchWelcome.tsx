import {
    ArrowBack as ArrowBackIcon, CheckCircle as CheckCircleIcon, Fingerprint as FingerprintIcon,
    Lock as LockIcon, Refresh as RefreshIcon, Security as SecurityIcon, Visibility,
    VisibilityOff
} from '@mui/icons-material';
import {
    Alert, alpha, Box, Button, Card,
    CardContent, Chip, Dialog, DialogActions, DialogContent, FormControl, FormControlLabel, IconButton,
    InputAdornment,
    LinearProgress,
    Radio,
    RadioGroup, TextField, Typography
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import React, { useEffect, useState } from 'react';
import { useAuth } from '../../contexts/AuthContext';
import { bridgeClient } from '../../services/BridgeClient';
import { fourWordsToDisplay } from '../../utils/identity';
import { isTauriApp } from '../../utils/tauri';

interface FirstLaunchWelcomeProps {
  open: boolean;
  onClose: () => void;
}

type OnboardingStep = 'welcome' | 'generate' | 'display-name' | 'security-choice' | 'password' | 'passkey' | 'summary';
type SecurityMethod = 'password' | 'passkey';

interface PasswordStrength {
  score: number; // 0-100
  label: string;
  color: 'error' | 'warning' | 'success';
}

export const FirstLaunchWelcome: React.FC<FirstLaunchWelcomeProps> = ({ open, onClose }) => {
  const { createIdentity, getOsUsername, enableAutoLogin } = useAuth();

  // Step management
  const [currentStep, setCurrentStep] = useState<OnboardingStep>('welcome');

  // Identity data
  const [fourWords, setFourWords] = useState<string>('');
  const [displayName, setDisplayName] = useState<string>('');
  const [securityMethod, setSecurityMethod] = useState<SecurityMethod>('password');

  // Password state
  const [password, setPassword] = useState<string>('');
  const [passwordConfirm, setPasswordConfirm] = useState<string>('');
  const [showPassword, setShowPassword] = useState(false);
  const [showPasswordConfirm, setShowPasswordConfirm] = useState(false);
  const [rememberMe, setRememberMe] = useState(true);

  // Passkey state
  const [passkeyRegistering, setPasskeyRegistering] = useState(false);
  const [isTauri, setIsTauri] = useState(false);

  // UI state
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Detect if running in Tauri desktop app
  useEffect(() => {
    setIsTauri(!!(window as any)._TAURI_);
  }, []);

  // Initialize display name from OS username
  useEffect(() => {
    if (open && currentStep === 'welcome') {
      loadOsUsername();
    }
  }, [open, currentStep]);

  const loadOsUsername = async () => {
    try {
      const osUsername = await getOsUsername();
      setDisplayName(osUsername);
    } catch (err) {
      console.warn('Failed to get OS username:', err);
      setDisplayName('');
    }
  };

  // Password strength calculation
  const calculatePasswordStrength = (pwd: string): PasswordStrength => {
    if (!pwd) return { score: 0, label: 'No password', color: 'error' };

    let score = 0;

    // Length
    if (pwd.length >= 8) score += 25;
    if (pwd.length >= 12) score += 25;

    // Complexity
    if (/[a-z]/.test(pwd)) score += 10;
    if (/[A-Z]/.test(pwd)) score += 10;
    if (/[0-9]/.test(pwd)) score += 10;
    if (/[^a-zA-Z0-9]/.test(pwd)) score += 20;

    if (score < 40) return { score, label: 'Weak', color: 'error' };
    if (score < 70) return { score, label: 'Medium', color: 'warning' };
    return { score, label: 'Strong', color: 'success' };
  };

  const passwordStrength = calculatePasswordStrength(password);

  // Generate a random four-word identity for browser mode
  const generateBrowserIdentity = (): string => {
    // Simple word list for demo - in production this would use the full four-word-networking dictionary
    const words = [
      'ocean', 'forest', 'mountain', 'river', 'desert', 'valley', 'meadow', 'canyon',
      'star', 'moon', 'sun', 'comet', 'galaxy', 'nebula', 'planet', 'asteroid',
      'thunder', 'lightning', 'rainbow', 'breeze', 'storm', 'cloud', 'rain', 'snow',
      'fire', 'water', 'earth', 'wind', 'ice', 'stone', 'metal', 'wood'
    ];

    const randomWord = () => words[Math.floor(Math.random() * words.length)];
    return `${randomWord()}-${randomWord()}-${randomWord()}-${randomWord()}`;
  };

  // Step handlers
  const handleGenerateIdentity = async () => {
    setLoading(true);
    setError(null);
    try {
      let generated: string;

      if (isTauriApp()) {
        // Use Tauri backend
        generated = await invoke<string>('generate_four_word_identity');
      } else {
        // Use browser mode - generate random four-word identity
        generated = generateBrowserIdentity();
      }

      setFourWords(generated);
      setCurrentStep('generate');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate identity');
    } finally {
      setLoading(false);
    }
  };

  const handleRegenerateIdentity = async () => {
    setLoading(true);
    setError(null);
    try {
      let generated: string;

      if (isTauriApp()) {
        // Use Tauri backend
        generated = await invoke<string>('generate_four_word_identity');
      } else {
        // Use browser mode - generate random four-word identity
        generated = generateBrowserIdentity();
      }

      setFourWords(generated);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate identity');
    } finally {
      setLoading(false);
    }
  };

  const handleContinueToDisplayName = () => {
    setCurrentStep('display-name');
  };

  const handleContinueToSecurity = () => {
    if (!displayName.trim()) {
      setError('Please enter a display name');
      return;
    }
    setError(null);
    setCurrentStep('security-choice');
  };

  const handleSecurityChoice = (method: SecurityMethod) => {
    setSecurityMethod(method);
    setCurrentStep(method);
  };

  const handlePasswordContinue = () => {
    // Validate password
    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return;
    }
    if (password !== passwordConfirm) {
      setError('Passwords do not match');
      return;
    }
    if (passwordStrength.score < 40) {
      setError('Please choose a stronger password');
      return;
    }
    setError(null);
    handleCreateIdentity();
  };

  const handlePasskeyRegister = async () => {
    setPasskeyRegistering(true);
    setError(null);

    try {
      // First create the vault with a strong random password
      const randomPassword = await generateSecurePassword();

      // Create identity
      await createIdentity(displayName, {
        fourWords: fourWords,
        password: randomPassword,
      });

      // Register passkey
      await invoke('auth_passkey_register', {
        fourWords: fourWords,
        deviceName: displayName,
      });

      // Enable auto-login with passkey
      await enableAutoLogin(fourWords, randomPassword);

      setCurrentStep('summary');
    } catch (err) {
      console.error('Passkey registration failed:', err);
      setError(err instanceof Error ? err.message : 'Passkey registration failed. Please try password method instead.');
    } finally {
      setPasskeyRegistering(false);
    }
  };

  const handleCreateIdentity = async () => {
    setLoading(true);
    setError(null);

    try {
      // In browser mode, initialize bridge server
      if (!isTauriApp()) {
        console.log('🌐 Browser mode: Initializing via bridge server');
        await bridgeClient.initialize({
          four_words: fourWords,
          display_name: displayName,
          device_name: 'Browser'
        });
        console.log('✅ Bridge initialization successful');
      }

      // Create vault with user's chosen password (works in both modes)
      await createIdentity(displayName, {
        fourWords: fourWords,
        password: password,
      });

      // Enable auto-login if requested
      if (rememberMe) {
        await enableAutoLogin(fourWords, password);
      }

      setCurrentStep('summary');
    } catch (err) {
      console.error('Identity creation failed:', err);
      setError(err instanceof Error ? err.message : 'Failed to create identity');
    } finally {
      setLoading(false);
    }
  };

  const generateSecurePassword = async (): Promise<string> => {
    // Generate a cryptographically secure random password for passkey users
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
  };

  const handleBack = () => {
    const stepOrder: OnboardingStep[] = ['welcome', 'generate', 'display-name', 'security-choice', 'password'];
    const currentIndex = stepOrder.indexOf(currentStep);
    if (currentIndex > 0) {
      setCurrentStep(stepOrder[currentIndex - 1]);
      setError(null);
    }
  };

  const handleStart = () => {
    console.log('✅ Onboarding complete, starting app');
    onClose();
  };

  // Render step content
  const renderStepContent = () => {
    switch (currentStep) {
      case 'welcome':
        return (
          <Box sx={{ textAlign: 'center', py: 2 }}>
            <Typography variant="h4" fontWeight={600} gutterBottom>
              Welcome to Communitas! 🎉
            </Typography>
            <Typography variant="body1" color="text.secondary" paragraph sx={{ mt: 3 }}>
              Let's set up your secure identity
            </Typography>

            <Box sx={{ mt: 4, mb: 3, display: 'flex', flexDirection: 'column', gap: 2, alignItems: 'flex-start', maxWidth: 400, mx: 'auto' }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                <SecurityIcon color="primary" />
                <Typography variant="body2" textAlign="left">Post-quantum secure encryption</Typography>
              </Box>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                <CheckCircleIcon color="primary" />
                <Typography variant="body2" textAlign="left">Peer-to-peer networking</Typography>
              </Box>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                <CheckCircleIcon color="primary" />
                <Typography variant="body2" textAlign="left">No central servers</Typography>
              </Box>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                <CheckCircleIcon color="primary" />
                <Typography variant="body2" textAlign="left">Human-readable identities</Typography>
              </Box>
            </Box>

            <Button
              variant="contained"
              size="large"
              onClick={handleGenerateIdentity}
              disabled={loading}
              sx={{ mt: 3, minWidth: 200 }}
            >
              Get Started
            </Button>
          </Box>
        );

      case 'generate':
        return (
          <Box sx={{ textAlign: 'center' }}>
            <Typography variant="h5" fontWeight={600} gutterBottom>
              Your Identity
            </Typography>
            <Typography variant="body2" color="text.secondary" paragraph sx={{ mt: 2 }}>
              This is your network address. You can regenerate it if you'd like a different one.
            </Typography>

            <Card
              sx={{
                mt: 3,
                mb: 3,
                bgcolor: (theme) => alpha(theme.palette.primary.main, 0.1),
                border: 2,
                borderColor: 'primary.main',
              }}
            >
              <CardContent sx={{ py: 4 }}>
                <Typography
                  variant="h4"
                  fontWeight={600}
                  sx={{
                    fontFamily: 'monospace',
                    color: 'primary.main',
                    letterSpacing: 2,
                  }}
                >
                  {fourWordsToDisplay(fourWords)}
                </Typography>
                <Typography variant="caption" color="text.secondary" sx={{ mt: 2, display: 'block' }}>
                  This is your unique address on the network
                </Typography>
              </CardContent>
            </Card>

            <Button
              startIcon={<RefreshIcon />}
              onClick={handleRegenerateIdentity}
              disabled={loading}
              sx={{ mb: 2 }}
            >
              Generate New Identity
            </Button>
          </Box>
        );

      case 'display-name':
        return (
          <Box>
            <Typography variant="h5" fontWeight={600} gutterBottom textAlign="center">
              What should we call you?
            </Typography>
            <Typography variant="body2" color="text.secondary" paragraph textAlign="center" sx={{ mt: 2, mb: 4 }}>
              This name will be shown to others when you collaborate
            </Typography>

            <TextField
              fullWidth
              label="Display Name"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="Enter your name"
              autoFocus
              sx={{ mb: 2 }}
            />

            <Typography variant="caption" color="text.secondary">
              You can change this later in settings
            </Typography>
          </Box>
        );

      case 'security-choice':
        return (
          <Box sx={{ textAlign: 'center' }}>
            <Typography variant="h5" fontWeight={600} gutterBottom>
              Secure Your Identity
            </Typography>
            <Typography variant="body2" color="text.secondary" paragraph sx={{ mt: 2, mb: 4 }}>
              Choose how you'd like to authenticate
            </Typography>

            <FormControl component="fieldset" fullWidth>
              <RadioGroup>
                <Card
                  sx={{
                    mb: 2,
                    cursor: 'pointer',
                    border: 2,
                    borderColor: 'transparent',
                    '&:hover': { borderColor: 'primary.main' },
                  }}
                  onClick={() => handleSecurityChoice('password')}
                >
                  <CardContent sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                    <FormControlLabel
                      value="password"
                      control={<Radio />}
                      label=""
                      sx={{ m: 0 }}
                    />
                    <LockIcon color="action" />
                    <Box sx={{ textAlign: 'left', flex: 1 }}>
                      <Typography variant="subtitle1" fontWeight={600}>
                        Password (traditional)
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        Set a password to protect your identity
                      </Typography>
                    </Box>
                  </CardContent>
                </Card>

                {/* Only show passkey option in web version (not Tauri desktop) */}
                {!isTauri && (
                  <Card
                    sx={{
                      cursor: 'pointer',
                      border: 2,
                      borderColor: 'transparent',
                      '&:hover': { borderColor: 'primary.main' },
                    }}
                    onClick={() => handleSecurityChoice('passkey')}
                  >
                    <CardContent sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                      <FormControlLabel
                        value="passkey"
                        control={<Radio />}
                        label=""
                        sx={{ m: 0 }}
                      />
                      <FingerprintIcon color="action" />
                      <Box sx={{ textAlign: 'left', flex: 1 }}>
                        <Typography variant="subtitle1" fontWeight={600}>
                          Passkey (recommended) 🔐
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          Use biometrics or security key
                        </Typography>
                      </Box>
                      <Chip label="Requires compatible device" size="small" />
                    </CardContent>
                  </Card>
                )}
              </RadioGroup>
            </FormControl>
          </Box>
        );

      case 'password':
        return (
          <Box>
            <Typography variant="h5" fontWeight={600} gutterBottom textAlign="center">
              Set Password
            </Typography>
            <Typography variant="body2" color="text.secondary" paragraph textAlign="center" sx={{ mt: 2, mb: 4 }}>
              Choose a strong password to protect your identity
            </Typography>

            <TextField
              fullWidth
              type={showPassword ? 'text' : 'password'}
              label="Password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoFocus
              sx={{ mb: 2 }}
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    <IconButton onClick={() => setShowPassword(!showPassword)} edge="end">
                      {showPassword ? <VisibilityOff /> : <Visibility />}
                    </IconButton>
                  </InputAdornment>
                ),
              }}
            />

            {password && (
              <Box sx={{ mb: 2 }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                  <Typography variant="caption">Strength:</Typography>
                  <Typography variant="caption" color={`${passwordStrength.color}.main`} fontWeight={600}>
                    {passwordStrength.label}
                  </Typography>
                </Box>
                <LinearProgress
                  variant="determinate"
                  value={passwordStrength.score}
                  color={passwordStrength.color}
                  sx={{ height: 6, borderRadius: 3 }}
                />
              </Box>
            )}

            <TextField
              fullWidth
              type={showPasswordConfirm ? 'text' : 'password'}
              label="Confirm Password"
              value={passwordConfirm}
              onChange={(e) => setPasswordConfirm(e.target.value)}
              sx={{ mb: 2 }}
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    <IconButton onClick={() => setShowPasswordConfirm(!showPasswordConfirm)} edge="end">
                      {showPasswordConfirm ? <VisibilityOff /> : <Visibility />}
                    </IconButton>
                  </InputAdornment>
                ),
              }}
            />

            <FormControlLabel
              control={
                <input
                  type="checkbox"
                  checked={rememberMe}
                  onChange={(e) => setRememberMe(e.target.checked)}
                />
              }
              label={
                <Typography variant="body2">
                  Remember me (save in system keyring)
                </Typography>
              }
            />
          </Box>
        );

      case 'passkey':
        return (
          <Box sx={{ textAlign: 'center' }}>
            <Typography variant="h5" fontWeight={600} gutterBottom>
              Register Passkey
            </Typography>
            <Typography variant="body2" color="text.secondary" paragraph sx={{ mt: 2, mb: 4 }}>
              Your device will prompt you to use biometric authentication
            </Typography>

            <Box sx={{ py: 6 }}>
              <FingerprintIcon sx={{ fontSize: 80, color: 'primary.main', mb: 2 }} />
              <Typography variant="body1" color="text.secondary">
                {passkeyRegistering ? 'Waiting for authentication...' : 'Touch your security key or use biometric'}
              </Typography>
            </Box>

            {!passkeyRegistering && (
              <Button
                variant="contained"
                size="large"
                onClick={handlePasskeyRegister}
                fullWidth
                sx={{ mb: 2 }}
              >
                Register Passkey
              </Button>
            )}

            <Button
              variant="text"
              size="small"
              onClick={() => window.open('https://webauthn.guide/', '_blank')}
            >
              Learn about passkeys
            </Button>
          </Box>
        );

      case 'summary':
        return (
          <Box sx={{ textAlign: 'center' }}>
            <CheckCircleIcon sx={{ fontSize: 64, color: 'success.main', mb: 2 }} />
            <Typography variant="h4" fontWeight={600} gutterBottom>
              ✅ Identity Created!
            </Typography>

            <Card sx={{ mt: 3, mb: 3, textAlign: 'left' }}>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Your Details:
                </Typography>
                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1, mt: 2 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" fontWeight={600}>Name:</Typography>
                    <Typography variant="body2">{displayName}</Typography>
                  </Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" fontWeight={600}>Address:</Typography>
                    <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                      {fourWordsToDisplay(fourWords)}
                    </Typography>
                  </Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" fontWeight={600}>Security:</Typography>
                    <Typography variant="body2">
                      {securityMethod === 'passkey' ? 'Passkey 🔐' : 'Password'}
                    </Typography>
                  </Box>
                </Box>
              </CardContent>
            </Card>

            <Alert severity="warning" sx={{ mb: 2 }}>
              <Typography variant="body2" fontWeight={600} gutterBottom>
                ⚠️ Important:
              </Typography>
              <Typography variant="caption">
                Write down your four-word address! You'll need it to connect from other devices.
              </Typography>
            </Alert>

            <Button
              variant="contained"
              size="large"
              onClick={handleStart}
              fullWidth
            >
              Start Using Communitas
            </Button>
          </Box>
        );

      default:
        return null;
    }
  };

  return (
    <Dialog open={open} maxWidth="sm" fullWidth onClose={() => {}} disableEscapeKeyDown>
      <DialogContent sx={{ p: 4, minHeight: 400 }}>
        {error && (
          <Alert severity="error" sx={{ mb: 3 }} onClose={() => setError(null)}>
            {error}
          </Alert>
        )}

        {renderStepContent()}
      </DialogContent>

      {currentStep !== 'welcome' && currentStep !== 'summary' && (
        <DialogActions sx={{ px: 4, pb: 3, justifyContent: 'space-between' }}>
          <Button
            startIcon={<ArrowBackIcon />}
            onClick={handleBack}
            disabled={loading || passkeyRegistering}
          >
            Back
          </Button>

          <Box sx={{ display: 'flex', gap: 2 }}>
            {currentStep === 'generate' && (
              <Button
                variant="contained"
                onClick={handleContinueToDisplayName}
                disabled={loading}
              >
                Continue
              </Button>
            )}
            {currentStep === 'display-name' && (
              <Button
                variant="contained"
                onClick={handleContinueToSecurity}
                disabled={!displayName.trim()}
              >
                Continue
              </Button>
            )}
            {currentStep === 'password' && (
              <Button
                variant="contained"
                onClick={handlePasswordContinue}
                disabled={loading || !password || !passwordConfirm}
              >
                Create Identity
              </Button>
            )}
          </Box>
        </DialogActions>
      )}
    </Dialog>
  );
};
