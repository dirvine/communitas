import { keyframes } from '@mui/material/styles'

// Keyframe animations
export const fadeIn = keyframes`
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
`

export const slideIn = keyframes`
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(0);
  }
`

export const slideUp = keyframes`
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
`

export const pulse = keyframes`
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
`

export const shimmer = keyframes`
  0% {
    background-position: -1000px 0;
  }
  100% {
    background-position: 1000px 0;
  }
`

export const rotate = keyframes`
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
`

export const bounce = keyframes`
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
`

export const glow = keyframes`
  0%, 100% {
    box-shadow: 0 0 20px rgba(147, 51, 234, 0.3);
  }
  50% {
    box-shadow: 0 0 40px rgba(147, 51, 234, 0.6);
  }
`

// Animation presets
export const UnifiedAnimations = {
  // Page transitions
  pageTransition: {
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: -20 },
    transition: { duration: 0.3, ease: 'easeInOut' },
  },

  // Card animations
  cardHover: {
    whileHover: {
      scale: 1.02,
      y: -4,
      transition: { duration: 0.2 },
    },
    whileTap: { scale: 0.98 },
  },

  // Message bubble animation
  messageBubble: {
    initial: { scale: 0.8, opacity: 0 },
    animate: { scale: 1, opacity: 1 },
    transition: { type: 'spring', stiffness: 500, damping: 30 },
  },

  // Navigation slide
  navigationSlide: {
    initial: { x: -320 },
    animate: { x: 0 },
    exit: { x: -320 },
    transition: { type: 'tween', duration: 0.3 },
  },

  // Modal/Dialog
  modalOverlay: {
    initial: { opacity: 0 },
    animate: { opacity: 1 },
    exit: { opacity: 0 },
    transition: { duration: 0.2 },
  },

  modalContent: {
    initial: { scale: 0.9, opacity: 0 },
    animate: { scale: 1, opacity: 1 },
    exit: { scale: 0.9, opacity: 0 },
    transition: { type: 'spring', stiffness: 300, damping: 30 },
  },

  // Button press
  buttonPress: {
    whileTap: { scale: 0.98 },
    transition: { duration: 0.1 },
  },

  // List item
  listItem: {
    initial: { opacity: 0, x: -20 },
    animate: { opacity: 1, x: 0 },
    transition: { duration: 0.2 },
  },

  // Stagger children
  staggerContainer: {
    animate: {
      transition: {
        staggerChildren: 0.1,
      },
    },
  },

  staggerItem: {
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
  },

  // Glassmorphism effect
  glassEffect: {
    whileHover: {
      backdropFilter: 'blur(25px)',
      background: 'rgba(255, 255, 255, 0.9)',
      transition: { duration: 0.2 },
    },
  },

  // Loading skeleton
  skeleton: {
    animation: `${shimmer} 2s infinite linear`,
    background: 'linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%)',
    backgroundSize: '1000px 100%',
  },

  // Presence indicator
  presenceIndicator: {
    online: {
      animation: `${pulse} 2s ease-in-out infinite`,
    },
    away: {
      animation: `${pulse} 4s ease-in-out infinite`,
    },
    busy: {
      animation: 'none',
    },
    offline: {
      animation: 'none',
    },
  },

  // Floating action button
  fabExpand: {
    initial: { scale: 0, opacity: 0 },
    animate: { scale: 1, opacity: 1 },
    exit: { scale: 0, opacity: 0 },
    transition: { type: 'spring', stiffness: 500, damping: 30 },
  },

  // Notification
  notification: {
    initial: { x: 400, opacity: 0 },
    animate: { x: 0, opacity: 1 },
    exit: { x: 400, opacity: 0 },
    transition: { type: 'spring', stiffness: 400, damping: 40 },
  },

  // Tooltip
  tooltip: {
    initial: { opacity: 0, y: 10 },
    animate: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: 10 },
    transition: { duration: 0.15 },
  },

  // Progress bar
  progressBar: {
    initial: { scaleX: 0 },
    animate: { scaleX: 1 },
    transition: { duration: 0.5, ease: 'easeOut' },
  },

  // Typing indicator
  typingDot: {
    animate: {
      scale: [1, 1.2, 1],
      opacity: [0.5, 1, 0.5],
    },
    transition: {
      duration: 1.5,
      repeat: Infinity,
      repeatDelay: 0.2,
    },
  },
}

// CSS animation classes
export const animationClasses = {
  fadeIn: {
    animation: `${fadeIn} 0.3s ease-out`,
  },
  slideIn: {
    animation: `${slideIn} 0.3s ease-out`,
  },
  slideUp: {
    animation: `${slideUp} 0.3s ease-out`,
  },
  pulse: {
    animation: `${pulse} 2s ease-in-out infinite`,
  },
  shimmer: {
    animation: `${shimmer} 2s infinite linear`,
  },
  rotate: {
    animation: `${rotate} 1s linear infinite`,
  },
  bounce: {
    animation: `${bounce} 1s ease-in-out infinite`,
  },
  glow: {
    animation: `${glow} 2s ease-in-out infinite`,
  },
}

// Duration presets
export const durations = {
  instant: 0,
  fast: 150,
  normal: 300,
  slow: 500,
  slower: 750,
  slowest: 1000,
}

// Easing functions
export const easings = {
  easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
  easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
  easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
  spring: 'cubic-bezier(0.175, 0.885, 0.32, 1.275)',
  bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
}