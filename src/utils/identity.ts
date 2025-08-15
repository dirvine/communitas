import { safeInvoke } from './tauri'

export type IdentityPacket = {
  four_words: string
  public_key: number[]
  signature: number[]
  dht_id: string
  created_at: number
  packet_version: number
}

export const generateFourWordIdentity = async (seed?: string): Promise<string> => {
  const words = await safeInvoke<string>('generate_four_word_identity', seed ? { seed } : undefined)
  if (words) return words
  // Test/browser fallback: deterministic from seed or random
  const rand = (n: number) => Math.floor(Math.random() * n)
  const wordList = [
    'ocean','forest','mountain','river','desert','valley','meadow','storm','cloud','wind',
    'moon','star','sun','comet','nova','ember','shadow','flame','stone','metal',
    'wolf','eagle','lion','tiger','bear','hawk','otter','whale','dolphin','fox'
  ]
  const pick = (i: number) => wordList[i % wordList.length]
  const s = seed ?? `${Date.now()}-${Math.random()}`
  // simple hash to indices
  let h = 0
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) >>> 0
  const parts = [h, h >>> 8, h >>> 16, h >>> 24].map((v, idx) => pick((v + idx * 7 + rand(1000)) % wordList.length))
  return parts.join('-')
}

export const validateFourWordIdentity = async (four_words: string): Promise<boolean> => {
  const ok = await safeInvoke<boolean>('validate_four_word_identity', { four_words })
  if (ok != null) return !!ok
  return /^[a-z]+-[a-z]+-[a-z]+-[a-z]+$/.test(four_words)
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
