import { CustomThemeOptions } from './index';

export const lightThemeOptions: CustomThemeOptions = {
  palette: {
    mode: 'light',
    primary: {
      main: '#2563EB',
      light: '#3B82F6',
      dark: '#1D4ED8',
      contrastText: '#FFFFFF',
    },
    secondary: {
      main: '#7C3AED',
      light: '#8B5CF6',
      dark: '#5B21B6',
      contrastText: '#FFFFFF',
    },
    error: {
      main: '#DC2626',
      light: '#EF4444',
      dark: '#991B1B',
      contrastText: '#FFFFFF',
    },
    warning: {
      main: '#D97706',
      light: '#F59E0B',
      dark: '#B45309',
      contrastText: '#FFFFFF',
    },
    info: {
      main: '#0891B2',
      light: '#06B6D4',
      dark: '#0E7490',
      contrastText: '#FFFFFF',
    },
    success: {
      main: '#059669',
      light: '#10B981',
      dark: '#065F46',
      contrastText: '#FFFFFF',
    },
    grey: {
      50: '#F8FAFC',
      100: '#F1F5F9',
      200: '#E2E8F0',
      300: '#CBD5E1',
      400: '#94A3B8',
      500: '#64748B',
      600: '#475569',
      700: '#334155',
      800: '#1E293B',
      900: '#0F172A',
    },
    background: {
      default: '#F8FAFC',
      paper: '#FFFFFF',
    },
    text: {
      primary: '#0F172A',
      secondary: '#475569',
      disabled: '#94A3B8',
    },
    divider: '#E2E8F0',
    action: {
      active: '#475569',
      hover: 'rgba(71, 85, 105, 0.04)',
      selected: 'rgba(71, 85, 105, 0.08)',
      disabled: 'rgba(71, 85, 105, 0.26)',
      disabledBackground: 'rgba(71, 85, 105, 0.12)',
      focus: 'rgba(71, 85, 105, 0.12)',
    },
  },

  // Custom gradients for light mode
  gradients: {
    primary: 'linear-gradient(135deg, #2563EB 0%, #1D4ED8 100%)',
    secondary: 'linear-gradient(135deg, #7C3AED 0%, #5B21B6 100%)',
    accent: 'linear-gradient(135deg, #059669 0%, #047857 100%)',
    background: 'linear-gradient(180deg, #F8FAFC 0%, #F1F5F9 100%)',
  },

  // Light mode specific shadows
  customShadows: {
    card: '0 4px 20px rgba(15, 23, 42, 0.08)',
    dropdown: '0 8px 24px rgba(15, 23, 42, 0.12)',
    modal: '0 24px 48px rgba(15, 23, 42, 0.16)',
    fab: '0 8px 32px rgba(37, 99, 235, 0.24)',
    navigation: '0 2px 12px rgba(15, 23, 42, 0.06)',
  },
};