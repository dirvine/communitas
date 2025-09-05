import React from 'react';
import {
  Button,
  ButtonProps,
  CircularProgress,
} from '@mui/material';
import { styled } from '@mui/material/styles';

export interface UnifiedButtonProps extends Omit<ButtonProps, 'variant' | 'size'> {
  loading?: boolean;
  fullWidth?: boolean;
}

// Styled button component with design token integration
const StyledButton = styled(Button)(({ theme }) => ({
  borderRadius: theme.shape.borderRadius,
  textTransform: 'none',
  fontWeight: 500,
  boxShadow: 'none',
  transition: theme.transitions.create(
    ['background-color', 'color', 'border-color', 'box-shadow', 'transform'],
    {
      duration: theme.transitions.duration.short,
      easing: theme.transitions.easing.easeInOut,
    }
  ),

  '&:hover': {
    boxShadow: theme.customShadows?.card || theme.shadows[2],
    transform: 'translateY(-1px)',
  },

  '&:disabled': {
    opacity: 0.5,
    cursor: 'not-allowed',
    transform: 'none !important',
    boxShadow: 'none !important',
  },
}));

export const UnifiedButton: React.FC<UnifiedButtonProps> = ({
  children,
  loading = false,
  disabled,
  startIcon,
  endIcon,
  ...props
}) => {
  return (
    <StyledButton
      disabled={disabled || loading}
      startIcon={loading ? <CircularProgress size={16} color="inherit" /> : startIcon}
      endIcon={endIcon}
      {...props}
    >
      {children}
    </StyledButton>
  );
};

export default UnifiedButton;