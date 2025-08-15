import { CustomThemeOptions } from './index';

export const darkThemeOptions: CustomThemeOptions = {
  palette: {
    mode: 'dark',
    primary: {
      main: '#3B82F6',
      light: '#60A5FA',
      dark: '#1E40AF',
      contrastText: '#FFFFFF',
    },
    secondary: {
      main: '#8B5CF6',
      light: '#A78BFA',
      dark: '#6D28D9',
      contrastText: '#FFFFFF',
    },
    error: {
      main: '#EF4444',
      light: '#F87171',
      dark: '#DC2626',
      contrastText: '#FFFFFF',
    },
    warning: {
      main: '#F59E0B',
      light: '#FBBF24',
      dark: '#D97706',
      contrastText: '#000000',
    },
    info: {
      main: '#06B6D4',
      light: '#22D3EE',
      dark: '#0891B2',
      contrastText: '#FFFFFF',
    },
    success: {
      main: '#10B981',
      light: '#34D399',
      dark: '#059669',
      contrastText: '#FFFFFF',
    },
    grey: {
      50: '#0F172A',
      100: '#1E293B',
      200: '#334155',
      300: '#475569',
      400: '#64748B',
      500: '#94A3B8',
      600: '#CBD5E1',
      700: '#E2E8F0',
      800: '#F1F5F9',
      900: '#F8FAFC',
    },
    background: {
      default: '#0F172A',
      paper: '#1E293B',
    },
    text: {
      primary: '#F8FAFC',
      secondary: '#CBD5E1',
      disabled: '#64748B',
    },
    divider: '#334155',
    action: {
      active: '#CBD5E1',
      hover: 'rgba(248, 250, 252, 0.04)',
      selected: 'rgba(248, 250, 252, 0.08)',
      disabled: 'rgba(248, 250, 252, 0.26)',
      disabledBackground: 'rgba(248, 250, 252, 0.12)',
      focus: 'rgba(248, 250, 252, 0.12)',
    },
  },

  // Custom gradients for dark mode
  gradients: {
    primary: 'linear-gradient(135deg, #3B82F6 0%, #1E40AF 100%)',
    secondary: 'linear-gradient(135deg, #8B5CF6 0%, #6D28D9 100%)',
    accent: 'linear-gradient(135deg, #10B981 0%, #059669 100%)',
    background: 'linear-gradient(180deg, #111827 0%, #0F172A 100%)',
  },

  // Dark mode specific shadows
  customShadows: {
    card: '0 4px 20px rgba(0, 0, 0, 0.32)',
    dropdown: '0 8px 24px rgba(0, 0, 0, 0.40)',
    modal: '0 24px 48px rgba(0, 0, 0, 0.48)',
    fab: '0 8px 32px rgba(59, 130, 246, 0.40)',
    navigation: '0 2px 12px rgba(0, 0, 0, 0.24)',
  },

  shadows: [
    'none',
    '0 1px 3px rgba(0, 0, 0, 0.32)',
    '0 1px 5px rgba(0, 0, 0, 0.32)',
    '0 1px 8px rgba(0, 0, 0, 0.32)',
    '0 2px 4px rgba(0, 0, 0, 0.32)',
    '0 2px 8px rgba(0, 0, 0, 0.32)',
    '0 4px 8px rgba(0, 0, 0, 0.32)',
    '0 4px 16px rgba(0, 0, 0, 0.32)',
    '0 8px 16px rgba(0, 0, 0, 0.32)',
    '0 8px 24px rgba(0, 0, 0, 0.32)',
    '0 12px 24px rgba(0, 0, 0, 0.32)',
    '0 16px 32px rgba(0, 0, 0, 0.32)',
    '0 24px 48px rgba(0, 0, 0, 0.32)',
    '0 32px 64px rgba(0, 0, 0, 0.40)',
    '0 40px 80px rgba(0, 0, 0, 0.40)',
    '0 48px 96px rgba(0, 0, 0, 0.48)',
    '0 56px 112px rgba(0, 0, 0, 0.48)',
    '0 64px 128px rgba(0, 0, 0, 0.56)',
    '0 72px 144px rgba(0, 0, 0, 0.56)',
    '0 80px 160px rgba(0, 0, 0, 0.64)',
    '0 88px 176px rgba(0, 0, 0, 0.64)',
    '0 96px 192px rgba(0, 0, 0, 0.72)',
    '0 104px 208px rgba(0, 0, 0, 0.72)',
    '0 112px 224px rgba(0, 0, 0, 0.80)',
    '0 120px 240px rgba(0, 0, 0, 0.80)',
  ],
};