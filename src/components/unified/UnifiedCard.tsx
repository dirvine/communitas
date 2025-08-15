import React, { useMemo, useCallback, forwardRef } from 'react'
import { Card, CardProps, useTheme, alpha } from '@mui/material'
import { SxProps, Theme } from '@mui/material/styles'
import { motion, HTMLMotionProps } from 'framer-motion'
import { generateFourWordGradient } from '../../utils/fourWords'

export interface UnifiedCardProps extends Omit<CardProps, 'variant'> {
  variant?: 'glass' | 'solid' | 'elevated' | 'floating'
  fourWordTheme?: string
  interactive?: boolean
  blur?: number
  opacity?: number
  gradient?: boolean
  onHover?: () => void
  children: React.ReactNode
}

const MotionCard = motion(Card)
const MotionCardAny = MotionCard as any

export const UnifiedCard = forwardRef<HTMLDivElement, UnifiedCardProps>(
  (
    {
      variant = 'glass',
      fourWordTheme,
      interactive = false,
      blur = 20,
      opacity = 0.85,
      gradient = false,
      onHover,
      onClick,
      children,
      sx,
      ...props
    },
    ref
  ) => {
    const theme = useTheme()
    const isDark = theme.palette.mode === 'dark'

    // Memoize gradient calculation
    const fourWordGradient = useMemo(() => {
      if (!fourWordTheme) return null
      return generateFourWordGradient(fourWordTheme)
    }, [fourWordTheme])

    // Get variant styles
    const getVariantStyles = useCallback((): SxProps<Theme> => {
      const baseStyles: SxProps<Theme> = {
        borderRadius: 2,
        transition: theme.transitions.create(
          ['transform', 'box-shadow', 'backdrop-filter'],
          { duration: 300 }
        ),
      }

      switch (variant) {
        case 'glass':
          return {
            ...baseStyles,
            background: isDark
              ? alpha('#1a1a1a', opacity)
              : alpha('#ffffff', opacity),
            backdropFilter: `blur(${blur}px)`,
            WebkitBackdropFilter: `blur(${blur}px)`,
            border: `1px solid ${alpha('#ffffff', isDark ? 0.1 : 0.2)}`,
            boxShadow: '0 8px 32px 0 rgba(31, 38, 135, 0.37)',
          }
        case 'solid':
          return {
            ...baseStyles,
            background: isDark ? '#1a1a1a' : '#ffffff',
            backdropFilter: 'none',
            boxShadow: theme.shadows[2],
          }
        case 'elevated':
          return {
            ...baseStyles,
            background: isDark
              ? alpha('#1a1a1a', 0.95)
              : alpha('#ffffff', 0.95),
            backdropFilter: `blur(${blur / 2}px)`,
            WebkitBackdropFilter: `blur(${blur / 2}px)`,
            transform: 'translateY(-2px)',
            boxShadow: theme.shadows[8],
          }
        case 'floating':
          return {
            ...baseStyles,
            position: 'relative',
            background: isDark
              ? alpha('#1a1a1a', opacity)
              : alpha('#ffffff', opacity),
            backdropFilter: `blur(${blur}px)`,
            WebkitBackdropFilter: `blur(${blur}px)`,
            boxShadow: '0 20px 60px 0 rgba(31, 38, 135, 0.5)',
          }
        default:
          return baseStyles
      }
    }, [variant, isDark, opacity, blur, theme])

    // Get interactive styles
    const getInteractiveStyles = useCallback((): SxProps<Theme> => {
      if (!interactive && !onClick) return {}

      const prefersReducedMotion = window.matchMedia?.(
        '(prefers-reduced-motion: reduce)'
      ).matches

      if (prefersReducedMotion) {
        return {
          cursor: 'pointer',
          transition: 'none',
        }
      }

      return {
        cursor: 'pointer',
        '&:hover': {
          transform: 'translateY(-4px)',
          backdropFilter: `blur(${blur + 5}px)`,
          WebkitBackdropFilter: `blur(${blur + 5}px)`,
          boxShadow: '0 12px 40px rgba(31, 38, 135, 0.5)',
        },
        '&:active': {
          transform: 'scale(0.98)',
        },
      }
    }, [interactive, onClick, blur])

    // Combine all styles
    const combinedStyles = useMemo((): SxProps<Theme> => {
      const styles: Record<string, any> = {
        ...getVariantStyles(),
        ...getInteractiveStyles(),
      } as Record<string, any>

      // Apply four-word gradient
      if (fourWordGradient) {
        if (gradient) {
          styles['background'] = fourWordGradient
        } else {
          styles['borderImage'] = fourWordGradient
          styles['borderImageSlice'] = 1
        }
      }

      return { ...(styles as any), ...(sx as any) }
    }, [getVariantStyles, getInteractiveStyles, fourWordGradient, gradient, sx])

    // Handle hover
    const handleMouseEnter = useCallback(() => {
      onHover?.()
    }, [onHover])

    // Handle keyboard interaction
    const handleKeyDown = useCallback(
      (event: React.KeyboardEvent) => {
        if (event.key === 'Enter' && onClick) {
          onClick(event as any)
        }
      },
      [onClick]
    )

    // Motion props for animations
    const motionProps: HTMLMotionProps<'div'> = interactive
      ? {
          whileHover: { scale: 1.02 },
          whileTap: { scale: 0.98 },
          transition: { type: 'spring', stiffness: 400, damping: 30 },
        }
      : {}

    return (
      <MotionCardAny
        ref={ref as any}
        sx={combinedStyles}
        onClick={onClick}
        onMouseEnter={handleMouseEnter}
        onKeyDown={handleKeyDown}
        role={interactive || onClick ? 'button' : undefined}
        tabIndex={interactive || onClick ? 0 : undefined}
        {...motionProps}
        {...props}
      >
        {children}
      </MotionCardAny>
    )
  }
)

UnifiedCard.displayName = 'UnifiedCard'