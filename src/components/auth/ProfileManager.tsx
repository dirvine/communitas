import React, { useState, useCallback } from 'react';
import {
  Box,
  Paper,
  Typography,
  TextField,
  Button,
  Avatar,
  Stack,
  Chip,
  IconButton,
  Card,
  CardContent,
  Grid,
  Alert,
  Divider,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Switch,
  FormControlLabel,
  Tooltip,
  CircularProgress,
} from '@mui/material';
import {
  Edit as EditIcon,
  Save as SaveIcon,
  Cancel as CancelIcon,
  Key as KeyIcon,
  Security as SecurityIcon,
  Person as PersonIcon,
  Email as EmailIcon,
  Public as PublicIcon,
  Lock as LockIcon,
  Fingerprint as FingerprintIcon,
  NetworkCheck as NetworkIcon,
  Settings as SettingsIcon,
  Shield as ShieldIcon,
  ContentCopy as CopyIcon,
  Refresh as RefreshIcon,
  GitHub as GitHubIcon,
  Twitter as TwitterIcon,
  LinkedIn as LinkedInIcon,
} from '@mui/icons-material';
import { motion } from 'framer-motion';
import { useAuth, UserIdentity, Permission } from '../../contexts/AuthContext';
import { useResponsive, ResponsiveContainer, ResponsiveGrid } from '../responsive';

interface ProfileManagerProps {
  onClose?: () => void;
  compact?: boolean;
}

export const ProfileManager: React.FC<ProfileManagerProps> = ({
  onClose,
  compact = false,
}) => {
  const { authState, updateProfile, signMessage, getNetworkStatus, logout } = useAuth();
  const { isMobile } = useResponsive();
  const [editing, setEditing] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [showKeyDialog, setShowKeyDialog] = useState(false);
  const [networkStatus, setNetworkStatus] = useState<{ connected: boolean; peers: number } | null>(null);
  
  const [formData, setFormData] = useState({
    name: authState.user?.name || '',
    bio: authState.user?.profile.bio || '',
    organization: authState.user?.profile.organization || '',
    location: authState.user?.profile.location || '',
    website: authState.user?.profile.website || '',
    github: authState.user?.profile.socialLinks?.github || '',
    twitter: authState.user?.profile.socialLinks?.twitter || '',
    linkedin: authState.user?.profile.socialLinks?.linkedin || '',
  });

  const user = authState.user as UserIdentity;

  const handleSave = async () => {
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await updateProfile({
        bio: formData.bio,
        organization: formData.organization,
        location: formData.location,
        website: formData.website,
        socialLinks: {
          github: formData.github,
          twitter: formData.twitter,
          linkedin: formData.linkedin,
        },
      });

      setEditing(false);
      setSuccess('Profile updated successfully');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update profile');
    } finally {
      setLoading(false);
    }
  };

  const handleCancel = () => {
    setFormData({
      name: user?.name || '',
      bio: user?.profile.bio || '',
      organization: user?.profile.organization || '',
      location: user?.profile.location || '',
      website: user?.profile.website || '',
      github: user?.profile.socialLinks?.github || '',
      twitter: user?.profile.socialLinks?.twitter || '',
      linkedin: user?.profile.socialLinks?.linkedin || '',
    });
    setEditing(false);
    setError(null);
  };

  const copyToClipboard = useCallback(async (text: string, label: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setSuccess(`${label} copied to clipboard`);
    } catch (err) {
      setError('Failed to copy to clipboard');
    }
  }, []);

  const checkNetworkStatus = async () => {
    try {
      const status = await getNetworkStatus();
      setNetworkStatus(status);
    } catch (err) {
      console.error('Failed to get network status:', err);
    }
  };

  React.useEffect(() => {
    checkNetworkStatus();
    const interval = setInterval(checkNetworkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const renderPermissionChip = (permission: Permission) => (
    <Chip
      key={`${permission.resource}-${permission.actions.join(',')}`}
      label={`${permission.resource}: ${permission.actions.join(', ')}`}
      variant="outlined"
      size="small"
      icon={<ShieldIcon />}
      sx={{ margin: 0.5 }}
    />
  );

  if (!user) {
    return (
      <Card>
        <CardContent>
          <Alert severity="error">No user data available</Alert>
        </CardContent>
      </Card>
    );
  }

  return (
    <ResponsiveContainer maxWidth="lg">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
      >
        <Stack spacing={3}>
          {/* Header */}
          <Paper sx={{ p: 3 }}>
            <Stack
              direction={isMobile ? 'column' : 'row'}
              alignItems={isMobile ? 'center' : 'flex-start'}
              spacing={3}
            >
              <Avatar
                sx={{
                  width: 80,
                  height: 80,
                  bgcolor: 'primary.main',
                  fontSize: '2rem',
                  fontWeight: 600,
                }}
              >
                {user.name.charAt(0).toUpperCase()}
              </Avatar>
              
              <Box flex={1}>
                <Stack
                  direction={isMobile ? 'column' : 'row'}
                  alignItems={isMobile ? 'center' : 'flex-start'}
                  justifyContent="space-between"
                  spacing={2}
                >
                  <Box textAlign={isMobile ? 'center' : 'left'}>
                    <Typography variant="h4" fontWeight={600} gutterBottom>
                      {user.name}
                    </Typography>
                    
                    <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap" gap={1}>
                      <Chip
                        icon={<FingerprintIcon />}
                        label={user.fourWordAddress}
                        variant="outlined"
                        size="small"
                        onClick={() => copyToClipboard(user.fourWordAddress, 'Four-word address')}
                        clickable
                      />
                      
                      {networkStatus && (
                        <Chip
                          icon={<NetworkIcon />}
                          label={`${networkStatus.connected ? 'Connected' : 'Disconnected'} • ${networkStatus.peers} peers`}
                          color={networkStatus.connected ? 'success' : 'warning'}
                          variant="outlined"
                          size="small"
                        />
                      )}
                    </Stack>
                    
                    {user.profile.bio && (
                      <Typography variant="body1" color="text.secondary" sx={{ mt: 1 }}>
                        {user.profile.bio}
                      </Typography>
                    )}
                  </Box>

                  <Stack direction="row" spacing={1}>
                    <Button
                      variant={editing ? 'outlined' : 'contained'}
                      startIcon={editing ? <CancelIcon /> : <EditIcon />}
                      onClick={editing ? handleCancel : () => setEditing(true)}
                      disabled={loading}
                    >
                      {editing ? 'Cancel' : 'Edit Profile'}
                    </Button>
                    
                    {editing && (
                      <Button
                        variant="contained"
                        startIcon={loading ? <CircularProgress size={20} /> : <SaveIcon />}
                        onClick={handleSave}
                        disabled={loading}
                      >
                        {loading ? 'Saving...' : 'Save'}
                      </Button>
                    )}
                  </Stack>
                </Stack>
              </Box>
            </Stack>

            {/* Status Messages */}
            {error && (
              <Alert severity="error" sx={{ mt: 2 }} onClose={() => setError(null)}>
                {error}
              </Alert>
            )}
            {success && (
              <Alert severity="success" sx={{ mt: 2 }} onClose={() => setSuccess(null)}>
                {success}
              </Alert>
            )}
          </Paper>

          {/* Profile Details */}
          <ResponsiveGrid columns={{ xs: 1, md: 2 }} spacing={3}>
            {/* Basic Information */}
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" fontWeight={600} gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <PersonIcon color="primary" />
                Basic Information
              </Typography>
              
              <Stack spacing={2}>
                <TextField
                  fullWidth
                  label="Display Name"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  disabled={!editing}
                />
                
                <TextField
                  fullWidth
                  multiline
                  rows={3}
                  label="Bio"
                  value={formData.bio}
                  onChange={(e) => setFormData(prev => ({ ...prev, bio: e.target.value }))}
                  disabled={!editing}
                  placeholder="Tell others about yourself..."
                />
                
                <TextField
                  fullWidth
                  label="Organization"
                  value={formData.organization}
                  onChange={(e) => setFormData(prev => ({ ...prev, organization: e.target.value }))}
                  disabled={!editing}
                />
                
                <TextField
                  fullWidth
                  label="Location"
                  value={formData.location}
                  onChange={(e) => setFormData(prev => ({ ...prev, location: e.target.value }))}
                  disabled={!editing}
                />
                
                <TextField
                  fullWidth
                  label="Website"
                  value={formData.website}
                  onChange={(e) => setFormData(prev => ({ ...prev, website: e.target.value }))}
                  disabled={!editing}
                  placeholder="https://example.com"
                />
              </Stack>
            </Paper>

            {/* Social Links */}
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" fontWeight={600} gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <PublicIcon color="primary" />
                Social Links
              </Typography>
              
              <Stack spacing={2}>
                <TextField
                  fullWidth
                  label="GitHub"
                  value={formData.github}
                  onChange={(e) => setFormData(prev => ({ ...prev, github: e.target.value }))}
                  disabled={!editing}
                  InputProps={{
                    startAdornment: <GitHubIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                  }}
                  placeholder="username"
                />
                
                <TextField
                  fullWidth
                  label="Twitter"
                  value={formData.twitter}
                  onChange={(e) => setFormData(prev => ({ ...prev, twitter: e.target.value }))}
                  disabled={!editing}
                  InputProps={{
                    startAdornment: <TwitterIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                  }}
                  placeholder="@username"
                />
                
                <TextField
                  fullWidth
                  label="LinkedIn"
                  value={formData.linkedin}
                  onChange={(e) => setFormData(prev => ({ ...prev, linkedin: e.target.value }))}
                  disabled={!editing}
                  InputProps={{
                    startAdornment: <LinkedInIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                  }}
                  placeholder="username"
                />
              </Stack>
            </Paper>

            {/* Identity & Security */}
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" fontWeight={600} gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <SecurityIcon color="primary" />
                Identity & Security
              </Typography>
              
              <List dense>
                <ListItem>
                  <ListItemIcon>
                    <FingerprintIcon />
                  </ListItemIcon>
                  <ListItemText
                    primary="Four-Word Address"
                    secondary={user.fourWordAddress}
                  />
                  <Tooltip title="Copy address">
                    <IconButton
                      size="small"
                      onClick={() => copyToClipboard(user.fourWordAddress, 'Four-word address')}
                    >
                      <CopyIcon />
                    </IconButton>
                  </Tooltip>
                </ListItem>
                
                <ListItem>
                  <ListItemIcon>
                    <KeyIcon />
                  </ListItemIcon>
                  <ListItemText
                    primary="Public Key"
                    secondary={`${user.publicKey.substring(0, 16)}...${user.publicKey.substring(-8)}`}
                  />
                  <Tooltip title="Copy public key">
                    <IconButton
                      size="small"
                      onClick={() => copyToClipboard(user.publicKey, 'Public key')}
                    >
                      <CopyIcon />
                    </IconButton>
                  </Tooltip>
                </ListItem>
                
                <ListItem>
                  <ListItemIcon>
                    <EmailIcon />
                  </ListItemIcon>
                  <ListItemText
                    primary="Email"
                    secondary={user.email || 'Not provided'}
                  />
                </ListItem>
              </List>
              
              <Button
                fullWidth
                variant="outlined"
                startIcon={<KeyIcon />}
                onClick={() => setShowKeyDialog(true)}
                sx={{ mt: 2 }}
              >
                Manage Keys
              </Button>
            </Paper>

            {/* Permissions */}
            <Paper sx={{ p: 3 }}>
              <Typography variant="h6" fontWeight={600} gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <ShieldIcon color="primary" />
                Permissions
              </Typography>
              
              <Box sx={{ mt: 2 }}>
                {user.permissions.length > 0 ? (
                  user.permissions.map(renderPermissionChip)
                ) : (
                  <Typography variant="body2" color="text.secondary">
                    No special permissions assigned
                  </Typography>
                )}
              </Box>
              
              <Alert severity="info" sx={{ mt: 2 }}>
                <Typography variant="body2">
                  Permissions are managed by organization administrators
                </Typography>
              </Alert>
            </Paper>
          </ResponsiveGrid>

          {/* Account Actions */}
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" fontWeight={600} gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <SettingsIcon color="primary" />
              Account Actions
            </Typography>
            
            <Stack direction={isMobile ? 'column' : 'row'} spacing={2} sx={{ mt: 2 }}>
              <Button
                variant="outlined"
                startIcon={<RefreshIcon />}
                onClick={checkNetworkStatus}
              >
                Refresh Status
              </Button>
              
              <Button
                variant="outlined"
                color="error"
                startIcon={<LockIcon />}
                onClick={logout}
              >
                Sign Out
              </Button>
            </Stack>
          </Paper>
        </Stack>
      </motion.div>

      {/* Key Management Dialog */}
      <Dialog
        open={showKeyDialog}
        onClose={() => setShowKeyDialog(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Key Management</DialogTitle>
        <DialogContent>
          <Alert severity="info" sx={{ mb: 2 }}>
            <Typography variant="body2">
              Key management features are coming soon. This will allow you to:
            </Typography>
            <Box component="ul" sx={{ mt: 1, mb: 0 }}>
              <li>Export your private key</li>
              <li>Import existing keys</li>
              <li>Generate new key pairs</li>
              <li>Set up key rotation</li>
            </Box>
          </Alert>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShowKeyDialog(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </ResponsiveContainer>
  );
};

export default ProfileManager;