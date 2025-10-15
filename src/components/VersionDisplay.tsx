// Copyright (c) 2025 Saorsa Labs Limited
//
// Version Display Component
// Shows app version from Tauri backend

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Box, Typography } from '@mui/material';

interface VersionDisplayProps {
  /**
   * Show prefix "v" before version number
   * @default true
   */
  showPrefix?: boolean;
  /**
   * Typography variant
   * @default 'caption'
   */
  variant?: 'caption' | 'body2' | 'body1';
  /**
   * Text color
   * @default 'text.secondary'
   */
  color?: string;
  /**
   * Additional className for styling
   */
  className?: string;
}

/**
 * VersionDisplay Component
 *
 * Fetches and displays the application version from Tauri backend.
 *
 * @example
 * ```tsx
 * // Default usage
 * <VersionDisplay />
 *
 * // Custom styling
 * <VersionDisplay variant="body2" color="primary.main" />
 *
 * // Without "v" prefix
 * <VersionDisplay showPrefix={false} />
 * ```
 */
export function VersionDisplay({
  showPrefix = true,
  variant = 'caption',
  color = 'text.secondary',
  className
}: VersionDisplayProps) {
  const [version, setVersion] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchVersion = async () => {
      try {
        const ver = await invoke<string>('get_app_version');
        setVersion(ver);
        setError(null);
      } catch (err) {
        console.error('Failed to fetch app version:', err);
        setError('Unknown');
        setVersion('Unknown');
      } finally {
        setLoading(false);
      }
    };

    fetchVersion();
  }, []);

  if (loading) {
    return null; // Don't show anything while loading
  }

  if (error || !version) {
    return (
      <Typography variant={variant} color="text.disabled" className={className}>
        Version Unknown
      </Typography>
    );
  }

  const displayVersion = showPrefix ? `v${version}` : version;

  return (
    <Box component="span" className={className}>
      <Typography
        variant={variant}
        color={color}
        sx={{
          fontFamily: 'monospace',
          letterSpacing: 0.5
        }}
      >
        {displayVersion}
      </Typography>
    </Box>
  );
}

/**
 * VersionBadge Component
 *
 * Displays version as a styled badge/chip.
 * Useful for headers, settings pages, or about dialogs.
 */
export function VersionBadge() {
  const [version, setVersion] = useState<string>('');

  useEffect(() => {
    invoke<string>('get_app_version')
      .then(setVersion)
      .catch(err => {
        console.error('Failed to fetch app version:', err);
        setVersion('Unknown');
      });
  }, []);

  if (!version) return null;

  return (
    <Box
      sx={{
        display: 'inline-flex',
        alignItems: 'center',
        px: 1.5,
        py: 0.5,
        borderRadius: 1,
        bgcolor: 'action.hover',
        border: '1px solid',
        borderColor: 'divider',
      }}
    >
      <Typography
        variant="caption"
        sx={{
          fontFamily: 'monospace',
          fontWeight: 600,
          color: 'text.secondary',
          letterSpacing: 0.5
        }}
      >
        v{version}
      </Typography>
    </Box>
  );
}

export default VersionDisplay;
