import React, { useState, useEffect } from 'react';
import {
  Box,
  Paper,
  Tabs,
  Tab,
  Typography,
  Chip,
  Stack,
  IconButton,
  Tooltip,
  Alert,
  CircularProgress,
  Avatar,
  Badge,
  alpha,
  useTheme,
} from '@mui/material';
import { styled } from '@mui/material/styles';
import { motion } from 'framer-motion';
import {
  ChatBubbleOutline as MessagesIcon,
  FolderOutlined as StorageIcon,
  LanguageOutlined as WebsiteIcon,
  SecurityOutlined as SecurityIcon,
  GroupOutlined as GroupIcon,
  PersonOutlined as PersonIcon,
  BusinessOutlined as ProjectIcon,
  TagOutlined as ChannelIcon,
  RefreshOutlined as RefreshIcon,
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { useNavigation } from '../../contexts/NavigationContext';
import MessagesPanel from './MessagesPanel';
import StoragePanel from './StoragePanel';
import WebsitePanel from './WebsitePanel';
import { GlassCard } from '../ui/GlassCard';
import { ModernButton } from '../ui/ModernButton';
import { designTokens } from '../../styles/theme';

interface EncryptionStatus {
  enabled: boolean;
  threshold: { k: number; m: number };
  available_shards: number;
  health_status: 'healthy' | 'degraded' | 'critical';
}

interface EntityContentViewProps {
  entityType: 'individual' | 'group' | 'channel' | 'project';
  entityId: string;
  entityName: string;
  fourWords: string;
}

// Styled components for glassmorphism
const StyledTabs = styled(Tabs)(({ theme }) => ({
  background: alpha(theme.palette.background.paper, 0.8),
  backdropFilter: 'blur(20px)',
  borderRadius: designTokens.borderRadius.lg,
  padding: theme.spacing(0.5),
  '& .MuiTabs-indicator': {
    background: designTokens.colors.primary.gradient,
    height: 3,
    borderRadius: designTokens.borderRadius.full,
  },
}));

const StyledTab = styled(Tab)(({ theme }) => ({
  borderRadius: designTokens.borderRadius.md,
  transition: `all ${designTokens.transitions.normal}`,
  '&:hover': {
    background: alpha(theme.palette.primary.main, 0.1),
  },
  '&.Mui-selected': {
    color: theme.palette.primary.main,
  },
}));

const ProfileHeader = styled(GlassCard)(({ theme }) => ({
  padding: theme.spacing(3),
  marginBottom: theme.spacing(2),
  background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.1)} 0%, ${alpha(theme.palette.secondary.main, 0.1)} 100%)`,
}));

const OnlineIndicator = styled(Badge)(({ theme }) => ({
  '& .MuiBadge-badge': {
    backgroundColor: '#44b700',
    color: '#44b700',
    boxShadow: `0 0 0 2px ${theme.palette.background.paper}`,
    '&::after': {
      position: 'absolute',
      top: 0,
      left: 0,
      width: '100%',
      height: '100%',
      borderRadius: '50%',
      animation: 'ripple 1.2s infinite ease-in-out',
      border: '1px solid currentColor',
      content: '""',
    },
  },
  '@keyframes ripple': {
    '0%': {
      transform: 'scale(.8)',
      opacity: 1,
    },
    '100%': {
      transform: 'scale(2.4)',
      opacity: 0,
    },
  },
}));

const MotionBox = motion(Box);

const EntityContentView: React.FC<EntityContentViewProps> = ({
  entityType,
  entityId,
  entityName,
  fourWords,
}) => {
  const [activeTab, setActiveTab] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [encryptionStatus, setEncryptionStatus] = useState<EncryptionStatus | null>(null);
  const [permissions, setPermissions] = useState<string[]>([]);
  const { state: navState } = useNavigation();

  // Get entity icon based on type
  const getEntityIcon = () => {
    switch (entityType) {
      case 'individual':
        return <PersonIcon />;
      case 'group':
        return <GroupIcon />;
      case 'channel':
        return <ChannelIcon />;
      case 'project':
        return <ProjectIcon />;
      default:
        return <PersonIcon />;
    }
  };

  // Get entity color based on type
  const getEntityColor = () => {
    switch (entityType) {
      case 'individual':
        return 'primary';
      case 'group':
        return 'secondary';
      case 'channel':
        return 'info';
      case 'project':
        return 'success';
      default:
        return 'default';
    }
  };

  // Load encryption status
  const loadEncryptionStatus = async () => {
    try {
      const status = await invoke('core_entity_get_encryption_status', {
        entity_id: entityId,
      });
      setEncryptionStatus(status as EncryptionStatus);
    } catch (err) {
      console.error('Failed to load encryption status:', err);
    }
  };

  // Load user permissions for this entity
  const loadPermissions = async () => {
    try {
      const perms = await invoke('core_entity_get_permissions', {
        entity_id: entityId,
      });
      setPermissions(perms as string[]);
    } catch (err) {
      console.error('Failed to load permissions:', err);
      // Default to read-only if we can't load permissions
      setPermissions(['read']);
    }
  };

  useEffect(() => {
    setLoading(true);
    Promise.all([loadEncryptionStatus(), loadPermissions()])
      .finally(() => setLoading(false));
  }, [entityId]);

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const handleRefresh = () => {
    loadEncryptionStatus();
    // Trigger refresh in active panel
    window.dispatchEvent(new CustomEvent('entity:refresh', { detail: { entityId } }));
  };

  // Encryption health indicator
  const EncryptionIndicator: React.FC = () => {
    if (!encryptionStatus) return null;

    const getColor = () => {
      switch (encryptionStatus.health_status) {
        case 'healthy':
          return 'success';
        case 'degraded':
          return 'warning';
        case 'critical':
          return 'error';
        default:
          return 'default';
      }
    };

    return (
      <Tooltip title={`Threshold encryption: ${encryptionStatus.threshold.k}-of-${encryptionStatus.threshold.k + encryptionStatus.threshold.m}`}>
        <Chip
          icon={<SecurityIcon />}
          label={`${encryptionStatus.available_shards}/${encryptionStatus.threshold.k + encryptionStatus.threshold.m} shards`}
          color={getColor()}
          variant="outlined"
          size="small"
        />
      </Tooltip>
    );
  };

  if (loading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
        <CircularProgress />
      </Box>
    );
  }

  const theme = useTheme();

  return (
    <MotionBox
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}
    >
      {/* Entity Header with Glassmorphism */}
      <ProfileHeader
        variant="light"
        elevation={0}
        sx={{
          mb: 2,
          background: theme => theme.palette.mode === 'dark'
            ? 'linear-gradient(135deg, rgba(66, 66, 66, 0.8) 0%, rgba(33, 33, 33, 0.8) 100%)'
            : 'linear-gradient(135deg, rgba(255, 255, 255, 0.9) 0%, rgba(245, 245, 245, 0.9) 100%)',
        }}
      >
        <Stack direction="row" alignItems="center" spacing={2}>
          {/* Entity Icon and Name */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flex: 1 }}>
            {entityType === 'individual' ? (
              <OnlineIndicator
                overlap="circular"
                anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
                variant="dot"
              >
                <Avatar
                  sx={{
                    width: 56,
                    height: 56,
                    background: designTokens.colors.primary.gradient,
                    fontSize: '1.5rem',
                    fontWeight: 600,
                  }}
                >
                  {entityName.charAt(0).toUpperCase()}
                </Avatar>
              </OnlineIndicator>
            ) : (
              <Box
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  width: 56,
                  height: 56,
                  borderRadius: 2,
                  background: designTokens.colors.primary.gradient,
                  color: 'white',
                }}
              >
                {getEntityIcon()}
              </Box>
            )}
            <Box>
              <Typography variant="h5" fontWeight="bold">
                {entityName}
              </Typography>
              <Stack direction="row" spacing={1} alignItems="center">
                <Typography variant="body2" color="text.secondary">
                  {fourWords}
                </Typography>
                {entityType === 'individual' && (
                  <Chip
                    label="Online"
                    size="small"
                    color="success"
                    variant="filled"
                    sx={{ height: 20, fontSize: '0.75rem' }}
                  />
                )}
              </Stack>
            </Box>
          </Box>

          {/* Status Indicators */}
          <Stack direction="row" spacing={1} alignItems="center">
            <EncryptionIndicator />

            <Chip
              label={entityType}
              color={getEntityColor()}
              variant="filled"
              size="small"
            />

            {permissions.includes('admin') && (
              <Chip
                label="Admin"
                color="warning"
                variant="outlined"
                size="small"
              />
            )}

            <Tooltip title="Refresh">
              <IconButton onClick={handleRefresh} size="small">
                <RefreshIcon />
              </IconButton>
            </Tooltip>
          </Stack>
        </Stack>
      </ProfileHeader>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Content Tabs with Glassmorphism */}
      <GlassCard variant="light" sx={{ flex: 1, display: 'flex', flexDirection: 'column', p: 2 }}>
        <StyledTabs
          value={activeTab}
          onChange={handleTabChange}
          indicatorColor="primary"
          textColor="primary"
        >
          <StyledTab
            icon={<MessagesIcon />}
            label="Messages"
            iconPosition="start"
            data-testid="messages-tab"
          />
          <StyledTab
            icon={<StorageIcon />}
            label="Storage"
            iconPosition="start"
            data-testid="storage-tab"
          />
          <StyledTab
            icon={<WebsiteIcon />}
            label="Website"
            iconPosition="start"
            data-testid="website-tab"
          />
        </StyledTabs>

        {/* Tab Panels */}
        <Box sx={{ flex: 1, overflow: 'hidden', mt: 2 }}>
          {activeTab === 0 && (
            <MessagesPanel
              entityType={entityType}
              entityId={entityId}
              entityName={entityName}
              fourWords={fourWords}
              permissions={permissions}
            />
          )}
          {activeTab === 1 && (
            <StoragePanel
              entityType={entityType}
              entityId={entityId}
              entityName={entityName}
              fourWords={fourWords}
              permissions={permissions}
              encryptionStatus={encryptionStatus}
            />
          )}
          {activeTab === 2 && (
            <WebsitePanel
              entityType={entityType}
              entityId={entityId}
              entityName={entityName}
              fourWords={fourWords}
              permissions={permissions}
            />
          )}
        </Box>
      </GlassCard>
    </MotionBox>
  );
};

export default EntityContentView;