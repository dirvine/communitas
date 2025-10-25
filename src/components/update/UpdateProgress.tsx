// Copyright (c) 2025 Saorsa Labs Limited
//
// Update Progress Component
// Displays download/install progress with speed and time remaining
// Design: Matches STORYBOARD.md storage meter component

import {
    CheckCircleOutline, DownloadOutlined, ErrorOutline, InstallDesktopOutlined, UpdateOutlined
} from '@mui/icons-material';
import { Box, Chip, LinearProgress, Stack, Typography } from '@mui/material';
import React from 'react';

// STORYBOARD.md Design Tokens
const TOKENS = {
  bgPrimary: '#161C20',
  bgSecondary: '#1a1f24',
  bgTertiary: '#101518',
  borderColor: '#2a3038',
  borderSubtle: '#1F262C',
  accentGreen: '#2EB67D',
  accentGreenDark: '#26A86B',
  accentYellow: '#F5B759',
  accentRed: '#E25555',
  textPrimary: '#F4F6F8',
  textSecondary: '#9AA2AB',
  textTertiary: '#6B7280',
};

export interface UpdateProgressProps {
  /**
   * Current update status
   */
  status: 'checking' | 'downloading' | 'installing' | 'completed' | 'error' | 'idle';

  /**
   * Download/install progress (0-100)
   */
  progress: number;

  /**
   * Current version being installed
   */
  version?: string;

  /**
   * Download speed (bytes per second)
   */
  downloadSpeed?: number;

  /**
   * Time remaining (seconds)
   */
  timeRemaining?: number;

  /**
   * Total file size (bytes)
   */
  totalSize?: number;

  /**
   * Downloaded bytes
   */
  downloadedBytes?: number;

  /**
   * Error message if status is 'error'
   */
  errorMessage?: string;

  /**
   * Show in compact mode
   */
  compact?: boolean;

  /**
   * Cancel download callback
   */
  onCancel?: () => void;

  /**
   * Retry on error callback
   */
  onRetry?: () => void;
}

/**
 * Formats bytes to human-readable string
 */
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${Math.round(bytes / Math.pow(k, i) * 100) / 100} ${sizes[i]}`;
};

/**
 * Formats seconds to time string (e.g., "2m 30s")
 */
const formatTime = (seconds: number): string => {
  if (seconds < 60) return `${Math.round(seconds)}s`;
  const minutes = Math.floor(seconds / 60);
  const secs = Math.round(seconds % 60);
  return `${minutes}m ${secs}s`;
};

/**
 * Gets status color based on update state
 */
const getStatusColor = (status: UpdateProgressProps['status']): string => {
  switch (status) {
    case 'checking':
    case 'downloading':
    case 'installing':
      return TOKENS.accentGreen;
    case 'completed':
      return TOKENS.accentGreen;
    case 'error':
      return TOKENS.accentRed;
    default:
      return TOKENS.textSecondary;
  }
};

/**
 * Gets status icon based on update state
 */
const getStatusIcon = (status: UpdateProgressProps['status']) => {
  switch (status) {
    case 'checking':
      return <UpdateOutlined sx={{ fontSize: 20 }} />;
    case 'downloading':
      return <DownloadOutlined sx={{ fontSize: 20 }} />;
    case 'installing':
      return <InstallDesktopOutlined sx={{ fontSize: 20 }} />;
    case 'completed':
      return <CheckCircleOutline sx={{ fontSize: 20 }} />;
    case 'error':
      return <ErrorOutline sx={{ fontSize: 20 }} />;
    default:
      return null;
  }
};

/**
 * Gets status label text
 */
const getStatusLabel = (status: UpdateProgressProps['status']): string => {
  switch (status) {
    case 'checking':
      return 'Checking for updates...';
    case 'downloading':
      return 'Downloading update...';
    case 'installing':
      return 'Installing update...';
    case 'completed':
      return 'Update completed!';
    case 'error':
      return 'Update failed';
    default:
      return 'No update in progress';
  }
};

/**
 * UpdateProgress Component
 *
 * Displays update progress following STORYBOARD.md storage meter design.
 * Shows download speed, time remaining, and progress bar.
 *
 * @example
 * ```tsx
 * <UpdateProgress
 *   status="downloading"
 *   progress={42}
 *   version="0.2.0"
 *   downloadSpeed={1024000}
 *   timeRemaining={120}
 *   totalSize={52428800}
 *   downloadedBytes={22020096}
 *   onCancel={() => console.log('Cancelled')}
 * />
 * ```
 */
export const UpdateProgress: React.FC<UpdateProgressProps> = ({
  status,
  progress,
  version,
  downloadSpeed,
  timeRemaining,
  totalSize,
  downloadedBytes,
  errorMessage,
  compact = false,
  onCancel,
  onRetry,
}) => {
  const statusColor = getStatusColor(status);
  const statusIcon = getStatusIcon(status);
  const statusLabel = getStatusLabel(status);

  if (status === 'idle') {
    return null;
  }

  return (
    <Box
      sx={{
        bgcolor: TOKENS.bgSecondary,
        border: `1px solid ${TOKENS.borderColor}`,
        borderRadius: compact ? 1 : 2,
        p: compact ? 1.5 : 2,
        transition: 'all 0.3s ease',
        '&:hover': {
          borderColor: `${statusColor}40`,
        },
      }}
    >
      {/* Header */}
      <Stack direction="row" alignItems="center" spacing={1.5} mb={compact ? 1 : 1.5}>
        <Box sx={{ color: statusColor }}>
          {statusIcon}
        </Box>
        <Box flex={1}>
          <Typography
            variant="body2"
            fontWeight={600}
            sx={{ color: TOKENS.textPrimary }}
          >
            {statusLabel}
          </Typography>
          {version && (
            <Typography
              variant="caption"
              sx={{ color: TOKENS.textSecondary, fontFamily: 'monospace' }}
            >
              Version {version}
            </Typography>
          )}
        </Box>
        {status === 'completed' && (
          <Chip
            label="Restart to apply"
            size="small"
            sx={{
              bgcolor: `${TOKENS.accentGreen}20`,
              color: TOKENS.accentGreen,
              fontSize: 11,
              height: 24,
            }}
          />
        )}
      </Stack>

      {/* Error Message */}
      {status === 'error' && errorMessage && (
        <Typography
          variant="body2"
          sx={{
            color: TOKENS.accentRed,
            mb: 1.5,
            p: 1,
            bgcolor: `${TOKENS.accentRed}10`,
            borderRadius: 1,
          }}
        >
          {errorMessage}
        </Typography>
      )}

      {/* Progress Info */}
      {(status === 'downloading' || status === 'installing') && (
        <Stack spacing={0.5} mb={1.5}>
          {/* Download Stats */}
          <Stack direction="row" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
              {downloadedBytes && totalSize
                ? `${formatBytes(downloadedBytes)} / ${formatBytes(totalSize)}`
                : 'Calculating...'}
            </Typography>
            <Typography
              variant="caption"
              fontWeight={600}
              sx={{ color: statusColor }}
            >
              {Math.round(progress)}%
            </Typography>
          </Stack>

          {/* Speed and Time */}
          {!compact && (
            <Stack direction="row" spacing={2}>
              {downloadSpeed && downloadSpeed > 0 && (
                <Typography variant="caption" sx={{ color: TOKENS.textTertiary }}>
                  ↓ {formatBytes(downloadSpeed)}/s
                </Typography>
              )}
              {timeRemaining && timeRemaining > 0 && (
                <Typography variant="caption" sx={{ color: TOKENS.textTertiary }}>
                  ⏱ {formatTime(timeRemaining)} remaining
                </Typography>
              )}
            </Stack>
          )}
        </Stack>
      )}

      {/* Progress Bar - STORYBOARD.md storage meter style */}
      {(status === 'downloading' || status === 'installing') && (
        <Box
          sx={{
            height: 8,
            bgcolor: TOKENS.borderSubtle,
            borderRadius: 1,
            overflow: 'hidden',
            position: 'relative',
          }}
        >
          <Box
            sx={{
              height: '100%',
              width: `${progress}%`,
              bgcolor: statusColor,
              transition: 'width 0.5s ease',
              position: 'relative',
              '&::after': {
                content: '""',
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                background: `linear-gradient(90deg, transparent, ${statusColor}40, transparent)`,
                animation: status === 'downloading' ? 'shimmer 2s infinite' : 'none',
              },
            }}
          />
        </Box>
      )}

      {/* Checking Progress (indeterminate) */}
      {status === 'checking' && (
        <LinearProgress
          sx={{
            height: 8,
            borderRadius: 1,
            bgcolor: TOKENS.borderSubtle,
            '& .MuiLinearProgress-bar': {
              bgcolor: statusColor,
            },
          }}
        />
      )}

      {/* Actions */}
      {!compact && (
        <Stack direction="row" spacing={1} mt={1.5} justifyContent="flex-end">
          {status === 'downloading' && onCancel && (
            <Box
              component="button"
              onClick={onCancel}
              sx={{
                px: 2,
                py: 0.75,
                bgcolor: 'transparent',
                border: `1px solid ${TOKENS.borderColor}`,
                borderRadius: 1,
                color: TOKENS.textSecondary,
                fontSize: 12,
                cursor: 'pointer',
                transition: 'all 0.2s',
                '&:hover': {
                  bgcolor: `${TOKENS.accentRed}10`,
                  borderColor: TOKENS.accentRed,
                  color: TOKENS.accentRed,
                },
              }}
            >
              Cancel
            </Box>
          )}
          {status === 'error' && onRetry && (
            <Box
              component="button"
              onClick={onRetry}
              sx={{
                px: 2,
                py: 0.75,
                bgcolor: TOKENS.accentGreen,
                border: 'none',
                borderRadius: 1,
                color: TOKENS.bgTertiary,
                fontSize: 12,
                fontWeight: 600,
                cursor: 'pointer',
                transition: 'all 0.2s',
                '&:hover': {
                  bgcolor: TOKENS.accentGreenDark,
                  transform: 'translateY(-1px)',
                },
              }}
            >
              Retry
            </Box>
          )}
        </Stack>
      )}

      {/* Shimmer animation for progress bar */}
      <style>
        {`
          @keyframes shimmer {
            0% { transform: translateX(-100%); }
            100% { transform: translateX(100%); }
          }
        `}
      </style>
    </Box>
  );
};

export default UpdateProgress;
