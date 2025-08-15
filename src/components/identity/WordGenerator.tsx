/**
 * Word Generator for Four-Word Identity System
 * Uses the four-word-networking crate through Rust bindings
 */

import { invoke } from '@tauri-apps/api/core'

export interface WordGeneratorOptions {
  seed?: string
  wordLists?: any // For testing with custom lists
  weights?: any
  constraints?: any
}

export interface GeneratorStatistics {
  totalCombinations: number
  totalGenerated: number
  uniqueGenerated: number
  collisionRate: number
  wordFrequency?: Record<string, number>
}

/**
 * WordGenerator class that interfaces with the four-word-networking crate
 * through Tauri commands to generate and validate four-word identities
 */
export class WordGenerator {
  protected claimed: Set<string> = new Set()
  private blacklist: Set<string> = new Set()
  private generated: string[] = []
  private options: WordGeneratorOptions

  constructor(options: WordGeneratorOptions = {}) {
    this.options = options
  }

  /**
   * Generate a single four-word identity using the Rust backend
   */
  async generate(): Promise<string> {
    try {
      // Call Rust backend to generate four-word identity
      const fourWords = await invoke<string>('generate_four_word_identity', {
        seed: this.options.seed
      })
      
      this.generated.push(fourWords)
      return fourWords
    } catch (error) {
      throw new Error(`Failed to generate four-word identity: ${error}`)
    }
  }

  /**
   * Generate multiple candidates for user selection
   */
  async generateCandidates(count: number): Promise<string[]> {
    if (count < 1) {
      throw new Error('Must generate at least 1 candidate')
    }

    const candidates: string[] = []
    const seen = new Set<string>()

    while (candidates.length < count) {
      const fourWords = await this.generate()
      
      // Ensure uniqueness within this batch
      if (!seen.has(fourWords) && !this.blacklist.has(fourWords)) {
        candidates.push(fourWords)
        seen.add(fourWords)
      }
    }

    return candidates
  }

  /**
   * Generate a batch of unique four-word identities
   */
  async generateBatch(size: number): Promise<string[]> {
    if (size < 1) {
      throw new Error('Batch size must be positive')
    }
    if (size > 10000) {
      throw new Error('Batch size exceeds maximum')
    }

    const batch: string[] = []
    const seen = new Set<string>()

    for (let i = 0; i < size; i++) {
      let attempts = 0
      let fourWords: string

      do {
        fourWords = await this.generate()
        attempts++

        if (attempts > 100) {
          throw new Error('Unable to generate unique combinations')
        }
      } while (seen.has(fourWords) || this.blacklist.has(fourWords))

      batch.push(fourWords)
      seen.add(fourWords)
    }

    return batch
  }

  /**
   * Generate a unique unclaimed four-word identity
   */
  async generateUnique(): Promise<string> {
    let attempts = 0
    const maxAttempts = 1000

    while (attempts < maxAttempts) {
      const fourWords = await this.generate()
      
      if (!this.claimed.has(fourWords) && 
          !this.blacklist.has(fourWords) &&
          await this.isAvailable(fourWords)) {
        return fourWords
      }

      attempts++
    }

    throw new Error('Unable to generate unique combination after ' + maxAttempts + ' attempts')
  }

  /**
   * Validate a four-word identity format and dictionary membership
   */
  async isValid(fourWords: string): Promise<boolean> {
    try {
      // Call Rust backend to validate using four-word-networking crate
      return await invoke<boolean>('validate_four_word_identity', {
        fourWords
      })
    } catch (error) {
      console.error('Validation error:', error)
      return false
    }
  }

  /**
   * Check if a four-word identity is available (not claimed)
   */
  async isAvailable(fourWords: string): Promise<boolean> {
    if (this.claimed.has(fourWords)) {
      return false
    }

    try {
      // Check with backend/DHT if identity is claimed
      const available = await invoke<boolean>('check_identity_availability', {
        fourWords
      })
      return available
    } catch (error) {
      console.error('Availability check error:', error)
      return false
    }
  }

  /**
   * Claim a four-word identity
   */
  async claim(fourWords: string): Promise<boolean> {
    if (!await this.isValid(fourWords)) {
      throw new Error('Invalid four-word identity format')
    }

    if (!await this.isAvailable(fourWords)) {
      return false
    }

    try {
      // Register with backend/DHT
      const success = await invoke<boolean>('claim_four_word_identity', {
        fourWords
      })

      if (success) {
        this.claimed.add(fourWords)
      }

      return success
    } catch (error) {
      throw new Error(`Failed to claim identity: ${error}`)
    }
  }

  /**
   * Add a four-word identity to the blacklist
   */
  addToBlacklist(fourWords: string): boolean {
    this.blacklist.add(fourWords)
    return true
  }

  /**
   * Get statistics about generated identities
   */
  getStatistics(): GeneratorStatistics {
    // Calculate word frequency
    const wordFrequency: Record<string, number> = {}
    
    for (const fourWords of this.generated) {
      const words = fourWords.split('-')
      for (const word of words) {
        wordFrequency[word] = (wordFrequency[word] || 0) + 1
      }
    }

    // Estimate total combinations (4096^4 if using standard dictionary)
    const wordsPerPosition = 4096 // Standard four-word-networking dictionary size
    const totalCombinations = Math.pow(wordsPerPosition, 4)

    // Calculate unique generated
    const uniqueGenerated = new Set(this.generated).size
    const collisionRate = this.generated.length > 0 
      ? (this.generated.length - uniqueGenerated) / this.generated.length 
      : 0

    return {
      totalCombinations,
      totalGenerated: this.generated.length,
      uniqueGenerated,
      collisionRate,
      wordFrequency
    }
  }

  /**
   * Clear all claimed identities (for testing)
   */
  clearClaimed(): void {
    this.claimed.clear()
  }

  /**
   * Clear blacklist (for testing)
   */
  clearBlacklist(): void {
    this.blacklist.clear()
  }
}

// Export word lists for backward compatibility and testing
export const wordLists = {
  nature: [], // Will be populated from four-word-networking crate
  colors: [],
  objects: [],
  concepts: []
}

// Helper function to check if we're in a test environment
export function isTestEnvironment(): boolean {
  return process.env.NODE_ENV === 'test' || process.env.JEST_WORKER_ID !== undefined
}

// Mock implementation for testing
export class MockWordGenerator extends WordGenerator {
  private mockWords = [
    'ocean-azure-stone-dream',
    'forest-crimson-crystal-wisdom',
    'mountain-golden-pearl-courage',
    'river-silver-diamond-harmony',
    'valley-emerald-sword-justice',
    'desert-ruby-shield-honor',
    'glacier-sapphire-crown-truth',
    'canyon-amber-throne-power'
  ]
  
  private mockIndex = 0

  async generate(): Promise<string> {
    const fourWords = this.mockWords[this.mockIndex % this.mockWords.length]
    this.mockIndex++
    return fourWords
  }

  async isValid(fourWords: string): Promise<boolean> {
    // Basic format validation for testing
    const parts = fourWords.split('-')
    return parts.length === 4 && parts.every(word => /^[a-z]+$/.test(word))
  }

  async isAvailable(fourWords: string): Promise<boolean> {
    return !this.claimed.has(fourWords)
  }

  async claim(fourWords: string): Promise<boolean> {
    if (await this.isAvailable(fourWords)) {
      this.claimed.add(fourWords)
      return true
    }
    return false
  }
}