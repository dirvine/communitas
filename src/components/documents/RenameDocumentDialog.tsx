/**
 * RenameDocumentDialog - Dialog for renaming documents with validation
 *
 * Features:
 * - Pre-filled with current document name
 * - Real-time validation (no slashes, max length, non-empty)
 * - Shows storage mode indicator
 * - Error handling for duplicate names and failures
 * - Prevents submission with invalid names
 */

import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Button,
  Alert,
  Stack,
  Chip,
  Typography,
  CircularProgress,
} from '@mui/material';
import {
  Lock as PrivateIcon,
  Language as PublicIcon,
  Edit as RenameIcon,
} from '@mui/icons-material';
import { ModernButton } from '../ui/ModernButton';
import { Document, DocumentStorageMode, isValidDocumentName } from '../../types/documents';

interface RenameDocumentDialogProps {
  /** Whether dialog is open */
  open: boolean;
  /** Document to rename */
  document: Document | null;
  /** Callback when dialog closes */
  onClose: () => void;
  /** Callback when rename confirmed with new name */
  onRename: (document: Document, newName: string) => Promise<void>;
}

// Get storage mode icon and label
const getStorageModeInfo = (mode: DocumentStorageMode) => {
  switch (mode) {
    case 'files':
      return {
        icon: <PrivateIcon fontSize="small" />,
        label: 'Private (Encrypted)',
        color: 'primary' as const,
      };
    case 'web':
      return {
        icon: <PublicIcon fontSize="small" />,
        label: 'Public (Website)',
        color: 'info' as const,
      };
    case 'both':
      return {
        icon: <PublicIcon fontSize="small" />,
        label: 'Both (Private + Public)',
        color: 'secondary' as const,
      };
  }
};

export const RenameDocumentDialog: React.FC<RenameDocumentDialogProps> = ({
  open,
  document,
  onClose,
  onRename,
}) => {
  const [newName, setNewName] = useState('');
  const [validationError, setValidationError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  // Reset state when document changes or dialog opens
  useEffect(() => {
    if (document && open) {
      setNewName(document.name);
      setValidationError(null);
      setSubmitError(null);
      setIsSubmitting(false);
    }
  }, [document, open]);

  // Validate name as user types
  useEffect(() => {
    if (!newName) {
      setValidationError('Name cannot be empty');
      return;
    }

    if (newName === document?.name) {
      setValidationError('Name is unchanged');
      return;
    }

    if (!isValidDocumentName(newName)) {
      if (newName.includes('/')) {
        setValidationError('Name cannot contain slashes');
      } else if (newName.length > 255) {
        setValidationError('Name too long (max 255 characters)');
      } else {
        setValidationError('Invalid document name');
      }
      return;
    }

    setValidationError(null);
  }, [newName, document]);

  const handleSubmit = async () => {
    if (!document || validationError || isSubmitting) return;

    setIsSubmitting(true);
    setSubmitError(null);

    try {
      await onRename(document, newName.trim());
      onClose(); // Close dialog on success
    } catch (error) {
      console.error('Failed to rename document:', error);
      const errorMessage = error instanceof Error ? error.message : 'Failed to rename document';
      setSubmitError(errorMessage);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' && !validationError && !isSubmitting) {
      handleSubmit();
    }
  };

  if (!document) return null;

  const storageModeInfo = getStorageModeInfo(document.storageMode);
  const canSubmit = !validationError && newName.trim() && !isSubmitting && newName !== document.name;

  return (
    <Dialog
      open={open}
      onClose={isSubmitting ? undefined : onClose}
      maxWidth="sm"
      fullWidth
      PaperProps={{
        sx: {
          borderRadius: 2,
          backgroundImage: 'none',
        },
      }}
    >
      <DialogTitle sx={{ pb: 1 }}>
        <Stack direction="row" alignItems="center" spacing={1}>
          <RenameIcon color="primary" />
          <Typography variant="h6" component="span">
            Rename Document
          </Typography>
        </Stack>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={2} sx={{ mt: 1 }}>
          {/* Current name display */}
          <Stack direction="row" alignItems="center" spacing={1}>
            <Typography variant="body2" color="text.secondary">
              Current name:
            </Typography>
            <Typography variant="body2" sx={{ fontWeight: 500 }}>
              {document.name}
            </Typography>
            <Chip
              icon={storageModeInfo.icon}
              label={storageModeInfo.label}
              size="small"
              color={storageModeInfo.color}
              variant="outlined"
            />
          </Stack>

          {/* New name input */}
          <TextField
            autoFocus
            fullWidth
            label="New Document Name"
            placeholder="Enter new name"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyPress={handleKeyPress}
            error={Boolean(validationError)}
            helperText={validationError || `${newName.length}/255 characters`}
            disabled={isSubmitting}
            variant="outlined"
            inputProps={{
              maxLength: 255,
            }}
          />

          {/* Submit error */}
          {submitError && (
            <Alert severity="error" onClose={() => setSubmitError(null)}>
              {submitError}
            </Alert>
          )}

          {/* Info message */}
          <Alert severity="info" icon={false}>
            <Typography variant="caption">
              The document will be renamed while preserving all content and version history.
              {document.storageMode === 'both' && ' This will update the name in both private and public storage.'}
            </Typography>
          </Alert>
        </Stack>
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button
          onClick={onClose}
          disabled={isSubmitting}
          color="inherit"
        >
          Cancel
        </Button>
        <ModernButton
          variant="contained"
          gradient={true}
          onClick={handleSubmit}
          disabled={!canSubmit}
          startIcon={isSubmitting ? <CircularProgress size={16} /> : <RenameIcon />}
        >
          {isSubmitting ? 'Renaming...' : 'Rename'}
        </ModernButton>
      </DialogActions>
    </Dialog>
  );
};
