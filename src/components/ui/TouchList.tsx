import React, { useCallback, useState, useRef } from 'react';
import {
  List,
  ListProps,
  ListItem,
  ListItemButton,
  ListItemText,
  ListItemIcon,
  IconButton,
  Box,
  Typography,
  useTheme,
  alpha,
  CircularProgress,
  LinearProgress,
  SxProps,
  Theme,
} from '@mui/material';
import {
  ArrowUpward as ArrowUpIcon,
  ArrowDownward as ArrowDownIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
} from '@mui/icons-material';
import { motion, PanInfo } from 'framer-motion';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing } from '../../hooks/useResponsive';

export interface TouchListItem {
  id: string;
  primary: string;
  secondary?: string;
  icon?: React.ReactNode;
  avatar?: React.ReactNode;
  actions?: React.ReactNode;
  disabled?: boolean;
  selected?: boolean;
  metadata?: Record<string, any>;
}

export interface TouchListProps extends Omit<ListProps, 'onScroll'> {
  /** Custom sx styles */
  sx?: SxProps<Theme>;
  /** List items */
  items: TouchListItem[];
  /** Enable pull-to-refresh */
  enablePullToRefresh?: boolean;
  /** Pull-to-refresh threshold */
  pullThreshold?: number;
  /** Enable swipe actions */
  enableSwipeActions?: boolean;
  /** Swipe threshold for actions */
  swipeThreshold?: number;
  /** Swipe actions for items */
  swipeActions?: {
    left?: (item: TouchListItem) => React.ReactNode;
    right?: (item: TouchListItem) => React.ReactNode;
  };
  /** Enable haptic feedback */
  hapticFeedback?: boolean;
  /** Custom haptic duration */
  hapticDuration?: number;
  /** Loading state */
  loading?: boolean;
  /** Loading more items */
  loadingMore?: boolean;
  /** Refresh callback */
  onRefresh?: () => void | Promise<void>;
  /** Load more callback */
  onLoadMore?: () => void;
  /** Item click handler */
  onItemClick?: (item: TouchListItem, index: number) => void;
  /** Item swipe handlers */
  onItemSwipeLeft?: (item: TouchListItem, event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => void;
  onItemSwipeRight?: (item: TouchListItem, event: MouseEvent | TouchEvent | PointerEvent, info: PanInfo) => void;
  /** Item action handlers */
  onItemDelete?: (item: TouchListItem) => void;
  onItemEdit?: (item: TouchListItem) => void;
  /** Custom item renderer */
  renderItem?: (item: TouchListItem, index: number) => React.ReactNode;
  /** Empty state component */
  emptyState?: React.ReactNode;
  /** Virtual scrolling */
  virtualScroll?: boolean;
  /** Item height for virtual scrolling */
  itemHeight?: number;
  /** Container height for virtual scrolling */
  containerHeight?: number;
}

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};

// Pull-to-refresh hook
const usePullToRefresh = (
  onRefresh?: () => void | Promise<void>,
  threshold: number = 80,
  enabled: boolean = true
) => {
  const [pullDistance, setPullDistance] = useState(0);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const startY = useRef<number | null>(null);

  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    if (!enabled) return;
    startY.current = e.touches[0].clientY;
  }, [enabled]);

  const handleTouchMove = useCallback((e: React.TouchEvent) => {
    if (!enabled || !startY.current || isRefreshing) return;

    const currentY = e.touches[0].clientY;
    const distance = Math.max(0, currentY - startY.current);

    if (distance > 0) {
      setPullDistance(Math.min(distance, threshold * 2));
    }
  }, [enabled, isRefreshing, threshold]);

  const handleTouchEnd = useCallback(async () => {
    if (!enabled || !startY.current || isRefreshing) return;

    if (pullDistance >= threshold) {
      setIsRefreshing(true);
      try {
        await onRefresh?.();
      } finally {
        setIsRefreshing(false);
      }
    }

    setPullDistance(0);
    startY.current = null;
  }, [enabled, isRefreshing, pullDistance, threshold, onRefresh]);

  return {
    pullDistance,
    isRefreshing,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
  };
};

// Touch-friendly list component
export const TouchList: React.FC<TouchListProps> = ({
  items,
  enablePullToRefresh = false,
  pullThreshold = 80,
  enableSwipeActions = false,
  swipeThreshold = 100,
  swipeActions,
  hapticFeedback = false,
  hapticDuration = 50,
  loading = false,
  loadingMore = false,
  onRefresh,
  onLoadMore,
  onItemClick,
  onItemSwipeLeft,
  onItemSwipeRight,
  onItemDelete,
  onItemEdit,
  renderItem,
  emptyState,
  virtualScroll = false,
  itemHeight = 72,
  containerHeight = 400,
  sx,
  ...props
}) => {
  const theme = useTheme();
  const isTouch = useTouchDevice();
  const touchSizing = useTouchFriendlySizing();
  const [swipedItems, setSwipedItems] = useState<Set<string>>(new Set());

  const {
    pullDistance,
    isRefreshing,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
  } = usePullToRefresh(onRefresh, pullThreshold, enablePullToRefresh);

  // Enhanced item click handler with haptic feedback
  const handleItemClick = useCallback((item: TouchListItem, index: number) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    onItemClick?.(item, index);
  }, [onItemClick, hapticFeedback, isTouch, hapticDuration]);

  // Default item renderer
  const defaultRenderItem = (item: TouchListItem, index: number) => (
    <motion.div
      key={item.id}
      animate={{
        x: swipedItems.has(item.id) ? (swipedItems.has(item.id) ? -20 : 20) : 0,
      }}
      transition={{ duration: 0.2 }}
    >
      <ListItem
        disablePadding
        sx={{
          position: 'relative',
          '& .item-actions': { opacity: 0 },
          '&:hover .item-actions': { opacity: 1 },
          // Touch-friendly sizing
          minHeight: isTouch ? touchSizing.minTouchTarget : 48,
          // Enhanced touch feedback
          '&:active': {
            backgroundColor: alpha(theme.palette.action.selected, 0.1),
          },
        }}
        secondaryAction={
          item.actions || (
            <Box className="item-actions" sx={{ display: 'flex', gap: 0.5 }}>
              {onItemEdit && (
                <IconButton
                  size="small"
                  onClick={(e) => {
                    e.stopPropagation();
                    if (hapticFeedback && isTouch) {
                      triggerHapticFeedback(hapticDuration);
                    }
                    onItemEdit(item);
                  }}
                  sx={{
                    minHeight: isTouch ? touchSizing.minTouchTarget : 'auto',
                    minWidth: isTouch ? touchSizing.minTouchTarget : 'auto',
                  }}
                >
                  <EditIcon fontSize="small" />
                </IconButton>
              )}
              {onItemDelete && (
                <IconButton
                  size="small"
                  onClick={(e) => {
                    e.stopPropagation();
                    if (hapticFeedback && isTouch) {
                      triggerHapticFeedback(hapticDuration);
                    }
                    onItemDelete(item);
                  }}
                  sx={{
                    color: 'error.main',
                    minHeight: isTouch ? touchSizing.minTouchTarget : 'auto',
                    minWidth: isTouch ? touchSizing.minTouchTarget : 'auto',
                  }}
                >
                  <DeleteIcon fontSize="small" />
                </IconButton>
              )}
            </Box>
          )
        }
      >
        <ListItemButton
          onClick={() => handleItemClick(item, index)}
          disabled={item.disabled}
          selected={item.selected}
          sx={{
            // Touch-friendly padding
            padding: isTouch ? theme.spacing(1.5, 2) : theme.spacing(1, 2),
            // Better touch targets
            minHeight: isTouch ? touchSizing.minTouchTarget : 48,
          }}
        >
          {item.icon && (
            <ListItemIcon sx={{ minWidth: isTouch ? 56 : 40 }}>
              {item.icon}
            </ListItemIcon>
          )}
          {item.avatar && (
            <ListItemIcon sx={{ minWidth: isTouch ? 56 : 40 }}>
              {item.avatar}
            </ListItemIcon>
          )}
          <ListItemText
            primary={
              <Typography
                variant="body1"
                fontWeight={item.selected ? 600 : 400}
                sx={{ fontSize: isTouch ? '1rem' : '0.875rem' }}
              >
                {item.primary}
              </Typography>
            }
            secondary={
              item.secondary && (
                <Typography
                  variant="body2"
                  color="text.secondary"
                  sx={{ fontSize: isTouch ? '0.875rem' : '0.75rem' }}
                >
                  {item.secondary}
                </Typography>
              )
            }
          />
        </ListItemButton>
      </ListItem>
    </motion.div>
  );

  // Touch-friendly styles
  const listStyles: SxProps<Theme> = {
    position: 'relative',
    // Touch-friendly scrolling
    WebkitOverflowScrolling: 'touch',
    // Better touch targets
    '& .MuiListItemButton-root': {
      minHeight: isTouch ? touchSizing.minTouchTarget : 48,
      padding: isTouch ? theme.spacing(1.5, 2) : theme.spacing(1, 2),
    },
    '& .MuiIconButton-root': {
      minHeight: isTouch ? touchSizing.minTouchTarget : 'auto',
      minWidth: isTouch ? touchSizing.minTouchTarget : 'auto',
    },
    ...sx,
  };

  return (
    <Box
      sx={{
        position: 'relative',
        height: virtualScroll ? containerHeight : 'auto',
        overflow: virtualScroll ? 'auto' : 'visible',
        WebkitOverflowScrolling: 'touch',
      }}
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
    >
      {/* Pull-to-refresh indicator */}
      {enablePullToRefresh && (
        <Box
          sx={{
            position: 'absolute',
            top: -pullDistance,
            left: 0,
            right: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            height: pullThreshold,
            backgroundColor: 'background.paper',
            borderBottom: `1px solid ${theme.palette.divider}`,
            transform: `translateY(${pullDistance > 0 ? -pullThreshold + pullDistance : -pullThreshold}px)`,
            transition: 'transform 0.2s ease',
            zIndex: 10,
          }}
        >
          <motion.div
            animate={{ rotate: isRefreshing ? 360 : pullDistance > pullThreshold ? 180 : 0 }}
            transition={{ duration: isRefreshing ? 1 : 0.2, repeat: isRefreshing ? Infinity : 0 }}
          >
            {isRefreshing ? (
              <CircularProgress size={24} />
            ) : pullDistance > pullThreshold ? (
              <ArrowDownIcon />
            ) : (
              <ArrowUpIcon />
            )}
          </motion.div>
          <Typography variant="body2" color="text.secondary" sx={{ ml: 1 }}>
            {isRefreshing ? 'Refreshing...' : pullDistance > pullThreshold ? 'Release to refresh' : 'Pull to refresh'}
          </Typography>
        </Box>
      )}

      {/* Loading indicator */}
      {loading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
          <CircularProgress />
        </Box>
      )}

      {/* Empty state */}
      {!loading && items.length === 0 && emptyState && (
        <Box sx={{ p: 4, textAlign: 'center' }}>
          {emptyState}
        </Box>
      )}

      {/* List content */}
      <List sx={listStyles} {...props}>
        {items.map((item, index) =>
          renderItem ? renderItem(item, index) : defaultRenderItem(item, index)
        )}
      </List>

      {/* Load more indicator */}
      {loadingMore && (
        <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
          <LinearProgress sx={{ width: '100%', maxWidth: 200 }} />
        </Box>
      )}

      {/* Infinite scroll trigger */}
      {onLoadMore && !loadingMore && (
        <Box
          sx={{ height: 1 }}
          ref={(node: HTMLDivElement | null) => {
            if (node && virtualScroll) {
              const observer = new IntersectionObserver(
                (entries) => {
                  if (entries[0].isIntersecting) {
                    onLoadMore();
                  }
                },
                { threshold: 0.1 }
              );
              observer.observe(node);
              return () => observer.disconnect();
            }
          }}
        />
      )}
    </Box>
  );
};

export default TouchList;