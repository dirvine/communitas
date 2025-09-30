import React from 'react';
import { Card, CardProps } from '@mui/material';
import { styled } from '@mui/material/styles';
import { designTokens } from '../../styles/theme';

interface GlassCardProps extends CardProps {
  variant?: 'light' | 'dark' | 'colored' | 'gradient';
  blur?: number;
  hover?: boolean;
  glow?: boolean;
}

const StyledGlassCard = styled(Card, {
  shouldForwardProp: (prop) => !['variant', 'blur', 'hover', 'glow'].includes(prop as string),
})<GlassCardProps>(({ theme, variant = 'light', blur = 20, hover = true, glow }) => ({
  position: 'relative',
  borderRadius: designTokens.borderRadius.xl,
  overflow: 'hidden',
  transition: `all ${designTokens.transitions.smooth}`,

  ...(variant === 'light' && {
    background: theme.palette.mode === 'light'
      ? 'rgba(255, 255, 255, 0.85)'
      : 'rgba(255, 255, 255, 0.08)',
    backdropFilter: `blur(${blur}px) saturate(180%)`,
    WebkitBackdropFilter: `blur(${blur}px) saturate(180%)`,
    border: theme.palette.mode === 'light'
      ? '1px solid rgba(255, 255, 255, 0.5)'
      : '1px solid rgba(255, 255, 255, 0.1)',
    boxShadow: theme.palette.mode === 'light'
      ? '0 8px 32px 0 rgba(31, 38, 135, 0.15)'
      : '0 8px 32px 0 rgba(0, 0, 0, 0.4)',
  }),

  ...(variant === 'dark' && {
    background: theme.palette.mode === 'light'
      ? 'rgba(17, 25, 40, 0.05)'
      : 'rgba(17, 25, 40, 0.75)',
    backdropFilter: `blur(${blur}px) saturate(180%)`,
    WebkitBackdropFilter: `blur(${blur}px) saturate(180%)`,
    border: theme.palette.mode === 'light'
      ? '1px solid rgba(17, 25, 40, 0.1)'
      : '1px solid rgba(255, 255, 255, 0.125)',
    boxShadow: theme.palette.mode === 'light'
      ? '0 8px 32px 0 rgba(31, 38, 135, 0.2)'
      : '0 8px 32px 0 rgba(0, 0, 0, 0.6)',
  }),

  ...(variant === 'colored' && {
    background: theme.palette.mode === 'light'
      ? 'linear-gradient(135deg, rgba(99, 102, 241, 0.15) 0%, rgba(236, 72, 153, 0.15) 100%)'
      : 'linear-gradient(135deg, rgba(99, 102, 241, 0.25) 0%, rgba(236, 72, 153, 0.25) 100%)',
    backdropFilter: `blur(${blur}px) saturate(200%)`,
    WebkitBackdropFilter: `blur(${blur}px) saturate(200%)`,
    border: '1px solid rgba(255, 255, 255, 0.2)',
    boxShadow: '0 8px 32px 0 rgba(99, 102, 241, 0.2)',

    '&::before': {
      content: '""',
      position: 'absolute',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      background: 'linear-gradient(135deg, transparent 30%, rgba(255, 255, 255, 0.1) 100%)',
      pointerEvents: 'none',
    },
  }),

  ...(variant === 'gradient' && {
    background: designTokens.colors.primary.gradient,
    color: '#ffffff',
    border: 'none',
    boxShadow: designTokens.shadows.primary,

    '&::before': {
      content: '""',
      position: 'absolute',
      inset: 0,
      borderRadius: 'inherit',
      padding: '1px',
      background: 'linear-gradient(135deg, rgba(255,255,255,0.4), transparent)',
      WebkitMask: 'linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)',
      WebkitMaskComposite: 'xor',
      maskComposite: 'exclude',
      pointerEvents: 'none',
    },
  }),

  ...(hover && {
    cursor: 'pointer',
    '&:hover': {
      transform: 'translateY(-4px) scale(1.01)',
      boxShadow: variant === 'gradient'
        ? `${designTokens.shadows.primary}, 0 10px 40px rgba(99, 102, 241, 0.3)`
        : theme.palette.mode === 'light'
        ? '0 12px 48px 0 rgba(31, 38, 135, 0.25)'
        : '0 12px 48px 0 rgba(0, 0, 0, 0.7)',
    },
    '&:active': {
      transform: 'translateY(-2px) scale(1.005)',
    },
  }),

  ...(glow && {
    '&::after': {
      content: '""',
      position: 'absolute',
      top: '-50%',
      left: '-50%',
      width: '200%',
      height: '200%',
      background: variant === 'gradient'
        ? 'radial-gradient(circle, rgba(99, 102, 241, 0.4) 0%, transparent 70%)'
        : variant === 'colored'
        ? 'radial-gradient(circle, rgba(236, 72, 153, 0.3) 0%, transparent 70%)'
        : 'radial-gradient(circle, rgba(255, 255, 255, 0.2) 0%, transparent 70%)',
      opacity: 0,
      filter: 'blur(30px)',
      transition: `opacity ${designTokens.transitions.slow}`,
      pointerEvents: 'none',
      zIndex: -1,
    },
    '&:hover::after': {
      opacity: 1,
    },
  }),

  // Animated gradient border effect
  ...(variant === 'colored' && {
    '&::after': {
      content: '""',
      position: 'absolute',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      borderRadius: 'inherit',
      background: 'linear-gradient(45deg, #667eea, #764ba2, #f093fb, #f5576c, #4facfe, #00f2fe)',
      backgroundSize: '300% 300%',
      animation: 'gradientShift 8s ease infinite',
      opacity: 0.3,
      zIndex: -1,
      filter: 'blur(10px)',
    },
    '@keyframes gradientShift': {
      '0%': { backgroundPosition: '0% 50%' },
      '50%': { backgroundPosition: '100% 50%' },
      '100%': { backgroundPosition: '0% 50%' },
    },
  }),
}));

// Floating particles effect component
const FloatingParticles = styled('div')({
  position: 'absolute',
  top: 0,
  left: 0,
  width: '100%',
  height: '100%',
  overflow: 'hidden',
  pointerEvents: 'none',
  '& .particle': {
    position: 'absolute',
    width: '4px',
    height: '4px',
    background: 'rgba(255, 255, 255, 0.6)',
    borderRadius: '50%',
    animation: 'float 20s infinite linear',
  },
  '@keyframes float': {
    from: {
      transform: 'translateY(100vh) translateX(0)',
      opacity: 0,
    },
    '10%': {
      opacity: 1,
    },
    '90%': {
      opacity: 1,
    },
    to: {
      transform: 'translateY(-10vh) translateX(100px)',
      opacity: 0,
    },
  },
});

interface GlassCardContentProps {
  children: React.ReactNode;
  particles?: boolean;
}

export const GlassCardContent: React.FC<GlassCardContentProps> = ({ children, particles = false }) => {
  const particlesRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    if (!particles || !particlesRef.current) return;

    const particleCount = 5;
    const container = particlesRef.current;

    for (let i = 0; i < particleCount; i++) {
      const particle = document.createElement('div');
      particle.className = 'particle';
      particle.style.left = `${Math.random() * 100}%`;
      particle.style.animationDelay = `${Math.random() * 20}s`;
      particle.style.animationDuration = `${15 + Math.random() * 10}s`;
      container.appendChild(particle);
    }

    return () => {
      container.innerHTML = '';
    };
  }, [particles]);

  return (
    <div style={{ position: 'relative', zIndex: 1 }}>
      {particles && <FloatingParticles ref={particlesRef} />}
      {children}
    </div>
  );
};

export const GlassCard = React.forwardRef<HTMLDivElement, GlassCardProps>(
  ({ children, ...props }, ref) => {
    return (
      <StyledGlassCard ref={ref} {...props}>
        {children}
      </StyledGlassCard>
    );
  }
);

GlassCard.displayName = 'GlassCard';