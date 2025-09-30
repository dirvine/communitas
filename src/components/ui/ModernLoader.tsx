import React from 'react';
import { Box, CircularProgress } from '@mui/material';
import { styled, keyframes, alpha } from '@mui/material/styles';
import { designTokens } from '../../styles/theme';

interface ModernLoaderProps {
  variant?: 'pulse' | 'wave' | 'orbit' | 'dots' | 'spinner' | 'gradient';
  size?: 'small' | 'medium' | 'large';
  color?: string;
  text?: string;
}

// Animations
const pulse = keyframes`
  0% {
    transform: scale(0);
    opacity: 1;
  }
  100% {
    transform: scale(1);
    opacity: 0;
  }
`;

const wave = keyframes`
  0%, 60%, 100% {
    transform: translateY(0);
  }
  30% {
    transform: translateY(-15px);
  }
`;

const orbit = keyframes`
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
`;

const dotPulse = keyframes`
  0%, 80%, 100% {
    transform: scale(0);
    opacity: 0.5;
  }
  40% {
    transform: scale(1);
    opacity: 1;
  }
`;

const gradientSpin = keyframes`
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
`;

const gradientMove = keyframes`
  0% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
  100% {
    background-position: 0% 50%;
  }
`;

// Styled components for different loader variants
const LoaderContainer = styled(Box)(({ theme }) => ({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  flexDirection: 'column',
  gap: theme.spacing(2),
}));

const PulseLoader = styled('div')<{ size: string }>(({ size }) => ({
  display: 'inline-block',
  position: 'relative',
  width: size === 'small' ? '40px' : size === 'large' ? '80px' : '60px',
  height: size === 'small' ? '40px' : size === 'large' ? '80px' : '60px',

  '& div': {
    position: 'absolute',
    border: `4px solid ${designTokens.colors.primary.main}`,
    borderRadius: '50%',
    animation: `${pulse} 1.5s cubic-bezier(0, 0.2, 0.8, 1) infinite`,

    '&:nth-of-type(2)': {
      animationDelay: '-0.5s',
    },
  },

  '& div:nth-of-type(1)': {
    inset: 0,
  },
  '& div:nth-of-type(2)': {
    inset: 0,
  },
}));

const WaveLoader = styled('div')<{ size: string }>(({ size }) => ({
  display: 'flex',
  gap: size === 'small' ? '4px' : size === 'large' ? '8px' : '6px',

  '& div': {
    width: size === 'small' ? '8px' : size === 'large' ? '16px' : '12px',
    height: size === 'small' ? '30px' : size === 'large' ? '60px' : '45px',
    background: designTokens.colors.primary.gradient,
    borderRadius: designTokens.borderRadius.full,
    animation: `${wave} 1.2s ease-in-out infinite`,

    '&:nth-of-type(2)': {
      animationDelay: '-1.1s',
    },
    '&:nth-of-type(3)': {
      animationDelay: '-1s',
    },
    '&:nth-of-type(4)': {
      animationDelay: '-0.9s',
    },
    '&:nth-of-type(5)': {
      animationDelay: '-0.8s',
    },
  },
}));

const OrbitLoader = styled('div')<{ size: string }>(({ size }) => ({
  display: 'inline-block',
  position: 'relative',
  width: size === 'small' ? '40px' : size === 'large' ? '80px' : '60px',
  height: size === 'small' ? '40px' : size === 'large' ? '80px' : '60px',

  '& div': {
    position: 'absolute',
    width: '100%',
    height: '100%',
    border: '3px solid transparent',
    borderTopColor: designTokens.colors.primary.main,
    borderRadius: '50%',
    animation: `${orbit} 1.2s linear infinite`,

    '&:nth-of-type(1)': {
      animationDelay: '0s',
    },
    '&:nth-of-type(2)': {
      width: '75%',
      height: '75%',
      margin: '12.5%',
      borderTopColor: designTokens.colors.secondary.main,
      animationDelay: '-0.3s',
      animationDuration: '1s',
    },
    '&:nth-of-type(3)': {
      width: '50%',
      height: '50%',
      margin: '25%',
      borderTopColor: designTokens.colors.success.main,
      animationDelay: '-0.6s',
      animationDuration: '0.8s',
    },
  },
}));

const DotsLoader = styled('div')<{ size: string }>(({ size }) => ({
  display: 'flex',
  gap: size === 'small' ? '6px' : size === 'large' ? '12px' : '8px',
  alignItems: 'center',

  '& div': {
    width: size === 'small' ? '10px' : size === 'large' ? '20px' : '15px',
    height: size === 'small' ? '10px' : size === 'large' ? '20px' : '15px',
    borderRadius: '50%',
    background: designTokens.colors.primary.main,
    animation: `${dotPulse} 1.4s ease-in-out infinite`,

    '&:nth-of-type(1)': {
      animationDelay: '-0.32s',
    },
    '&:nth-of-type(2)': {
      animationDelay: '-0.16s',
    },
    '&:nth-of-type(3)': {
      animationDelay: '0s',
    },
  },
}));

const GradientSpinner = styled('div')<{ size: string }>(({ size }) => ({
  width: size === 'small' ? '40px' : size === 'large' ? '80px' : '60px',
  height: size === 'small' ? '40px' : size === 'large' ? '80px' : '60px',
  borderRadius: '50%',
  background: designTokens.colors.primary.gradient,
  backgroundSize: '300% 300%',
  animation: `${gradientSpin} 1s linear infinite, ${gradientMove} 3s ease infinite`,
  position: 'relative',

  '&::before': {
    content: '""',
    position: 'absolute',
    inset: '3px',
    borderRadius: '50%',
    background: 'white',
  },

  '&::after': {
    content: '""',
    position: 'absolute',
    inset: 0,
    borderRadius: '50%',
    background: designTokens.colors.primary.gradient,
    backgroundSize: '300% 300%',
    animation: `${gradientMove} 3s ease infinite`,
    clipPath: 'polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%, 50% 0%)',
  },
}));

const LoaderText = styled('div')(({ theme }) => ({
  marginTop: theme.spacing(1),
  fontSize: '0.875rem',
  fontWeight: 500,
  color: theme.palette.text.secondary,
  letterSpacing: '0.025em',
  animation: `${pulse} 2s ease-in-out infinite`,
}));

const SkeletonLoader = styled('div')(({ theme }) => ({
  width: '100%',
  height: '100%',
  background: `linear-gradient(90deg,
    ${alpha(theme.palette.action.hover, 0.05)} 25%,
    ${alpha(theme.palette.action.hover, 0.15)} 50%,
    ${alpha(theme.palette.action.hover, 0.05)} 75%)`,
  backgroundSize: '200% 100%',
  animation: `${gradientMove} 1.5s ease infinite`,
  borderRadius: designTokens.borderRadius.md,
}));

export const ModernLoader: React.FC<ModernLoaderProps> = ({
  variant = 'spinner',
  size = 'medium',
  color,
  text,
}) => {
  const getSize = () => {
    switch (size) {
      case 'small': return 24;
      case 'large': return 48;
      default: return 36;
    }
  };

  const renderLoader = () => {
    switch (variant) {
      case 'pulse':
        return (
          <PulseLoader size={size}>
            <div />
            <div />
          </PulseLoader>
        );

      case 'wave':
        return (
          <WaveLoader size={size}>
            <div />
            <div />
            <div />
            <div />
            <div />
          </WaveLoader>
        );

      case 'orbit':
        return (
          <OrbitLoader size={size}>
            <div />
            <div />
            <div />
          </OrbitLoader>
        );

      case 'dots':
        return (
          <DotsLoader size={size}>
            <div />
            <div />
            <div />
          </DotsLoader>
        );

      case 'gradient':
        return <GradientSpinner size={size} />;

      case 'spinner':
      default:
        return (
          <CircularProgress
            size={getSize()}
            sx={{
              color: color || designTokens.colors.primary.main,
              '& .MuiCircularProgress-circle': {
                strokeLinecap: 'round',
              },
            }}
          />
        );
    }
  };

  return (
    <LoaderContainer>
      {renderLoader()}
      {text && <LoaderText>{text}</LoaderText>}
    </LoaderContainer>
  );
};

// Export skeleton loader for use in loading states
export const SkeletonBox: React.FC<{ width?: string | number; height?: string | number }> = ({
  width = '100%',
  height = '20px',
}) => {
  return <SkeletonLoader style={{ width, height }} />;
};