import {
    Button,
    ButtonProps, Fab,
    FabProps, IconButton,
    IconButtonProps, useTheme
} from '@mui/material';
import { motion } from 'framer-motion';
import React, { useCallback } from 'react';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing } from '../../hooks/useResponsive';

export interface TouchButtonProps extends Omit<ButtonProps, 'size'> {
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
}

export interface TouchIconButtonProps extends Omit<IconButtonProps, 'size'> {
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
}

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
}

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};

// Animation variants
const buttonVariants = {
  tap: {
    scale: 0.95,
    transition: { duration: 0.1 }
  },
  scale: {
    scale: 1.05,
    transition: { duration: 0.2 }
  },
  none: {}
};

// Base touch-friendly button component
const BaseTouchButton: React.FC<{
  children: React.ReactNode;
  onClick: (event: React.MouseEvent<HTMLButtonElement>) => void;
  componentProps: any;
  hapticFeedback?: boolean;
  hapticDuration?: number;
  enableRipple?: boolean;
  touchTargetSize?: number;
  animationVariant?: 'tap' | 'scale' | 'none';
  animationDuration?: number;
  size?: 'small' | 'medium' | 'large' | 'touch';
}> = ({
  children,
  onClick,
  componentProps,
  hapticFeedback = false,
  hapticDuration = 50,
  enableRipple = true,
  touchTargetSize,
  animationVariant = 'tap',
  animationDuration = 200,
  size = 'medium',
}) => {
  const theme = useTheme();
  const isTouch = useTouchDevice();
  const touchSizing = useTouchFriendlySizing();

  // Determine button size
  const getButtonSize = () => {
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
    onClick(event);
  }, [onClick, hapticFeedback, isTouch, hapticDuration]);

  // Common styles for all touch buttons
  const commonStyles = {
    minHeight: isTouch ? minTouchSize : 'auto',
    minWidth: isTouch ? minTouchSize : 'auto',
    transition: `all ${theme.transitions.duration.standard}ms ${theme.transitions.easing.easeInOut}`,
    // Enhanced touch feedback
    '&:active': {
      transform: isTouch ? 'scale(0.95)' : 'none',
    },
    '&:focus-visible': {
      outline: `2px solid ${theme.palette.primary.main}`,
      outlineOffset: 2,
    },
    // Better touch targets
    padding: isTouch ? theme.spacing(1.5, 3) : undefined,
  };

  const MotionComponent = motion.div;

  return (
    <MotionComponent
      variants={buttonVariants}
      whileTap={animationVariant !== 'none' ? animationVariant : undefined}
      transition={{ duration: animationDuration / 1000 }}
    >
      {React.cloneElement(children as React.ReactElement, {
        ...componentProps,
        onClick: handleClick,
        disableRipple: !enableRipple,
        size: getButtonSize(),
        sx: {
          ...commonStyles,
          ...((children as React.ReactElement).props?.sx || {}),
        },
      })}
    </MotionComponent>
  );
};

// TouchButton component
export const TouchButton: React.FC<TouchButtonProps> = ({
  hapticFeedback = false,
  hapticDuration = 50,
  enableRipple = true,
  touchTargetSize,
  animationVariant = 'tap',
  animationDuration = 200,
  size = 'medium',
  children,
  onClick,
  ...props
}) => {
  const buttonProps = {
    ...props,
    onClick,
  };

  return (
    <BaseTouchButton
      onClick={onClick!}
      componentProps={buttonProps}
      hapticFeedback={hapticFeedback}
      hapticDuration={hapticDuration}
      enableRipple={enableRipple}
      touchTargetSize={touchTargetSize}
      animationVariant={animationVariant}
      animationDuration={animationDuration}
      size={size}
      variant="button"
    >
      <Button {...buttonProps}>{children}</Button>
    </BaseTouchButton>
  );
};

// TouchIconButton component
export const TouchIconButton: React.FC<TouchIconButtonProps> = ({
  hapticFeedback = false,
  hapticDuration = 50,
  enableRipple = true,
  touchTargetSize,
  animationVariant = 'tap',
  animationDuration = 200,
  size = 'medium',
  children,
  onClick,
  ...props
}) => {
  const iconButtonProps = {
    ...props,
    onClick,
  };

  return (
    <BaseTouchButton
      onClick={onClick!}
      componentProps={iconButtonProps}
      hapticFeedback={hapticFeedback}
      hapticDuration={hapticDuration}
      enableRipple={enableRipple}
      touchTargetSize={touchTargetSize}
      animationVariant={animationVariant}
      animationDuration={animationDuration}
      size={size}
      variant="iconButton"
    >
      <IconButton {...iconButtonProps}>{children}</IconButton>
    </BaseTouchButton>
  );
};

// TouchFab component
export const TouchFab: React.FC<TouchFabProps> = ({
  hapticFeedback = false,
  hapticDuration = 50,
  enableRipple = true,
  touchTargetSize,
  animationVariant = 'scale',
  animationDuration = 200,
  size = 'medium',
  children,
  onClick,
  ...props
}) => {
  const fabProps = {
    ...props,
    onClick,
  };

  return (
    <BaseTouchButton
      onClick={onClick!}
      componentProps={fabProps}
      hapticFeedback={hapticFeedback}
      hapticDuration={hapticDuration}
      enableRipple={enableRipple}
      touchTargetSize={touchTargetSize}
      animationVariant={animationVariant}
      animationDuration={animationDuration}
      size={size}
      variant="fab"
    >
      <Fab {...fabProps}>{children}</Fab>
    </BaseTouchButton>
  );
};

export default TouchButton;