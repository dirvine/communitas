import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ensureIdentity } from '../identity'

// Simple localStorage shim for node
class MemoryStorage {
  private store = new Map<string, string>()
  getItem(key: string) { return this.store.get(key) ?? null }
  setItem(key: string, value: string) { this.store.set(key, value) }
  removeItem(key: string) { this.store.delete(key) }
  clear() { this.store.clear() }
}

// @ts-ignore
globalThis.localStorage = new MemoryStorage()

// Provide blake3 shim if not present
// @ts-ignore
if (!globalThis.__BLAKE3_SHIM__) {
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const nodeCrypto = require('crypto')
  // @ts-ignore
  globalThis.__BLAKE3_SHIM__ = (input: string) => nodeCrypto.createHash('sha256').update(input).digest('hex')
}

describe('ensureIdentity', () => {
  beforeEach(() => {
    // @ts-ignore
    globalThis.localStorage.clear()
  })

  it('generates and persists a four-word identity when absent', async () => {
    const id = await ensureIdentity('test-four-words')
    expect(typeof id).toBe('string')
    expect(id.split('-')).toHaveLength(4)
    // @ts-ignore
    expect(globalThis.localStorage.getItem('test-four-words')).toBe(id)
  })

  it('reuses an existing valid identity', async () => {
    // @ts-ignore
    globalThis.localStorage.setItem('test-four-words', 'ocean-forest-moon-star')
    const id = await ensureIdentity('test-four-words')
    expect(id).toBe('ocean-forest-moon-star')
  })
})
