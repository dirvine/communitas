import { createTheme, alpha } from '@mui/material/styles';

// Ultra-modern design tokens
const designTokens = {
  // Modern color palette with vibrant gradients
  colors: {
    primary: {
      main: '#6366f1',      // Modern indigo
      light: '#818cf8',
      dark: '#4f46e5',
      gradient: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    },
    secondary: {
      main: '#ec4899',      // Modern pink
      light: '#f472b6',
      dark: '#db2777',
      gradient: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
    },
    success: {
      main: '#10b981',
      light: '#34d399',
      dark: '#059669',
      gradient: 'linear-gradient(135deg, #11998e 0%, #38ef7d 100%)',
    },
    warning: {
      main: '#f59e0b',
      light: '#fbbf24',
      dark: '#d97706',
      gradient: 'linear-gradient(135deg, #f2994a 0%, #f2c94c 100%)',
    },
    error: {
      main: '#ef4444',
      light: '#f87171',
      dark: '#dc2626',
      gradient: 'linear-gradient(135deg, #eb3349 0%, #f45c43 100%)',
    },
    neutral: {
      50: '#fafafa',
      100: '#f4f4f5',
      200: '#e4e4e7',
      300: '#d4d4d8',
      400: '#a1a1aa',
      500: '#71717a',
      600: '#52525b',
      700: '#3f3f46',
      800: '#27272a',
      900: '#18181b',
    },
  },
  // Modern typography with fluid sizing
  typography: {
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    displayLarge: 'clamp(2.5rem, 5vw, 4rem)',
    displayMedium: 'clamp(2rem, 4vw, 3rem)',
    displaySmall: 'clamp(1.5rem, 3vw, 2.5rem)',
    headlineLarge: 'clamp(1.25rem, 2.5vw, 2rem)',
    headlineMedium: 'clamp(1.125rem, 2vw, 1.5rem)',
    headlineSmall: 'clamp(1rem, 1.5vw, 1.25rem)',
    bodyLarge: '1.125rem',
    bodyMedium: '1rem',
    bodySmall: '0.875rem',
    labelLarge: '0.875rem',
    labelMedium: '0.75rem',
    labelSmall: '0.625rem',
  },
  // Modern spacing system (8px base)
  spacing: {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
    xxl: 48,
    xxxl: 64,
  },
  // Modern border radius
  borderRadius: {
    xs: 4,
    sm: 8,
    md: 12,
    lg: 16,
    xl: 24,
    xxl: 32,
    full: '9999px',
  },
  // Modern shadows with colored tints
  shadows: {
    xs: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    sm: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px -1px rgba(0, 0, 0, 0.1)',
    md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
    lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)',
    xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)',
    xxl: '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
    primary: '0 20px 25px -5px rgba(99, 102, 241, 0.3), 0 8px 10px -6px rgba(99, 102, 241, 0.2)',
    secondary: '0 20px 25px -5px rgba(236, 72, 153, 0.3), 0 8px 10px -6px rgba(236, 72, 153, 0.2)',
    glow: '0 0 20px rgba(99, 102, 241, 0.5)',
  },
  // Modern transitions
  transitions: {
    fast: '150ms cubic-bezier(0.4, 0, 0.2, 1)',
    normal: '250ms cubic-bezier(0.4, 0, 0.2, 1)',
    slow: '350ms cubic-bezier(0.4, 0, 0.2, 1)',
    bounce: '500ms cubic-bezier(0.68, -0.55, 0.265, 1.55)',
    smooth: '300ms cubic-bezier(0.25, 0.8, 0.25, 1)',
  },
  // Glassmorphism effects
  glass: {
    light: {
      background: 'rgba(255, 255, 255, 0.8)',
      backdropFilter: 'blur(20px) saturate(180%)',
      border: '1px solid rgba(255, 255, 255, 0.3)',
    },
    dark: {
      background: 'rgba(17, 25, 40, 0.75)',
      backdropFilter: 'blur(16px) saturate(180%)',
      border: '1px solid rgba(255, 255, 255, 0.125)',
    },
    colored: {
      background: 'linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(236, 72, 153, 0.1) 100%)',
      backdropFilter: 'blur(20px) saturate(200%)',
      border: '1px solid rgba(255, 255, 255, 0.2)',
    },
  },
};

// Create Material-UI theme with ultra-modern design
export const theme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: designTokens.colors.primary.main,
      light: designTokens.colors.primary.light,
      dark: designTokens.colors.primary.dark,
    },
    secondary: {
      main: designTokens.colors.secondary.main,
      light: designTokens.colors.secondary.light,
      dark: designTokens.colors.secondary.dark,
    },
    success: {
      main: designTokens.colors.success.main,
      light: designTokens.colors.success.light,
      dark: designTokens.colors.success.dark,
    },
    warning: {
      main: designTokens.colors.warning.main,
      light: designTokens.colors.warning.light,
      dark: designTokens.colors.warning.dark,
    },
    error: {
      main: designTokens.colors.error.main,
      light: designTokens.colors.error.light,
      dark: designTokens.colors.error.dark,
    },
    background: {
      default: '#fafafa',
      paper: '#ffffff',
    },
    text: {
      primary: designTokens.colors.neutral[900],
      secondary: designTokens.colors.neutral[600],
    },
  },
  typography: {
    fontFamily: designTokens.typography.fontFamily,
    h1: {
      fontSize: designTokens.typography.displayLarge,
      fontWeight: 700,
      letterSpacing: '-0.02em',
      lineHeight: 1.2,
    },
    h2: {
      fontSize: designTokens.typography.displayMedium,
      fontWeight: 600,
      letterSpacing: '-0.01em',
      lineHeight: 1.3,
    },
    h3: {
      fontSize: designTokens.typography.displaySmall,
      fontWeight: 600,
      letterSpacing: '-0.01em',
      lineHeight: 1.4,
    },
    h4: {
      fontSize: designTokens.typography.headlineLarge,
      fontWeight: 500,
      lineHeight: 1.4,
    },
    h5: {
      fontSize: designTokens.typography.headlineMedium,
      fontWeight: 500,
      lineHeight: 1.5,
    },
    h6: {
      fontSize: designTokens.typography.headlineSmall,
      fontWeight: 500,
      lineHeight: 1.5,
    },
    body1: {
      fontSize: designTokens.typography.bodyMedium,
      lineHeight: 1.6,
    },
    body2: {
      fontSize: designTokens.typography.bodySmall,
      lineHeight: 1.6,
    },
    button: {
      fontSize: designTokens.typography.labelLarge,
      fontWeight: 500,
      textTransform: 'none',
      letterSpacing: '0.02em',
    },
  },
  shape: {
    borderRadius: designTokens.borderRadius.md,
  },
  shadows: [
    'none',
    designTokens.shadows.xs,
    designTokens.shadows.sm,
    designTokens.shadows.sm,
    designTokens.shadows.md,
    designTokens.shadows.md,
    designTokens.shadows.md,
    designTokens.shadows.md,
    designTokens.shadows.lg,
    designTokens.shadows.lg,
    designTokens.shadows.lg,
    designTokens.shadows.lg,
    designTokens.shadows.xl,
    designTokens.shadows.xl,
    designTokens.shadows.xl,
    designTokens.shadows.xl,
    designTokens.shadows.xl,
    designTokens.shadows.xl,
    designTokens.shadows.xl,
    designTokens.shadows.xl,
    designTokens.shadows.xxl,
    designTokens.shadows.xxl,
    designTokens.shadows.xxl,
    designTokens.shadows.xxl,
    designTokens.shadows.xxl,
  ],
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: designTokens.borderRadius.lg,
          padding: '12px 24px',
          fontSize: designTokens.typography.labelLarge,
          fontWeight: 500,
          transition: designTokens.transitions.normal,
          textTransform: 'none',
          boxShadow: 'none',
          '&:hover': {
            transform: 'translateY(-2px)',
            boxShadow: designTokens.shadows.lg,
          },
          '&:active': {
            transform: 'translateY(0)',
          },
        },
        contained: {
          background: designTokens.colors.primary.gradient,
          color: '#ffffff',
          '&:hover': {
            background: designTokens.colors.primary.gradient,
            boxShadow: designTokens.shadows.primary,
          },
        },
        outlined: {
          borderWidth: '2px',
          '&:hover': {
            borderWidth: '2px',
            background: alpha(designTokens.colors.primary.main, 0.05),
          },
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          borderRadius: designTokens.borderRadius.xl,
          boxShadow: designTokens.shadows.sm,
          transition: designTokens.transitions.normal,
          overflow: 'hidden',
          '&:hover': {
            transform: 'translateY(-4px)',
            boxShadow: designTokens.shadows.xl,
          },
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          borderRadius: designTokens.borderRadius.lg,
          boxShadow: designTokens.shadows.sm,
        },
        elevation1: {
          boxShadow: designTokens.shadows.sm,
        },
        elevation2: {
          boxShadow: designTokens.shadows.md,
        },
        elevation3: {
          boxShadow: designTokens.shadows.lg,
        },
      },
    },
    MuiTextField: {
      styleOverrides: {
        root: {
          '& .MuiOutlinedInput-root': {
            borderRadius: designTokens.borderRadius.md,
            transition: designTokens.transitions.normal,
            '& fieldset': {
              borderWidth: '2px',
              transition: designTokens.transitions.normal,
            },
            '&:hover fieldset': {
              borderColor: designTokens.colors.primary.light,
            },
            '&.Mui-focused fieldset': {
              borderColor: designTokens.colors.primary.main,
              borderWidth: '2px',
            },
            '&.Mui-focused': {
              boxShadow: `0 0 0 4px ${alpha(designTokens.colors.primary.main, 0.1)}`,
            },
          },
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: {
          borderRadius: designTokens.borderRadius.full,
          fontWeight: 500,
          transition: designTokens.transitions.normal,
          '&:hover': {
            transform: 'scale(1.05)',
          },
        },
      },
    },
    MuiAvatar: {
      styleOverrides: {
        root: {
          fontWeight: 600,
          boxShadow: designTokens.shadows.md,
        },
      },
    },
    MuiDialog: {
      styleOverrides: {
        paper: {
          borderRadius: designTokens.borderRadius.xl,
          boxShadow: designTokens.shadows.xxl,
        },
      },
    },
    MuiTooltip: {
      styleOverrides: {
        tooltip: {
          ...designTokens.glass.dark,
          borderRadius: designTokens.borderRadius.md,
          fontSize: designTokens.typography.bodySmall,
          padding: '8px 12px',
        },
      },
    },
  },
});

// Export design tokens for use in styled components
export { designTokens };

// Dark theme variant
export const darkTheme = createTheme({
  ...theme,
  palette: {
    mode: 'dark',
    primary: {
      main: designTokens.colors.primary.light,
      light: designTokens.colors.primary.main,
      dark: designTokens.colors.primary.dark,
    },
    secondary: {
      main: designTokens.colors.secondary.light,
      light: designTokens.colors.secondary.main,
      dark: designTokens.colors.secondary.dark,
    },
    background: {
      default: designTokens.colors.neutral[900],
      paper: designTokens.colors.neutral[800],
    },
    text: {
      primary: designTokens.colors.neutral[50],
      secondary: designTokens.colors.neutral[400],
    },
  },
  components: {
    ...theme.components,
    MuiCard: {
      styleOverrides: {
        root: {
          ...theme.components?.MuiCard?.styleOverrides?.root,
          ...designTokens.glass.dark,
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          ...theme.components?.MuiPaper?.styleOverrides?.root,
          backgroundImage: 'none',
        },
      },
    },
  },
});