import {
    AccessTime as AccessTimeIcon, Add as AddIcon,
    ArrowForward as ArrowForwardIcon, Delete as DeleteIcon, Fingerprint as FingerprintIcon, MoreVert as MoreVertIcon, QrCode as QrCodeIcon,
    QrCodeScanner as QrCodeScannerIcon,
    Search as SearchIcon, Security as SecurityIcon
} from '@mui/icons-material';
import {
    Alert, alpha, Avatar, Box, Button, Card,
    CardContent, Chip, CircularProgress, Divider, IconButton, InputAdornment, ListItemIcon,
    ListItemText, Menu,
    MenuItem, Stack, TextField, Tooltip, Typography
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { ask } from '@tauri-apps/plugin-dialog';
import React, { useEffect, useState } from 'react';
import { QRCodeExportDialog } from './QRCodeExportDialog';
import { QRCodeImportDialog } from './QRCodeImportDialog';

// Recent identity from backend
interface RecentIdentity {
  four_words: string;
  display_name: string;
  last_used: number;
  has_passkey: boolean;
}

interface IdentityPickerProps {
  onSelectIdentity: (fourWords: string, usePasskey: boolean) => Promise<void>;
  onCreateNew: () => void;
  onManualEntry?: (fourWords: string) => void;
}

export const IdentityPicker: React.FC<IdentityPickerProps> = ({
  onSelectIdentity,
  onCreateNew,
  onManualEntry,
}) => {
  const [identities, setIdentities] = useState<RecentIdentity[]>([]);
  const [loading, setLoading] = useState(true);
  const [authenticating, setAuthenticating] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [manualFourWords, setManualFourWords] = useState<string>('');
  const [showQRExport, setShowQRExport] = useState(false);
  const [showQRImport, setShowQRImport] = useState(false);
  const [selectedQRIdentity, setSelectedQRIdentity] = useState<RecentIdentity | null>(null);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [menuAnchor, setMenuAnchor] = useState<{ element: HTMLElement; identity: RecentIdentity } | null>(null);

  // Filter identities based on search query
  const filteredIdentities = identities.filter((identity) => {
    if (!searchQuery) return true;
    const query = searchQuery.toLowerCase();
    return (
      identity.display_name.toLowerCase().includes(query) ||
      identity.four_words.toLowerCase().includes(query)
    );
  });

  // Load recent identities on mount
  useEffect(() => {
    loadRecentIdentities();
  }, []);

  const loadRecentIdentities = async () => {
    try {
      setLoading(true);
      setError(null);

      // Initialize encrypted storage
      await invoke('auth_initialize');

      // Get recent identities
      const recent = await invoke<RecentIdentity[]>('auth_get_recent_identities');
      setIdentities(recent);
    } catch (err) {
      console.error('Failed to load identities:', err);
      setError('Failed to load identities. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleSelectIdentity = async (fourWords: string, usePasskey: boolean) => {
    try {
      setAuthenticating(fourWords);
      setError(null);
      await onSelectIdentity(fourWords, usePasskey);
    } catch (err: any) {
      console.error('Authentication failed:', err);
      setError(err?.message || 'Authentication failed. Please try again.');
    } finally {
      setAuthenticating(null);
    }
  };

  const formatLastUsed = (timestamp: number): string => {
    const now = Date.now();
    const secondsAgo = Math.floor((now - timestamp * 1000) / 1000);

    if (secondsAgo < 60) return 'Just now';
    if (secondsAgo < 3600) return `${Math.floor(secondsAgo / 60)}m ago`;
    if (secondsAgo < 86400) return `${Math.floor(secondsAgo / 3600)}h ago`;
    if (secondsAgo < 604800) return `${Math.floor(secondsAgo / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  const getAvatarColor = (fourWords: string): string => {
    // Generate consistent color from four-word address
    let hash = 0;
    for (let i = 0; i < fourWords.length; i++) {
      hash = fourWords.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue = Math.abs(hash % 360);
    return `hsl(${hue}, 65%, 55%)`;
  };

  const handleQRImport = async (qrData: string) => {
    try {
      const data = JSON.parse(qrData);

      // Validate QR code format
      if (data.type !== 'communitas-identity' || !data.fourWords || !data.displayName) {
        throw new Error('Invalid QR code format');
      }

      // Use manual entry handler to prompt for password
      if (onManualEntry) {
        onManualEntry(data.fourWords);
      }

      setShowQRImport(false);
    } catch (err) {
      console.error('Failed to import from QR code:', err);
      throw new Error('Invalid identity QR code');
    }
  };

  if (loading) {
    return (
      <Box
        sx={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          minHeight: '400px',
          gap: 2,
        }}
      >
        <CircularProgress size={48} />
        <Typography variant="body1" color="text.secondary">
          Loading identities...
        </Typography>
      </Box>
    );
  }

  return (
    <Box
      sx={{
        maxWidth: 600,
        width: '100%',
        margin: '0 auto',
        padding: 3,
      }}
    >
      {/* Header */}
      <Box sx={{ textAlign: 'center', mb: 4 }}>
        <Typography variant="h4" gutterBottom fontWeight={600}>
          Welcome to Communitas
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Select your identity to continue
        </Typography>
      </Box>

      {/* Import from QR Code Button */}
      <Button
        fullWidth
        variant="outlined"
        startIcon={<QrCodeScannerIcon />}
        onClick={() => setShowQRImport(true)}
        disabled={authenticating !== null}
        sx={{
          mb: 3,
          py: 1.5,
          borderStyle: 'dashed',
          borderWidth: 2,
          '&:hover': {
            borderStyle: 'dashed',
            borderWidth: 2,
            bgcolor: (theme) => alpha(theme.palette.primary.main, 0.05),
          },
        }}
      >
        Import Identity from QR Code
      </Button>

      {/* Search Field */}
      {identities.length > 3 && (
        <TextField
          fullWidth
          placeholder="Search identities..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          disabled={authenticating !== null}
          sx={{ mb: 3 }}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <SearchIcon />
              </InputAdornment>
            ),
          }}
        />
      )}

      {/* Error Alert */}
      {error && (
        <Alert severity="error" sx={{ mb: 3 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {/* Identity Cards */}
      <Stack spacing={2}>
        {filteredIdentities.length === 0 && searchQuery && (
          <Alert severity="info">
            No identities found matching "{searchQuery}"
          </Alert>
        )}
        {filteredIdentities.map((identity) => (
          <Card
            key={identity.four_words}
            elevation={authenticating === identity.four_words ? 8 : 2}
            sx={{
              transition: 'all 0.2s ease',
              cursor: authenticating ? 'default' : 'pointer',
              '&:hover': authenticating
                ? {}
                : {
                    elevation: 4,
                    transform: 'translateY(-2px)',
                  },
              position: 'relative',
              overflow: 'visible',
            }}
          >
            <CardContent sx={{ p: 3 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                {/* Avatar */}
                <Avatar
                  sx={{
                    width: 56,
                    height: 56,
                    bgcolor: getAvatarColor(identity.four_words),
                    fontSize: '1.5rem',
                    fontWeight: 600,
                  }}
                >
                  {identity.display_name.charAt(0).toUpperCase()}
                </Avatar>

                {/* Identity Info */}
                <Box sx={{ flex: 1, minWidth: 0 }}>
                  <Typography variant="h6" fontWeight={600} noWrap>
                    {identity.display_name}
                  </Typography>
                  <Typography
                    variant="body2"
                    color="text.secondary"
                    sx={{
                      fontFamily: 'monospace',
                      fontSize: '0.85rem',
                    }}
                  >
                    {identity.four_words}
                  </Typography>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mt: 0.5 }}>
                    <AccessTimeIcon sx={{ fontSize: 14, color: 'text.secondary' }} />
                    <Typography variant="caption" color="text.secondary">
                      {formatLastUsed(identity.last_used)}
                    </Typography>
                    {identity.has_passkey && (
                      <Chip
                        icon={<SecurityIcon />}
                        label="Passkey"
                        size="small"
                        color="primary"
                        variant="outlined"
                        sx={{ ml: 1, height: 20 }}
                      />
                    )}
                  </Box>
                </Box>

                {/* Action Buttons */}
                <Stack direction="row" spacing={1}>
                  <Tooltip title="More options">
                    <IconButton
                      size="small"
                      disabled={authenticating !== null}
                      onClick={(e) => {
                        e.stopPropagation();
                        setMenuAnchor({ element: e.currentTarget, identity });
                      }}
                      sx={{
                        color: 'text.secondary',
                        '&:hover': {
                          bgcolor: (theme) => alpha(theme.palette.text.secondary, 0.1),
                        },
                      }}
                    >
                      <MoreVertIcon fontSize="small" />
                    </IconButton>
                  </Tooltip>
                  {identity.has_passkey && (
                    <Tooltip title="Sign in with biometric">
                      <IconButton
                        color="primary"
                        disabled={authenticating !== null}
                        onClick={() => handleSelectIdentity(identity.four_words, true)}
                        sx={{
                          bgcolor: (theme) => alpha(theme.palette.primary.main, 0.1),
                          '&:hover': {
                            bgcolor: (theme) => alpha(theme.palette.primary.main, 0.2),
                          },
                        }}
                      >
                        {authenticating === identity.four_words ? (
                          <CircularProgress size={24} />
                        ) : (
                          <FingerprintIcon />
                        )}
                      </IconButton>
                    </Tooltip>
                  )}
                  <Tooltip title="Sign in with password">
                    <IconButton
                      disabled={authenticating !== null}
                      onClick={() => handleSelectIdentity(identity.four_words, false)}
                    >
                      <ArrowForwardIcon />
                    </IconButton>
                  </Tooltip>
                </Stack>
              </Box>
            </CardContent>
          </Card>
        ))}
      </Stack>

      {/* Manual Four-Word Entry */}
      {onManualEntry && (
        <>
          <Divider sx={{ my: 3 }}>
            <Typography variant="body2" color="text.secondary">
              or
            </Typography>
          </Divider>

          <Box
            component="form"
            onSubmit={(e: React.FormEvent) => {
              e.preventDefault()
              if (manualFourWords.trim() && onManualEntry) {
                onManualEntry(manualFourWords.trim())
              }
            }}
          >
            <Stack spacing={2}>
              <Typography variant="body2" color="text.secondary" sx={{ textAlign: 'center' }}>
                Enter your four-word identity to sign in from a new device
              </Typography>
              <TextField
                fullWidth
                label="Four-Word Identity"
                placeholder="ocean-forest-moon-star"
                value={manualFourWords}
                onChange={(e) => setManualFourWords(e.target.value)}
                disabled={authenticating !== null}
                helperText="Enter your four-word address from another device"
                InputProps={{
                  sx: {
                    fontFamily: 'monospace',
                  },
                }}
              />
              <Button
                type="submit"
                fullWidth
                variant="contained"
                disabled={!manualFourWords.trim() || authenticating !== null}
                sx={{ py: 1.5 }}
              >
                Sign In with Four Words
              </Button>
            </Stack>
          </Box>
        </>
      )}

      {/* Create New Identity */}
      <Divider sx={{ my: 3 }}>
        <Typography variant="body2" color="text.secondary">
          {onManualEntry ? 'or' : 'or'}
        </Typography>
      </Divider>

      <Button
        fullWidth
        variant="outlined"
        size="large"
        startIcon={<AddIcon />}
        onClick={onCreateNew}
        disabled={authenticating !== null}
        sx={{
          py: 1.5,
          borderStyle: 'dashed',
          borderWidth: 2,
          '&:hover': {
            borderStyle: 'dashed',
            borderWidth: 2,
          },
        }}
      >
        Create New Identity
      </Button>

      {/* Info Text */}
      <Typography
        variant="caption"
        color="text.secondary"
        sx={{ display: 'block', textAlign: 'center', mt: 3 }}
      >
        Your identities are stored securely with end-to-end encryption
      </Typography>

      {/* Context Menu for Identity Options */}
      <Menu
        anchorEl={menuAnchor?.element}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
        transformOrigin={{ horizontal: 'right', vertical: 'top' }}
        anchorOrigin={{ horizontal: 'right', vertical: 'bottom' }}
      >
        <MenuItem
          onClick={() => {
            if (menuAnchor) {
              setSelectedQRIdentity(menuAnchor.identity);
              setShowQRExport(true);
              setMenuAnchor(null);
            }
          }}
        >
          <ListItemIcon>
            <QrCodeIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Export as QR Code</ListItemText>
        </MenuItem>
        <Divider />
        <MenuItem
          onClick={async (e) => {
            e.preventDefault();
            e.stopPropagation();

            console.log('🔍 Remove menu item clicked', { menuAnchor });

            if (!menuAnchor) {
              console.log('⚠️ No menuAnchor found');
              return;
            }

            const identity = menuAnchor.identity;
            console.log('📝 Identity to remove:', identity);

            // Store identity data before closing menu
            const displayName = identity.display_name;
            const fourWords = identity.four_words;

            console.log('🔒 Captured identity data:', { displayName, fourWords });
            setMenuAnchor(null); // Close menu

            // Add a small delay to ensure menu closes before dialog
            console.log('⏱️ Waiting for menu to close...');
            await new Promise(resolve => setTimeout(resolve, 100));

            console.log('💬 About to show confirmation dialog...');

            try {
              console.log('💬 Calling ask() function...');
              const confirmed = await ask(
                `This will remove "${displayName}" from the recent identities list but will not delete the vault.`,
                {
                  title: 'Remove Identity?',
                  kind: 'warning',
                  okLabel: 'Remove',
                  cancelLabel: 'Cancel',
                }
              );
              console.log('💬 User confirmation result:', confirmed);

              if (confirmed) {
                console.log('🗑️ User confirmed removal, calling backend for:', fourWords);

                invoke('auth_remove_recent_identity', {
                  fourWords: fourWords
                })
                  .then(() => {
                    console.log('✅ Identity removed successfully');
                    // Refresh the identities list
                    loadRecentIdentities();
                  })
                  .catch((err) => {
                    console.error('❌ Failed to remove identity:', err);
                    setError(err instanceof Error ? err.message : 'Failed to remove identity');
                  });
              } else {
                console.log('🚫 User cancelled removal');
              }
            } catch (err) {
              console.error('❌ Dialog error:', err);
              console.error('❌ Error details:', {
                message: err instanceof Error ? err.message : String(err),
                stack: err instanceof Error ? err.stack : undefined,
                type: typeof err
              });
            }
          }}
          sx={{ color: 'error.main' }}
        >
          <ListItemIcon>
            <DeleteIcon fontSize="small" color="error" />
          </ListItemIcon>
          <ListItemText>Remove from Device</ListItemText>
        </MenuItem>
      </Menu>

      {/* QR Code Export Dialog */}
      {selectedQRIdentity && (
        <QRCodeExportDialog
          open={showQRExport}
          onClose={() => {
            setShowQRExport(false);
            setSelectedQRIdentity(null);
          }}
          fourWords={selectedQRIdentity.four_words}
          displayName={selectedQRIdentity.display_name}
        />
      )}

      {/* QR Code Import Dialog */}
      <QRCodeImportDialog
        open={showQRImport}
        onClose={() => setShowQRImport(false)}
        onImport={handleQRImport}
      />
    </Box>
  );
};
