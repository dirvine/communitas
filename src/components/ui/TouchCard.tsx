import {
    Favorite as FavoriteIcon, MoreVert as MoreVertIcon, Share as ShareIcon
} from '@mui/icons-material';
import {
    alpha, Box, Card, CardActions, CardContent, CardHeader,
    CardMedia, CardProps, IconButton, SxProps,
    Theme, useTheme
} from '@mui/material';
import { motion, PanInfo } from 'framer-motion';
import React, { useCallback, useState } from 'react';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing } from '../../hooks/useResponsive';

export interface TouchCardProps extends Omit<CardProps, 'onClick' | 'onDrag' | 'onDragEnd' | 'onDragStart' | 'onDragOver' | 'onDragEnter' | 'onDragLeave' | 'onDrop'> {
  /** Card title */
  title?: string;
  /** Card subtitle */
  subtitle?: string;
  /** Card content */
  children?: React.ReactNode;
  /** Card media (image/video) */
  media?: string;
  /** Media height */
  mediaHeight?: number | string;
  /** Enable swipe gestures */
  enableSwipe?: boolean;
  /** Swipe threshold for actions */
  swipeThreshold?: number;
  /** Actions to show on swipe */
  swipeActions?: {
    left?: React.ReactNode;
    right?: React.ReactNode;
  };
  /** Enable haptic feedback */
  hapticFeedback?: boolean;
  /** Custom haptic duration */
  hapticDuration?: number;
  /** Enable tap animations */
  enableAnimations?: boolean;
  /** Animation duration */
  animationDuration?: number;
  /** Touch-friendly padding */
  touchPadding?: boolean;
  /** Show favorite button */
  showFavorite?: boolean;
  /** Show share button */
  showShare?: boolean;
  /** Show more options button */
  showMoreOptions?: boolean;
  /** Custom action buttons */
  actions?: React.ReactNode;
  /** Click handler */
  onClick?: (event: React.MouseEvent<HTMLDivElement>) => void;
  /** Swipe handlers */
  onSwipeLeft?: (event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => void;
  onSwipeRight?: (event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => void;
  /** Action handlers */
  onFavorite?: () => void;
  onShare?: () => void;
  onMoreOptions?: () => void;
  /** Custom sx styles */
  sx?: SxProps<Theme>;
}

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};

// Touch-friendly card component
export const TouchCard: React.FC<TouchCardProps> = ({
  title,
  subtitle,
  children,
  media,
  mediaHeight = 200,
  enableSwipe = false,
  swipeThreshold = 100,
  swipeActions,
  hapticFeedback = false,
  hapticDuration = 50,
  enableAnimations = true,
  animationDuration = 300,
  touchPadding = true,
  showFavorite = false,
  showShare = false,
  showMoreOptions = false,
  actions,
  onClick,
  onSwipeLeft,
  onSwipeRight,
  onFavorite,
  onShare,
  onMoreOptions,
  sx,
  ...props
}) => {
  const theme = useTheme();
  const isTouch = useTouchDevice();
  const touchSizing = useTouchFriendlySizing();
  const [isSwiped, setIsSwiped] = useState(false);
  const [swipeDirection, setSwipeDirection] = useState<'left' | 'right' | null>(null);

  // Enhanced click handler with haptic feedback
  const handleClick = useCallback((event: React.MouseEvent<HTMLDivElement>) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onClick?.(event);
  }, [onClick, hapticFeedback, isTouch, hapticDuration]);

  // Swipe gesture handlers
  const _handleDragEnd = useCallback((event: any, info: PanInfo) => {
    const { offset, velocity } = info;
    const absOffset = Math.abs(offset.x);
    const absVelocity = Math.abs(velocity.x);

    // Check if swipe meets threshold
    if (absOffset > swipeThreshold || absVelocity > 500) {
      if (offset.x > 0) {
        // Swipe right
        setSwipeDirection('right');
        onSwipeRight?.(event, info);
      } else {
        // Swipe left
        setSwipeDirection('left');
        onSwipeLeft?.(event, info);
      }
      setIsSwiped(true);

      // Reset swipe state after animation
      setTimeout(() => {
        setIsSwiped(false);
        setSwipeDirection(null);
      }, animationDuration);
    }
  }, [swipeThreshold, onSwipeRight, onSwipeLeft, animationDuration]);

  // Default action handlers
  const handleFavorite = useCallback(() => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onFavorite?.();
  }, [onFavorite, hapticFeedback, isTouch, hapticDuration]);

  const handleShare = useCallback(() => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onShare?.();
  }, [onShare, hapticFeedback, isTouch, hapticDuration]);

  const handleMoreOptions = useCallback(() => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onMoreOptions?.();
  }, [onMoreOptions, hapticFeedback, isTouch, hapticDuration]);

  // Touch-friendly styles
  const cardStyles: SxProps<Theme> = {
    position: 'relative',
    cursor: onClick ? 'pointer' : 'default',
    transition: `all ${theme.transitions.duration.standard}ms ${theme.transitions.easing.easeInOut}`,
    // Touch-friendly sizing
    minHeight: isTouch ? 80 : 'auto',
    // Enhanced touch feedback
    '&:active': {
      transform: isTouch ? 'scale(0.98)' : 'none',
      backgroundColor: alpha(theme.palette.action.selected, 0.1),
    },
    '&:hover': {
      transform: !isTouch ? 'translateY(-2px)' : 'none',
      boxShadow: !isTouch ? theme.shadows[4] : theme.shadows[2],
    },
    // Touch-friendly padding
    padding: touchPadding ? theme.spacing(2) : undefined,
    // Better touch targets
    '& .MuiIconButton-root': {
      minHeight: isTouch ? touchSizing.minTouchTarget : 'auto',
      minWidth: isTouch ? touchSizing.minTouchTarget : 'auto',
    },
    '& .MuiButton-root': {
      minHeight: isTouch ? touchSizing.minTouchTarget : 'auto',
    },
    ...sx,
  };

  const _MotionCard = motion(Card);

  return (
    <motion.div
      animate={{
        x: isSwiped ? (swipeDirection === 'left' ? -20 : swipeDirection === 'right' ? 20 : 0) : 0,
        rotate: isSwiped ? (swipeDirection === 'left' ? -2 : swipeDirection === 'right' ? 2 : 0) : 0,
      }}
      transition={{ duration: animationDuration / 1000 }}
      whileTap={enableAnimations ? { scale: 0.98 } : undefined}
    >
      <Card
        sx={cardStyles}
        onClick={handleClick}
        {...props}
      >
      {/* Swipe action indicators */}
      {enableSwipe && isTouch && (
        <>
          {/* Left swipe indicator */}
          <Box
            sx={{
              position: 'absolute',
              left: -60,
              top: '50%',
              transform: 'translateY(-50%)',
              opacity: swipeDirection === 'left' ? 1 : 0,
              transition: 'opacity 0.2s ease',
              zIndex: 10,
            }}
          >
            {swipeActions?.left}
          </Box>

          {/* Right swipe indicator */}
          <Box
            sx={{
              position: 'absolute',
              right: -60,
              top: '50%',
              transform: 'translateY(-50%)',
              opacity: swipeDirection === 'right' ? 1 : 0,
              transition: 'opacity 0.2s ease',
              zIndex: 10,
            }}
          >
            {swipeActions?.right}
          </Box>
        </>
      )}

      {/* Card Header */}
      {(title || subtitle || showMoreOptions) && (
        <CardHeader
          title={title}
          subheader={subtitle}
          action={
            showMoreOptions ? (
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  handleMoreOptions();
                }}
                sx={{
                  minHeight: isTouch ? touchSizing.minTouchTarget : 'auto',
                  minWidth: isTouch ? touchSizing.minTouchTarget : 'auto',
                }}
              >
                <MoreVertIcon fontSize="small" />
              </IconButton>
            ) : undefined
          }
          sx={{
            padding: touchPadding ? theme.spacing(2, 2, 1, 2) : undefined,
            '& .MuiCardHeader-title': {
              fontSize: isTouch ? '1.1rem' : '1rem',
              fontWeight: 600,
            },
            '& .MuiCardHeader-subheader': {
              fontSize: isTouch ? '0.875rem' : '0.75rem',
            },
          }}
        />
      )}

      {/* Card Media */}
      {media && (
        <CardMedia
          component="img"
          height={mediaHeight}
          image={media}
          alt={title || 'Card media'}
          sx={{
            objectFit: 'cover',
            // Touch-friendly media sizing
            height: isTouch ? mediaHeight : mediaHeight,
          }}
        />
      )}

      {/* Card Content */}
      {children && (
        <CardContent
          sx={{
            padding: touchPadding ? theme.spacing(2) : undefined,
            '&:last-child': {
              paddingBottom: touchPadding ? theme.spacing(2) : undefined,
            },
          }}
        >
          {children}
        </CardContent>
      )}

      {/* Card Actions */}
      {(showFavorite || showShare || actions) && (
        <CardActions
          sx={{
            padding: touchPadding ? theme.spacing(1, 2, 2, 2) : undefined,
            justifyContent: 'space-between',
            gap: 1,
          }}
        >
          <Box sx={{ display: 'flex', gap: 1 }}>
            {showFavorite && (
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  handleFavorite();
                }}
                sx={{
                  color: 'error.main',
                  minHeight: isTouch ? touchSizing.minTouchTarget : 'auto',
                  minWidth: isTouch ? touchSizing.minTouchTarget : 'auto',
                }}
              >
                <FavoriteIcon fontSize="small" />
              </IconButton>
            )}

            {showShare && (
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  handleShare();
                }}
                sx={{
                  color: 'primary.main',
                  minHeight: isTouch ? touchSizing.minTouchTarget : 'auto',
                  minWidth: isTouch ? touchSizing.minTouchTarget : 'auto',
                }}
              >
                <ShareIcon fontSize="small" />
              </IconButton>
            )}
          </Box>

          {actions && (
            <Box sx={{ display: 'flex', gap: 1 }}>
              {actions}
            </Box>
          )}
        </CardActions>
      )}
      </Card>
    </motion.div>
  );
};

export default TouchCard;