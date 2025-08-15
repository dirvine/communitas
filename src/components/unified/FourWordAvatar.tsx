import React, { useMemo, forwardRef } from 'react'
import {
  Avatar,
  Badge,
  Box,
  Typography,
  Tooltip,
  useTheme,
  keyframes,
} from '@mui/material'
import { generateFourWordGradient } from '../../utils/fourWords'

export interface FourWordAvatarProps {
  fourWords: string
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  showWords?: boolean
  showTooltip?: boolean
  presence?: 'online' | 'away' | 'busy' | 'offline'
  type?: 'personal' | 'organization' | 'project'
  gradient?: 'radial' | 'linear' | 'conic'
  animated?: boolean
  onClick?: () => void
}

const pulse = keyframes`
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
`

const sizeMap = {
  xs: 24,
  sm: 32,
  md: 48,
  lg: 64,
  xl: 96,
}

const presenceColorMap = {
  online: 'success',
  away: 'warning',
  busy: 'error',
  offline: 'default',
} as const

export const FourWordAvatar = forwardRef<HTMLDivElement, FourWordAvatarProps>(
  (
    {
      fourWords,
      size = 'md',
      showWords = false,
      showTooltip = false,
      presence,
      type = 'personal',
      gradient: gradientType = 'linear',
      animated = false,
      onClick,
    },
    ref
  ) => {
    const theme = useTheme()
    const dimension = sizeMap[size]

    // Generate initials from four words
    const initials = useMemo(() => {
      if (!fourWords) return '?'
      const words = fourWords.split('-').filter(Boolean)
      if (words.length === 0) return '?'
      return words.map(w => w[0]?.toUpperCase() || '').join('')
    }, [fourWords])

    // Generate gradient background
    const gradientBackground = useMemo(() => {
      if (!fourWords) return theme.palette.primary.main
      
      const baseGradient = generateFourWordGradient(fourWords)
      
      // Convert to different gradient types
      switch (gradientType) {
        case 'radial':
          return baseGradient.replace('linear-gradient', 'radial-gradient')
        case 'conic':
          return baseGradient.replace('linear-gradient', 'conic-gradient')
        default:
          return baseGradient
      }
    }, [fourWords, gradientType, theme])

    // Get type-specific border
    const getBorder = () => {
      switch (type) {
        case 'organization':
          return '2px solid #FFD700'
        case 'project':
          return '2px solid #C0C0C0'
        default:
          return 'none'
      }
    }

    // Get animation styles
    const getAnimationStyles = () => {
      if (!animated || presence !== 'online') return {}
      return {
        animation: `${pulse} 2s ease-in-out infinite`,
      }
    }

    const avatarContent = (
      <Badge
        overlap="circular"
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        variant="dot"
        color={presence ? presenceColorMap[presence] : 'default'}
        invisible={!presence}
        componentsProps={{
          badge: {
            'aria-label': presence,
          },
        }}
      >
        <Avatar
          ref={ref}
          sx={{
            width: dimension,
            height: dimension,
            background: gradientBackground,
            fontWeight: 600,
            fontSize: dimension / 3,
            border: getBorder(),
            boxShadow: '0 4px 20px rgba(0,0,0,0.15)',
            cursor: onClick ? 'pointer' : 'default',
            ...getAnimationStyles(),
          }}
          onClick={onClick}
          aria-label={fourWords}
        >
          {initials}
        </Avatar>
      </Badge>
    )

    const content = (
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        {showTooltip ? (
          <Tooltip title={fourWords} arrow>
            {avatarContent}
          </Tooltip>
        ) : (
          avatarContent
        )}
        
        {showWords && (
          <Typography
            variant="body2"
            fontWeight={500}
            sx={{
              background: gradientBackground,
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text',
            }}
          >
            {fourWords}
          </Typography>
        )}
      </Box>
    )

    return content
  }
)

FourWordAvatar.displayName = 'FourWordAvatar'