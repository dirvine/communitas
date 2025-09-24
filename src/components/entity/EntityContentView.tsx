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
} from '@mui/material';
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
        entityId,
        path: '/',
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
        entityId,
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

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Entity Header */}
      <Paper
        elevation={2}
        sx={{
          p: 2,
          mb: 2,
          background: theme => theme.palette.mode === 'dark'
            ? 'linear-gradient(135deg, rgba(66, 66, 66, 0.8) 0%, rgba(33, 33, 33, 0.8) 100%)'
            : 'linear-gradient(135deg, rgba(255, 255, 255, 0.9) 0%, rgba(245, 245, 245, 0.9) 100%)',
        }}
      >
        <Stack direction="row" alignItems="center" spacing={2}>
          {/* Entity Icon and Name */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flex: 1 }}>
            <Box
              sx={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                width: 48,
                height: 48,
                borderRadius: 2,
                bgcolor: `${getEntityColor()}.main`,
                color: 'white',
              }}
            >
              {getEntityIcon()}
            </Box>
            <Box>
              <Typography variant="h5" fontWeight="bold">
                {entityName}
              </Typography>
              <Typography variant="caption" color="text.secondary">
                {fourWords}
              </Typography>
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
      </Paper>

      {/* Error Alert */}
      {error && (
        <Alert severity="error" onClose={() => setError(null)} sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}

      {/* Content Tabs */}
      <Paper sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
        <Tabs
          value={activeTab}
          onChange={handleTabChange}
          indicatorColor="primary"
          textColor="primary"
          sx={{ borderBottom: 1, borderColor: 'divider' }}
        >
          <Tab
            icon={<MessagesIcon />}
            label="Messages"
            iconPosition="start"
            data-testid="messages-tab"
          />
          <Tab
            icon={<StorageIcon />}
            label="Storage"
            iconPosition="start"
            data-testid="storage-tab"
          />
          <Tab
            icon={<WebsiteIcon />}
            label="Website"
            iconPosition="start"
            data-testid="website-tab"
          />
        </Tabs>

        {/* Tab Panels */}
        <Box sx={{ flex: 1, overflow: 'hidden' }}>
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
      </Paper>
    </Box>
  );
};

export default EntityContentView;