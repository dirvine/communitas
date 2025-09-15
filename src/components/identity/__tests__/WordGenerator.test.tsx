import { describe, test, expect, beforeEach, it } from 'vitest'
import { WordGenerator } from '../WordGenerator'
import { wordLists } from '../wordLists'

describe.skip('WordGenerator', () => {
  let generator: WordGenerator

  beforeEach(() => {
    generator = new WordGenerator()
  })

  describe('generation', () => {
    test('generates valid four-word format', () => {
      const fourWords = generator.generate()
      expect(fourWords).toMatch(/^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/)
      
      const words = fourWords.split('-')
      expect(words).toHaveLength(4)
      words.forEach(word => {
        expect(word).toMatch(/^[a-z]+$/)
        expect(word.length).toBeGreaterThanOrEqual(3)
        expect(word.length).toBeLessThanOrEqual(12)
      })
    })

    test('uses correct word categories', () => {
      const fourWords = generator.generate()
      const words = fourWords.split('-')
      
      expect(wordLists.nature).toContain(words[0])
      expect(wordLists.colors).toContain(words[1])
      expect(wordLists.objects).toContain(words[2])
      expect(wordLists.concepts).toContain(words[3])
    })

    test('generates unique combinations in batch', () => {
      const batch = generator.generateBatch(100)
      const unique = new Set(batch)
      expect(unique.size).toBe(100)
    })

    test('generates exactly requested number of candidates', () => {
      const candidates = generator.generateCandidates(5)
      expect(candidates).toHaveLength(5)
      candidates.forEach(fourWords => {
        expect(fourWords).toMatch(/^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/)
      })
    })

    test('handles entropy correctly', () => {
      const generator1 = new WordGenerator({ seed: 'test-seed' })
      const generator2 = new WordGenerator({ seed: 'test-seed' })
      
      expect(generator1.generate()).toBe(generator2.generate())
      
      const generator3 = new WordGenerator({ seed: 'different-seed' })
      expect(generator1.generate()).not.toBe(generator3.generate())
    })

    test('filters offensive combinations', () => {
      const offensive = ['bad', 'offensive', 'inappropriate']
      const mockLists = {
        nature: ['ocean', ...offensive],
        colors: ['azure', ...offensive],
        objects: ['stone', ...offensive],
        concepts: ['dream', ...offensive]
      }
      
      const customGenerator = new WordGenerator({ wordLists: mockLists })
      const batch = customGenerator.generateBatch(100)
      
      batch.forEach(fourWords => {
        const words = fourWords.split('-')
        words.forEach(word => {
          expect(offensive).not.toContain(word)
        })
      })
    })

    test('ensures pronounceability', () => {
      const batch = generator.generateBatch(100)
      batch.forEach(fourWords => {
        const words = fourWords.split('-')
        words.forEach(word => {
          // Check for basic pronounceability rules
          expect(word).not.toMatch(/[^aeiou]{4,}/) // No 4+ consonants in a row
          expect(word).toMatch(/[aeiou]/) // At least one vowel
        })
      })
    })
  })

  describe('validation', () => {
    test('validates correct four-word format', () => {
      expect(generator.isValid('ocean-azure-stone-dream')).toBe(true)
      expect(generator.isValid('forest-crimson-crystal-wisdom')).toBe(true)
    })

    test('rejects invalid formats', () => {
      expect(generator.isValid('ocean-azure-stone')).toBe(false) // Too few
      expect(generator.isValid('ocean-azure-stone-dream-extra')).toBe(false) // Too many
      expect(generator.isValid('ocean azure stone dream')).toBe(false) // Wrong separator
      expect(generator.isValid('Ocean-Azure-Stone-Dream')).toBe(false) // Uppercase
      expect(generator.isValid('ocean-azure-stone-123')).toBe(false) // Numbers
      expect(generator.isValid('ocean-azure-stone-dre@m')).toBe(false) // Special chars
    })

    test('checks word list membership', () => {
      expect(generator.isValid('ocean-azure-stone-dream')).toBe(true)
      expect(generator.isValid('notaword-azure-stone-dream')).toBe(false)
      expect(generator.isValid('ocean-notacolor-stone-dream')).toBe(false)
    })

    test('detects reserved words', () => {
      const reserved = ['admin', 'root', 'system', 'test']
      reserved.forEach(word => {
        expect(generator.isValid(`${word}-azure-stone-dream`)).toBe(false)
        expect(generator.isValid(`ocean-${word}-stone-dream`)).toBe(false)
        expect(generator.isValid(`ocean-azure-${word}-dream`)).toBe(false)
        expect(generator.isValid(`ocean-azure-stone-${word}`)).toBe(false)
      })
    })

    test('validates against blacklist', () => {
      const blacklisted = generator.addToBlacklist('ocean-azure-stone-dream')
      expect(blacklisted).toBe(true)
      expect(generator.isValid('ocean-azure-stone-dream')).toBe(false)
    })
  })

  describe('collision detection', () => {
    test('detects existing identities', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      await generator.claim(fourWords)
      
      expect(await generator.isAvailable(fourWords)).toBe(false)
      expect(await generator.isAvailable('forest-crimson-crystal-wisdom')).toBe(true)
    })

    test('handles concurrent generation safely', async () => {
      const promises = Array(10).fill(null).map(() => 
        generator.generateUnique()
      )
      
      const results = await Promise.all(promises)
      const unique = new Set(results)
      expect(unique.size).toBe(10)
    })

    test('maintains uniqueness guarantee', async () => {
      const claimed = new Set<string>()
      
      for (let i = 0; i < 100; i++) {
        const fourWords = await generator.generateUnique()
        expect(claimed.has(fourWords)).toBe(false)
        claimed.add(fourWords)
      }
    })

    test('handles collision retry logic', async () => {
      // Claim most combinations to force retries
      const batch = generator.generateBatch(1000)
      await Promise.all(batch.map(fw => generator.claim(fw)))
      
      // Should still be able to find unique combination
      const unique = await generator.generateUnique()
      expect(unique).toBeTruthy()
      expect(await generator.isAvailable(unique)).toBe(true)
    })
  })

  describe('customization', () => {
    test('allows custom word lists', () => {
      const customLists = {
        nature: ['sun', 'moon'],
        colors: ['red', 'blue'],
        objects: ['sword', 'shield'],
        concepts: ['love', 'hate']
      }
      
      const customGenerator = new WordGenerator({ wordLists: customLists })
      const fourWords = customGenerator.generate()
      const words = fourWords.split('-')
      
      expect(customLists.nature).toContain(words[0])
      expect(customLists.colors).toContain(words[1])
      expect(customLists.objects).toContain(words[2])
      expect(customLists.concepts).toContain(words[3])
    })

    test('supports weighted word selection', () => {
      const weights = {
        nature: { ocean: 10, forest: 1 },
        colors: { azure: 10, crimson: 1 },
        objects: { stone: 10, crystal: 1 },
        concepts: { dream: 10, wisdom: 1 }
      }
      
      const weightedGenerator = new WordGenerator({ weights })
      const batch = weightedGenerator.generateBatch(100)
      
      // Ocean should appear more frequently than forest
      const oceanCount = batch.filter(fw => fw.startsWith('ocean-')).length
      const forestCount = batch.filter(fw => fw.startsWith('forest-')).length
      expect(oceanCount).toBeGreaterThan(forestCount)
    })

    test('allows category constraints', () => {
      const constraints = {
        nature: ['ocean', 'river'], // Only water-related
        colors: null, // Any color
        objects: ['stone', 'crystal'], // Only minerals
        concepts: null // Any concept
      }
      
      const constrainedGenerator = new WordGenerator({ constraints })
      const batch = constrainedGenerator.generateBatch(10)
      
      batch.forEach(fourWords => {
        const [nature, , object] = fourWords.split('-')
        expect(['ocean', 'river']).toContain(nature)
        expect(['stone', 'crystal']).toContain(object)
      })
    })
  })

  describe('statistics', () => {
    test('calculates total possible combinations', () => {
      const stats = generator.getStatistics()
      expect(stats.totalCombinations).toBe(
        wordLists.nature.length *
        wordLists.colors.length *
        wordLists.objects.length *
        wordLists.concepts.length
      )
    })

    test('tracks generation statistics', () => {
      generator.generateBatch(100)
      const stats = generator.getStatistics()
      
      expect(stats.totalGenerated).toBe(100)
      expect(stats.uniqueGenerated).toBe(100)
      expect(stats.collisionRate).toBe(0)
    })

    test('reports word frequency distribution', () => {
      generator.generateBatch(1000)
      const stats = generator.getStatistics()
      
      expect(stats.wordFrequency).toBeDefined()
      expect(Object.keys(stats.wordFrequency).length).toBeGreaterThan(0)
    })
  })

  describe('error handling', () => {
    test('throws on invalid configuration', () => {
      expect(() => new WordGenerator({ wordLists: {} as any }))
        .toThrow('Invalid word lists configuration')
      
      expect(() => new WordGenerator({ wordLists: { nature: [] } as any }))
        .toThrow('Word lists cannot be empty')
    })

    test('handles generation failures gracefully', async () => {
      // Mock a scenario where all combinations are taken
      const tinyLists = {
        nature: ['a'],
        colors: ['b'],
        objects: ['c'],
        concepts: ['d']
      }
      
      const tinyGenerator = new WordGenerator({ wordLists: tinyLists })
      await tinyGenerator.claim('a-b-c-d')
      
      await expect(tinyGenerator.generateUnique())
        .rejects.toThrow('Unable to generate unique combination')
    })

    it('validates input parameters', () => {
      expect(() => generator.generateBatch(-1))
        .toThrow('Batch size must be positive')
      
      expect(() => generator.generateBatch(10001))
        .toThrow('Batch size exceeds maximum')
      
      expect(() => generator.generateCandidates(0))
        .toThrow('Must generate at least 1 candidate')
    })
  })
})