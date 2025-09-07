import { describe, it, expect, beforeEach } from 'vitest'
import { cryptoManager } from '../cryptoManager'

describe('CryptoManager', () => {
  beforeEach(() => {
    cryptoManager.clearAll()
  })

  it('derives key fails for short password', async () => {
    await expect(
      cryptoManager.deriveKey('short', new Uint8Array(16))
    ).rejects.toThrow('Password must be at least 8 characters long')
  })

  it('derives key fails for short salt', async () => {
    await expect(
      cryptoManager.deriveKey('long-enough', new Uint8Array(8))
    ).rejects.toThrow('Salt must be at least 16 bytes long')
  })

  it('encrypt/decrypt roundtrip works', async () => {
    const key = new Uint8Array(32)
    crypto.getRandomValues(key)
    const data = new TextEncoder().encode('hello world')

    const enc = await cryptoManager.encryptData(data, key, 'k1')
    const dec = await cryptoManager.decryptData(enc, key)

    expect(new TextDecoder().decode(dec)).toBe('hello world')
  })

  it('encrypt fails for wrong key length', async () => {
    const badKey = new Uint8Array(16)
    await expect(
      cryptoManager.encryptData(new Uint8Array([1, 2, 3]), badKey)
    ).rejects.toThrow('Key must be 32 bytes long')
  })

  it('sign fails for missing key', async () => {
    await expect(cryptoManager.signData(new Uint8Array([1]), 'missing'))
      .rejects.toThrow('Key pair with ID missing not found')
  })

  it('generate key and sign/verify succeeds', async () => {
    const kp = await cryptoManager.generateKeyPair('test-key')
    const msg = new TextEncoder().encode('message')
    const sig = await cryptoManager.signData(msg, kp.keyId)

    const ok = await cryptoManager.verifySignature(
      msg, Array.from(sig), kp.publicKey, kp.algorithm
    )
    expect(ok).toBe(true)
  })
})
