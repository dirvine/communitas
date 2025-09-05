import React, { createContext, useContext, useEffect, useState } from 'react';
import { ThemeProvider as MuiThemeProvider, CssBaseline, GlobalStyles } from '@mui/material';
import { createCustomTheme, ThemeMode, ColorPreset } from '../../theme';

// Theme context interface
interface ThemeContextType {
  mode: ThemeMode;
  colorPreset: ColorPreset;
  toggleMode: () => void;
  setColorPreset: (preset: ColorPreset) => void;
  isDarkMode: boolean;
}

// Create theme context
const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

// Hook to use theme context
export const useTheme = (): ThemeContextType => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

// Global styles for custom scrollbar and animations
const globalStyles = {
  // Custom scrollbar styling
  '*::-webkit-scrollbar': {
    width: '8px',
    height: '8px',
  },
  '*::-webkit-scrollbar-track': {
    background: 'transparent',
  },
  '*::-webkit-scrollbar-thumb': {
    background: (theme: any) => theme.palette.divider,
    borderRadius: '4px',
    '&:hover': {
      background: (theme: any) => theme.palette.text.secondary,
    },
  },
  
  // Smooth transitions for theme switching
  '*': {
    transition: 'background-color 0.3s ease, color 0.3s ease, border-color 0.3s ease !important',
  },
  
  // Enhanced focus styles for accessibility
  '.MuiFocusVisible-root': {
    outline: (theme: any) => `2px solid ${theme.palette.primary.main}`,
    outlineOffset: '2px',
  },

  // High contrast focus for better accessibility
  '*:focus-visible': {
    outline: (theme: any) => `2px solid ${theme.palette.primary.main}`,
    outlineOffset: '2px',
  },

  // Ensure sufficient color contrast
  'button, [role="button"]': {
    '&:focus-visible': {
      outline: (theme: any) => `2px solid ${theme.palette.primary.main}`,
      outlineOffset: '2px',
    },
  },

  // Improve readability
  'p, span, div': {
    lineHeight: 1.5,
  },

  // Ensure interactive elements have proper cursor
  'button, [role="button"], [tabindex]:not([tabindex="-1"])': {
    cursor: 'pointer',
  },
  
  // Custom animations
  '@keyframes fadeIn': {
    from: { opacity: 0 },
    to: { opacity: 1 },
  },
  
  '@keyframes slideInFromRight': {
    from: { 
      opacity: 0,
      transform: 'translateX(20px)',
    },
    to: { 
      opacity: 1,
      transform: 'translateX(0)',
    },
  },
  
  '@keyframes slideInFromTop': {
    from: { 
      opacity: 0,
      transform: 'translateY(-20px)',
    },
    to: { 
      opacity: 1,
      transform: 'translateY(0)',
    },
  },
  
  // Animation utility classes
  '.animate-fade-in': {
    animation: 'fadeIn 0.3s ease-out',
  },
  
  '.animate-slide-in-right': {
    animation: 'slideInFromRight 0.3s ease-out',
  },
  
  '.animate-slide-in-top': {
    animation: 'slideInFromTop 0.3s ease-out',
  },
  
  // Glass morphism effect
  '.glass-effect': {
    backgroundColor: (theme: any) => theme.palette.mode === 'light' 
      ? 'rgba(255, 255, 255, 0.8)'
      : 'rgba(30, 41, 59, 0.8)',
    backdropFilter: 'blur(20px)',
    border: (theme: any) => `1px solid ${theme.palette.divider}`,
  },
  
  // Gradient text effect
  '.gradient-text': {
    background: (theme: any) => theme.gradients?.primary || theme.palette.primary.main,
    WebkitBackgroundClip: 'text',
    WebkitTextFillColor: 'transparent',
    backgroundClip: 'text',
  },
  
  // Enhanced shadow on hover
  '.hover-lift': {
    transition: 'transform 0.2s ease, box-shadow 0.2s ease',
    '&:hover': {
      transform: 'translateY(-2px)',
      boxShadow: (theme: any) => theme.customShadows?.card || theme.shadows[4],
    },
  },
};

// Local storage keys
const STORAGE_KEYS = {
  THEME_MODE: 'communitas_theme_mode',
  COLOR_PRESET: 'communitas_color_preset',
} as const;

// Theme provider props
interface ThemeProviderProps {
  children: React.ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  // Initialize state from localStorage or defaults
  const [mode, setMode] = useState<ThemeMode>(() => {
    const savedMode = localStorage.getItem(STORAGE_KEYS.THEME_MODE);
    if (savedMode === 'light' || savedMode === 'dark') {
      return savedMode;
    }
    // Auto-detect system preference
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  });

  const [colorPreset, setColorPresetState] = useState<ColorPreset>(() => {
    const savedPreset = localStorage.getItem(STORAGE_KEYS.COLOR_PRESET);
    return (savedPreset as ColorPreset) || 'professional';
  });

  // Create theme based on current mode and preset with performance optimization
  const theme = React.useMemo(() => createCustomTheme(mode, colorPreset), [mode, colorPreset]);

  // Toggle between light and dark modes
  const toggleMode = () => {
    const newMode = mode === 'light' ? 'dark' : 'light';
    setMode(newMode);
    localStorage.setItem(STORAGE_KEYS.THEME_MODE, newMode);
  };

  // Set color preset
  const setColorPreset = (preset: ColorPreset) => {
    setColorPresetState(preset);
    localStorage.setItem(STORAGE_KEYS.COLOR_PRESET, preset);
  };

  // Listen for system theme changes
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleChange = (e: MediaQueryListEvent) => {
      // Only auto-switch if user hasn't manually set a preference
      const savedMode = localStorage.getItem(STORAGE_KEYS.THEME_MODE);
      if (!savedMode) {
        setMode(e.matches ? 'dark' : 'light');
      }
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, []);

  // Update document metadata for theme
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', mode);
    document.documentElement.setAttribute('data-color-preset', colorPreset);
    
    // Update meta theme-color for mobile browsers
    const metaThemeColor = document.querySelector('meta[name="theme-color"]');
    if (metaThemeColor) {
      metaThemeColor.setAttribute('content', theme.palette.primary.main);
    }
  }, [mode, colorPreset, theme]);

  const contextValue: ThemeContextType = {
    mode,
    colorPreset,
    toggleMode,
    setColorPreset,
    isDarkMode: mode === 'dark',
  };

  return (
    <ThemeContext.Provider value={contextValue}>
      <MuiThemeProvider theme={theme}>
        <CssBaseline />
        <GlobalStyles styles={globalStyles} />
        {children}
      </MuiThemeProvider>
    </ThemeContext.Provider>
  );
};

export default ThemeProvider;