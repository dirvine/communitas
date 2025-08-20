import React from 'react';
import {
  Box,
  CircularProgress,
  Typography,
  Paper,
  alpha,
  useTheme,
} from '@mui/material';
import { styled, keyframes } from '@mui/material/styles';

interface LoadingSpinnerProps {
  message?: string;
  size?: 'small' | 'medium' | 'large';
  overlay?: boolean;
  progress?: number; // 0-100 for determinate progress
}

// Animated gradient background
const gradientAnimation = keyframes`
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
`;

const GradientBackground = styled(Box)(({ theme }) => ({
  background: `linear-gradient(-45deg, ${theme.palette.primary.main}, ${theme.palette.secondary.main}, ${theme.palette.primary.light}, ${theme.palette.secondary.light})`,
  backgroundSize: '400% 400%',
  animation: `${gradientAnimation} 15s ease infinite`,
  borderRadius: theme.shape.borderRadius,
}));

const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({
  message = 'Loading...',
  size = 'medium',
  overlay = false,
  progress,
}) => {
  const theme = useTheme();

  const sizeMap = {
    small: { spinner: 24, fontSize: 'body2' },
    medium: { spinner: 40, fontSize: 'body1' },
    large: { spinner: 64, fontSize: 'h6' },
  };

  const { spinner, fontSize } = sizeMap[size];

  const content = (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: 2,
        p: 3,
        minHeight: size === 'large' ? 200 : 120,
      }}
    >
      <CircularProgress
        size={spinner}
        thickness={4}
        variant={progress !== undefined ? 'determinate' : 'indeterminate'}
        value={progress}
        sx={{
          color: theme.palette.primary.main,
          '& .MuiCircularProgress-circle': {
            strokeLinecap: 'round',
          },
        }}
      />
      <Typography
        variant={fontSize as any}
        color="text.secondary"
        align="center"
        sx={{ fontWeight: 500 }}
      >
        {message}
      </Typography>
      {progress !== undefined && (
        <Typography variant="caption" color="text.secondary">
          {Math.round(progress)}%
        </Typography>
      )}
    </Box>
  );

  if (overlay) {
    return (
      <Box
        sx={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: alpha(theme.palette.background.default, 0.8),
          backdropFilter: 'blur(4px)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: theme.zIndex.modal,
        }}
      >
        <Paper
          elevation={6}
          sx={{
            borderRadius: 2,
            overflow: 'hidden',
            minWidth: 280,
          }}
        >
          <GradientBackground>
            {content}
          </GradientBackground>
        </Paper>
      </Box>
    );
  }

  return (
    <GradientBackground>
      {content}
    </GradientBackground>
  );
};

export default LoadingSpinner;