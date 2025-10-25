// Copyright (c) 2025 Saorsa Labs Limited
//
// Update Settings Component
// Settings panel for auto-update configuration
// Design: Matches STORYBOARD.md dashboard card component

import {
    CheckCircleOutline, DownloadOutlined, InfoOutlined, UpdateOutlined
} from '@mui/icons-material';
import {
    Alert, Box, Button, Chip, Divider, FormControlLabel, MenuItem, Select, SelectChangeEvent, Stack,
    Switch, Typography
} from '@mui/material';
import React, { useEffect, useState } from 'react';
import { VersionDisplay } from '../VersionDisplay';

// STORYBOARD.md Design Tokens
const TOKENS = {
  bgPrimary: '#161C20',
  bgSecondary: '#1a1f24',
  bgTertiary: '#101518',
  borderColor: '#2a3038',
  borderSubtle: '#1F262C',
  accentGreen: '#2EB67D',
  accentGreenDark: '#26A86B',
  accentBlue: '#1E88E5',
  accentYellow: '#F5B759',
  textPrimary: '#F4F6F8',
  textSecondary: '#9AA2AB',
  textTertiary: '#6B7280',
};

export interface UpdateSettingsProps {
  /**
   * Current auto-update enabled state
   */
  autoUpdateEnabled: boolean;

  /**
   * Update check frequency in hours
   */
  checkFrequency: number;

  /**
   * Update channel (stable/beta)
   */
  updateChannel: 'stable' | 'beta';

  /**
   * Last check timestamp
   */
  lastChecked?: Date;

  /**
   * Is currently checking for updates
   */
  isChecking?: boolean;

  /**
   * Latest available version
   */
  latestVersion?: string;

  /**
   * Update available
   */
  updateAvailable?: boolean;

  /**
   * Callbacks
   */
  onAutoUpdateChange: (enabled: boolean) => void;
  onCheckFrequencyChange: (hours: number) => void;
  onUpdateChannelChange: (channel: 'stable' | 'beta') => void;
  onCheckNow: () => void;
  onInstallUpdate?: () => void;
}

/**
 * Formats timestamp to relative time string
 */
const formatRelativeTime = (date: Date): string => {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);

  if (seconds < 60) return 'Just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
};

/**
 * UpdateSettings Component
 *
 * Comprehensive settings panel for update management following STORYBOARD.md
 * dashboard card design with proper spacing, colors, and interactive states.
 *
 * @example
 * ```tsx
 * <UpdateSettings
 *   autoUpdateEnabled={true}
 *   checkFrequency={24}
 *   updateChannel="stable"
 *   lastChecked={new Date()}
 *   updateAvailable={true}
 *   latestVersion="0.2.0"
 *   onAutoUpdateChange={(enabled) => console.log('Auto-update:', enabled)}
 *   onCheckFrequencyChange={(hours) => console.log('Frequency:', hours)}
 *   onUpdateChannelChange={(channel) => console.log('Channel:', channel)}
 *   onCheckNow={() => console.log('Checking...')}
 *   onInstallUpdate={() => console.log('Installing...')}
 * />
 * ```
 */
export const UpdateSettings: React.FC<UpdateSettingsProps> = ({
  autoUpdateEnabled,
  checkFrequency,
  updateChannel,
  lastChecked,
  isChecking = false,
  latestVersion,
  updateAvailable = false,
  onAutoUpdateChange,
  onCheckFrequencyChange,
  onUpdateChannelChange,
  onCheckNow,
  onInstallUpdate,
}) => {
  const [localAutoUpdate, setLocalAutoUpdate] = useState(autoUpdateEnabled);
  const [localFrequency, setLocalFrequency] = useState(checkFrequency);
  const [localChannel, setLocalChannel] = useState(updateChannel);

  useEffect(() => {
    setLocalAutoUpdate(autoUpdateEnabled);
  }, [autoUpdateEnabled]);

  useEffect(() => {
    setLocalFrequency(checkFrequency);
  }, [checkFrequency]);

  useEffect(() => {
    setLocalChannel(updateChannel);
  }, [updateChannel]);

  const handleAutoUpdateToggle = (event: React.ChangeEvent<HTMLInputElement>) => {
    const enabled = event.target.checked;
    setLocalAutoUpdate(enabled);
    onAutoUpdateChange(enabled);
  };

  const handleFrequencyChange = (event: SelectChangeEvent<number>) => {
    const hours = Number(event.target.value);
    setLocalFrequency(hours);
    onCheckFrequencyChange(hours);
  };

  const handleChannelChange = (event: SelectChangeEvent<string>) => {
    const channel = event.target.value as 'stable' | 'beta';
    setLocalChannel(channel);
    onUpdateChannelChange(channel);
  };

  return (
    <Box
      sx={{
        bgcolor: TOKENS.bgSecondary,
        border: `1px solid ${TOKENS.borderColor}`,
        borderRadius: 2,
        p: 2,
        transition: 'border-color 0.3s ease',
        '&:hover': {
          borderColor: `${TOKENS.accentGreen}40`,
        },
      }}
    >
      {/* Header */}
      <Stack direction="row" alignItems="center" spacing={1.5} mb={2}>
        <Box
          sx={{
            width: 40,
            height: 40,
            borderRadius: '50%',
            background: `linear-gradient(135deg, ${TOKENS.accentGreen}, ${TOKENS.accentBlue})`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'white',
          }}
        >
          <UpdateOutlined />
        </Box>
        <Box flex={1}>
          <Typography variant="h6" fontWeight={600} sx={{ color: TOKENS.textPrimary, fontSize: 16 }}>
            Software Updates
          </Typography>
          <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
            Manage automatic updates and version channels
          </Typography>
        </Box>
        <VersionDisplay variant="body2" color={TOKENS.textSecondary} />
      </Stack>

      {/* Update Available Alert */}
      {updateAvailable && latestVersion && (
        <Alert
          severity="info"
          icon={<DownloadOutlined />}
          sx={{
            mb: 2,
            bgcolor: `${TOKENS.accentGreen}10`,
            border: `1px solid ${TOKENS.accentGreen}`,
            color: TOKENS.textPrimary,
            '& .MuiAlert-icon': {
              color: TOKENS.accentGreen,
            },
          }}
          action={
            onInstallUpdate && (
              <Button
                size="small"
                onClick={onInstallUpdate}
                sx={{
                  color: TOKENS.accentGreen,
                  fontWeight: 600,
                  '&:hover': {
                    bgcolor: `${TOKENS.accentGreen}20`,
                  },
                }}
              >
                Install
              </Button>
            )
          }
        >
          <Typography variant="body2" fontWeight={600}>
            Version {latestVersion} is available
          </Typography>
          <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
            A new update is ready to install
          </Typography>
        </Alert>
      )}

      {/* Auto-Update Toggle */}
      <Stack spacing={2}>
        <FormControlLabel
          control={
            <Switch
              checked={localAutoUpdate}
              onChange={handleAutoUpdateToggle}
              sx={{
                '& .MuiSwitch-switchBase.Mui-checked': {
                  color: TOKENS.accentGreen,
                },
                '& .MuiSwitch-switchBase.Mui-checked + .MuiSwitch-track': {
                  bgcolor: TOKENS.accentGreen,
                },
              }}
            />
          }
          label={
            <Box>
              <Typography variant="body2" fontWeight={500} sx={{ color: TOKENS.textPrimary }}>
                Automatic Updates
              </Typography>
              <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
                Download and install updates automatically
              </Typography>
            </Box>
          }
          sx={{ m: 0, alignItems: 'flex-start', gap: 1.5 }}
        />

        <Divider sx={{ borderColor: TOKENS.borderSubtle }} />

        {/* Check Frequency */}
        <Box>
          <Typography
            variant="body2"
            fontWeight={500}
            sx={{ color: TOKENS.textPrimary, mb: 1 }}
          >
            Check Frequency
          </Typography>
          <Select
            value={localFrequency}
            onChange={handleFrequencyChange}
            disabled={!localAutoUpdate}
            size="small"
            fullWidth
            sx={{
              bgcolor: TOKENS.bgTertiary,
              color: TOKENS.textPrimary,
              '& .MuiOutlinedInput-notchedOutline': {
                borderColor: TOKENS.borderColor,
              },
              '&:hover .MuiOutlinedInput-notchedOutline': {
                borderColor: TOKENS.accentGreen,
              },
              '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
                borderColor: TOKENS.accentGreen,
              },
              '& .MuiSelect-icon': {
                color: TOKENS.textSecondary,
              },
            }}
            MenuProps={{
              PaperProps: {
                sx: {
                  bgcolor: TOKENS.bgSecondary,
                  border: `1px solid ${TOKENS.borderColor}`,
                  '& .MuiMenuItem-root': {
                    color: TOKENS.textPrimary,
                    '&:hover': {
                      bgcolor: `${TOKENS.accentGreen}10`,
                    },
                    '&.Mui-selected': {
                      bgcolor: `${TOKENS.accentGreen}20`,
                      '&:hover': {
                        bgcolor: `${TOKENS.accentGreen}30`,
                      },
                    },
                  },
                },
              },
            }}
          >
            <MenuItem value={1}>Every hour</MenuItem>
            <MenuItem value={6}>Every 6 hours</MenuItem>
            <MenuItem value={12}>Every 12 hours</MenuItem>
            <MenuItem value={24}>Daily</MenuItem>
            <MenuItem value={168}>Weekly</MenuItem>
          </Select>
        </Box>

        <Divider sx={{ borderColor: TOKENS.borderSubtle }} />

        {/* Update Channel */}
        <Box>
          <Stack direction="row" alignItems="center" spacing={1} mb={1}>
            <Typography
              variant="body2"
              fontWeight={500}
              sx={{ color: TOKENS.textPrimary }}
            >
              Update Channel
            </Typography>
            {localChannel === 'beta' && (
              <Chip
                label="Beta"
                size="small"
                sx={{
                  bgcolor: `${TOKENS.accentYellow}20`,
                  color: TOKENS.accentYellow,
                  fontSize: 10,
                  height: 20,
                }}
              />
            )}
          </Stack>
          <Select
            value={localChannel}
            onChange={handleChannelChange}
            size="small"
            fullWidth
            sx={{
              bgcolor: TOKENS.bgTertiary,
              color: TOKENS.textPrimary,
              '& .MuiOutlinedInput-notchedOutline': {
                borderColor: TOKENS.borderColor,
              },
              '&:hover .MuiOutlinedInput-notchedOutline': {
                borderColor: TOKENS.accentGreen,
              },
              '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
                borderColor: TOKENS.accentGreen,
              },
              '& .MuiSelect-icon': {
                color: TOKENS.textSecondary,
              },
            }}
            MenuProps={{
              PaperProps: {
                sx: {
                  bgcolor: TOKENS.bgSecondary,
                  border: `1px solid ${TOKENS.borderColor}`,
                  '& .MuiMenuItem-root': {
                    color: TOKENS.textPrimary,
                    '&:hover': {
                      bgcolor: `${TOKENS.accentGreen}10`,
                    },
                    '&.Mui-selected': {
                      bgcolor: `${TOKENS.accentGreen}20`,
                      '&:hover': {
                        bgcolor: `${TOKENS.accentGreen}30`,
                      },
                    },
                  },
                },
              },
            }}
          >
            <MenuItem value="stable">
              <Stack>
                <Typography variant="body2">Stable</Typography>
                <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
                  Recommended for most users
                </Typography>
              </Stack>
            </MenuItem>
            <MenuItem value="beta">
              <Stack>
                <Typography variant="body2">Beta</Typography>
                <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
                  Early access to new features
                </Typography>
              </Stack>
            </MenuItem>
          </Select>
          <Box
            sx={{
              mt: 1,
              p: 1,
              bgcolor: `${TOKENS.accentBlue}10`,
              borderRadius: 1,
              display: 'flex',
              gap: 1,
            }}
          >
            <InfoOutlined sx={{ fontSize: 16, color: TOKENS.accentBlue, mt: 0.25 }} />
            <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
              Beta releases may contain experimental features and are less stable
            </Typography>
          </Box>
        </Box>

        <Divider sx={{ borderColor: TOKENS.borderSubtle }} />

        {/* Manual Check */}
        <Box>
          <Typography
            variant="body2"
            fontWeight={500}
            sx={{ color: TOKENS.textPrimary, mb: 1 }}
          >
            Manual Check
          </Typography>
          <Stack direction="row" justifyContent="space-between" alignItems="center">
            <Typography variant="caption" sx={{ color: TOKENS.textSecondary }}>
              {lastChecked ? (
                <>Last checked {formatRelativeTime(lastChecked)}</>
              ) : (
                'Never checked'
              )}
            </Typography>
            <Button
              variant="contained"
              size="small"
              onClick={onCheckNow}
              disabled={isChecking}
              startIcon={isChecking ? <UpdateOutlined sx={{ animation: 'spin 1s linear infinite' }} /> : <CheckCircleOutline />}
              sx={{
                bgcolor: TOKENS.accentGreen,
                color: TOKENS.bgTertiary,
                fontWeight: 600,
                textTransform: 'none',
                px: 2,
                '&:hover': {
                  bgcolor: TOKENS.accentGreenDark,
                  transform: 'translateY(-1px)',
                },
                '&.Mui-disabled': {
                  bgcolor: TOKENS.borderColor,
                  color: TOKENS.textTertiary,
                },
              }}
            >
              {isChecking ? 'Checking...' : 'Check Now'}
            </Button>
          </Stack>
        </Box>
      </Stack>

      {/* Spin animation for checking icon */}
      <style>
        {`
          @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
          }
        `}
      </style>
    </Box>
  );
};

export default UpdateSettings;
