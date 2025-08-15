import { CustomThemeOptions } from './index';

export const baseThemeOptions: CustomThemeOptions = {
  shape: {
    borderRadius: 12,
  },
  
  spacing: 8,

  breakpoints: {
    values: {
      xs: 0,
      sm: 600,
      md: 900,
      lg: 1200,
      xl: 1536,
    },
  },

  transitions: {
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
  },

  // Custom shadows with modern design
  customShadows: {
    card: '0 4px 20px rgba(0, 0, 0, 0.08)',
    dropdown: '0 8px 24px rgba(0, 0, 0, 0.12)',
    modal: '0 24px 48px rgba(0, 0, 0, 0.16)',
    fab: '0 8px 32px rgba(0, 0, 0, 0.24)',
    navigation: '0 2px 12px rgba(0, 0, 0, 0.06)',
  },

  // Enhanced shadow system
  shadows: [
    'none',
    '0 1px 3px rgba(0, 0, 0, 0.12)', // 1
    '0 1px 5px rgba(0, 0, 0, 0.12)', // 2
    '0 1px 8px rgba(0, 0, 0, 0.12)', // 3
    '0 2px 4px rgba(0, 0, 0, 0.12)', // 4
    '0 2px 8px rgba(0, 0, 0, 0.12)', // 5
    '0 4px 8px rgba(0, 0, 0, 0.12)', // 6
    '0 4px 16px rgba(0, 0, 0, 0.12)', // 7
    '0 8px 16px rgba(0, 0, 0, 0.12)', // 8
    '0 8px 24px rgba(0, 0, 0, 0.12)', // 9
    '0 12px 24px rgba(0, 0, 0, 0.12)', // 10
    '0 16px 32px rgba(0, 0, 0, 0.12)', // 11
    '0 24px 48px rgba(0, 0, 0, 0.12)', // 12
    '0 32px 64px rgba(0, 0, 0, 0.16)', // 13
    '0 40px 80px rgba(0, 0, 0, 0.16)', // 14
    '0 48px 96px rgba(0, 0, 0, 0.20)', // 15
    '0 56px 112px rgba(0, 0, 0, 0.20)', // 16
    '0 64px 128px rgba(0, 0, 0, 0.24)', // 17
    '0 72px 144px rgba(0, 0, 0, 0.24)', // 18
    '0 80px 160px rgba(0, 0, 0, 0.28)', // 19
    '0 88px 176px rgba(0, 0, 0, 0.28)', // 20
    '0 96px 192px rgba(0, 0, 0, 0.32)', // 21
    '0 104px 208px rgba(0, 0, 0, 0.32)', // 22
    '0 112px 224px rgba(0, 0, 0, 0.36)', // 23
    '0 120px 240px rgba(0, 0, 0, 0.36)', // 24
  ],

  zIndex: {
    mobileStepper: 1000,
    fab: 1050,
    speedDial: 1050,
    appBar: 1100,
    drawer: 1200,
    modal: 1300,
    snackbar: 1400,
    tooltip: 1500,
  },
};