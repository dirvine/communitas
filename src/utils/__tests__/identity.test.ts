import { describe, it, expect, beforeEach, vi } from 'vitest'
import {
  generateFourWordIdentity,
  validateFourWordIdentity,
  parseIdentity,
  identityToSeed,
  isFourWordValid,
  sanitizeIdentity,
} from '../identity'
import { fourWordsLib } from '../fourWords'

// Mock the four words library
const mockFourWordsLib = {
  generate_four_words: vi.fn(),
  four_words_to_bytes: vi.fn(),
  bytes_to_four_words: vi.fn(),
  validate: vi.fn(),
}

vi.mock('../fourWords', () => ({
  fourWordsLib: mockFourWordsLib
}))

describe('identity utilities', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('generateFourWordIdentity', () => {
    it('generates valid four-word identity', async () => {
      const mockIdentity = 'ocean-forest-moon-star'
      mockFourWordsLib.generate_four_words.mockReturnValue(mockIdentity)

      const result = await generateFourWordIdentity()

      expect(mockFourWordsLib.generate_four_words).toHaveBeenCalled()
      expect(result).toBe(mockIdentity)
    })

    it('handles generation failure', async () => {
      mockFourWordsLib.generate_four_words.mockImplementation(() => {
        throw new Error('Generation failed')
      })

      await expect(generateFourWordIdentity()).rejects.toThrow('Generation failed')
    })
  })

  describe('validateFourWordIdentity', () => {
    it('validates correct four-word identity', async () => {
      const validIdentity = 'ocean-forest-moon-star'
      mockFourWordsLib.validate.mockReturnValue(true)

      const result = await validateFourWordIdentity(validIdentity)

      expect(mockFourWordsLib.validate.mockReturnValue(true)
      expect(result).toBe(true)
    })

    it('rejects invalid four-word identity', async () => {
      const invalidIdentity = 'invalid-identity'
      mockFourWordsLib.validate.mockReturnValue(false)

      const result = await validateFourWordIdentity(invalidIdentity)

      expect(result).toBe(false)
    })

    it('handles validation errors gracefully', async () => {
      const identity = 'test-identity'
      mockFourWordsLib.validate.mockImplementation(() => {
        throw new Error('Validation error')
      })

      const result = await validateFourWordIdentity(identity)

      expect(result).toBe(false)
    })
  })

  describe('parseIdentity', () => {
    it('parses valid four-word identity', async () => {
      const identity = 'ocean-forest-moon-star'
      const mockBytes = new Uint8Array([1, 2, 3, 4])
      mockFourWordsLib.four_words_to_bytes.mockReturnValue(mockBytes)

      const result = await parseIdentity(identity)

      expect(mockFourWordsLib.four_words_to_bytes).toHaveBeenCalledWith(identity)
      expect(result).toEqual(mockBytes)
    })

    it('handles parsing failure', async () => {
      const identity = 'invalid-identity'
      mockFourWordsLib.four_words_to_bytes.mockImplementation(() => {
        throw new Error('Parse error')
      })

      await expect(parseIdentity(identity)).rejects.toThrow('Parse error')
    })
  })

  describe('identityToSeed', () => {
    it('converts identity to seed', async () => {
      const identity = 'ocean-forest-moon-star'
      const mockBytes = new Uint8Array([1, 2, 3, 4])
      const mockSeed = new Uint8Array([5, 6, 7, 8])
      
      mockFourWordsLib.four_words_to_bytes.mockReturnValue(mockBytes)
      
      // Mock webcrypto subtle digest
      const mockCrypto = {
        subtle: {
          digest: vi.fn().mockResolvedValue(mockSeed.buffer)
        }
      }
      Object.defineProperty(global, 'crypto', {
        value: mockCrypto,
        writable: true
      })

      const result = await identityToSeed(identity)

      expect(mockFourWordsLib.four_words_to_bytes).toHaveBeenCalledWith(identity)
      expect(mockCrypto.subtle.digest).toHaveBeenCalledWith('SHA-256', mockBytes)
      expect(result).toEqual(mockSeed)
    })
  })

  describe('isFourWordValid', () => {
    it('returns true for valid four-word identities', async () => {
      const validIdentity = 'ocean-forest-moon-star'
      mockFourWordsLib.validate.mockReturnValue(true)

      const result = await isFourWordValid(validIdentity)

      expect(result).toBe(true)
    })

    it('returns false for invalid identities', async () => {
      const invalidIdentity = 'not-four-words'
      mockFourWordsLib.validate.mockReturnValue(false)

      const result = await isFourWordValid(invalidIdentity)

      expect(result).toBe(false)
    })

    it('handles non-string inputs', async () => {
      const result = await isFourWordValid(null as any)
      expect(result).toBe(false)
    })
  })

  describe('sanitizeIdentity', () => {
    it('trims whitespace', () => {
      const identity = '  ocean-forest-moon-star  '
      const result = sanitizeIdentity(identity)
      expect(result).toBe('ocean-forest-moon-star')
    })

    it('converts to lowercase', () => {
      const identity = 'OCEAN-FOREST-MOON-STAR'
      const result = sanitizeIdentity(identity)
      expect(result).toBe('ocean-forest-moon-star')
    })

    it('handles empty string', () => {
      const result = sanitizeIdentity('')
      expect(result).toBe('')
    })

    it('handles null/undefined', () => {
      expect(sanitizeIdentity(null as any)).toBe('')
      expect(sanitizeIdentity(undefined as any)).toBe('')
    })

    it('removes extra spaces', () => {
      const identity = '  ocean   forest  moon  star  '
      const result = sanitizeIdentity(identity)
      expect(result).toBe('ocean-forest-moon-star')
    })
  })

  describe('edge cases and error handling', () => {
    it('handles extremely long identities', async () => {
      const longIdentity = 'a'.repeat(1000)
      mockFourWordsLib.validate.mockReturnValue(false)

      const result = await isFourWordValid(longIdentity)
      expect(result).toBe(false)
    })

    it('handles identities with special characters', () => {
      const identityWithSpecial = 'ocean@forest#moon$star'
      const result = sanitizeIdentity(identityWithSpecial)
      expect(result).toBe('oceanforestmoonstar')
    })

    it('handles repeated validation calls', async () => {
      const identity = 'ocean-forest-moon-star'
      mockFourWordsLib.validate.mockReturnValue(true)

      await isFourWordValid(identity)
      await isFourWordValid(identity)

      expect(mockFourWordsLib.validate).toHaveBeenCalledTimes(2)
    })
  })

  describe('integration tests', () => {
    it('generates and validates identity end-to-end', async () => {
      const identity = 'ocean-forest-moon-star'
      const mockBytes = new Uint8Array([1, 2, 3, 4])
      
      mockFourWordsLib.generate_four_words.mockReturnValue(identity)
      mockFourWordsLib.validate.mockReturnValue(true)
      mockFourWordsLib.four_words_to_bytes.mockReturnValue(mockBytes)

      // Generate
      const generated = await generateFourWordIdentity()
      
      // Validate
      const isValid = await validateFourWordIdentity(generated)
      
      // Parse
      const parsed = await parseIdentity(generated)
      
      expect(generated).toBe(identity)
      expect(isValid).toBe(true)
      expect(parsed).toEqual(mockBytes)
    })
  })
})
