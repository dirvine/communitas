import { InputAdornment, TextField, TextFieldProps } from '@mui/material';
import { alpha, styled } from '@mui/material/styles';
import React from 'react';
import { designTokens } from '../../styles/theme';

interface ModernInputProps extends Omit<TextFieldProps, 'variant'> {
  icon?: React.ReactNode;
  iconPosition?: 'start' | 'end';
  glowing?: boolean;
  animated?: boolean;
}

const StyledTextField = styled(TextField, {
  shouldForwardProp: (prop) => !['glowing', 'animated'].includes(prop as string),
})<{ glowing?: boolean; animated?: boolean }>(({ theme, glowing, animated }) => ({
  '& .MuiOutlinedInput-root': {
    borderRadius: designTokens.borderRadius.lg,
    background: theme.palette.mode === 'light'
      ? 'rgba(255, 255, 255, 0.9)'
      : 'rgba(255, 255, 255, 0.05)',
    backdropFilter: 'blur(10px)',
    transition: `all ${designTokens.transitions.smooth}`,
    fontSize: '0.95rem',

    '& fieldset': {
      borderWidth: '2px',
      borderColor: theme.palette.mode === 'light'
        ? 'rgba(0, 0, 0, 0.1)'
        : 'rgba(255, 255, 255, 0.1)',
      transition: `all ${designTokens.transitions.normal}`,
    },

    '&:hover': {
      background: theme.palette.mode === 'light'
        ? 'rgba(255, 255, 255, 1)'
        : 'rgba(255, 255, 255, 0.08)',

      '& fieldset': {
        borderColor: theme.palette.primary.light,
      },
    },

    '&.Mui-focused': {
      background: theme.palette.mode === 'light'
        ? 'rgba(255, 255, 255, 1)'
        : 'rgba(255, 255, 255, 0.08)',

      '& fieldset': {
        borderWidth: '2px',
        borderColor: theme.palette.primary.main,
      },

      ...(glowing && {
        boxShadow: `0 0 0 4px ${alpha(theme.palette.primary.main, 0.15)}`,
      }),
    },

    '& input': {
      padding: '14px 16px',
      fontWeight: 500,

      '&::placeholder': {
        color: theme.palette.text.secondary,
        opacity: 0.7,
      },
    },

    '& .MuiInputAdornment-root': {
      marginLeft: 0,
      marginRight: 0,

      '& svg': {
        color: theme.palette.text.secondary,
        transition: `color ${designTokens.transitions.normal}`,
      },
    },

    ...(animated && {
      position: 'relative',
      overflow: 'hidden',

      '&::before': {
        content: '""',
        position: 'absolute',
        top: 0,
        left: '-100%',
        width: '100%',
        height: '100%',
        background: `linear-gradient(90deg, transparent, ${alpha(theme.palette.primary.main, 0.1)}, transparent)`,
        transition: `left ${designTokens.transitions.slow}`,
      },

      '&.Mui-focused::before': {
        left: '100%',
        transition: `left 1s ease-out`,
      },
    }),
  },

  '& .MuiInputLabel-root': {
    fontSize: '0.9rem',
    fontWeight: 500,
    transform: 'translate(14px, 14px) scale(1)',

    '&.MuiInputLabel-shrink': {
      transform: 'translate(14px, -9px) scale(0.75)',
      background: theme.palette.background.paper,
      padding: '0 8px',
      borderRadius: designTokens.borderRadius.xs,
    },
  },

  '& .MuiFormHelperText-root': {
    marginLeft: 8,
    fontSize: '0.75rem',
    marginTop: 6,
  },
}));

const FloatingLabel = styled('label')<{ focused: boolean; hasValue: boolean }>(({ theme, focused, hasValue }) => ({
  position: 'absolute',
  top: focused || hasValue ? '-8px' : '14px',
  left: '12px',
  fontSize: focused || hasValue ? '0.75rem' : '0.95rem',
  fontWeight: 500,
  color: focused ? theme.palette.primary.main : theme.palette.text.secondary,
  background: theme.palette.background.paper,
  padding: focused || hasValue ? '0 8px' : '0',
  borderRadius: designTokens.borderRadius.xs,
  transition: `all ${designTokens.transitions.normal}`,
  pointerEvents: 'none',
  zIndex: 1,
}));

const InputWrapper = styled('div')({
  position: 'relative',
  width: '100%',
});

export const ModernInput: React.FC<ModernInputProps> = ({
  icon,
  iconPosition = 'start',
  glowing = true,
  animated = true,
  label,
  value,
  onChange,
  onFocus,
  onBlur,
  ...props
}) => {
  const [focused, setFocused] = React.useState(false);
  const [hasValue, setHasValue] = React.useState(Boolean(value));

  React.useEffect(() => {
    setHasValue(Boolean(value));
  }, [value]);

  const handleFocus = (e: React.FocusEvent<HTMLInputElement>) => {
    setFocused(true);
    if (onFocus) onFocus(e);
  };

  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    setFocused(false);
    if (onBlur) onBlur(e);
  };

  const inputProps = icon
    ? {
        [iconPosition === 'start' ? 'startAdornment' : 'endAdornment']: (
          <InputAdornment position={iconPosition}>
            {icon}
          </InputAdornment>
        ),
      }
    : {};

  return (
    <StyledTextField
      variant="outlined"
      fullWidth
      glowing={glowing}
      animated={animated}
      value={value}
      onChange={onChange}
      onFocus={handleFocus}
      onBlur={handleBlur}
      label={label}
      InputProps={inputProps}
      {...props}
    />
  );
};

// Search Input Variant
const StyledSearchInput = styled('div')(({ theme }) => ({
  position: 'relative',
  borderRadius: designTokens.borderRadius.full,
  background: theme.palette.mode === 'light'
    ? 'rgba(255, 255, 255, 0.9)'
    : 'rgba(255, 255, 255, 0.05)',
  backdropFilter: 'blur(10px)',
  border: `2px solid ${theme.palette.mode === 'light'
    ? 'rgba(0, 0, 0, 0.05)'
    : 'rgba(255, 255, 255, 0.05)'}`,
  transition: `all ${designTokens.transitions.smooth}`,
  overflow: 'hidden',

  '&:hover': {
    background: theme.palette.mode === 'light'
      ? 'rgba(255, 255, 255, 1)'
      : 'rgba(255, 255, 255, 0.08)',
    borderColor: theme.palette.primary.light,
  },

  '&:focus-within': {
    background: theme.palette.mode === 'light'
      ? 'rgba(255, 255, 255, 1)'
      : 'rgba(255, 255, 255, 0.08)',
    borderColor: theme.palette.primary.main,
    boxShadow: `0 0 0 4px ${alpha(theme.palette.primary.main, 0.15)}`,
  },
}));

const SearchInputField = styled('input')(({ theme }) => ({
  width: '100%',
  padding: '12px 20px 12px 48px',
  border: 'none',
  background: 'transparent',
  fontSize: '0.95rem',
  fontWeight: 500,
  color: theme.palette.text.primary,
  outline: 'none',

  '&::placeholder': {
    color: theme.palette.text.secondary,
    opacity: 0.7,
  },
}));

const SearchIcon = styled('div')(({ theme }) => ({
  position: 'absolute',
  left: '16px',
  top: '50%',
  transform: 'translateY(-50%)',
  color: theme.palette.text.secondary,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  pointerEvents: 'none',
}));

interface SearchInputProps {
  value?: string;
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
  placeholder?: string;
  icon?: React.ReactNode;
}

export const SearchInput: React.FC<SearchInputProps> = ({
  value,
  onChange,
  placeholder = 'Search...',
  icon,
}) => {
  return (
    <StyledSearchInput>
      <SearchIcon>{icon}</SearchIcon>
      <SearchInputField
        type="text"
        value={value}
        onChange={onChange}
        placeholder={placeholder}
      />
    </StyledSearchInput>
  );
};