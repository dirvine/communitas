import React, { useMemo, useCallback, forwardRef, useState } from 'react'
import { Card, CardProps, useTheme, alpha, useMediaQuery } from '@mui/material'
import { SxProps, Theme } from '@mui/material/styles'
import { generateFourWordGradient } from '../../utils/fourWords'

export interface UnifiedCardProps extends Omit<CardProps, 'variant'> {
  variant?: 'glass' | 'solid' | 'elevated' | 'floating' | 'minimal'
  fourWordTheme?: string
  interactive?: boolean
  blur?: number
  opacity?: number
  gradient?: boolean
  onHover?: () => void
  animateOnHover?: boolean
  responsive?: boolean
  children: React.ReactNode
}

const UnifiedCardComponent = forwardRef<HTMLDivElement, UnifiedCardProps>(
  (
    {
      variant = 'glass',
      fourWordTheme,
      interactive = false,
      blur = 20,
      opacity = 0.85,
      gradient = false,
      animateOnHover = true,
      responsive = true,
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
    const isMobile = useMediaQuery(theme.breakpoints.down('sm'))
    const [isHovered, setIsHovered] = useState(false)

    // Memoize gradient calculation
    const fourWordGradient = useMemo(() => {
      if (!fourWordTheme) return null
      return generateFourWordGradient(fourWordTheme)
    }, [fourWordTheme])

    // Get variant styles
    const getVariantStyles = useCallback((): SxProps<Theme> => {
      const baseStyles: SxProps<Theme> = {
        borderRadius: responsive ? (isMobile ? 1.5 : 2) : 2,
        transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
        minHeight: responsive ? (isMobile ? '40px' : '50px') : '50px',
        padding: responsive ? (isMobile ? 1.5 : 2) : 2,
        position: 'relative',
        backdropFilter: `blur(${blur}px)`,
        WebkitBackdropFilter: `blur(${blur}px)`,
      }

      switch (variant) {
        case 'glass':
          return {
            ...baseStyles,
            background: isDark
              ? `rgba(26, 26, 26, ${opacity})`
              : `rgba(255, 255, 255, ${opacity})`,
            border: `1px solid ${alpha(isDark ? '#ffffff' : '#000000', isDark ? 0.1 : 0.08)}`,
            boxShadow: isDark
              ? '0 8px 32px 0 rgba(0, 0, 0, 0.37), 0 2px 8px 0 rgba(0, 0, 0, 0.2)'
              : '0 8px 32px 0 rgba(31, 38, 135, 0.37), 0 2px 8px 0 rgba(31, 38, 135, 0.1)',
          }
        case 'solid':
          return {
            ...baseStyles,
            background: isDark ? '#1a1a1a' : '#ffffff',
            border: `1px solid ${theme.palette.divider}`,
            boxShadow: theme.shadows[2],
            backdropFilter: 'none',
            WebkitBackdropFilter: 'none',
          }
        case 'elevated':
          return {
            ...baseStyles,
            background: isDark
              ? `rgba(26, 26, 26, ${Math.min(opacity + 0.1, 1)})`
              : `rgba(255, 255, 255, ${Math.min(opacity + 0.1, 1)})`,
            border: `1px solid ${alpha(isDark ? '#ffffff' : '#000000', isDark ? 0.08 : 0.06)}`,
            transform: 'translateY(-2px)',
            boxShadow: isDark
              ? '0 12px 40px 0 rgba(0, 0, 0, 0.4), 0 4px 16px 0 rgba(0, 0, 0, 0.2)'
              : '0 12px 40px 0 rgba(31, 38, 135, 0.4), 0 4px 16px 0 rgba(31, 38, 135, 0.15)',
          }
        case 'floating':
          return {
            ...baseStyles,
            background: isDark
              ? `rgba(26, 26, 26, ${opacity})`
              : `rgba(255, 255, 255, ${opacity})`,
            border: `1px solid ${alpha(isDark ? '#ffffff' : '#000000', isDark ? 0.12 : 0.1)}`,
            boxShadow: isDark
              ? '0 20px 60px 0 rgba(0, 0, 0, 0.5), 0 8px 24px 0 rgba(0, 0, 0, 0.3)'
              : '0 20px 60px 0 rgba(31, 38, 135, 0.5), 0 8px 24px 0 rgba(31, 38, 135, 0.2)',
          }
        case 'minimal':
          return {
            ...baseStyles,
            background: 'transparent',
            border: `1px solid ${alpha(theme.palette.divider, 0.5)}`,
            boxShadow: 'none',
            backdropFilter: 'none',
            WebkitBackdropFilter: 'none',
            padding: responsive ? (isMobile ? 1 : 1.5) : 1.5,
          }
        default:
          return baseStyles
      }
    }, [variant, isDark, opacity, blur, theme, responsive, isMobile])

    // Get interactive styles
    const getInteractiveStyles = useCallback((): SxProps<Theme> => {
      if (!interactive && !onClick) return {}

      const hoverTransform = animateOnHover ? 'translateY(-4px) scale(1.02)' : 'translateY(-2px)'
      const hoverShadow = isDark
        ? '0 16px 48px rgba(0, 0, 0, 0.4), 0 8px 24px rgba(0, 0, 0, 0.2)'
        : '0 16px 48px rgba(31, 38, 135, 0.4), 0 8px 24px rgba(31, 38, 135, 0.15)'

      return {
        cursor: 'pointer',
        userSelect: 'none',
        '&:hover': {
          transform: hoverTransform,
          boxShadow: hoverShadow,
        },
        '&:active': {
          transform: 'scale(0.98)',
          transition: 'transform 0.1s ease',
        },
        '&:focus-visible': {
          outline: `2px solid ${theme.palette.primary.main}`,
          outlineOffset: 2,
        },
      }
    }, [interactive, onClick, animateOnHover, isDark, theme])

    // Combine all styles
    const combinedStyles = useMemo((): SxProps<Theme> => {
      const styles = {
        ...getVariantStyles(),
        ...getInteractiveStyles(),
      } as Record<string, any>

      // Apply four-word gradient
      if (fourWordGradient && gradient) {
        styles.background = fourWordGradient
      }

      // Merge with custom sx prop
      return sx ? { ...styles, ...sx } : styles
    }, [getVariantStyles, getInteractiveStyles, fourWordGradient, gradient, sx])

    // Handle hover
    const handleMouseEnter = useCallback(() => {
      setIsHovered(true)
      onHover?.()
    }, [onHover])

    const handleMouseLeave = useCallback(() => {
      setIsHovered(false)
    }, [])

    // Handle keyboard interaction
    const handleKeyDown = useCallback(
      (event: React.KeyboardEvent) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault()
          if (onClick) {
            onClick(event as any)
          }
        }
      },
      [onClick]
    )

    return (
      <Card
        ref={ref}
        sx={combinedStyles}
        onClick={onClick}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        onKeyDown={handleKeyDown}
        role={interactive || onClick ? 'button' : undefined}
        tabIndex={interactive || onClick ? 0 : undefined}
        aria-label={interactive || onClick ? 'Interactive card' : undefined}
        aria-pressed={interactive || onClick ? isHovered : undefined}
        aria-describedby={fourWordTheme ? `card-description-${fourWordTheme}` : undefined}
        {...props}
      >
        {children}
      </Card>
    )
  }
)

UnifiedCardComponent.displayName = 'UnifiedCard'

export const UnifiedCard = React.memo(UnifiedCardComponent)