import { createTheme, Theme, ThemeOptions } from '@mui/material/styles';
import { deepmerge } from '@mui/utils';
import { baseThemeOptions } from './base';
import { lightThemeOptions } from './light';
import { darkThemeOptions } from './dark';
import { components } from './components';
import { typography } from './typography';
import { getTokens, createCssVariables } from './tokens';

export type ThemeMode = 'light' | 'dark' | 'auto';

// Exported preset type for consumers
export type ColorPreset = 'professional' | 'creative' | 'tech' | 'warm';

export interface CustomThemeOptions extends ThemeOptions {
  customShadows?: {
    card: string;
    dropdown: string;
    modal: string;
    fab: string;
    navigation: string;
  };
  gradients?: {
    primary: string;
    secondary: string;
    accent: string;
    background: string;
  };
}

export const createCustomTheme = (mode: ThemeMode, colorPreset?: string): Theme => {
  const isDark = mode === 'dark' || (mode === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  const themeMode: 'light' | 'dark' = isDark ? 'dark' : 'light';

  // Get design tokens for the current mode
  const tokens = getTokens(themeMode);

  // Base configuration with tokens
  const baseOptions: CustomThemeOptions = deepmerge(baseThemeOptions, {
    palette: {
      mode: themeMode,
      primary: {
        main: tokens.colors.brand.primary,
        light: tokens.colors.brand.primary, // Will be overridden by preset
        dark: tokens.colors.brand.primary,  // Will be overridden by preset
        contrastText: tokens.colors.semantic.text.inverse,
      },
      secondary: {
        main: tokens.colors.brand.secondary,
        light: tokens.colors.brand.secondary,
        dark: tokens.colors.brand.secondary,
        contrastText: tokens.colors.semantic.text.inverse,
      },
      error: {
        main: tokens.colors.brand.error,
        light: tokens.colors.brand.error,
        dark: tokens.colors.brand.error,
        contrastText: tokens.colors.semantic.text.inverse,
      },
      warning: {
        main: tokens.colors.brand.warning,
        light: tokens.colors.brand.warning,
        dark: tokens.colors.brand.warning,
        contrastText: tokens.colors.semantic.text.primary,
      },
      info: {
        main: tokens.colors.brand.info,
        light: tokens.colors.brand.info,
        dark: tokens.colors.brand.info,
        contrastText: tokens.colors.semantic.text.inverse,
      },
      success: {
        main: tokens.colors.brand.success,
        light: tokens.colors.brand.success,
        dark: tokens.colors.brand.success,
        contrastText: tokens.colors.semantic.text.inverse,
      },
      background: {
        default: tokens.colors.semantic.background.primary,
        paper: tokens.colors.semantic.surface.primary,
      },
      text: {
        primary: tokens.colors.semantic.text.primary,
        secondary: tokens.colors.semantic.text.secondary,
        disabled: tokens.colors.semantic.text.tertiary,
      },
      divider: tokens.colors.semantic.border.primary,
      action: {
        active: tokens.colors.semantic.text.secondary,
        hover: tokens.colors.semantic.background.secondary,
        selected: tokens.colors.semantic.background.tertiary,
        disabled: tokens.colors.semantic.text.tertiary,
        disabledBackground: tokens.colors.semantic.surface.secondary,
        focus: tokens.colors.semantic.border.focus,
      },
    },
    typography,
    components,
    // Add custom gradients and shadows from tokens
    gradients: tokens.colors.semantic.background,
    customShadows: {
      card: tokens.shadows.sm,
      dropdown: tokens.shadows.md,
      modal: tokens.shadows.lg,
      fab: tokens.shadows.lg,
      navigation: tokens.shadows.xs,
    },
  });

  // Apply mode-specific options
  const modeOptions = isDark ? darkThemeOptions : lightThemeOptions;
  const themeOptions = deepmerge(baseOptions, modeOptions);

  // Apply color preset if specified
  if (colorPreset) {
    const presetOptions = getColorPreset(colorPreset, isDark);
    const finalTheme = createTheme(deepmerge(themeOptions, presetOptions));

    // Inject CSS custom properties for design tokens
    const cssVariables = createCssVariables(tokens);
    const rootVars = Object.entries(cssVariables)
      .map(([key, value]) => `${key}: ${value}`)
      .join('; ');

    // Add CSS variables to document head
    if (typeof document !== 'undefined') {
      let styleElement = document.getElementById('design-tokens');
      if (!styleElement) {
        styleElement = document.createElement('style');
        styleElement.id = 'design-tokens';
        document.head.appendChild(styleElement);
      }
      styleElement.textContent = `:root { ${rootVars} }`;
    }

    return finalTheme;
  }

  return createTheme(themeOptions);
};

// Color presets for different brand personalities
const getColorPreset = (preset: string, isDark: boolean): CustomThemeOptions => {
  const presets: Record<string, CustomThemeOptions> = {
    // Professional Blue (Default)
    professional: {
      palette: {
        primary: {
          main: isDark ? '#3B82F6' : '#2563EB',
          light: isDark ? '#60A5FA' : '#3B82F6',
          dark: isDark ? '#1E40AF' : '#1D4ED8',
        },
        secondary: {
          main: isDark ? '#8B5CF6' : '#7C3AED',
          light: isDark ? '#A78BFA' : '#8B5CF6',
          dark: isDark ? '#6D28D9' : '#5B21B6',
        },
      },
      gradients: {
        primary: isDark 
          ? 'linear-gradient(135deg, #3B82F6 0%, #1E40AF 100%)'
          : 'linear-gradient(135deg, #2563EB 0%, #1D4ED8 100%)',
        secondary: isDark
          ? 'linear-gradient(135deg, #8B5CF6 0%, #6D28D9 100%)'
          : 'linear-gradient(135deg, #7C3AED 0%, #5B21B6 100%)',
        accent: isDark
          ? 'linear-gradient(135deg, #10B981 0%, #059669 100%)'
          : 'linear-gradient(135deg, #059669 0%, #047857 100%)',
        background: isDark
          ? 'linear-gradient(180deg, #111827 0%, #0F172A 100%)'
          : 'linear-gradient(180deg, #F8FAFC 0%, #F1F5F9 100%)',
      },
    },

    // Creative Purple
    creative: {
      palette: {
        primary: {
          main: isDark ? '#A855F7' : '#9333EA',
          light: isDark ? '#C084FC' : '#A855F7',
          dark: isDark ? '#7C3AED' : '#7C2D92',
        },
        secondary: {
          main: isDark ? '#EC4899' : '#E11D48',
          light: isDark ? '#F472B6' : '#EC4899',
          dark: isDark ? '#BE185D' : '#BE123C',
        },
      },
      gradients: {
        primary: isDark 
          ? 'linear-gradient(135deg, #A855F7 0%, #7C3AED 100%)'
          : 'linear-gradient(135deg, #9333EA 0%, #7C2D92 100%)',
        secondary: isDark
          ? 'linear-gradient(135deg, #EC4899 0%, #BE185D 100%)'
          : 'linear-gradient(135deg, #E11D48 0%, #BE123C 100%)',
        accent: isDark
          ? 'linear-gradient(135deg, #F59E0B 0%, #D97706 100%)'
          : 'linear-gradient(135deg, #D97706 0%, #B45309 100%)',
        background: isDark
          ? 'linear-gradient(180deg, #1E1B4B 0%, #0F0F23 100%)'
          : 'linear-gradient(180deg, #FAF5FF 0%, #F3E8FF 100%)',
      },
    },

    // Tech Green
    tech: {
      palette: {
        primary: {
          main: isDark ? '#10B981' : '#059669',
          light: isDark ? '#34D399' : '#10B981',
          dark: isDark ? '#047857' : '#065F46',
        },
        secondary: {
          main: isDark ? '#3B82F6' : '#2563EB',
          light: isDark ? '#60A5FA' : '#3B82F6',
          dark: isDark ? '#1E40AF' : '#1D4ED8',
        },
      },
      gradients: {
        primary: isDark 
          ? 'linear-gradient(135deg, #10B981 0%, #047857 100%)'
          : 'linear-gradient(135deg, #059669 0%, #065F46 100%)',
        secondary: isDark
          ? 'linear-gradient(135deg, #3B82F6 0%, #1E40AF 100%)'
          : 'linear-gradient(135deg, #2563EB 0%, #1D4ED8 100%)',
        accent: isDark
          ? 'linear-gradient(135deg, #F59E0B 0%, #D97706 100%)'
          : 'linear-gradient(135deg, #D97706 0%, #B45309 100%)',
        background: isDark
          ? 'linear-gradient(180deg, #064E3B 0%, #022C22 100%)'
          : 'linear-gradient(180deg, #F0FDF4 0%, #DCFCE7 100%)',
      },
    },

    // Warm Orange
    warm: {
      palette: {
        primary: {
          main: isDark ? '#F97316' : '#EA580C',
          light: isDark ? '#FB923C' : '#F97316',
          dark: isDark ? '#C2410C' : '#C2410C',
        },
        secondary: {
          main: isDark ? '#EF4444' : '#DC2626',
          light: isDark ? '#F87171' : '#EF4444',
          dark: isDark ? '#B91C1C' : '#991B1B',
        },
      },
      gradients: {
        primary: isDark 
          ? 'linear-gradient(135deg, #F97316 0%, #C2410C 100%)'
          : 'linear-gradient(135deg, #EA580C 0%, #C2410C 100%)',
        secondary: isDark
          ? 'linear-gradient(135deg, #EF4444 0%, #B91C1C 100%)'
          : 'linear-gradient(135deg, #DC2626 0%, #991B1B 100%)',
        accent: isDark
          ? 'linear-gradient(135deg, #8B5CF6 0%, #6D28D9 100%)'
          : 'linear-gradient(135deg, #7C3AED 0%, #5B21B6 100%)',
        background: isDark
          ? 'linear-gradient(180deg, #431407 0%, #1C1917 100%)'
          : 'linear-gradient(180deg, #FFF7ED 0%, #FED7AA 100%)',
      },
    },
  };

  return presets[preset] || presets.professional;
};

export * from './base';
export * from './light';
export * from './dark';
export * from './components';
export * from './typography';