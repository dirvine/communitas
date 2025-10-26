import { Button, ButtonProps, CircularProgress } from '@mui/material';
import { styled } from '@mui/material/styles';
import React from 'react';
import { designTokens } from '../../styles/theme';

interface ModernButtonProps extends ButtonProps {
  gradient?: boolean;
  glow?: boolean;
  loading?: boolean;
  ripple?: boolean;
}

const StyledButton = styled(Button, {
  shouldForwardProp: (prop) => !['gradient', 'glow', 'loading', 'ripple'].includes(prop as string),
})<ModernButtonProps>(({ theme, gradient, glow, variant }) => ({
  position: 'relative',
  borderRadius: designTokens.borderRadius.lg,
  padding: '14px 28px',
  fontSize: '0.95rem',
  fontWeight: 600,
  letterSpacing: '0.025em',
  transition: `all ${designTokens.transitions.smooth}`,
  textTransform: 'none',
  overflow: 'hidden',

  ...(gradient && variant === 'contained' && {
    background: designTokens.colors.primary.gradient,
    color: '#ffffff',
    border: 'none',
    '&::before': {
      content: '""',
      position: 'absolute',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      background: 'linear-gradient(135deg, rgba(255,255,255,0.2) 0%, transparent 100%)',
      opacity: 0,
      transition: `opacity ${designTokens.transitions.normal}`,
    },
    '&:hover': {
      transform: 'translateY(-2px) scale(1.02)',
      boxShadow: designTokens.shadows.primary,
      '&::before': {
        opacity: 1,
      },
    },
  }),

  ...(glow && {
    '&::after': {
      content: '""',
      position: 'absolute',
      top: '50%',
      left: '50%',
      width: '100%',
      height: '100%',
      transform: 'translate(-50%, -50%)',
      background: variant === 'contained'
        ? designTokens.colors.primary.gradient
        : `radial-gradient(circle, ${theme.palette.primary.main} 0%, transparent 70%)`,
      opacity: 0,
      filter: 'blur(20px)',
      transition: `opacity ${designTokens.transitions.normal}`,
      pointerEvents: 'none',
    },
    '&:hover::after': {
      opacity: 0.4,
    },
  }),

  ...(variant === 'outlined' && {
    borderWidth: '2px',
    borderColor: theme.palette.primary.main,
    background: 'transparent',
    position: 'relative',
    overflow: 'hidden',
    '&::before': {
      content: '""',
      position: 'absolute',
      top: '50%',
      left: '50%',
      width: '0%',
      height: '0%',
      borderRadius: '50%',
      background: theme.palette.mode === 'light'
        ? `${theme.palette.primary.main}10`
        : `${theme.palette.primary.main}20`,
      transform: 'translate(-50%, -50%)',
      transition: `all ${designTokens.transitions.smooth}`,
    },
    '&:hover': {
      borderColor: theme.palette.primary.light,
      transform: 'translateY(-2px)',
      '&::before': {
        width: '300%',
        height: '300%',
      },
    },
  }),

  ...(variant === 'text' && {
    position: 'relative',
    '&::after': {
      content: '""',
      position: 'absolute',
      bottom: '8px',
      left: '50%',
      width: '0%',
      height: '2px',
      background: designTokens.colors.primary.gradient,
      transform: 'translateX(-50%)',
      transition: `width ${designTokens.transitions.smooth}`,
    },
    '&:hover': {
      background: 'transparent',
      '&::after': {
        width: '80%',
      },
    },
  }),

  '&:active': {
    transform: 'translateY(0) scale(0.98)',
  },

  '&:disabled': {
    opacity: 0.5,
    cursor: 'not-allowed',
    transform: 'none',
  },
}));

const LoadingWrapper = styled('div')({
  position: 'absolute',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
});

export const ModernButton: React.FC<ModernButtonProps> = ({
  children,
  gradient = true,
  glow = false,
  loading = false,
  ripple = true,
  disabled,
  onClick,
  ...props
}) => {
  const buttonRef = React.useRef<HTMLButtonElement>(null);

  const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    if (ripple && buttonRef.current && !loading && !disabled) {
      const button = buttonRef.current;
      const rect = button.getBoundingClientRect();
      const rippleEl = document.createElement('span');
      const size = Math.max(rect.width, rect.height);
      const x = e.clientX - rect.left - size / 2;
      const y = e.clientY - rect.top - size / 2;

      rippleEl.style.width = rippleEl.style.height = size + 'px';
      rippleEl.style.left = x + 'px';
      rippleEl.style.top = y + 'px';
      rippleEl.className = 'ripple-effect';

      // Add styles dynamically
      rippleEl.style.position = 'absolute';
      rippleEl.style.borderRadius = '50%';
      rippleEl.style.background = 'rgba(255, 255, 255, 0.6)';
      rippleEl.style.transform = 'scale(0)';
      rippleEl.style.animation = 'ripple 0.6s ease-out';
      rippleEl.style.pointerEvents = 'none';

      button.appendChild(rippleEl);

      setTimeout(() => {
        rippleEl.remove();
      }, 600);
    }

    if (onClick) {
      onClick(e);
    }
  };

  return (
    <StyledButton
      ref={buttonRef}
      gradient={gradient}
      glow={glow}
      disabled={disabled || loading}
      onClick={handleClick}
      {...props}
    >
      <span style={{ opacity: loading ? 0 : 1, transition: 'opacity 0.2s' }}>
        {children}
      </span>
      {loading && (
        <LoadingWrapper>
          <CircularProgress size={20} color="inherit" />
        </LoadingWrapper>
      )}
      <style>{`
        @keyframes ripple {
          to {
            transform: scale(4);
            opacity: 0;
          }
        }
      `}</style>
    </StyledButton>
  );
};