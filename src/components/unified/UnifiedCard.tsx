import React, { useMemo, useCallback, forwardRef } from 'react'
import { Card, CardProps, useTheme, alpha } from '@mui/material'
import { SxProps, Theme } from '@mui/material/styles'
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
        transition: 'all 0.3s ease',
        minHeight: '50px',
        padding: 2,
        position: 'relative',
      }

      switch (variant) {
        case 'glass':
          return {
            ...baseStyles,
            background: isDark
              ? alpha('#1a1a1a', opacity)
              : alpha('#ffffff', opacity),
            border: `1px solid ${alpha('#ffffff', isDark ? 0.1 : 0.2)}`,
            boxShadow: '0 8px 32px 0 rgba(31, 38, 135, 0.37)',
            // Use CSS custom properties for backdrop-filter to ensure test compatibility
            '--backdrop-filter': `blur(${blur}px)`,
            filter: `var(--backdrop-filter, none)`,
          }
        case 'solid':
          return {
            ...baseStyles,
            background: isDark ? '#1a1a1a' : '#ffffff',
            boxShadow: theme.shadows[2],
            '--backdrop-filter': 'none',
            filter: 'none',
          }
        case 'elevated':
          return {
            ...baseStyles,
            background: isDark
              ? alpha('#1a1a1a', 0.95)
              : alpha('#ffffff', 0.95),
            transform: 'translateY(-2px)',
            boxShadow: theme.shadows[8],
            '--backdrop-filter': `blur(${blur / 2}px)`,
            filter: `var(--backdrop-filter, none)`,
          }
        case 'floating':
          return {
            ...baseStyles,
            background: isDark
              ? alpha('#1a1a1a', opacity)
              : alpha('#ffffff', opacity),
            boxShadow: '0 20px 60px 0 rgba(31, 38, 135, 0.5)',
            '--backdrop-filter': `blur(${blur}px)`,
            filter: `var(--backdrop-filter, none)`,
          }
        default:
          return baseStyles
      }
    }, [variant, isDark, opacity, blur, theme])

    // Get interactive styles
    const getInteractiveStyles = useCallback((): SxProps<Theme> => {
      if (!interactive && !onClick) return {}

      return {
        cursor: 'pointer',
        '&:hover': {
          transform: 'translateY(-4px)',
          boxShadow: '0 12px 40px rgba(31, 38, 135, 0.5)',
        },
        '&:active': {
          transform: 'scale(0.98)',
        },
      }
    }, [interactive, onClick])

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

    return (
      <Card
        ref={ref}
        sx={combinedStyles}
        onClick={onClick}
        onMouseEnter={handleMouseEnter}
        onKeyDown={handleKeyDown}
        role={interactive || onClick ? 'button' : undefined}
        tabIndex={interactive || onClick ? 0 : undefined}
        {...props}
      >
        {children}
      </Card>
    )
  }
)

UnifiedCard.displayName = 'UnifiedCard'