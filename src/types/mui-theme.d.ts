// MUI Theme module augmentation for custom fields used across the app
import type {} from '@mui/material/styles';

declare module '@mui/material/styles' {
  interface Theme {
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
  interface ThemeOptions {
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
}
