import React from 'react';
import {
  Box,
  CircularProgress,
  Tooltip,
  Badge,
  Chip,
  alpha,
} from '@mui/material';
import {
  Cloud,
  CloudOff,
  CloudQueue,
  CloudDone,
  Error as ErrorIcon,
  Schedule,
  SyncProblem,
  CheckCircle,
  Warning,
} from '@mui/icons-material';
import { EntitySyncStatus } from '../../types/collaboration';

interface EntitySyncIndicatorProps {
  syncStatus?: EntitySyncStatus;
  lastSyncedAt?: Date;
  syncError?: string;
  size?: 'small' | 'medium' | 'large';
  showLabel?: boolean;
  variant?: 'icon' | 'chip' | 'badge';
  onClick?: () => void;
}

export const EntitySyncIndicator: React.FC<EntitySyncIndicatorProps> = ({
  syncStatus = 'synced',
  lastSyncedAt,
  syncError,
  size = 'small',
  showLabel = false,
  variant = 'icon',
  onClick,
}) => {
  const getIcon = () => {
    switch (syncStatus) {
      case 'synced':
        return <CloudDone />;
      case 'new':
        return <CloudQueue />;
      case 'dirty':
        return <CloudQueue />;
      case 'deleted':
        return <CloudOff />;
      case 'error':
        return <SyncProblem />;
      default:
        return <Cloud />;
    }
  };

  const getColor = () => {
    switch (syncStatus) {
      case 'synced':
        return 'success';
      case 'new':
        return 'info';
      case 'dirty':
        return 'warning';
      case 'deleted':
        return 'default';
      case 'error':
        return 'error';
      default:
        return 'default';
    }
  };

  const getLabel = () => {
    switch (syncStatus) {
      case 'synced':
        return 'Synced';
      case 'new':
        return 'Pending sync';
      case 'dirty':
        return 'Modified';
      case 'deleted':
        return 'Deleted';
      case 'error':
        return 'Sync failed';
      default:
        return 'Unknown';
    }
  };

  const getTooltip = () => {
    let base = getLabel();
    if (lastSyncedAt) {
      const timeAgo = getTimeAgo(lastSyncedAt);
      base += ` • ${timeAgo}`;
    }
    if (syncError) {
      base += ` • Error: ${syncError}`;
    }
    return base;
  };

  const getTimeAgo = (date: Date): string => {
    const seconds = Math.floor((new Date().getTime() - date.getTime()) / 1000);
    if (seconds < 60) return 'just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
  };

  const iconSize = {
    small: 16,
    medium: 20,
    large: 24,
  }[size];

  if (variant === 'chip') {
    return (
      <Tooltip title={getTooltip()}>
        <Chip
          icon={getIcon()}
          label={showLabel ? getLabel() : undefined}
          color={getColor() as any}
          size={size}
          onClick={onClick}
          sx={{
            cursor: onClick ? 'pointer' : 'default',
            '& .MuiChip-icon': {
              fontSize: iconSize,
            },
          }}
        />
      </Tooltip>
    );
  }

  if (variant === 'badge') {
    const showBadge = syncStatus !== 'synced';
    return (
      <Tooltip title={getTooltip()}>
        <Badge
          variant="dot"
          color={getColor() as any}
          invisible={!showBadge}
          sx={{
            '& .MuiBadge-dot': {
              width: 8,
              height: 8,
              borderRadius: '50%',
              animation: syncStatus === 'dirty' || syncStatus === 'new'
                ? 'pulse 2s infinite'
                : 'none',
            },
            '@keyframes pulse': {
              '0%': {
                boxShadow: '0 0 0 0 rgba(255, 165, 0, 0.7)',
                opacity: 1,
              },
              '70%': {
                boxShadow: '0 0 0 10px rgba(255, 165, 0, 0)',
                opacity: 0.7,
              },
              '100%': {
                opacity: 1,
              },
            },
          }}
        >
          <Box
            onClick={onClick}
            sx={{
              cursor: onClick ? 'pointer' : 'default',
              color: `${getColor()}.main`,
              display: 'flex',
              alignItems: 'center',
              gap: 0.5,
            }}
          >
            <Box component="span" sx={{ fontSize: iconSize }}>
              {getIcon()}
            </Box>
            {showLabel && (
              <Box component="span" sx={{ fontSize: size === 'small' ? 12 : 14 }}>
                {getLabel()}
              </Box>
            )}
          </Box>
        </Badge>
      </Tooltip>
    );
  }

  // Default icon variant
  return (
    <Tooltip title={getTooltip()}>
      <Box
        onClick={onClick}
        sx={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: 0.5,
          cursor: onClick ? 'pointer' : 'default',
          color: `${getColor()}.main`,
          '&:hover': onClick ? {
            opacity: 0.8,
          } : {},
        }}
      >
        <Box
          component="span"
          sx={{
            fontSize: iconSize,
            display: 'flex',
            alignItems: 'center',
            animation: syncStatus === 'dirty' || syncStatus === 'new'
              ? 'rotate 2s linear infinite'
              : 'none',
            '@keyframes rotate': {
              '0%': { transform: 'rotate(0deg)' },
              '100%': { transform: 'rotate(360deg)' },
            },
          }}
        >
          {getIcon()}
        </Box>
        {showLabel && (
          <Box component="span" sx={{ fontSize: size === 'small' ? 12 : 14 }}>
            {getLabel()}
          </Box>
        )}
      </Box>
    </Tooltip>
  );
};

// Bulk sync status indicator for showing overall sync state
interface BulkSyncIndicatorProps {
  totalItems: number;
  syncedItems: number;
  pendingItems: number;
  errorItems: number;
  size?: 'small' | 'medium' | 'large';
  showDetails?: boolean;
}

export const BulkSyncIndicator: React.FC<BulkSyncIndicatorProps> = ({
  totalItems,
  syncedItems,
  pendingItems,
  errorItems,
  size = 'medium',
  showDetails = true,
}) => {
  const allSynced = syncedItems === totalItems;
  const hasErrors = errorItems > 0;
  const hasPending = pendingItems > 0;

  const getOverallStatus = () => {
    if (hasErrors) return 'error';
    if (hasPending) return 'warning';
    if (allSynced) return 'success';
    return 'info';
  };

  const getOverallIcon = () => {
    if (hasErrors) return <SyncProblem />;
    if (hasPending) return <CloudQueue />;
    if (allSynced) return <CheckCircle />;
    return <Cloud />;
  };

  const getTooltip = () => {
    const parts = [];
    if (syncedItems > 0) parts.push(`${syncedItems} synced`);
    if (pendingItems > 0) parts.push(`${pendingItems} pending`);
    if (errorItems > 0) parts.push(`${errorItems} failed`);
    return parts.join(' • ');
  };

  if (!showDetails) {
    return (
      <Tooltip title={getTooltip()}>
        <Box sx={{ color: `${getOverallStatus()}.main` }}>
          {getOverallIcon()}
        </Box>
      </Tooltip>
    );
  }

  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        gap: 1,
        p: 1,
        borderRadius: 1,
        bgcolor: alpha('#000', 0.03),
      }}
    >
      <Box sx={{ color: `${getOverallStatus()}.main`, display: 'flex' }}>
        {getOverallIcon()}
      </Box>
      <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
        {syncedItems > 0 && (
          <Chip
            label={syncedItems}
            size="small"
            color="success"
            variant="outlined"
            sx={{ height: 20 }}
          />
        )}
        {pendingItems > 0 && (
          <Chip
            label={pendingItems}
            size="small"
            color="warning"
            variant="outlined"
            sx={{ height: 20 }}
          />
        )}
        {errorItems > 0 && (
          <Chip
            label={errorItems}
            size="small"
            color="error"
            variant="outlined"
            sx={{ height: 20 }}
          />
        )}
      </Box>
      {hasPending && (
        <CircularProgress size={16} thickness={4} />
      )}
    </Box>
  );
};

export default EntitySyncIndicator;