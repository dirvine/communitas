import React from 'react';
import { Box, Chip, Tooltip, IconButton } from '@mui/material';
import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import WarningIcon from '@mui/icons-material/Warning';
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined';

export type SecurityStatus = 'verified' | 'tofu' | 'invalid' | 'unsigned';

interface PQCLockIndicatorProps {
  status: SecurityStatus;
  onClick?: () => void;
  showLabel?: boolean;
}

/**
 * PQC Lock Indicator Component
 * 
 * Displays the post-quantum cryptographic signature status:
 * - Verified (green): Signature valid, key trusted
 * - TOFU (orange): First-time key (Trust On First Use)
 * - Invalid (red): Signature verification failed
 * - Unsigned (gray): No signature present
 */
export const PQCLockIndicator: React.FC<PQCLockIndicatorProps> = ({
  status,
  onClick,
  showLabel = true,
}) => {
  const config = React.useMemo(() => {
    switch (status) {
      case 'verified':
        return {
          color: 'success' as const,
          icon: <LockIcon fontSize="small" />,
          label: 'Verified',
          tooltip: 'ML-DSA signature verified and key trusted',
        };
      case 'tofu':
        return {
          color: 'warning' as const,
          icon: <LockOpenIcon fontSize="small" />,
          label: 'First Time',
          tooltip: 'First-time key - Trust On First Use (TOFU)',
        };
      case 'invalid':
        return {
          color: 'error' as const,
          icon: <WarningIcon fontSize="small" />,
          label: 'Invalid',
          tooltip: 'Signature verification failed - DO NOT TRUST',
        };
      case 'unsigned':
        return {
          color: 'default' as const,
          icon: <InfoOutlinedIcon fontSize="small" />,
          label: 'Unsigned',
          tooltip: 'No signature present',
        };
    }
  }, [status]);

  if (!showLabel && onClick) {
    // Icon button mode (compact)
    return (
      <Tooltip title={config.tooltip}>
        <IconButton size="small" onClick={onClick} color={config.color}>
          {config.icon}
        </IconButton>
      </Tooltip>
    );
  }

  if (!showLabel) {
    // Icon only mode (even more compact)
    return (
      <Tooltip title={config.tooltip}>
        <Box sx={{ display: 'inline-flex', alignItems: 'center', color: `${config.color}.main` }}>
          {config.icon}
        </Box>
      </Tooltip>
    );
  }

  // Chip mode (default)
  return (
    <Tooltip title={config.tooltip}>
      <Chip
        icon={config.icon}
        label={config.label}
        color={config.color}
        size="small"
        onClick={onClick}
        clickable={Boolean(onClick)}
        sx={{ fontWeight: 600 }}
      />
    </Tooltip>
  );
};
