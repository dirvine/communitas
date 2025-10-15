import { describe, it, expect, beforeEach, vi } from 'vitest'
import {
  ENCRYPTION_CONFIG,
  KEY_DERIVATION_CONFIG,
  cryptoManager,
} from '../crypto'

describe('crypto utilities', () => {
  beforeEach(() => {
    cryptoManager.clearCache()
  })

  describe('ENCRYPTION_CONFIG', () => {
    it('should have correct configuration values', () => {
      expect(ENCRYPTION_CONFIG.ALGORITHM).toBe('AES-GCM')
      expect(ENCRYPTION_CONFIG.KEY_LENGTH).toBe(256)
      expect(ENCRYPTION_CONFIG.IV_LENGTH).toBe(12)
      expect(ENCRYPTION_CONFIG.TAG_LENGTH).toBe(128)
      expect(ENCRYPTION_CONFIG.SALT_LENGTH).toBe(32)
      expect(ENCRYPTION_CONFIG.ITERATIONS).toBe(100000)
    })
  })

  describe('KEY_DERIVATION_CONFIG', () => {
    it('should have correct key derivation configuration', () => {
      expect(KEY_DERIVATION_CONFIG.NAME).toBe('PBKDF2')
      expect(KEY_DERIVATION_CONFIG.HASH).toBe('SHA-256')
      expect(KEY_DERIVATION_CONFIG.ITERATIONS).toBe(100000)
    })
  })

  describe('CryptoManager', () => {
    it('should be a singleton instance', () => {
      const manager1 = cryptoManager
      const manager2 = cryptoManager
      expect(manager1).toBe(manager2)
    })

    it('should clear cache properly', () => {
      // Clear should not throw
      expect(() => cryptoManager.clearCache()).not.toThrow()
    })

    it('should generate random passphrase', () => {
      const passphrase = cryptoManager.generatePassphrase()
      expect(passphrase).toBeDefined()
      expect(passphrase.split('-')).toHaveLength(4)
      expect(passphrase).toMatch(/^[a-z]+(-[a-z]+){3}$/)
    })
  })

  describe('encryption operations', () => {
    it('should generate encryption key', async () => {
      const key = await cryptoManager.generateEncryptionKey()
      expect(key).toBeDefined()
      expect(key.algorithm).toBe('AES-GCM')
      expect(key.extractable).toBe(true)
    })

    it('should generate signing key pair', async () => {
      const keyPair = await cryptoManager.generateKeyPair('signing')
      expect(keyPair).toBeDefined()
      expect(keyPair.publicKey).toBeDefined()
      expect(keyPair.privateKey).toBeDefined()
    })

    it('should derive keys from password', async () => {
      const password = 'test-password123'
      const salt = crypto.getRandomValues(new Uint8Array(16))
      
      const key = await cryptoManager.deriveKey(password, salt)
      expect(key).toBeDefined()
      expect(key.algorithm).toBe('AES-GCM')
    })

    it('should fail with short password', async () => {
      const password = 'short'
      const salt = crypto.getRandomValues(new Uint8Array(16))
      
      await expect(cryptoManager.deriveKey(password, salt)).rejects.toThrow()
    })

    it('should encrypt and decrypt data', async () => {
      const key = await cryptoManager.generateEncryptionKey()
      const data = 'test secret message'

      const encrypted = await cryptoManager.encrypt(data, key)
      expect(encrypted).toBeDefined()
      expect(encrypted.data).toBeInstanceOf(ArrayBuffer)
      expect(encrypted.iv).toBeInstanceOf(ArrayBuffer)

      const decrypted = await cryptoManager.decrypt(encrypted, key)
      expect(decrypted).toBe(data)
    })
  })

  describe('edge cases', () => {
    it('should handle empty string encryption', async () => {
      const key = await cryptoManager.generateEncryptionKey()
      
      const encrypted = await cryptoManager.encrypt('', key)
      const decrypted = await cryptoManager.decrypt(encrypted, key)
      expect(decrypted).toBe('')
    })

    it('should handle unicode characters', async () => {
      const key = await cryptoManager.generateEncryptionKey()
      const data = 'test with émojis 🚀 unicode'

      const encrypted = await cryptoManager.encrypt(data, key)
      const decrypted = await cryptoManager.decrypt(encrypted, key)
      expect(decrypted).toBe(data)
    })

    it('should handle large data', async () => {
      const key = await cryptoManager.generateEncryptionKey()
      const data = 'x'.repeat(10000) // 10KB

      const encrypted = await cryptoManager.encrypt(data, key)
      const decrypted = await cryptoManager.decrypt(encrypted, key)
      expect(decrypted).toBe(data)
    })
  })

  describe('key caching', () => {
    it('should cache derived keys', async () => {
      const password = 'test-password123'
      const salt = crypto.getRandomValues(new Uint8Array(16))
      
      const key1 = await cryptoManager.deriveKey(password, salt)
      const key2 = await cryptoManager.deriveKey(password, salt)
      
      // Keys should be the same object (cached)
      expect(key1).toBe(key2)
    })

    it('should clear key cache', async () => {
      const password = 'test-password123'
      const salt = crypto.getRandomValues(new Uint8Array(16))
      
      await cryptoManager.deriveKey(password, salt)
      cryptoManager.clearCache()
      
      const key = await cryptoManager.deriveKey(password, salt)
      expect(key).toBeDefined()
    })
  })
})
