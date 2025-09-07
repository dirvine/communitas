import { describe, it, expect, beforeEach } from 'vitest'
import { cryptoManager } from '../cryptoManager'

describe('CryptoManager negative verification', () => {
  beforeEach(() => {
    cryptoManager.clearAll()
  })

  it('verifySignature returns false on bad public key JSON', async () => {
    const kp = await cryptoManager.generateKeyPair('neg-key')
    const msg = new TextEncoder().encode('bad-verify')
    const sig = await cryptoManager.signData(msg, kp.keyId)

    // Pass non-JSON public key string to force parsing error path
    const ok = await cryptoManager.verifySignature(msg, Array.from(sig), 'not-json', kp.algorithm)
    expect(ok).toBe(false)
  })
})

