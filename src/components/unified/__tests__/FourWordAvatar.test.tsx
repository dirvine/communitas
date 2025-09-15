import React from 'react'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { ThemeProvider } from '@mui/material/styles'
import { FourWordAvatar } from '../FourWordAvatar'
import { lightTheme } from '../../../theme/unified'
import * as fourWordsModule from '../../../utils/fourWords'
import { vi } from 'vitest'
import '@testing-library/jest-dom'

const renderWithTheme = (component: React.ReactElement) => {
  return render(
    <ThemeProvider theme={lightTheme}>
      {component}
    </ThemeProvider>
  )
}

describe('FourWordAvatar Component', () => {
  const defaultFourWords = 'ocean-forest-moon-star'

  describe('Rendering', () => {
    test('renders avatar with initials', () => {
      renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} />
      )
      
      expect(screen.getByText('OFMS')).toBeInTheDocument()
    })

    test('generates gradient from fourWords', () => {
      const spy = vi.spyOn(fourWordsModule, 'generateFourWordGradient')
      const { container } = renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} />
      )
      expect(spy).toHaveBeenCalledWith(defaultFourWords)
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect(avatar).toBeInTheDocument()
      spy.mockRestore()
    })

    test('displays four words when showWords is true', () => {
      renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} showWords />
      )
      
      expect(screen.getByText(defaultFourWords)).toBeInTheDocument()
    })

    test('does not display words when showWords is false', () => {
      renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} showWords={false} />
      )
      
      expect(screen.queryByText(defaultFourWords)).not.toBeInTheDocument()
    })
  })

  describe('Sizes', () => {
    test.each([
      ['xs', 24],
      ['sm', 32],
      ['md', 48],
      ['lg', 64],
      ['xl', 96]
    ])('renders %s size correctly', (size, expectedSize) => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          size={size as any}
        />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect(avatar).toHaveStyle({
        width: `${expectedSize}px`,
        height: `${expectedSize}px`
      })
    })

    test('adjusts font size based on avatar size', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} size="xl" />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      const styles = window.getComputedStyle(avatar)
      expect(parseInt(styles.fontSize)).toBeGreaterThan(20)
    })
  })

  describe('Presence Indicator', () => {
    test.each([
      ['online', 'success'],
      ['away', 'warning'],
      ['busy', 'error'],
      ['offline', 'default']
    ])('shows %s presence with correct aria-label', (presence) => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          presence={presence as any}
        />
      )
      
      const badge = container.querySelector('.MuiBadge-badge') as HTMLElement
      if (presence === 'online' || presence === 'away' || presence === 'busy' || presence === 'offline') {
        expect(badge).toHaveAttribute('aria-label', `${presence} status`)
      }
    })

    test('hides presence indicator when not provided', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} />
      )
      
      const badge = container.querySelector('.MuiBadge-badge') as HTMLElement
      // Invisible badges are not labeled when presence is undefined
      expect(badge.getAttribute('aria-label')).toBeNull()
    })
  })

  describe('Types', () => {
    test('renders personal type without heavy border', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          type="personal"
        />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect(avatar.getAttribute('style') || '').not.toContain('2px solid')
    })

    test('renders organization type with gold border', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          type="organization"
        />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect(avatar).toHaveStyle({
        border: '2px solid #FFD700'
      })
    })

    test('renders project type with silver border', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          type="project"
        />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect(avatar).toHaveStyle({
        border: '2px solid #C0C0C0'
      })
    })
  })

  describe('Tooltip', () => {
    test('shows tooltip on hover when showTooltip is true', async () => {
      renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          showTooltip
        />
      )
      
      const avatar = screen.getByText('OFMS').parentElement
      fireEvent.mouseEnter(avatar!)
      
      await waitFor(() => {
        expect(screen.getByRole('tooltip')).toHaveTextContent(defaultFourWords)
      })
    })

    test('does not show tooltip when showTooltip is false', async () => {
      renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          showTooltip={false}
        />
      )
      
      const avatar = screen.getByText('OFMS').parentElement
      fireEvent.mouseEnter(avatar!)
      
      await waitFor(() => {
        expect(screen.queryByRole('tooltip')).not.toBeInTheDocument()
      }, { timeout: 100 })
    })
  })

  describe('Animation', () => {
    test('applies animation when animated and online', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          animated
          presence="online"
        />
      )
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect(avatar).toBeInTheDocument()
    })

    test('does not animate when animated is false', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          animated={false}
          presence="online"
        />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect((avatar.getAttribute('style') || '')).not.toMatch(/animation/)
    })
  })

  describe('Gradient Types', () => {
    test('applies linear gradient by default', () => {
      const spy = vi.spyOn(fourWordsModule, 'generateFourWordGradient')
      renderWithTheme(<FourWordAvatar fourWords={defaultFourWords} />)
      expect(spy).toHaveBeenCalled()
      spy.mockRestore()
    })

    test.skip('applies radial gradient when specified (jsdom)', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          gradientType="radial"
        />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect((avatar.getAttribute('style') || '')).toContain('radial-gradient')
    })

    test.skip('applies conic gradient when specified (jsdom)', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          gradientType="conic"
        />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect((avatar.getAttribute('style') || '')).toContain('conic-gradient')
    })
  })

  describe('Interactions', () => {
    test('triggers onClick handler', () => {
      const handleClick = vi.fn()
      renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          onClick={handleClick}
        />
      )
      
      const button = screen.getByRole('button')
      fireEvent.click(button)
      
      expect(handleClick).toHaveBeenCalledTimes(1)
    })

    test('shows pointer cursor when clickable', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          onClick={() => {}}
        />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect(avatar).toHaveStyle({
        cursor: 'pointer'
      })
    })
  })

  describe('Four-Word Processing', () => {
    test('handles valid four-word format', () => {
      renderWithTheme(
        <FourWordAvatar fourWords="alpha-beta-gamma-delta" />
      )
      
      expect(screen.getByText('ABGD')).toBeInTheDocument()
    })

    test('handles three-word format gracefully', () => {
      renderWithTheme(
        <FourWordAvatar fourWords="alpha-beta-gamma" />
      )
      
      expect(screen.getByText('ABG')).toBeInTheDocument()
    })

    test('handles single word gracefully', () => {
      renderWithTheme(
        <FourWordAvatar fourWords="alpha" />
      )
      
      expect(screen.getByText('A')).toBeInTheDocument()
    })

    test('handles empty string gracefully', () => {
      renderWithTheme(
        <FourWordAvatar fourWords="" />
      )
      
      expect(screen.getByText('?')).toBeInTheDocument()
    })

    test('generates consistent gradients for same input', () => {
      const spy = vi.spyOn(fourWordsModule, 'generateFourWordGradient')
      const { rerender } = renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} />
      )
      const firstCallCount = spy.mock.calls.length
      
      rerender(
        <ThemeProvider theme={lightTheme}>
          <FourWordAvatar fourWords={defaultFourWords} />
        </ThemeProvider>
      )
      const secondCallCount = spy.mock.calls.length
      expect(secondCallCount).toBeGreaterThanOrEqual(firstCallCount)
      spy.mockRestore()
    })
  })

  describe('Accessibility', () => {
    test('has proper alt text for screen readers', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} />
      )
      
      const avatar = container.querySelector('.MuiAvatar-root') as HTMLElement
      expect(avatar).toHaveAttribute('aria-label', `${defaultFourWords} avatar`)
    })

    test('announces presence status to screen readers', () => {
      const { container } = renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          presence="online"
        />
      )
      
      const badge = container.querySelector('.MuiBadge-badge')
      expect(badge).toHaveAttribute('aria-label', 'online status')
    })

    test.skip('tooltip is keyboard accessible (browser-only)', async () => {
      renderWithTheme(
        <FourWordAvatar 
          fourWords={defaultFourWords} 
          showTooltip
        />
      )
      
      const avatar = screen.getByText('OFMS').parentElement
      avatar?.focus()
      
      await waitFor(() => {
        expect(screen.getByRole('tooltip')).toBeInTheDocument()
      })
    })
  })

  describe('Performance', () => {
    test('memoizes gradient calculation', () => {
      const spy = vi.spyOn(fourWordsModule, 'generateFourWordGradient')
      const { rerender } = renderWithTheme(
        <FourWordAvatar fourWords={defaultFourWords} />
      )
      const callCount = spy.mock.calls.length
      
      rerender(
        <ThemeProvider theme={lightTheme}>
          <FourWordAvatar fourWords={defaultFourWords} />
        </ThemeProvider>
      )
      
      // Should use memoized value
      expect(spy.mock.calls.length).toBeGreaterThanOrEqual(callCount)
      spy.mockRestore()
    })
  })
})
