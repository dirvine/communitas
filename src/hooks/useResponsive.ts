import { useTheme, useMediaQuery } from '@mui/material';
import { useState, useEffect } from 'react';

// Breakpoint names type
export type Breakpoint = 'xs' | 'sm' | 'md' | 'lg' | 'xl';

// Responsive hook with detailed breakpoint information
export const useResponsive = () => {
  const theme = useTheme();
  
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const isTablet = useMediaQuery(theme.breakpoints.between('sm', 'md'));
  const isDesktop = useMediaQuery(theme.breakpoints.up('md'));
  const isLargeScreen = useMediaQuery(theme.breakpoints.up('lg'));
  const isExtraLarge = useMediaQuery(theme.breakpoints.up('xl'));

  // Current breakpoint determination
  const getCurrentBreakpoint = (): Breakpoint => {
    if (isExtraLarge) return 'xl';
    if (isLargeScreen) return 'lg';
    if (isDesktop) return 'md';
    if (isTablet) return 'sm';
    return 'xs';
  };

  const currentBreakpoint = getCurrentBreakpoint();

  // Screen size categories
  const screenSize = {
    isMobile: isMobile,
    isTablet: isTablet,
    isDesktop: isDesktop,
    isLargeScreen: isLargeScreen,
    isExtraLarge: isExtraLarge,
    currentBreakpoint,
    // Convenience helpers
    isMobileOrTablet: isMobile || isTablet,
    isDesktopOrLarger: isDesktop,
    isLargeScreenOrBigger: isLargeScreen,
  };

  return screenSize;
};

// Hook for responsive values based on current breakpoint
export const useResponsiveValue = <T>(values: Partial<Record<Breakpoint, T>>, fallback?: T): T => {
  const { currentBreakpoint } = useResponsive();
  
  // Priority order: current breakpoint, then fallback to smaller breakpoints
  const breakpointOrder: Breakpoint[] = ['xl', 'lg', 'md', 'sm', 'xs'];
  const currentIndex = breakpointOrder.indexOf(currentBreakpoint);
  
  // Look for value starting from current breakpoint going down
  for (let i = currentIndex; i < breakpointOrder.length; i++) {
    const bp = breakpointOrder[i];
    if (values[bp] !== undefined) {
      return values[bp]!;
    }
  }
  
  // If no value found, return fallback or first available value
  if (fallback !== undefined) {
    return fallback;
  }
  
  const firstValue = Object.values(values).find(v => v !== undefined);
  return firstValue as T;
};

// Hook for window dimensions
export const useWindowSize = () => {
  const [windowSize, setWindowSize] = useState({
    width: typeof window !== 'undefined' ? window.innerWidth : 0,
    height: typeof window !== 'undefined' ? window.innerHeight : 0,
  });

  useEffect(() => {
    const handleResize = () => {
      setWindowSize({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return windowSize;
};

// Hook for touch device detection
export const useTouchDevice = () => {
  const [isTouch, setIsTouch] = useState(false);

  useEffect(() => {
    const checkTouch = () => {
      setIsTouch('ontouchstart' in window || navigator.maxTouchPoints > 0);
    };

    checkTouch();
    // Re-check on resize in case device orientation changes
    window.addEventListener('resize', checkTouch);
    return () => window.removeEventListener('resize', checkTouch);
  }, []);

  return isTouch;
};

// Hook for orientation detection
export const useOrientation = () => {
  const { width, height } = useWindowSize();
  
  return {
    isLandscape: width > height,
    isPortrait: height > width,
    aspectRatio: width / height,
  };
};

// Hook for responsive spacing based on screen size
export const useResponsiveSpacing = () => {
  const { isMobile, isTablet, isDesktop } = useResponsive();
  
  return {
    // Container padding
    containerPadding: {
      xs: 1,
      sm: 2,
      md: 3,
      lg: 4,
    },
    // Section spacing
    sectionSpacing: {
      xs: 2,
      sm: 3,
      md: 4,
      lg: 6,
    },
    // Component spacing
    componentSpacing: {
      xs: 1,
      sm: 1.5,
      md: 2,
      lg: 2.5,
    },
    // Grid gaps
    gridGap: {
      xs: 1,
      sm: 2,
      md: 3,
      lg: 4,
    },
    // Current values
    current: {
      containerPadding: isMobile ? 1 : isTablet ? 2 : isDesktop ? 3 : 4,
      sectionSpacing: isMobile ? 2 : isTablet ? 3 : isDesktop ? 4 : 6,
      componentSpacing: isMobile ? 1 : isTablet ? 1.5 : isDesktop ? 2 : 2.5,
      gridGap: isMobile ? 1 : isTablet ? 2 : isDesktop ? 3 : 4,
    },
  };
};

// Hook for responsive typography scaling
export const useResponsiveTypography = () => {
  const { currentBreakpoint } = useResponsive();
  
  const getScale = (baseSize: number): number => {
    const scales = {
      xs: 0.875, // 14/16
      sm: 0.9375, // 15/16
      md: 1, // 16/16
      lg: 1.0625, // 17/16
      xl: 1.125, // 18/16
    };
    
    return baseSize * scales[currentBreakpoint];
  };
  
  return {
    getScale,
    scaledSizes: {
      xs: getScale(12),
      sm: getScale(14),
      base: getScale(16),
      lg: getScale(18),
      xl: getScale(20),
      '2xl': getScale(24),
      '3xl': getScale(30),
      '4xl': getScale(36),
    },
  };
};

// Hook for responsive columns in grid layouts
export const useResponsiveColumns = (
  config: Partial<Record<Breakpoint, number>> = { xs: 1, sm: 2, md: 3, lg: 4 }
) => {
  const { currentBreakpoint } = useResponsive();
  
  return useResponsiveValue(config, 1);
};

// Hook for sidebar behavior on different screen sizes
export const useSidebarBehavior = () => {
  const { isMobile, isTablet, isDesktop } = useResponsive();
  
  return {
    // Default sidebar state based on screen size
    defaultOpen: isDesktop,
    // Sidebar variant based on screen size
    variant: isMobile ? 'temporary' : 'persistent',
    // Whether sidebar should overlay content
    overlay: isMobile,
    // Recommended sidebar width
    width: isMobile ? 280 : isTablet ? 300 : 320,
  };
};

// Export all hooks as a combined object for convenience
export const ResponsiveHooks = {
  useResponsive,
  useResponsiveValue,
  useWindowSize,
  useTouchDevice,
  useOrientation,
  useResponsiveSpacing,
  useResponsiveTypography,
  useResponsiveColumns,
  useSidebarBehavior,
};