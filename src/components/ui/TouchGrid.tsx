import React, { useCallback } from 'react';
import {
  Grid,
  GridProps,
  useTheme,
  alpha,
  SxProps,
  Theme,
} from '@mui/material';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing, useResponsiveColumns } from '../../hooks/useResponsive';

export interface TouchGridProps extends Omit<GridProps, 'columns' | 'spacing'> {
  /** Enable touch-friendly spacing */
  enableTouchSpacing?: boolean;
  /** Enable haptic feedback on item tap */
  hapticFeedback?: boolean;
  /** Custom haptic duration */
  hapticDuration?: number;
  /** Grid items */
  items?: React.ReactNode[];
  /** Item click handler */
  onItemClick?: (item: React.ReactNode, index: number) => void;
  /** Item tap handler */
  onItemTap?: (item: React.ReactNode, index: number) => void;
  /** Item long press handler */
  onItemLongPress?: (item: React.ReactNode, index: number) => void;
  /** Enable item animations */
  enableAnimations?: boolean;
  /** Animation duration */
  animationDuration?: number;
  /** Touch-friendly item sizing */
  touchSizing?: boolean;
  /** Custom item styles */
  itemSx?: SxProps<Theme>;
  /** Responsive columns configuration */
  responsiveColumns?: {
    xs?: number;
    sm?: number;
    md?: number;
    lg?: number;
    xl?: number;
  };
  /** Custom spacing values */
  spacing?: {
    xs?: number;
    sm?: number;
    md?: number;
    lg?: number;
    xl?: number;
  };
}

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};

// Touch-friendly grid component
export const TouchGrid: React.FC<TouchGridProps> = ({
  enableTouchSpacing = true,
  hapticFeedback = false,
  hapticDuration = 50,
  items = [],
  onItemClick,
  onItemTap,
  onItemLongPress,
  enableAnimations = true,
  animationDuration = 300,
  touchSizing = true,
  itemSx,
  responsiveColumns,
  spacing = { xs: 2, sm: 3, md: 4 },
  sx,
  children,
  ...props
}) => {
  const theme = useTheme();
  const isTouch = useTouchDevice();
  const touchSizingValues = useTouchFriendlySizing();
  const columns = useResponsiveColumns(responsiveColumns);

  // Enhanced item click handler with haptic feedback
  const handleItemClick = useCallback((item: React.ReactNode, index: number) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onItemClick?.(item, index);
  }, [onItemClick, hapticFeedback, isTouch, hapticDuration]);

  // Enhanced item tap handler with haptic feedback
  const handleItemTap = useCallback((item: React.ReactNode, index: number) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onItemTap?.(item, index);
  }, [onItemTap, hapticFeedback, isTouch, hapticDuration]);

  // Enhanced item long press handler with haptic feedback
  const handleItemLongPress = useCallback((item: React.ReactNode, index: number) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration * 2); // Longer vibration for long press
    }
    onItemLongPress?.(item, index);
  }, [onItemLongPress, hapticFeedback, isTouch, hapticDuration]);

  // Touch-friendly styles
  const gridStyles = {
    // Touch-friendly spacing
    gap: enableTouchSpacing ? 2 : 1,
    // Touch-friendly grid items
    '& > *': {
      minHeight: isTouch ? 44 : 'auto',
      minWidth: isTouch ? 44 : 'auto',
      // Enhanced touch feedback
      transition: 'all 0.2s ease-in-out',
      cursor: 'pointer',
      // Touch-friendly hover effects
      '&:hover': {
        transform: !isTouch ? 'translateY(-2px)' : 'none',
        boxShadow: !isTouch ? 4 : 2,
      },
      '&:active': {
        transform: isTouch ? 'scale(0.98)' : 'none',
        backgroundColor: 'action.selected',
      },
      '&:focus-visible': {
        outline: `2px solid`,
        outlineColor: 'primary.main',
        outlineOffset: 2,
      },
      // Animation support
      ...(enableAnimations && {
        animation: `fadeInUp ${animationDuration}ms ease-out`,
      }),
      ...itemSx,
    },
    ...sx,
  };

  return (
    <div
      style={{
        display: 'grid',
        gap: enableTouchSpacing ? '16px' : '8px',
        gridTemplateColumns: `repeat(${columns}, 1fr)`,
      }}
    >
      {children || items.map((item, index) => (
        <div
          key={index}
          onClick={() => handleItemClick(item, index)}
          onTouchStart={() => {
            // Handle touch start for potential long press
            const timer = setTimeout(() => {
              handleItemLongPress(item, index);
            }, 500);

            const handleTouchEnd = () => {
              clearTimeout(timer);
              handleItemTap(item, index);
            };

            document.addEventListener('touchend', handleTouchEnd, { once: true });
          }}
          style={{
            minHeight: isTouch ? '44px' : 'auto',
            minWidth: isTouch ? '44px' : 'auto',
            display: 'flex',
            flexDirection: 'column',
            cursor: 'pointer',
            userSelect: 'none',
            WebkitTapHighlightColor: 'transparent',
            transition: 'all 0.2s ease-in-out',
          }}
        >
          {item}
        </div>
      ))}

      {/* Animation keyframes */}
      <style>{`
        @keyframes fadeInUp {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
      `}</style>
    </div>
  );
};

export default TouchGrid;