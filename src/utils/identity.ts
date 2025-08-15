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
  if (!words) throw new Error('Failed to generate four-word identity')
  return words
}

export const validateFourWordIdentity = async (four_words: string): Promise<boolean> => {
  const ok = await safeInvoke<boolean>('validate_four_word_identity', { four_words })
  return !!ok
}

export const claimFourWordIdentity = async (four_words: string): Promise<boolean> => {
  const ok = await safeInvoke<boolean>('claim_four_word_identity', { four_words })
  return !!ok
}

export const getIdentityPacket = async (four_words: string): Promise<IdentityPacket | null> => {
  const res = await safeInvoke<IdentityPacket | null>('get_identity_packet', { four_words })
  return res ?? null
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
