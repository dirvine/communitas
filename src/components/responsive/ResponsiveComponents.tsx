import React, { useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  CardActions,
  Typography,
  IconButton,
  Collapse,
  Stack,
  Chip,
  Button,
  Paper,
  Divider,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  ListItemButton,
  SwipeableDrawer,
  useTheme,
  alpha,
} from '@mui/material';
import {
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
  SwipeUp as SwipeUpIcon,
  DragHandle as DragHandleIcon,
} from '@mui/icons-material';
import { motion } from 'framer-motion';
import { useResponsive, useResponsiveSpacing, useTouchDevice } from '../../hooks/useResponsive';

// Responsive card component that adapts layout based on screen size
interface ResponsiveCardProps {
  title: string;
  subtitle?: string;
  content?: React.ReactNode;
  actions?: React.ReactNode;
  image?: string;
  avatar?: React.ReactNode;
  collapsible?: boolean;
  initialCollapsed?: boolean;
  className?: string;
}

export const ResponsiveCard: React.FC<ResponsiveCardProps> = ({
  title,
  subtitle,
  content,
  actions,
  image,
  avatar,
  collapsible = false,
  initialCollapsed = false,
  className,
}) => {
  const { isMobile } = useResponsive();
  const { current } = useResponsiveSpacing();
  const [collapsed, setCollapsed] = useState(initialCollapsed);

  const handleToggleCollapse = () => {
    setCollapsed(!collapsed);
  };

  return (
    <Card
      className={className}
      component={motion.div}
      whileHover={{ y: -2 }}
      whileTap={{ scale: 0.98 }}
      sx={{
        height: 'fit-content',
        transition: 'all 0.2s ease',
      }}
    >
      <CardContent
        sx={{
          p: current.containerPadding,
          pb: collapsible && collapsed ? current.containerPadding : 1,
        }}
      >
        {/* Header with title and optional collapse button */}
        <Stack
          direction="row"
          alignItems="flex-start"
          justifyContent="space-between"
          spacing={1}
          mb={subtitle || content ? 1 : 0}
        >
          <Box flex={1}>
            <Stack direction={isMobile ? 'column' : 'row'} spacing={1} alignItems="flex-start">
              {avatar && <Box>{avatar}</Box>}
              <Box flex={1}>
                <Typography
                  variant={isMobile ? 'h6' : 'h5'}
                  component="h3"
                  sx={{
                    fontWeight: 600,
                    lineHeight: 1.2,
                    wordBreak: 'break-word',
                  }}
                >
                  {title}
                </Typography>
                {subtitle && (
                  <Typography
                    variant="body2"
                    color="text.secondary"
                    sx={{ mt: 0.5, lineHeight: 1.4 }}
                  >
                    {subtitle}
                  </Typography>
                )}
              </Box>
            </Stack>
          </Box>

          {collapsible && (
            <IconButton
              onClick={handleToggleCollapse}
              size="small"
              sx={{ mt: -0.5 }}
              aria-label={collapsed ? 'Expand' : 'Collapse'}
            >
              {collapsed ? <ExpandMoreIcon /> : <ExpandLessIcon />}
            </IconButton>
          )}
        </Stack>

        {/* Image */}
        {image && (
          <Box
            sx={{
              width: '100%',
              height: isMobile ? 200 : 240,
              borderRadius: 1,
              overflow: 'hidden',
              mb: 2,
              backgroundImage: `url(${image})`,
              backgroundSize: 'cover',
              backgroundPosition: 'center',
            }}
          />
        )}

        {/* Collapsible content */}
        <Collapse in={!collapsed} timeout="auto" unmountOnExit>
          {content && <Box>{content}</Box>}
        </Collapse>
      </CardContent>

      {/* Actions */}
      {actions && !collapsed && (
        <CardActions
          sx={{
            px: current.containerPadding,
            pt: 0,
            pb: current.containerPadding,
            flexDirection: isMobile ? 'column' : 'row',
            gap: 1,
            '& > *': {
              flex: isMobile ? '1 1 auto' : 'none',
              width: isMobile ? '100%' : 'auto',
            },
          }}
        >
          {actions}
        </CardActions>
      )}
    </Card>
  );
};

// Mobile-friendly bottom sheet component
interface BottomSheetProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
  maxHeight?: string | number;
  showHandle?: boolean;
}

export const BottomSheet: React.FC<BottomSheetProps> = ({
  open,
  onClose,
  title,
  children,
  maxHeight = '80vh',
  showHandle = true,
}) => {
  const theme = useTheme();
  const { isMobile } = useResponsive();
  const isTouch = useTouchDevice();

  // Use SwipeableDrawer for mobile, regular drawer for desktop
  const DrawerComponent = isMobile ? SwipeableDrawer : SwipeableDrawer;

  return (
    <DrawerComponent
      anchor="bottom"
      open={open}
      onClose={onClose}
      onOpen={() => {}} // Required for SwipeableDrawer
      disableSwipeToOpen={!isTouch}
      PaperProps={{
        sx: {
          borderTopLeftRadius: 16,
          borderTopRightRadius: 16,
          maxHeight,
          background: theme.palette.background.paper,
        },
      }}
    >
      <Box
        sx={{
          width: '100%',
          maxWidth: isMobile ? 'none' : 600,
          mx: 'auto',
        }}
      >
        {/* Handle for swiping */}
        {showHandle && isTouch && (
          <Box
            sx={{
              display: 'flex',
              justifyContent: 'center',
              pt: 1.5,
              pb: 1,
            }}
          >
            <DragHandleIcon
              sx={{
                color: theme.palette.divider,
                fontSize: 32,
              }}
            />
          </Box>
        )}

        {/* Title */}
        {title && (
          <Box sx={{ px: 2, pb: 2 }}>
            <Typography variant="h6" component="h2" fontWeight={600}>
              {title}
            </Typography>
          </Box>
        )}

        {/* Content */}
        <Box sx={{ px: 2, pb: 2 }}>{children}</Box>
      </Box>
    </DrawerComponent>
  );
};

// Responsive data display component
interface ResponsiveDataDisplayProps {
  data: Array<{
    label: string;
    value: React.ReactNode;
    icon?: React.ReactNode;
    color?: string;
  }>;
  variant?: 'list' | 'grid' | 'chips';
  maxColumns?: number;
}

export const ResponsiveDataDisplay: React.FC<ResponsiveDataDisplayProps> = ({
  data,
  variant = 'list',
  maxColumns = 3,
}) => {
  const { isMobile, isTablet } = useResponsive();
  const { current } = useResponsiveSpacing();

  if (variant === 'chips') {
    return (
      <Stack
        direction="row"
        flexWrap="wrap"
        gap={1}
        sx={{ mt: 1 }}
      >
        {data.map((item, index) => (
          <Chip
            key={index}
            label={`${item.label}: ${item.value}`}
            size={isMobile ? 'small' : 'medium'}
            variant="outlined"
            icon={item.icon as React.ReactElement}
            sx={{
              ...(item.color && {
                borderColor: item.color,
                color: item.color,
              }),
            }}
          />
        ))}
      </Stack>
    );
  }

  if (variant === 'grid') {
    const columns = Math.min(maxColumns, isMobile ? 1 : isTablet ? 2 : 3);
    
    return (
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: `repeat(${columns}, 1fr)`,
          gap: current.gridGap,
          mt: 1,
        }}
      >
        {data.map((item, index) => (
          <Paper
            key={index}
            variant="outlined"
            sx={{
              p: 2,
              textAlign: 'center',
              borderLeft: item.color ? `4px solid ${item.color}` : 'none',
            }}
          >
            {item.icon && (
              <Box sx={{ mb: 1, color: item.color }}>{item.icon}</Box>
            )}
            <Typography variant="body2" color="text.secondary" gutterBottom>
              {item.label}
            </Typography>
            <Typography variant="body1" fontWeight={500}>
              {item.value}
            </Typography>
          </Paper>
        ))}
      </Box>
    );
  }

  // Default list variant
  return (
    <List dense={isMobile} sx={{ mt: 1 }}>
      {data.map((item, index) => (
        <ListItem
          key={index}
          sx={{
            px: 0,
            borderLeft: item.color ? `4px solid ${item.color}` : 'none',
            pl: item.color ? 2 : 0,
          }}
        >
          {item.icon && (
            <ListItemIcon sx={{ color: item.color, minWidth: 36 }}>
              {item.icon}
            </ListItemIcon>
          )}
          <ListItemText
            primary={item.label}
            secondary={item.value}
            primaryTypographyProps={{
              variant: isMobile ? 'body2' : 'body1',
              fontWeight: 500,
            }}
            secondaryTypographyProps={{
              variant: isMobile ? 'caption' : 'body2',
            }}
          />
        </ListItem>
      ))}
    </List>
  );
};

// Responsive action buttons component
interface ResponsiveActionButtonsProps {
  actions: Array<{
    label: string;
    icon?: React.ReactNode;
    onClick: () => void;
    variant?: 'contained' | 'outlined' | 'text';
    color?: 'primary' | 'secondary' | 'error' | 'warning' | 'info' | 'success';
    disabled?: boolean;
  }>;
  orientation?: 'horizontal' | 'vertical' | 'auto';
  spacing?: number;
}

export const ResponsiveActionButtons: React.FC<ResponsiveActionButtonsProps> = ({
  actions,
  orientation = 'auto',
  spacing = 1,
}) => {
  const { isMobile } = useResponsive();
  
  const direction = orientation === 'auto' 
    ? (isMobile ? 'column' : 'row')
    : orientation === 'vertical' ? 'column' : 'row';

  return (
    <Stack
      direction={direction}
      spacing={spacing}
      sx={{
        width: direction === 'column' ? '100%' : 'auto',
        '& > *': {
          flex: direction === 'column' ? '1 1 auto' : 'none',
          width: direction === 'column' ? '100%' : 'auto',
        },
      }}
    >
      {actions.map((action, index) => (
        <Button
          key={index}
          variant={action.variant || 'contained'}
          color={action.color || 'primary'}
          onClick={action.onClick}
          disabled={action.disabled}
          startIcon={action.icon}
          size={isMobile ? 'large' : 'medium'}
          fullWidth={direction === 'column'}
          sx={{
            minHeight: isMobile ? 48 : 36,
            justifyContent: direction === 'column' ? 'flex-start' : 'center',
          }}
        >
          {action.label}
        </Button>
      ))}
    </Stack>
  );
};

export default {
  ResponsiveCard,
  BottomSheet,
  ResponsiveDataDisplay,
  ResponsiveActionButtons,
};