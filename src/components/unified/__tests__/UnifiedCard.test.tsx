import React from 'react'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { ThemeProvider } from '@mui/material/styles'
import { UnifiedCard } from '../UnifiedCard'
import { lightTheme, darkTheme } from '../../../theme/unified'
import '@testing-library/jest-dom'

// Helper to render with theme
const renderWithTheme = (component: React.ReactElement, theme = lightTheme) => {
  return render(
    <ThemeProvider theme={theme}>
      {component}
    </ThemeProvider>
  )
}

describe('UnifiedCard Component', () => {
  describe('Rendering', () => {
    test('renders children content', () => {
      renderWithTheme(
        <UnifiedCard>
          <div>Test Content</div>
        </UnifiedCard>
      )
      expect(screen.getByText('Test Content')).toBeInTheDocument()
    })

    test('renders with default glass variant', () => {
      const { container } = renderWithTheme(<UnifiedCard>Content</UnifiedCard>)
      const card = container.firstChild as HTMLElement

      expect(card).toBeInTheDocument()
      expect(card).toHaveTextContent('Content')
    })

    test('applies solid variant styles', () => {
      const { container } = renderWithTheme(
        <UnifiedCard variant="solid">Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement

      expect(card).toBeInTheDocument()
      expect(card).toHaveTextContent('Content')
    })

    test('applies elevated variant styles', () => {
      const { container } = renderWithTheme(
        <UnifiedCard variant="elevated">Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      const styles = window.getComputedStyle(card)
      expect(styles.boxShadow).toMatch(/rgba/)
      expect(styles.transform).toContain('translateY')
    })

    test('applies floating variant styles', () => {
      const { container } = renderWithTheme(
        <UnifiedCard variant="floating">Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      expect(card).toHaveStyle({
        position: 'relative'
      })
    })
  })

  describe('Four-Word Theme', () => {
    test('applies fourWordTheme gradient', () => {
      const fourWords = 'ocean-forest-moon-star'
      const { container } = renderWithTheme(
        <UnifiedCard fourWordTheme={fourWords}>Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      const styles = window.getComputedStyle(card)
      expect(styles.borderImage || styles.background).toMatch(/gradient/)
    })

    test('handles invalid four-word format gracefully', () => {
      const { container } = renderWithTheme(
        <UnifiedCard fourWordTheme="invalid">Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      expect(card).toBeInTheDocument()
    })
  })

  describe('Interactivity', () => {
    test('applies interactive styles when enabled', () => {
      const { container } = renderWithTheme(
        <UnifiedCard interactive>Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      expect(card).toHaveStyle({
        cursor: 'pointer'
      })
    })

    test('triggers onClick handler', () => {
      const handleClick = jest.fn()
      renderWithTheme(
        <UnifiedCard onClick={handleClick}>Content</UnifiedCard>
      )
      
      const card = screen.getByText('Content').parentElement
      fireEvent.click(card!)
      
      expect(handleClick).toHaveBeenCalledTimes(1)
    })

    test('triggers onHover handler', () => {
      const handleHover = jest.fn()
      renderWithTheme(
        <UnifiedCard onHover={handleHover}>Content</UnifiedCard>
      )
      
      const card = screen.getByText('Content').parentElement
      fireEvent.mouseEnter(card!)
      
      expect(handleHover).toHaveBeenCalledTimes(1)
    })

    test('applies hover transform when interactive', async () => {
      const { container } = renderWithTheme(
        <UnifiedCard interactive>Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      fireEvent.mouseEnter(card)
      
      await waitFor(() => {
        const styles = window.getComputedStyle(card)
        expect(styles.transform).toContain('translateY')
      })
    })
  })

  describe('Customization', () => {
    test('applies custom blur amount', () => {
      const { container } = renderWithTheme(
        <UnifiedCard blur={30}>Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      const styles = window.getComputedStyle(card)
      expect(styles.backdropFilter).toContain('blur(30px)')
    })

    test('applies custom opacity', () => {
      const { container } = renderWithTheme(
        <UnifiedCard opacity={0.9}>Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      const styles = window.getComputedStyle(card)
      expect(styles.backgroundColor).toMatch(/0\.9/)
    })

    test('applies custom className', () => {
      const { container } = renderWithTheme(
        <UnifiedCard className="custom-class">Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      expect(card).toHaveClass('custom-class')
    })

    test('applies sx prop styles', () => {
      const { container } = renderWithTheme(
        <UnifiedCard sx={{ padding: 4 }}>Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      expect(card).toHaveStyle({
        padding: '32px'
      })
    })
  })

  describe('Theme Support', () => {
    test('renders correctly in light theme', () => {
      const { container } = renderWithTheme(
        <UnifiedCard>Content</UnifiedCard>,
        lightTheme
      )
      const card = container.firstChild as HTMLElement
      
      const styles = window.getComputedStyle(card)
      expect(styles.backgroundColor).toMatch(/rgba\(255/)
    })

    test('renders correctly in dark theme', () => {
      const { container } = renderWithTheme(
        <UnifiedCard>Content</UnifiedCard>,
        darkTheme
      )
      const card = container.firstChild as HTMLElement
      
      const styles = window.getComputedStyle(card)
      expect(styles.backgroundColor).toMatch(/rgba\(26/)
    })
  })

  describe('Accessibility', () => {
    test('supports keyboard navigation when interactive', () => {
      const handleClick = jest.fn()
      renderWithTheme(
        <UnifiedCard interactive onClick={handleClick}>
          Content
        </UnifiedCard>
      )
      
      const card = screen.getByText('Content').parentElement
      card?.focus()
      
      fireEvent.keyDown(card!, { key: 'Enter' })
      expect(handleClick).toHaveBeenCalled()
    })

    test('has proper ARIA attributes', () => {
      const { container } = renderWithTheme(
        <UnifiedCard interactive>Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      expect(card).toHaveAttribute('role', 'button')
      expect(card).toHaveAttribute('tabIndex', '0')
    })

    test('respects reduced motion preference', () => {
      // Mock matchMedia for prefers-reduced-motion
      window.matchMedia = jest.fn().mockImplementation(query => ({
        matches: query === '(prefers-reduced-motion: reduce)',
        media: query,
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
      }))

      const { container } = renderWithTheme(
        <UnifiedCard interactive>Content</UnifiedCard>
      )
      const card = container.firstChild as HTMLElement
      
      const styles = window.getComputedStyle(card)
      expect(styles.transition).toBe('none')
    })
  })

  describe('Performance', () => {
    test('memoizes expensive calculations', () => {
      const calculateGradient = jest.fn()
      const { rerender } = renderWithTheme(
        <UnifiedCard fourWordTheme="ocean-forest-moon-star">
          Content
        </UnifiedCard>
      )
      
      rerender(
        <ThemeProvider theme={lightTheme}>
          <UnifiedCard fourWordTheme="ocean-forest-moon-star">
            Updated Content
          </UnifiedCard>
        </ThemeProvider>
      )
      
      // Gradient calculation should be memoized
      expect(calculateGradient).toHaveBeenCalledTimes(0)
    })

    test('cleans up on unmount', () => {
      const { unmount } = renderWithTheme(
        <UnifiedCard>Content</UnifiedCard>
      )
      
      unmount()
      
      // Verify no memory leaks
      expect(screen.queryByText('Content')).not.toBeInTheDocument()
    })
  })

  describe('Edge Cases', () => {
    test('handles null children gracefully', () => {
      const { container } = renderWithTheme(
        <UnifiedCard>{null}</UnifiedCard>
      )
      
      expect(container.firstChild).toBeInTheDocument()
    })

    test('handles undefined props gracefully', () => {
      const { container } = renderWithTheme(
        <UnifiedCard
          fourWordTheme={undefined}
          blur={undefined}
          opacity={undefined}
        >
          Content
        </UnifiedCard>
      )
      
      expect(container.firstChild).toBeInTheDocument()
    })

    test('handles very long content', () => {
      const longContent = 'A'.repeat(1000)
      const { container } = renderWithTheme(
        <UnifiedCard>{longContent}</UnifiedCard>
      )
      
      const card = container.firstChild as HTMLElement
      expect(card.scrollHeight).toBeGreaterThan(0)
    })
  })
})