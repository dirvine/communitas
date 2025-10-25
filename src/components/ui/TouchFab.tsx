import {
    alpha, Box, Fab,
    FabProps, SxProps,
    Theme, useTheme
} from '@mui/material';
import React, { useCallback } from 'react';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing } from '../../hooks/useResponsive';

export interface TouchFabProps extends Omit<FabProps, 'size'> {
  /** Enable haptic feedback on touch devices */
  hapticFeedback?: boolean;
  /** Custom haptic duration in milliseconds */
  hapticDuration?: number;
  /** Enable ripple effect */
  enableRipple?: boolean;
  /** Custom touch target size override */
  touchTargetSize?: number;
  /** Animation variant for interactions */
  animationVariant?: 'tap' | 'scale' | 'none';
  /** Custom animation duration */
  animationDuration?: number;
  /** Size variant optimized for touch */
  size?: 'small' | 'medium' | 'large' | 'touch';
  /** Show extended label */
  showLabel?: boolean;
  /** Extended label text */
  label?: string;
  /** Label position */
  labelPosition?: 'left' | 'right' | 'top' | 'bottom';
  /** Enable shadow animation */
  enableShadowAnimation?: boolean;
  /** Custom shadow color */
  shadowColor?: string;
}

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};

// Animation variants
const fabVariants = {
  tap: {
    scale: 0.9,
    transition: { duration: 0.1 }
  },
  scale: {
    scale: 1.1,
    transition: { duration: 0.2 }
  },
  none: {}
};

// Touch-friendly FAB component
export const TouchFab: React.FC<TouchFabProps> = ({
  hapticFeedback = false,
  hapticDuration = 50,
  enableRipple = true,
  touchTargetSize,
  animationVariant = 'tap',
  animationDuration = 200,
  size = 'medium',
  showLabel = false,
  label,
  labelPosition = 'right',
  enableShadowAnimation = true,
  shadowColor,
  children,
  onClick,
  sx,
  ...props
}) => {
  const theme = useTheme();
  const isTouch = useTouchDevice();
  const touchSizing = useTouchFriendlySizing();

  // Determine FAB size
  const getFabSize = () => {
    if (size === 'touch') return 'large';
    if (size === 'large') return 'large';
    if (size === 'medium') return 'medium';
    return 'small';
  };

  // Determine minimum touch target size
  const minTouchSize = touchTargetSize || touchSizing.minTouchTarget;

  // Enhanced click handler with haptic feedback
  const handleClick = useCallback((event: React.MouseEvent<HTMLButtonElement>) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onClick?.(event);
  }, [onClick, hapticFeedback, isTouch, hapticDuration]);

  // Touch-friendly styles
  const fabStyles: SxProps<Theme> = {
    // Touch-friendly sizing
    minHeight: isTouch ? minTouchSize : 'auto',
    minWidth: isTouch ? minTouchSize : 'auto',
    // Enhanced touch feedback
    transition: `all ${theme.transitions.duration.standard}ms ${theme.transitions.easing.easeInOut}`,
    // Touch-friendly positioning
    position: 'relative',
    // Better touch targets
    '&:active': {
      transform: isTouch ? 'scale(0.9)' : 'none',
    },
    '&:focus-visible': {
      outline: `2px solid ${theme.palette.primary.main}`,
      outlineOffset: 2,
    },
    // Shadow animation
    ...(enableShadowAnimation && {
      boxShadow: isTouch
        ? `0 4px 12px ${shadowColor || alpha(theme.palette.primary.main, 0.3)}`
        : theme.shadows[6],
      '&:active': {
        boxShadow: isTouch
          ? `0 2px 6px ${shadowColor || alpha(theme.palette.primary.main, 0.2)}`
          : theme.shadows[3],
      },
    }),
    ...sx,
  };

  // Label positioning styles
  const labelStyles: SxProps<Theme> = {
    position: 'absolute',
    backgroundColor: theme.palette.background.paper,
    color: theme.palette.text.primary,
    padding: theme.spacing(0.5, 1),
    borderRadius: theme.spacing(1),
    fontSize: '0.75rem',
    fontWeight: 500,
    whiteSpace: 'nowrap',
    boxShadow: theme.shadows[2],
    border: `1px solid ${theme.palette.divider}`,
    zIndex: 1,
    ...(labelPosition === 'left' && {
      right: minTouchSize + 8,
      top: '50%',
      transform: 'translateY(-50%)',
    }),
    ...(labelPosition === 'right' && {
      left: minTouchSize + 8,
      top: '50%',
      transform: 'translateY(-50%)',
    }),
    ...(labelPosition === 'top' && {
      bottom: minTouchSize + 8,
      left: '50%',
      transform: 'translateX(-50%)',
    }),
    ...(labelPosition === 'bottom' && {
      top: minTouchSize + 8,
      left: '50%',
      transform: 'translateX(-50%)',
    }),
  };

  return (
    <Box sx={{ position: 'relative', display: 'inline-block' }}>
      <Fab
        {...props}
        size={getFabSize()}
        onClick={handleClick}
        disableRipple={!enableRipple}
        sx={fabStyles}
      >
        {children}
      </Fab>

      {showLabel && label && (
        <Box
          component="span"
          sx={labelStyles}
        >
          {label}
        </Box>
      )}
    </Box>
  );
};

export default TouchFab;