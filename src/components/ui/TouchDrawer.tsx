import React, { useCallback, useState } from 'react';
import {
  Drawer,
  DrawerProps,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  IconButton,
  Box,
  useTheme,
  alpha,
  SxProps,
  Theme,
  Divider,
} from '@mui/material';
import {
  Close as CloseIcon,
} from '@mui/icons-material';
import { motion, PanInfo } from 'framer-motion';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing } from '../../hooks/useResponsive';

export interface TouchDrawerProps extends Omit<DrawerProps, 'onClose'> {
  /** Drawer title */
  title?: string;
  /** Drawer items */
  items?: Array<{
    id: string;
    label: string;
    icon?: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
    divider?: boolean;
  }>;
  /** Show close button */
  showCloseButton?: boolean;
  /** Enable swipe gestures */
  enableSwipe?: boolean;
  /** Swipe threshold for closing */
  swipeThreshold?: number;
  /** Enable haptic feedback */
  hapticFeedback?: boolean;
  /** Custom haptic duration */
  hapticDuration?: number;
  /** Drawer position */
  position?: 'left' | 'right' | 'top' | 'bottom';
  /** Drawer width/height */
  size?: number | string;
  /** Enable backdrop dismiss */
  enableBackdropDismiss?: boolean;
  /** Animation variant */
  animationVariant?: 'slide' | 'scale' | 'none';
  /** Animation duration */
  animationDuration?: number;
  /** Touch-friendly sizing */
  touchSizing?: boolean;
  /** Custom header content */
  headerContent?: React.ReactNode;
  /** Custom footer content */
  footerContent?: React.ReactNode;
  /** Close handler */
  onClose?: () => void;
  /** Item click handler */
  onItemClick?: (item: any) => void;
  /** Swipe handlers */
  onSwipeLeft?: (event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => void;
  onSwipeRight?: (event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => void;
  /** Custom sx styles */
  sx?: SxProps<Theme>;
  /** Custom paper sx styles */
  paperSx?: SxProps<Theme>;
}

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};

// Touch-friendly drawer component
export const TouchDrawer: React.FC<TouchDrawerProps> = ({
  title,
  items = [],
  showCloseButton = true,
  enableSwipe = true,
  swipeThreshold = 100,
  hapticFeedback = false,
  hapticDuration = 50,
  position = 'left',
  size = 280,
  enableBackdropDismiss = true,
  animationVariant = 'slide',
  animationDuration = 300,
  touchSizing = true,
  headerContent,
  footerContent,
  onClose,
  onItemClick,
  onSwipeLeft,
  onSwipeRight,
  sx,
  paperSx,
  ...props
}) => {
  const theme = useTheme();
  const isTouch = useTouchDevice();
  const touchSizingValues = useTouchFriendlySizing();
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState(0);

  // Enhanced close handler with haptic feedback
  const handleClose = useCallback(() => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onClose?.();
  }, [onClose, hapticFeedback, isTouch, hapticDuration]);

  // Enhanced item click handler with haptic feedback
  const handleItemClick = useCallback((item: any) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onItemClick?.(item);
    item.onClick?.();
  }, [onItemClick, hapticFeedback, isTouch, hapticDuration]);

  // Swipe gesture handlers
  const handleDragStart = useCallback(() => {
    setIsDragging(true);
  }, []);

  const handleDrag = useCallback((_event: any, info: PanInfo) => {
    if (!enableSwipe) return;

    const { offset } = info;
    const isHorizontal = position === 'left' || position === 'right';

    if (isHorizontal) {
      setDragOffset(offset.x);
    } else {
      setDragOffset(offset.y);
    }
  }, [enableSwipe, position]);

  const handleDragEnd = useCallback((event: any, info: PanInfo) => {
    if (!enableSwipe) return;

    const { offset, velocity } = info;
    const absOffset = Math.abs(isHorizontal ? offset.x : offset.y);
    const absVelocity = Math.abs(isHorizontal ? velocity.x : velocity.y);

    setIsDragging(false);
    setDragOffset(0);

    // Check if swipe meets threshold
    if (absOffset > swipeThreshold || absVelocity > 500) {
      if (position === 'left' && offset.x < 0) {
        // Swipe left to close left drawer
        onSwipeLeft?.(event, info);
        handleClose();
      } else if (position === 'right' && offset.x > 0) {
        // Swipe right to close right drawer
        onSwipeRight?.(event, info);
        handleClose();
      } else if (position === 'top' && offset.y < 0) {
        // Swipe up to close top drawer
        handleClose();
      } else if (position === 'bottom' && offset.y > 0) {
        // Swipe down to close bottom drawer
        handleClose();
      }
    }
  }, [enableSwipe, swipeThreshold, position, onSwipeLeft, onSwipeRight, handleClose]);

  // Determine drawer properties based on position
  const isHorizontal = position === 'left' || position === 'right';
  const isVertical = position === 'top' || position === 'bottom';

  // Touch-friendly styles
  const drawerStyles: SxProps<Theme> = {
    // Touch-friendly drawer sizing
    '& .MuiDrawer-paper': {
      width: isHorizontal ? size : '100%',
      height: isVertical ? size : '100%',
      maxWidth: isHorizontal ? size : '100%',
      maxHeight: isVertical ? size : '100%',
      // Touch-friendly padding and spacing
      padding: touchSizing ? theme.spacing(2) : theme.spacing(1),
      // Enhanced touch feedback
      transition: `all ${theme.transitions.duration.standard}ms ${theme.transitions.easing.easeInOut}`,
    },
    ...sx,
  };

  const paperStyles: SxProps<Theme> = {
    // Touch-friendly paper styles
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
    ...paperSx,
  };

  return (
    <Drawer
      {...props}
      onClose={enableBackdropDismiss ? handleClose : undefined}
      sx={drawerStyles}
      PaperProps={{
        sx: paperStyles,
        component: motion.div,
        drag: enableSwipe && isTouch ? (isHorizontal ? 'x' : 'y') : false,
        dragConstraints: { left: 0, right: 0, top: 0, bottom: 0 },
        dragElastic: 0.1,
        onDragStart: handleDragStart,
        onDrag: handleDrag,
        onDragEnd: handleDragEnd,
        animate: {
          x: isHorizontal && isDragging ? dragOffset : 0,
          y: isVertical && isDragging ? dragOffset : 0,
        },
        transition: { duration: 0.1 },
      }}
    >
      {/* Header */}
      {(title || showCloseButton || headerContent) && (
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: touchSizing ? theme.spacing(2) : theme.spacing(1),
            borderBottom: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
            minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
          }}
        >
          {headerContent || (
            <>
              {title && (
                <Box component="h3" sx={{ margin: 0, fontSize: '1.125rem', fontWeight: 600 }}>
                  {title}
                </Box>
              )}
              <Box sx={{ flex: 1 }} />
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
            </>
          )}
        </Box>
      )}

      {/* Content */}
      <Box
        sx={{
          flex: 1,
          overflow: 'auto',
          WebkitOverflowScrolling: 'touch',
        }}
      >
        {items.length > 0 ? (
          <List sx={{ padding: 0 }}>
            {items.map((item, index) => (
              <React.Fragment key={item.id}>
                <ListItem disablePadding>
                  <ListItemButton
                    onClick={() => handleItemClick(item)}
                    disabled={item.disabled}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      padding: touchSizing ? theme.spacing(2) : theme.spacing(1),
                      // Touch-friendly hover effects
                      '&:hover': {
                        backgroundColor: alpha(theme.palette.action.hover, 0.1),
                      },
                      '&:active': {
                        backgroundColor: alpha(theme.palette.action.selected, 0.2),
                      },
                    }}
                  >
                    {item.icon && (
                      <ListItemIcon
                        sx={{
                          minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                          marginRight: touchSizing ? theme.spacing(2) : theme.spacing(1),
                        }}
                      >
                        {item.icon}
                      </ListItemIcon>
                    )}
                    <ListItemText
                      primary={item.label}
                      sx={{
                        '& .MuiListItemText-primary': {
                          fontSize: isTouch ? '1rem' : '0.875rem',
                          fontWeight: item.disabled ? 400 : 500,
                        },
                      }}
                    />
                  </ListItemButton>
                </ListItem>
                {item.divider && index < items.length - 1 && (
                  <Divider sx={{ margin: touchSizing ? theme.spacing(1, 0) : 0 }} />
                )}
              </React.Fragment>
            ))}
          </List>
        ) : (
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              padding: theme.spacing(2),
              textAlign: 'center',
            }}
          >
            No items available
          </Box>
        )}
      </Box>

      {/* Footer */}
      {footerContent && (
        <Box
          sx={{
            borderTop: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
            padding: touchSizing ? theme.spacing(2) : theme.spacing(1),
          }}
        >
          {footerContent}
        </Box>
      )}
    </Drawer>
  );
};

export default TouchDrawer;