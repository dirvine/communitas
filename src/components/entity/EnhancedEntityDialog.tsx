import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  FormControl,
  FormLabel,
  Select,
  MenuItem,
  Box,
  Typography,
  Alert,
  CircularProgress,
  Stack,
  Chip,
  alpha,
} from '@mui/material';
import { styled } from '@mui/material/styles';
import {
  PersonAdd as CreateIcon,
  GroupAdd as AddExistingIcon,
  WifiOff as OfflineIcon,
  CheckCircle as OnlineIcon,
} from '@mui/icons-material';
import { ModernButton } from '../ui/ModernButton';
import { SearchInput } from '../ui/ModernInput';
import { designTokens } from '../../styles/theme';
import { useEntityDirectory } from '../../contexts/EntityDirectoryContext';
import type {
  CreateNewContactInput,
  AddExistingContactInput,
  EntityOperationMode,
  EntityOperationResult,
} from '../../types/entityOperations';

// Styled Components
const StyledDialog = styled(Dialog)(({ theme }) => ({
  '& .MuiDialog-paper': {
    borderRadius: designTokens.borderRadius.xl,
    background: theme.palette.mode === 'light'
      ? 'rgba(255, 255, 255, 0.95)'
      : 'rgba(17, 25, 40, 0.95)',
    backdropFilter: 'blur(20px) saturate(180%)',
    boxShadow: designTokens.shadows.xl,
    border: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
  },
}));

const ModeSelector = styled(Box)(({ theme }) => ({
  display: 'flex',
  gap: theme.spacing(1),
  marginBottom: theme.spacing(3),
  padding: theme.spacing(1),
  background: alpha(theme.palette.background.default, 0.5),
  borderRadius: designTokens.borderRadius.lg,
}));

const ModeButton = styled(ModernButton)<{ selected: boolean }>(({ theme, selected }) => ({
  flex: 1,
  background: selected
    ? designTokens.colors.primary.gradient
    : 'transparent',
  color: selected ? '#ffffff' : theme.palette.text.primary,
  border: selected ? 'none' : `1px solid ${alpha(theme.palette.divider, 0.2)}`,

  '&:hover': {
    background: selected
      ? designTokens.colors.primary.gradient
      : alpha(theme.palette.primary.main, 0.08),
  },
}));

const StatusBadge = styled(Chip)(({ theme }) => ({
  marginLeft: 'auto',
  fontWeight: 600,
  fontSize: '0.75rem',
}));

interface EnhancedEntityDialogProps {
  open: boolean;
  onClose: () => void;
  entityType: 'contact' | 'group' | 'organization' | 'channel' | 'project';
  mode?: EntityOperationMode;
  isOnline?: boolean;
}

export const EnhancedEntityDialog: React.FC<EnhancedEntityDialogProps> = ({
  open,
  onClose,
  entityType,
  mode: initialMode = 'create',
  isOnline = false,
}) => {
  const { createContact, addExistingContact } = useEntityDirectory();

  // State
  const [mode, setMode] = useState<EntityOperationMode>(initialMode);
  const [displayName, setDisplayName] = useState('');
  const [fourWords, setFourWords] = useState('');
  const [email, setEmail] = useState('');
  const [relationship, setRelationship] = useState<'friend' | 'family' | 'colleague' | 'acquaintance'>('colleague');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);

  // Reset form when dialog opens/closes
  useEffect(() => {
    if (!open) {
      setDisplayName('');
      setFourWords('');
      setEmail('');
      setRelationship('colleague');
      setError(null);
      setValidationError(null);
      setLoading(false);
    }
  }, [open]);

  // Force "create" mode when offline
  useEffect(() => {
    if (!isOnline && mode === 'add') {
      setMode('create');
    }
  }, [isOnline, mode]);

  // Validate four-words input
  const validateFourWords = (input: string): boolean => {
    const normalized = input.trim().toLowerCase().replace(/\s+/g, '-');
    const parts = normalized.split('-');

    if (parts.length !== 4) {
      setValidationError('Four-Words must contain exactly 4 words separated by dashes or spaces');
      return false;
    }

    // Basic validation - each word should be alphabetic
    for (const part of parts) {
      if (!/^[a-z]+$/.test(part)) {
        setValidationError('Each word must contain only letters');
        return false;
      }
    }

    setValidationError(null);
    return true;
  };

  const handleFourWordsChange = (value: string) => {
    setFourWords(value);
    if (value.trim()) {
      validateFourWords(value);
    } else {
      setValidationError(null);
    }
  };

  const handleSubmit = async () => {
    setError(null);
    setLoading(true);

    try {
      let result: EntityOperationResult;

      if (mode === 'create') {
        // Create new contact (works offline with temp four-words)
        const input: CreateNewContactInput = {
          displayName: displayName.trim(),
          email: email.trim() || undefined,
          relationship,
        };
        result = await createContact(input);
      } else {
        // Add existing contact (requires online, fetches from DHT)
        if (!isOnline) {
          setError('Must be online to add existing contacts from the network');
          setLoading(false);
          return;
        }

        if (!validateFourWords(fourWords)) {
          setLoading(false);
          return;
        }

        const input: AddExistingContactInput = {
          fourWords: fourWords.trim().toLowerCase().replace(/\s+/g, '-'),
          displayName: displayName.trim() || undefined,
          relationship,
        };
        result = await addExistingContact(input);
      }

      if (result.success) {
        // Success - close dialog
        onClose();
      } else {
        setError(result.error || 'Operation failed');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An unexpected error occurred');
    } finally {
      setLoading(false);
    }
  };

  const canSubmit = () => {
    if (loading) return false;

    if (mode === 'create') {
      return displayName.trim().length > 0;
    } else {
      return fourWords.trim().length > 0 && !validationError;
    }
  };

  const getDialogTitle = () => {
    if (mode === 'create') {
      return isOnline ? 'Create New Contact' : 'Create New Contact (Offline)';
    } else {
      return 'Add Existing Contact';
    }
  };

  const getSubmitButtonText = () => {
    if (loading) return 'Processing...';
    if (mode === 'create') {
      return isOnline ? 'Create Contact' : 'Create Offline';
    } else {
      return 'Add Contact';
    }
  };

  return (
    <StyledDialog
      open={open}
      onClose={loading ? undefined : onClose}
      maxWidth="sm"
      fullWidth
    >
      <DialogTitle>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Typography variant="h6" fontWeight={600}>
            {getDialogTitle()}
          </Typography>
          <StatusBadge
            icon={isOnline ? <OnlineIcon fontSize="small" /> : <OfflineIcon fontSize="small" />}
            label={isOnline ? 'Online' : 'Offline'}
            color={isOnline ? 'success' : 'warning'}
            size="small"
          />
        </Stack>
      </DialogTitle>

      <DialogContent>
        {/* Mode Selector */}
        <ModeSelector>
          <ModeButton
            selected={mode === 'create'}
            onClick={() => setMode('create')}
            disabled={loading}
            gradient={false}
            startIcon={<CreateIcon />}
          >
            Create New
          </ModeButton>
          <ModeButton
            selected={mode === 'add'}
            onClick={() => setMode('add')}
            disabled={loading || !isOnline}
            gradient={false}
            startIcon={<AddExistingIcon />}
          >
            Add Existing
          </ModeButton>
        </ModeSelector>

        {/* Offline Notice for Add Mode */}
        {!isOnline && (
          <Alert severity="warning" sx={{ mb: 2 }}>
            You're currently offline. You can create new contacts with temporary identities
            that will be upgraded when you reconnect. Adding existing contacts requires network access.
          </Alert>
        )}

        {/* Error Display */}
        {error && (
          <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
            {error}
          </Alert>
        )}

        {/* Form Fields */}
        <Stack spacing={2.5} sx={{ mt: 2 }}>
          {mode === 'add' && (
            <TextField
              autoFocus
              label="Four-Word Address"
              placeholder="ocean-blue-eagle-star"
              fullWidth
              variant="outlined"
              value={fourWords}
              onChange={(e) => handleFourWordsChange(e.target.value)}
              error={!!validationError}
              helperText={validationError || 'Enter the person\'s four-word network identity'}
              disabled={loading}
              required
              sx={{
                '& .MuiOutlinedInput-root': {
                  borderRadius: designTokens.borderRadius.md,
                },
              }}
            />
          )}

          <TextField
            autoFocus={mode === 'create'}
            label="Display Name"
            placeholder="Alice Smith"
            fullWidth
            variant="outlined"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            helperText={mode === 'add' ? 'Optional: Override the network display name' : 'Required'}
            disabled={loading}
            required={mode === 'create'}
            sx={{
              '& .MuiOutlinedInput-root': {
                borderRadius: designTokens.borderRadius.md,
              },
            }}
          />

          {mode === 'create' && (
            <TextField
              label="Email (Optional)"
              type="email"
              placeholder="alice@example.com"
              fullWidth
              variant="outlined"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              disabled={loading}
              sx={{
                '& .MuiOutlinedInput-root': {
                  borderRadius: designTokens.borderRadius.md,
                },
              }}
            />
          )}

          <FormControl fullWidth>
            <FormLabel sx={{ mb: 1, fontWeight: 500 }}>Relationship</FormLabel>
            <Select
              value={relationship}
              onChange={(e) => setRelationship(e.target.value as typeof relationship)}
              disabled={loading}
              sx={{
                borderRadius: designTokens.borderRadius.md,
              }}
            >
              <MenuItem value="colleague">Colleague</MenuItem>
              <MenuItem value="friend">Friend</MenuItem>
              <MenuItem value="family">Family</MenuItem>
              <MenuItem value="acquaintance">Acquaintance</MenuItem>
            </Select>
          </FormControl>

          {mode === 'create' && !isOnline && (
            <Alert severity="info" icon={<OfflineIcon />}>
              A temporary identity will be created. It will be upgraded to a permanent
              network identity when you reconnect.
            </Alert>
          )}
        </Stack>
      </DialogContent>

      <DialogActions sx={{ p: 2.5 }}>
        <ModernButton
          onClick={onClose}
          variant="outlined"
          gradient={false}
          disabled={loading}
        >
          Cancel
        </ModernButton>
        <ModernButton
          onClick={handleSubmit}
          disabled={!canSubmit()}
          startIcon={loading ? <CircularProgress size={16} /> : undefined}
        >
          {getSubmitButtonText()}
        </ModernButton>
      </DialogActions>
    </StyledDialog>
  );
};