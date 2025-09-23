import React, { useCallback, useEffect, useState } from 'react';
import {
  Dialog,
  DialogProps,
  DialogTitle,
  DialogContent,
  DialogActions,
  IconButton,
  Box,
  useTheme,
  alpha,
  SxProps,
  Theme,
} from '@mui/material';
import {
  Close as CloseIcon,
  ArrowBack as ArrowBackIcon,
} from '@mui/icons-material';
import { motion, PanInfo } from 'framer-motion';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing } from '../../hooks/useResponsive';

export interface TouchModalProps extends Omit<DialogProps, 'onClose' | 'onBackdropClick'> {
  /** Modal title */
  title?: string;
  /** Modal subtitle */
  subtitle?: string;
  /** Modal content */
  children?: React.ReactNode;
  /** Show close button */
  showCloseButton?: boolean;
  /** Show back button (for nested modals) */
  showBackButton?: boolean;
  /** Enable swipe down to close */
  enableSwipeDown?: boolean;
  /** Swipe threshold for closing */
  swipeThreshold?: number;
  /** Enable haptic feedback */
  hapticFeedback?: boolean;
  /** Custom haptic duration */
  hapticDuration?: number;
  /** Enable backdrop dismiss */
  enableBackdropDismiss?: boolean;
  /** Enable escape key dismiss */
  enableEscapeKey?: boolean;
  /** Animation variant */
  animationVariant?: 'slide' | 'fade' | 'scale' | 'none';
  /** Animation duration */
  animationDuration?: number;
  /** Touch-friendly sizing */
  touchSizing?: boolean;
  /** Full screen on mobile */
  fullScreenOnMobile?: boolean;
  /** Custom header actions */
  headerActions?: React.ReactNode;
  /** Custom footer actions */
  footerActions?: React.ReactNode;
  /** Close handler */
  onClose?: () => void;
  /** Back handler */
  onBack?: () => void;
  /** Swipe down handler */
  onSwipeDown?: (event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => void;
  /** Custom sx styles */
  sx?: SxProps<Theme>;
  /** Custom backdrop sx styles */
  backdropSx?: SxProps<Theme>;
  /** Custom paper sx styles */
  paperSx?: SxProps<Theme>;
}

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};



// Touch-friendly modal component
export const TouchModal: React.FC<TouchModalProps> = ({
  title,
  subtitle,
  children,
  showCloseButton = true,
  showBackButton = false,
  enableSwipeDown = true,
  swipeThreshold = 100,
  hapticFeedback = false,
  hapticDuration = 50,
  enableBackdropDismiss = true,
  enableEscapeKey = true,
  animationVariant = 'slide',
  animationDuration = 300,
  touchSizing = true,
  fullScreenOnMobile = true,
  headerActions,
  footerActions,
  onClose,
  onBack,
  onSwipeDown,
  sx,
  backdropSx,
  paperSx,
  ...props
}) => {
  const theme = useTheme();
  const isTouch = useTouchDevice();
  const touchSizingValues = useTouchFriendlySizing();
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState(0);
  const [canClose, setCanClose] = useState(false);

  // Handle escape key
  useEffect(() => {
    if (!enableEscapeKey) return;

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && onClose) {
        if (hapticFeedback && isTouch) {
          triggerHapticFeedback(hapticDuration);
        }
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [enableEscapeKey, onClose, hapticFeedback, isTouch, hapticDuration]);

  // Enhanced close handler with haptic feedback
  const handleClose = useCallback(() => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onClose?.();
  }, [onClose, hapticFeedback, isTouch, hapticDuration]);

  // Enhanced back handler with haptic feedback
  const handleBack = useCallback(() => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onBack?.();
  }, [onBack, hapticFeedback, isTouch, hapticDuration]);

  // Swipe gesture handlers
  const handleDragStart = useCallback(() => {
    setIsDragging(true);
  }, []);

  const handleDrag = useCallback((_event: any, info: PanInfo) => {
    if (!enableSwipeDown) return;

    const { offset } = info;
    setDragOffset(offset.y);

    // Show visual feedback when dragging down
    if (offset.y > 50) {
      setCanClose(true);
    } else {
      setCanClose(false);
    }
  }, [enableSwipeDown]);

  const handleDragEnd = useCallback((event: any, info: PanInfo) => {
    if (!enableSwipeDown) return;

    const { offset, velocity } = info;
    const absOffset = Math.abs(offset.y);
    const absVelocity = Math.abs(velocity.y);

    setIsDragging(false);
    setDragOffset(0);

    // Check if swipe meets threshold
    if (absOffset > swipeThreshold || absVelocity > 500) {
      if (offset.y > 0) {
        // Swipe down to close
        onSwipeDown?.(event, info);
        handleClose();
      }
    }

    setCanClose(false);
  }, [enableSwipeDown, swipeThreshold, onSwipeDown, handleClose]);

  // Touch-friendly styles
  const modalStyles: SxProps<Theme> = {
    // Touch-friendly sizing
    '& .MuiDialog-paper': {
      margin: isTouch ? theme.spacing(2) : theme.spacing(3),
      maxHeight: 'calc(100% - 32px)',
      maxWidth: isTouch ? '100%' : '90vw',
      width: isTouch ? '100%' : 'auto',
      borderRadius: isTouch ? theme.spacing(2) : theme.spacing(1),
      // Touch-friendly minimum size
      minHeight: isTouch ? 200 : 'auto',
      minWidth: isTouch ? 280 : 'auto',
      // Enhanced touch feedback
      transition: `all ${theme.transitions.duration.standard}ms ${theme.transitions.easing.easeInOut}`,
    },
    // Touch-friendly backdrop
    '& .MuiBackdrop-root': {
      backgroundColor: alpha(theme.palette.common.black, isTouch ? 0.6 : 0.5),
    },
    ...sx,
  };

  const backdropStyles: SxProps<Theme> = {
    // Enhanced backdrop for touch
    backgroundColor: alpha(theme.palette.common.black, isTouch ? 0.6 : 0.5),
    backdropFilter: isTouch ? 'blur(4px)' : 'blur(2px)',
    ...backdropSx,
  };

  const paperStyles: SxProps<Theme> = {
    // Touch-friendly paper styles
    borderRadius: isTouch ? theme.spacing(2) : theme.spacing(1),
    overflow: 'hidden',
    // Drag indicator for swipe gestures
    ...(enableSwipeDown && isTouch && {
      '&::before': {
        content: '""',
        position: 'absolute',
        top: 8,
        left: '50%',
        transform: 'translateX(-50%)',
        width: 32,
        height: 4,
        backgroundColor: alpha(theme.palette.text.secondary, 0.3),
        borderRadius: 2,
        zIndex: 1,
      },
    }),
    ...paperSx,
  };

  return (
    <Dialog
      {...props}
      onClose={enableBackdropDismiss ? handleClose : undefined}
      sx={modalStyles}
      PaperProps={{
        sx: paperStyles,
        component: motion.div,
        drag: enableSwipeDown && isTouch ? 'y' : false,
        dragConstraints: { top: 0, bottom: 0 },
        dragElastic: 0.1,
        onDragStart: handleDragStart,
        onDrag: handleDrag,
        onDragEnd: handleDragEnd,
        animate: {
          y: isDragging ? dragOffset : 0,
          opacity: canClose ? 0.9 : 1,
        },
        transition: { duration: 0.1 },
      }}
      BackdropProps={{
        sx: backdropStyles,
        component: motion.div,
      }}
      fullScreen={fullScreenOnMobile && isTouch}
    >
      {/* Header */}
      {(title || subtitle || showCloseButton || showBackButton || headerActions) && (
        <DialogTitle
          sx={{
            padding: touchSizing ? theme.spacing(2, 2, 1, 2) : theme.spacing(1),
            display: 'flex',
            alignItems: 'center',
            gap: 1,
            minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
            // Touch-friendly typography
            '& .MuiDialogTitle-root': {
              fontSize: isTouch ? '1.25rem' : '1.125rem',
              fontWeight: 600,
            },
          }}
        >
          {showBackButton && (
            <IconButton
              onClick={handleBack}
              sx={{
                minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                marginRight: 1,
              }}
            >
              <ArrowBackIcon />
            </IconButton>
          )}

          <Box sx={{ flex: 1 }}>
            {title && (
              <Box component="h2" sx={{ margin: 0, fontSize: 'inherit', fontWeight: 'inherit' }}>
                {title}
              </Box>
            )}
            {subtitle && (
              <Box
                component="p"
                sx={{
                  margin: 0,
                  marginTop: 0.5,
                  fontSize: isTouch ? '0.875rem' : '0.75rem',
                  color: 'text.secondary',
                }}
              >
                {subtitle}
              </Box>
            )}
          </Box>

          {headerActions}

          {showCloseButton && (
            <IconButton
              onClick={handleClose}
              sx={{
                minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
              }}
            >
              <CloseIcon />
            </IconButton>
          )}
        </DialogTitle>
      )}

      {/* Content */}
      <DialogContent
        sx={{
          padding: touchSizing ? theme.spacing(2) : theme.spacing(1),
          paddingTop: touchSizing ? theme.spacing(1) : theme.spacing(1),
          // Touch-friendly scrolling
          overflowY: 'auto',
          WebkitOverflowScrolling: 'touch',
          // Touch-friendly content spacing
          '& > * + *': {
            marginTop: touchSizing ? theme.spacing(2) : theme.spacing(1),
          },
        }}
      >
        {children}
      </DialogContent>

      {/* Footer */}
      {footerActions && (
        <DialogActions
          sx={{
            padding: touchSizing ? theme.spacing(2) : theme.spacing(1),
            gap: 1,
            justifyContent: 'flex-end',
            // Touch-friendly button spacing
            '& .MuiButton-root': {
              minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
              minWidth: isTouch ? 80 : 'auto',
            },
          }}
        >
          {footerActions}
        </DialogActions>
      )}
    </Dialog>
  );
};

export default TouchModal;