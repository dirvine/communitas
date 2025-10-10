import { safeInvoke } from './tauri'

export type IdentityPacket = {
  four_words: string
  public_key: number[]
  signature: number[]
  dht_id: string
  created_at: number
  packet_version: number
}

/**
 * Generate a valid four-word identity using the four-word-networking dictionary
 *
 * Returns identity with dashes internally, but UX should display/accept spaces.
 * Backend returns format: "word-word-word-word"
 *
 * @param seed - Optional seed for deterministic generation (not currently used)
 * @returns Four-word identity with dashes (e.g., "ocean-forest-moon-star")
 */
export const generateFourWordIdentity = async (seed?: string): Promise<string> => {
  const words = await safeInvoke<string>('generate_four_word_identity', seed ? { seed } : undefined)
  if (words) {
    return words
  }
  return generateFallbackFourWords()
}

/**
 * Validate a four-word identity format
 *
 * Accepts both space and dash separators for UX flexibility.
 * Internally converts to dash format before validation.
 *
 * @param four_words - Four-word identity (spaces or dashes)
 * @returns true if valid format
 */
export const validateFourWordIdentity = async (four_words: string): Promise<boolean> => {
  // Normalize: trim, lowercase, convert spaces to dashes for backend
  const normalized = four_words.trim().toLowerCase().replace(/\s+/g, '-')

  const ok = await safeInvoke<boolean>('validate_four_word_identity', { four_words: normalized })
  if (ok != null) return !!ok

  // Fallback validation: check format only (4 words)
  return /^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/.test(normalized)
}

export const claimFourWordIdentity = async (four_words: string): Promise<boolean> => {
  const ok = await safeInvoke<boolean>('claim_four_word_identity', { four_words })
  if (ok != null) return !!ok
  // No-op in tests/browser
  return true
}

export const getIdentityPacket = async (four_words: string): Promise<IdentityPacket | null> => {
  const res = await safeInvoke<IdentityPacket | null>('get_identity_packet', { four_words })
  if (res) return res
  return {
    four_words,
    public_key: [],
    signature: [],
    dht_id: four_words,
    created_at: Date.now(),
    packet_version: 1,
  }
}

export async function ensureIdentity(storageKey = 'communitas-four-words'): Promise<string> {
  // Try local storage first
  let four = localStorage.getItem(storageKey)
  if (four) {
    const valid = await validateFourWordIdentity(four)
    if (valid) return four
  }

  // Generate and claim
  four = await generateFourWordIdentity()
  await claimFourWordIdentity(four)
  localStorage.setItem(storageKey, four)
  return four
}

/**
 * Convert four-word address from storage format (dashes) to display format (spaces)
 *
 * @param fourWords - Four-word address with dashes (e.g., "ocean-forest-moon-star")
 * @returns Four-word address with spaces (e.g., "ocean forest moon star")
 */
export function fourWordsToDisplay(fourWords: string): string {
  if (!fourWords) return fourWords
  return fourWords.replace(/-/g, ' ')
}

/**
 * Convert four-word address from display format (spaces) to storage format (dashes)
 *
 * @param fourWords - Four-word address with spaces (e.g., "ocean forest moon star")
 * @returns Four-word address with dashes (e.g., "ocean-forest-moon-star")
 */
export function fourWordsToStorage(fourWords: string): string {
  if (!fourWords) return fourWords
  return fourWords.trim().toLowerCase().replace(/\s+/g, '-')
}

const FALLBACK_WORDS: string[][] = [
  ['ocean', 'forest', 'prairie', 'valley', 'desert', 'island', 'sunset', 'harbor'],
  ['bright', 'golden', 'silver', 'crystal', 'ember', 'shadow', 'morning', 'midnight'],
  ['hawk', 'otter', 'lynx', 'sparrow', 'storm', 'firefly', 'orca', 'aurora'],
  ['star', 'moon', 'nova', 'cloud', 'rain', 'wind', 'flame', 'glow'],
]

const generateFallbackFourWords = (): string => {
  const words = FALLBACK_WORDS.map(group => group[Math.floor(Math.random() * group.length)])
  return words.join('-')
}
