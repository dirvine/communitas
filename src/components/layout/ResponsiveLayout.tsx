import React, { useState, useEffect } from 'react';
import {
  Box,
  Drawer,
  useTheme,
  useMediaQuery,
  IconButton,
  AppBar,
  Toolbar,
  Typography,
  Fab,
  Zoom,
  useScrollTrigger,
} from '@mui/material';
import {
  Menu as MenuIcon,
  Close as CloseIcon,
  KeyboardArrowUp as KeyboardArrowUpIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';

// Breakpoint utilities
export const useBreakpoints = () => {
  const theme = useTheme();
  return {
    isMobile: useMediaQuery(theme.breakpoints.down('sm')),
    isTablet: useMediaQuery(theme.breakpoints.between('sm', 'md')),
    isDesktop: useMediaQuery(theme.breakpoints.up('md')),
    isLargeScreen: useMediaQuery(theme.breakpoints.up('lg')),
    isExtraLarge: useMediaQuery(theme.breakpoints.up('xl')),
  };
};

// Responsive drawer widths
const DRAWER_WIDTHS = {
  mobile: 280,
  tablet: 300,
  desktop: 320,
} as const;

// Scroll to top button component
const ScrollToTop: React.FC = () => {
  const trigger = useScrollTrigger({
    target: window,
    disableHysteresis: true,
    threshold: 100,
  });

  const handleClick = () => {
    window.scrollTo({
      top: 0,
      behavior: 'smooth',
    });
  };

  return (
    <Zoom in={trigger}>
      <Box
        onClick={handleClick}
        role="presentation"
        sx={{ position: 'fixed', bottom: 16, right: 16, zIndex: 1000 }}
      >
        <Fab color="primary" size="small" aria-label="scroll back to top">
          <KeyboardArrowUpIcon />
        </Fab>
      </Box>
    </Zoom>
  );
};

// Responsive layout props
interface ResponsiveLayoutProps {
  children: React.ReactNode;
  sidebar?: React.ReactNode;
  header?: React.ReactNode;
  footer?: React.ReactNode;
  showScrollToTop?: boolean;
  sidebarOpen?: boolean;
  onSidebarToggle?: () => void;
  maxWidth?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | false;
}

export const ResponsiveLayout: React.FC<ResponsiveLayoutProps> = ({
  children,
  sidebar,
  header,
  footer,
  showScrollToTop = true,
  sidebarOpen = false,
  onSidebarToggle,
  maxWidth = 'xl',
}) => {
  const theme = useTheme();
  const { isMobile, isTablet, isDesktop } = useBreakpoints();
  const [mobileDrawerOpen, setMobileDrawerOpen] = useState(false);

  // Get responsive drawer width
  const getDrawerWidth = () => {
    if (isMobile) return DRAWER_WIDTHS.mobile;
    if (isTablet) return DRAWER_WIDTHS.tablet;
    return DRAWER_WIDTHS.desktop;
  };

  const drawerWidth = getDrawerWidth();

  // Handle sidebar toggle
  const handleDrawerToggle = () => {
    if (isMobile) {
      setMobileDrawerOpen(!mobileDrawerOpen);
    } else if (onSidebarToggle) {
      onSidebarToggle();
    }
  };

  // Close mobile drawer on route change or resize
  useEffect(() => {
    if (!isMobile && mobileDrawerOpen) {
      setMobileDrawerOpen(false);
    }
  }, [isMobile, mobileDrawerOpen]);

  // Responsive drawer component
  const DrawerContent = sidebar ? (
    <Box
      sx={{
        width: drawerWidth,
        height: '100%',
        overflow: 'auto',
        borderRight: `1px solid ${theme.palette.divider}`,
        background: theme.palette.background.paper,
      }}
    >
      {/* Mobile drawer header */}
      {isMobile && (
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: theme.spacing(1, 2),
            borderBottom: `1px solid ${theme.palette.divider}`,
          }}
        >
          <Typography variant="h6" fontWeight={600}>
            Navigation
          </Typography>
          <IconButton onClick={() => setMobileDrawerOpen(false)} size="small">
            <CloseIcon />
          </IconButton>
        </Box>
      )}
      {sidebar}
    </Box>
  ) : null;

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      {/* Header */}
      {header && (
        <AppBar
          position="sticky"
          sx={{
            zIndex: theme.zIndex.drawer + 1,
            background: theme.gradients?.background,
          }}
        >
          {React.cloneElement(header as React.ReactElement, {
            onMenuClick: handleDrawerToggle,
            showMenuButton: Boolean(sidebar),
          })}
        </AppBar>
      )}

      <Box sx={{ display: 'flex', flex: 1 }}>
        {/* Sidebar - Desktop permanent drawer */}
        {sidebar && !isMobile && (
          <Drawer
            variant="persistent"
            anchor="left"
            open={sidebarOpen}
            sx={{
              width: sidebarOpen ? drawerWidth : 0,
              flexShrink: 0,
              transition: theme.transitions.create('width', {
                easing: theme.transitions.easing.sharp,
                duration: theme.transitions.duration.enteringScreen,
              }),
              '& .MuiDrawer-paper': {
                width: drawerWidth,
                boxSizing: 'border-box',
                position: 'relative',
                height: 'auto',
              },
            }}
          >
            {DrawerContent}
          </Drawer>
        )}

        {/* Sidebar - Mobile temporary drawer */}
        {sidebar && isMobile && (
          <Drawer
            variant="temporary"
            anchor="left"
            open={mobileDrawerOpen}
            onClose={() => setMobileDrawerOpen(false)}
            ModalProps={{
              keepMounted: true, // Better mobile performance
            }}
            sx={{
              '& .MuiDrawer-paper': {
                width: drawerWidth,
                boxSizing: 'border-box',
              },
            }}
          >
            {DrawerContent}
          </Drawer>
        )}

        {/* Main content */}
        <Box
          component="main"
          sx={{
            flexGrow: 1,
            display: 'flex',
            flexDirection: 'column',
            minHeight: 0, // Allows flex items to shrink below content size
            transition: theme.transitions.create('margin', {
              easing: theme.transitions.easing.sharp,
              duration: theme.transitions.duration.leavingScreen,
            }),
            marginLeft: sidebar && !isMobile && sidebarOpen ? 0 : sidebar && !isMobile ? `-${drawerWidth}px` : 0,
          }}
        >
          <Box
            sx={{
              flex: 1,
              overflow: 'auto',
              maxWidth: maxWidth ? theme.breakpoints.values[maxWidth] : 'none',
              width: '100%',
              mx: maxWidth ? 'auto' : 0,
              px: {
                xs: 1,
                sm: 2,
                md: 3,
              },
              py: {
                xs: 1,
                sm: 2,
              },
            }}
          >
            <AnimatePresence mode="wait">
              <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
                transition={{ duration: 0.2 }}
                style={{ height: '100%' }}
              >
                {children}
              </motion.div>
            </AnimatePresence>
          </Box>
        </Box>
      </Box>

      {/* Footer */}
      {footer && (
        <Box
          component="footer"
          sx={{
            mt: 'auto',
            borderTop: `1px solid ${theme.palette.divider}`,
            backgroundColor: theme.palette.background.paper,
          }}
        >
          {footer}
        </Box>
      )}

      {/* Scroll to top button */}
      {showScrollToTop && <ScrollToTop />}
    </Box>
  );
};

// Responsive container component
interface ResponsiveContainerProps {
  children: React.ReactNode;
  maxWidth?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | false;
  disableGutters?: boolean;
  className?: string;
}

export const ResponsiveContainer: React.FC<ResponsiveContainerProps> = ({
  children,
  maxWidth = 'lg',
  disableGutters = false,
  className,
}) => {
  const theme = useTheme();
  const { isMobile } = useBreakpoints();

  return (
    <Box
      className={className}
      sx={{
        width: '100%',
        maxWidth: maxWidth ? theme.breakpoints.values[maxWidth] : 'none',
        mx: 'auto',
        px: disableGutters ? 0 : {
          xs: 2,
          sm: 3,
          md: 4,
        },
        py: disableGutters ? 0 : {
          xs: 1,
          sm: 2,
        },
      }}
    >
      {children}
    </Box>
  );
};

// Responsive grid component
interface ResponsiveGridProps {
  children: React.ReactNode;
  spacing?: number;
  columns?: {
    xs?: number;
    sm?: number;
    md?: number;
    lg?: number;
    xl?: number;
  };
}

export const ResponsiveGrid: React.FC<ResponsiveGridProps> = ({
  children,
  spacing = 2,
  columns = { xs: 1, sm: 2, md: 3, lg: 4 },
}) => {
  const theme = useTheme();

  return (
    <Box
      sx={{
        display: 'grid',
        gap: theme.spacing(spacing),
        gridTemplateColumns: {
          xs: `repeat(${columns.xs || 1}, 1fr)`,
          sm: `repeat(${columns.sm || 2}, 1fr)`,
          md: `repeat(${columns.md || 3}, 1fr)`,
          lg: `repeat(${columns.lg || 4}, 1fr)`,
          xl: `repeat(${columns.xl || columns.lg || 4}, 1fr)`,
        },
      }}
    >
      {children}
    </Box>
  );
};

export default ResponsiveLayout;