import { alpha } from '@mui/material/styles'

export const UnifiedDesignTokens = {
  // Glassmorphism effects
  glass: {
    light: {
      background: 'rgba(255, 255, 255, 0.85)',
      border: 'rgba(255, 255, 255, 0.2)',
      shadow: 'rgba(31, 38, 135, 0.37)',
    },
    dark: {
      background: 'rgba(26, 26, 26, 0.85)',
      border: 'rgba(255, 255, 255, 0.1)',
      shadow: 'rgba(0, 0, 0, 0.5)',
    },
  },

  // Blur levels
  blur: {
    none: 0,
    sm: 10,
    md: 20,
    lg: 30,
    xl: 40,
  },

  // Border radius
  radius: {
    xs: 4,
    sm: 8,
    md: 12,
    lg: 16,
    xl: 24,
    round: 9999,
  },

  // Transitions
  transition: {
    fast: '150ms ease',
    normal: '300ms ease',
    slow: '500ms ease',
    spring: 'cubic-bezier(0.4, 0, 0.2, 1)',
  },

  // Z-index layers
  zIndex: {
    base: 0,
    card: 10,
    navigation: 100,
    modal: 1000,
    tooltip: 1100,
    notification: 1200,
  },

  // Spacing
  spacing: {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
  },

  // Typography
  typography: {
    fontFamily: {
      sans: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      mono: '"JetBrains Mono", "SF Mono", Monaco, monospace',
    },
    fontSize: {
      xs: 12,
      sm: 14,
      md: 16,
      lg: 18,
      xl: 20,
      '2xl': 24,
      '3xl': 30,
    },
    fontWeight: {
      light: 300,
      normal: 400,
      medium: 500,
      semibold: 600,
      bold: 700,
    },
  },

  // Shadows
  shadows: {
    sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    md: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
    lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
    xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1)',
    '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
    glass: '0 8px 32px 0 rgba(31, 38, 135, 0.37)',
    glow: '0 0 40px rgba(147, 51, 234, 0.3)',
  },

  // Gradients
  gradients: {
    primary: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    secondary: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
    accent: 'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)',
    success: 'linear-gradient(135deg, #00b09b 0%, #96c93d 100%)',
    warning: 'linear-gradient(135deg, #fcb69f 0%, #ff9a00 100%)',
    error: 'linear-gradient(135deg, #ff758c 0%, #ff7eb3 100%)',
    dark: 'linear-gradient(135deg, #2d3436 0%, #000000 100%)',
    light: 'linear-gradient(135deg, #ffffff 0%, #f5f5f5 100%)',
  },

  // Breakpoints
  breakpoints: {
    xs: 0,
    sm: 600,
    md: 960,
    lg: 1280,
    xl: 1920,
  },

  // Animation durations
  duration: {
    instant: 0,
    fast: 150,
    normal: 300,
    slow: 500,
    slower: 750,
    slowest: 1000,
  },

  // Presence colors
  presence: {
    online: '#4caf50',
    away: '#ff9800',
    busy: '#f44336',
    offline: '#9e9e9e',
  },

  // Organization colors
  organization: {
    personal: 'transparent',
    org: '#FFD700',
    project: '#C0C0C0',
  },
}

// Helper functions
export const getGlassEffect = (theme: 'light' | 'dark' = 'light', opacity = 0.85) => ({
  background: theme === 'light'
    ? alpha('#ffffff', opacity)
    : alpha('#1a1a1a', opacity),
  backdropFilter: `blur(${UnifiedDesignTokens.blur.md}px)`,
  WebkitBackdropFilter: `blur(${UnifiedDesignTokens.blur.md}px)`,
  border: `1px solid ${theme === 'light'
    ? UnifiedDesignTokens.glass.light.border
    : UnifiedDesignTokens.glass.dark.border}`,
  boxShadow: UnifiedDesignTokens.shadows.glass,
})

export const getHoverEffect = () => ({
  transform: 'translateY(-4px)',
  backdropFilter: `blur(${UnifiedDesignTokens.blur.lg}px)`,
  WebkitBackdropFilter: `blur(${UnifiedDesignTokens.blur.lg}px)`,
  boxShadow: '0 12px 40px rgba(31, 38, 135, 0.5)',
  transition: UnifiedDesignTokens.transition.normal,
})

export const getPressEffect = () => ({
  transform: 'scale(0.98)',
  transition: UnifiedDesignTokens.transition.fast,
})