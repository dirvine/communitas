import { describe, it, expect, beforeEach, vi } from 'vitest'
import {
  generateFourWordIdentity,
  validateFourWordIdentity,
} from '../identity'

// Mock the tauri safeInvoke utility
vi.mock('../tauri', () => ({
  safeInvoke: vi.fn(),
}))

import { safeInvoke } from '../tauri'

describe('identity utilities', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('generateFourWordIdentity', () => {
    it('generates valid four-word identity from backend', async () => {
      const mockIdentity = 'ocean-forest-moon-star'
      ;(safeInvoke as any).mockResolvedValue(mockIdentity)

      const result = await generateFourWordIdentity()

      expect(safeInvoke).toHaveBeenCalledWith('generate_four_word_identity', undefined)
      expect(result).toBe(mockIdentity)
    })

    it('uses fallback when backend unavailable', async () => {
      ;(safeInvoke as any).mockResolvedValue(null)

      const result = await generateFourWordIdentity()

      expect(result).toBe('test-identity-not-valid')
    })

    it('passes seed parameter to backend', async () => {
      const mockIdentity = 'ocean-forest-moon-star'
      ;(safeInvoke as any).mockResolvedValue(mockIdentity)

      const result = await generateFourWordIdentity('test-seed')

      expect(safeInvoke).toHaveBeenCalledWith('generate_four_word_identity', { seed: 'test-seed' })
      expect(result).toBe(mockIdentity)
    })
  })

  describe('validateFourWordIdentity', () => {
    it('validates correct four-word identity', async () => {
      ;(safeInvoke as any).mockResolvedValue(true)

      const result = await validateFourWordIdentity('ocean-forest-moon-star')

      expect(safeInvoke).toHaveBeenCalledWith('validate_four_word_identity', { four_words: 'ocean-forest-moon-star' })
      expect(result).toBe(true)
    })

    it('rejects invalid four-word identity', async () => {
      ;(safeInvoke as any).mockResolvedValue(false)

      const result = await validateFourWordIdentity('invalid-identity')

      expect(result).toBe(false)
    })

    it('normalizes spaces to dashes', async () => {
      ;(safeInvoke as any).mockResolvedValue(true)

      const result = await validateFourWordIdentity('ocean forest moon star')

      expect(safeInvoke).toHaveBeenCalledWith('validate_four_word_identity', { four_words: 'ocean-forest-moon-star' })
      expect(result).toBe(true)
    })

    it('handles case insensitive validation', async () => {
      ;(safeInvoke as any).mockResolvedValue(true)

      const result = await validateFourWordIdentity('OCEAN-FOREST-MOON-STAR')

      expect(safeInvoke).toHaveBeenCalledWith('validate_four_word_identity', { four_words: 'ocean-forest-moon-star' })
      expect(result).toBe(true)
    })

    it('uses fallback validation when backend unavailable', async () => {
      ;(safeInvoke as any).mockResolvedValue(null)

      const result = await validateFourWordIdentity('ocean-forest-moon-star')

      expect(result).toBe(true) // Fallback regex validation
    })

    it('fallback rejects malformed identities', async () => {
      ;(safeInvoke as any).mockResolvedValue(null)

      const result = await validateFourWordIdentity('not-four-words')

      expect(result).toBe(false)
    })

    it('handles whitespace trimming', async () => {
      ;(safeInvoke as any).mockResolvedValue(true)

      const result = await validateFourWordIdentity('  ocean-forest-moon-star  ')

      expect(safeInvoke).toHaveBeenCalledWith('validate_four_word_identity', { four_words: 'ocean-forest-moon-star' })
      expect(result).toBe(true)
    })
  })

  describe('integration tests', () => {
    it('generates and validates identity end-to-end', async () => {
      const mockIdentity = 'ocean-forest-moon-star'

      ;(safeInvoke as any)
        .mockResolvedValueOnce(mockIdentity) // generate
        .mockResolvedValueOnce(true) // validate

      // Generate
      const generated = await generateFourWordIdentity()
      expect(generated).toBe(mockIdentity)

      // Validate
      const isValid = await validateFourWordIdentity(generated)
      expect(isValid).toBe(true)
    })

    it('handles backend unavailability gracefully', async () => {
      ;(safeInvoke as any).mockResolvedValue(null)

      // Generate falls back
      const generated = await generateFourWordIdentity()
      expect(generated).toBe('test-identity-not-valid')

      // Validate uses regex fallback
      const isValid = await validateFourWordIdentity('ocean-forest-moon-star')
      expect(isValid).toBe(true)
    })
  })
})
