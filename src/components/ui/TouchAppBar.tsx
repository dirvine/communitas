import {
    AccountCircle as AccountCircleIcon,
    ArrowBack as ArrowBackIcon,
    Home as HomeIcon, Menu as MenuIcon,
    MoreVert as MoreVertIcon, Notifications as NotificationsIcon, Search as SearchIcon, Settings as SettingsIcon
} from '@mui/icons-material';
import {
    alpha, AppBar,
    AppBarProps, Badge, Box, IconButton, Menu,
    MenuItem, SxProps,
    Theme, Toolbar, Typography, useTheme
} from '@mui/material';
import React, { useCallback, useState } from 'react';

// Enhanced responsive hooks
import { useTouchDevice, useTouchFriendlySizing } from '../../hooks/useResponsive';

export interface TouchAppBarProps extends Omit<AppBarProps, 'onMenuClick'> {
  /** App bar title */
  title?: string;
  /** Show back button */
  showBackButton?: boolean;
  /** Show menu button */
  showMenuButton?: boolean;
  /** Show search button */
  showSearchButton?: boolean;
  /** Show notifications button */
  showNotificationsButton?: boolean;
  /** Show account button */
  showAccountButton?: boolean;
  /** Show settings button */
  showSettingsButton?: boolean;
  /** Show home button */
  showHomeButton?: boolean;
  /** Enable haptic feedback */
  hapticFeedback?: boolean;
  /** Custom haptic duration */
  hapticDuration?: number;
  /** App bar position */
  position?: 'fixed' | 'absolute' | 'sticky' | 'static' | 'relative';
  /** App bar elevation */
  elevation?: number;
  /** Touch-friendly sizing */
  touchSizing?: boolean;
  /** Custom left actions */
  leftActions?: React.ReactNode;
  /** Custom right actions */
  rightActions?: React.ReactNode;
  /** Custom center content */
  centerContent?: React.ReactNode;
  /** Menu items for more options */
  menuItems?: Array<{
    id: string;
    label: string;
    icon?: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
    divider?: boolean;
  }>;
  /** Notification count */
  notificationCount?: number;
  /** Back button handler */
  onBack?: () => void;
  /** Menu button handler */
  onMenu?: () => void;
  /** Search button handler */
  onSearch?: () => void;
  /** Notifications handler */
  onNotifications?: () => void;
  /** Account handler */
  onAccount?: () => void;
  /** Settings handler */
  onSettings?: () => void;
  /** Home handler */
  onHome?: () => void;
  /** Custom sx styles */
  sx?: SxProps<Theme>;
}

// Haptic feedback utility
const triggerHapticFeedback = (duration: number = 50) => {
  if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
    navigator.vibrate(duration);
  }
};

// Touch-friendly app bar component
export const TouchAppBar: React.FC<TouchAppBarProps> = ({
  title,
  showBackButton = false,
  showMenuButton = true,
  showSearchButton = false,
  showNotificationsButton = false,
  showAccountButton = false,
  showSettingsButton = false,
  showHomeButton = false,
  hapticFeedback = false,
  hapticDuration = 50,
  position = 'fixed',
  elevation = 4,
  touchSizing = true,
  leftActions,
  rightActions,
  centerContent,
  menuItems = [],
  notificationCount = 0,
  onBack,
  onMenu,
  onSearch,
  onNotifications,
  onAccount,
  onSettings,
  onHome,
  sx,
  ...props
}) => {
  const theme = useTheme();
  const isTouch = useTouchDevice();
  const touchSizingValues = useTouchFriendlySizing();
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);

  // Enhanced button click handler with haptic feedback
  const handleButtonClick = useCallback((handler?: () => void) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    handler?.();
  }, [hapticFeedback, isTouch, hapticDuration]);

  // Menu handlers
  const handleMenuOpen = useCallback((event: React.MouseEvent<HTMLButtonElement>) => {
    setMenuAnchor(event.currentTarget);
  }, []);

  const handleMenuClose = useCallback(() => {
    setMenuAnchor(null);
  }, []);

  const handleMenuItemClick = useCallback((item: any) => {
    if (hapticFeedback && isTouch) {
      triggerHapticFeedback(hapticDuration);
    }
    item.onClick?.();
    handleMenuClose();
  }, [hapticFeedback, isTouch, hapticDuration, handleMenuClose]);

  // Touch-friendly styles
  const appBarStyles: SxProps<Theme> = {
    // Touch-friendly app bar sizing
    minHeight: isTouch ? touchSizingValues.minTouchTarget : 64,
    padding: touchSizing ? theme.spacing(1, 2) : theme.spacing(0.5, 1),
    // Enhanced touch feedback
    transition: `all ${theme.transitions.duration.standard}ms ${theme.transitions.easing.easeInOut}`,
    // Touch-friendly backdrop
    backdropFilter: 'blur(10px)',
    backgroundColor: alpha(theme.palette.background.paper, 0.9),
    borderBottom: `1px solid ${alpha(theme.palette.divider, 0.1)}`,
    ...sx,
  };

  return (
    <>
      <AppBar
        {...props}
        position={position}
        elevation={elevation}
        sx={appBarStyles}
      >
        <Toolbar
          sx={{
            minHeight: isTouch ? touchSizingValues.minTouchTarget : 64,
            padding: touchSizing ? theme.spacing(0, 1) : theme.spacing(0, 0.5),
            gap: 1,
          }}
        >
          {/* Left Actions */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {leftActions || (
              <>
                {showBackButton && (
                  <IconButton
                    onClick={() => handleButtonClick(onBack)}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                    }}
                  >
                    <ArrowBackIcon />
                  </IconButton>
                )}

                {showHomeButton && (
                  <IconButton
                    onClick={() => handleButtonClick(onHome)}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                    }}
                  >
                    <HomeIcon />
                  </IconButton>
                )}

                {showMenuButton && (
                  <IconButton
                    onClick={() => handleButtonClick(onMenu)}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                    }}
                  >
                    <MenuIcon />
                  </IconButton>
                )}
              </>
            )}
          </Box>

          {/* Center Content */}
          <Box sx={{ flex: 1, display: 'flex', justifyContent: 'center' }}>
            {centerContent || (
              title && (
                <Typography
                  variant="h6"
                  component="h1"
                  sx={{
                    fontSize: isTouch ? '1.125rem' : '1rem',
                    fontWeight: 600,
                    textAlign: 'center',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                    maxWidth: '60%',
                  }}
                >
                  {title}
                </Typography>
              )
            )}
          </Box>

          {/* Right Actions */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {rightActions || (
              <>
                {showSearchButton && (
                  <IconButton
                    onClick={() => handleButtonClick(onSearch)}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                    }}
                  >
                    <SearchIcon />
                  </IconButton>
                )}

                {showNotificationsButton && (
                  <IconButton
                    onClick={() => handleButtonClick(onNotifications)}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                    }}
                  >
                    <Badge badgeContent={notificationCount} color="error">
                      <NotificationsIcon />
                    </Badge>
                  </IconButton>
                )}

                {showAccountButton && (
                  <IconButton
                    onClick={() => handleButtonClick(onAccount)}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                    }}
                  >
                    <AccountCircleIcon />
                  </IconButton>
                )}

                {showSettingsButton && (
                  <IconButton
                    onClick={() => handleButtonClick(onSettings)}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                    }}
                  >
                    <SettingsIcon />
                  </IconButton>
                )}

                {menuItems.length > 0 && (
                  <IconButton
                    onClick={handleMenuOpen}
                    sx={{
                      minHeight: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                      minWidth: isTouch ? touchSizingValues.minTouchTarget : 'auto',
                    }}
                  >
                    <MoreVertIcon />
                  </IconButton>
                )}
              </>
            )}
          </Box>
        </Toolbar>
      </AppBar>

      {/* More Options Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={handleMenuClose}
        anchorOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
        slotProps={{
          paper: {
            sx: {
              minWidth: 200,
              marginTop: theme.spacing(1),
            },
          },
        }}
      >
        {menuItems.map((item, index) => (
          <React.Fragment key={item.id}>
            <MenuItem
              onClick={() => handleMenuItemClick(item)}
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
                <Box sx={{ marginRight: 2, display: 'flex', alignItems: 'center' }}>
                  {item.icon}
                </Box>
              )}
              <Typography
                sx={{
                  fontSize: isTouch ? '1rem' : '0.875rem',
                  fontWeight: item.disabled ? 400 : 500,
                }}
              >
                {item.label}
              </Typography>
            </MenuItem>
            {item.divider && index < menuItems.length - 1 && (
              <Box sx={{ height: 1, backgroundColor: theme.palette.divider, margin: theme.spacing(1, 0) }} />
            )}
          </React.Fragment>
        ))}
      </Menu>
    </>
  );
};

export default TouchAppBar;