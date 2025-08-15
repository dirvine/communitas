/**
 * Unified Design System for Communitas
 * Beautiful, modern theme combining the best of WhatsApp, Slack, and Dropbox
 */

import { createTheme, alpha } from '@mui/material/styles'

// Color Palette
const colors = {
  // Primary - Deep Blue (Trust and stability)
  primary: {
    main: '#1A73E8',
    light: '#4285F4',
    dark: '#1557B0',
    contrastText: '#FFFFFF',
  },
  
  // Secondary - Vibrant Teal (Innovation and connectivity)
  secondary: {
    main: '#00ACC1',
    light: '#26C6DA',
    dark: '#00838F',
    contrastText: '#FFFFFF',
  },
  
  // Accent - Warm Orange (Human connection)
  accent: {
    main: '#FF6B35',
    light: '#FF8A65',
    dark: '#E55100',
    contrastText: '#FFFFFF',
  },
  
  // Semantic Colors
  success: {
    main: '#34A853',
    light: '#81C784',
    dark: '#2E7D32',
  },
  warning: {
    main: '#FBBC05',
    light: '#FFD54F',
    dark: '#F9A825',
  },
  error: {
    main: '#EA4335',
    light: '#EF5350',
    dark: '#C62828',
  },
  info: {
    main: '#00ACC1',
    light: '#4FC3F7',
    dark: '#0288D1',
  },
  
  // Neutral Colors
  grey: {
    50: '#FAFAFA',
    100: '#F5F5F5',
    200: '#EEEEEE',
    300: '#E0E0E0',
    400: '#BDBDBD',
    500: '#9E9E9E',
    600: '#757575',
    700: '#616161',
    800: '#424242',
    900: '#212121',
  },
  
  // Background Colors
  background: {
    default: '#FAFAFA',
    paper: '#FFFFFF',
    elevated: '#FFFFFF',
    glass: 'rgba(255, 255, 255, 0.8)',
  },
  
  // Dark Mode Colors
  dark: {
    background: {
      default: '#0A0A0A',
      paper: '#1A1A1A',
      elevated: '#2A2A2A',
      glass: 'rgba(26, 26, 26, 0.8)',
    },
    text: {
      primary: '#FFFFFF',
      secondary: 'rgba(255, 255, 255, 0.7)',
      disabled: 'rgba(255, 255, 255, 0.5)',
    },
  },
}

// Typography System
const typography = {
  fontFamily: '"Inter", "SF Pro Text", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  
  // Headlines
  h1: {
    fontFamily: '"Inter", sans-serif',
    fontWeight: 700,
    fontSize: '3rem',
    lineHeight: 1.2,
    letterSpacing: '-0.02em',
  },
  h2: {
    fontFamily: '"Inter", sans-serif',
    fontWeight: 600,
    fontSize: '2.25rem',
    lineHeight: 1.3,
    letterSpacing: '-0.01em',
  },
  h3: {
    fontFamily: '"Inter", sans-serif',
    fontWeight: 600,
    fontSize: '1.875rem',
    lineHeight: 1.4,
  },
  h4: {
    fontFamily: '"Inter", sans-serif',
    fontWeight: 600,
    fontSize: '1.5rem',
    lineHeight: 1.4,
  },
  h5: {
    fontFamily: '"Inter", sans-serif',
    fontWeight: 500,
    fontSize: '1.25rem',
    lineHeight: 1.5,
  },
  h6: {
    fontFamily: '"Inter", sans-serif',
    fontWeight: 500,
    fontSize: '1.125rem',
    lineHeight: 1.5,
  },
  
  // Body Text
  body1: {
    fontSize: '1rem',
    lineHeight: 1.6,
  },
  body2: {
    fontSize: '0.875rem',
    lineHeight: 1.5,
  },
  
  // Other
  button: {
    fontWeight: 500,
    fontSize: '0.875rem',
    textTransform: 'none' as const,
    letterSpacing: '0.02em',
  },
  caption: {
    fontSize: '0.75rem',
    lineHeight: 1.4,
  },
  overline: {
    fontSize: '0.75rem',
    fontWeight: 600,
    letterSpacing: '0.08em',
    textTransform: 'uppercase' as const,
  },
  
  // Code
  code: {
    fontFamily: '"JetBrains Mono", "Courier New", monospace',
    fontSize: '0.875rem',
  },
}

// Spacing (8px grid system)
const spacing = 8

// Shape
const shape = {
  borderRadius: 12,
  borderRadiusSmall: 8,
  borderRadiusLarge: 16,
}

// Shadows (subtle depth)
const shadows = {
  sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
  md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
  lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
  xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
  '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
  glass: '0 8px 32px 0 rgba(31, 38, 135, 0.37)',
}

// Glassmorphism effects
const glass = {
  light: {
    background: 'rgba(255, 255, 255, 0.8)',
    backdropFilter: 'blur(10px)',
    border: '1px solid rgba(255, 255, 255, 0.3)',
  },
  dark: {
    background: 'rgba(26, 26, 26, 0.8)',
    backdropFilter: 'blur(10px)',
    border: '1px solid rgba(255, 255, 255, 0.1)',
  },
}

// Animation
const transitions = {
  duration: {
    shortest: 150,
    shorter: 200,
    short: 250,
    standard: 300,
    complex: 375,
    enteringScreen: 225,
    leavingScreen: 195,
  },
  easing: {
    easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
    easeOut: 'cubic-bezier(0.0, 0, 0.2, 1)',
    easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
    sharp: 'cubic-bezier(0.4, 0, 0.6, 1)',
  },
}

// Component Overrides
const components = {
  MuiButton: {
    styleOverrides: {
      root: {
        borderRadius: shape.borderRadius,
        padding: '10px 20px',
        fontWeight: 500,
        transition: `all ${transitions.duration.standard}ms ${transitions.easing.easeInOut}`,
        '&:hover': {
          transform: 'translateY(-1px)',
          boxShadow: shadows.md,
        },
      },
      contained: {
        boxShadow: 'none',
        '&:hover': {
          boxShadow: shadows.md,
        },
      },
    },
  },
  
  MuiPaper: {
    styleOverrides: {
      root: {
        borderRadius: shape.borderRadius,
        boxShadow: shadows.md,
      },
      elevation1: {
        boxShadow: shadows.sm,
      },
      elevation2: {
        boxShadow: shadows.md,
      },
      elevation3: {
        boxShadow: shadows.lg,
      },
    },
  },
  
  MuiCard: {
    styleOverrides: {
      root: {
        borderRadius: shape.borderRadiusLarge,
        boxShadow: shadows.md,
        transition: `all ${transitions.duration.standard}ms ${transitions.easing.easeInOut}`,
        '&:hover': {
          transform: 'translateY(-2px)',
          boxShadow: shadows.lg,
        },
      },
    },
  },
  
  MuiTextField: {
    styleOverrides: {
      root: {
        '& .MuiOutlinedInput-root': {
          borderRadius: shape.borderRadius,
          '&:hover fieldset': {
            borderColor: colors.primary.main,
          },
        },
      },
    },
  },
  
  MuiAvatar: {
    styleOverrides: {
      root: {
        fontWeight: 500,
      },
    },
  },
  
  MuiChip: {
    styleOverrides: {
      root: {
        borderRadius: shape.borderRadiusSmall,
        fontWeight: 500,
      },
    },
  },
  
  MuiListItemButton: {
    styleOverrides: {
      root: {
        borderRadius: shape.borderRadius,
        margin: '2px 8px',
        '&.Mui-selected': {
          backgroundColor: alpha(colors.primary.main, 0.1),
          '&:hover': {
            backgroundColor: alpha(colors.primary.main, 0.15),
          },
        },
      },
    },
  },
}

// Create Light Theme
export const lightTheme = createTheme({
  palette: {
    mode: 'light',
    primary: colors.primary,
    secondary: colors.secondary,
    success: colors.success,
    warning: colors.warning,
    error: colors.error,
    info: colors.info,
    grey: colors.grey,
    background: colors.background,
    text: {
      primary: 'rgba(0, 0, 0, 0.87)',
      secondary: 'rgba(0, 0, 0, 0.6)',
      disabled: 'rgba(0, 0, 0, 0.38)',
    },
  },
  typography,
  spacing,
  shape: {
    borderRadius: shape.borderRadius,
  },
  shadows: [
    'none',
    shadows.sm,
    shadows.sm,
    shadows.md,
    shadows.md,
    shadows.md,
    shadows.lg,
    shadows.lg,
    shadows.lg,
    shadows.lg,
    shadows.xl,
    shadows.xl,
    shadows.xl,
    shadows.xl,
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
  ] as any,
  transitions,
  components,
})

// Create Dark Theme
export const darkTheme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      ...colors.primary,
      main: '#4285F4', // Slightly brighter for dark mode
    },
    secondary: colors.secondary,
    success: colors.success,
    warning: colors.warning,
    error: colors.error,
    info: colors.info,
    grey: colors.grey,
    background: colors.dark.background,
    text: colors.dark.text,
  },
  typography,
  spacing,
  shape: {
    borderRadius: shape.borderRadius,
  },
  shadows: [
    'none',
    shadows.sm,
    shadows.sm,
    shadows.md,
    shadows.md,
    shadows.md,
    shadows.lg,
    shadows.lg,
    shadows.lg,
    shadows.lg,
    shadows.xl,
    shadows.xl,
    shadows.xl,
    shadows.xl,
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
    shadows['2xl'],
  ] as any,
  transitions,
  components: {
    ...components,
    MuiPaper: {
      styleOverrides: {
        root: {
          borderRadius: shape.borderRadius,
          backgroundImage: 'none',
          backgroundColor: colors.dark.background.paper,
        },
      },
    },
  },
})

// Export additional design tokens
export const designTokens = {
  colors,
  typography,
  spacing,
  shape,
  shadows,
  glass,
  transitions,
}

// Utility functions for glassmorphism
export const getGlassStyle = (isDark: boolean) => ({
  ...glass[isDark ? 'dark' : 'light'],
  boxShadow: shadows.glass,
})

// Utility for creating gradient avatars from four-word identities
export const getFourWordGradient = (fourWords: string) => {
  const hash = fourWords.split('-').reduce((acc, word) => {
    return acc + word.charCodeAt(0)
  }, 0)
  
  const hue1 = (hash * 137) % 360
  const hue2 = (hue1 + 60) % 360
  
  return `linear-gradient(135deg, hsl(${hue1}, 70%, 60%) 0%, hsl(${hue2}, 70%, 60%) 100%)`
}

// Export themes
export default {
  light: lightTheme,
  dark: darkTheme,
  tokens: designTokens,
  utils: {
    getGlassStyle,
    getFourWordGradient,
  },
}