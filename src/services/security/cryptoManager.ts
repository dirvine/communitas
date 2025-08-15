/**
 * Secure Cryptographic Key Management Service
 * Addresses critical cryptographic vulnerabilities identified in security analysis
 */

import crypto from 'crypto'

export interface KeyPair {
  publicKey: string
  privateKey: string
  keyId: string
  algorithm: string
  createdAt: number
  expiresAt?: number
}

export interface EncryptionResult {
  ciphertext: Uint8Array
  iv: Uint8Array
  authTag: Uint8Array
  keyId: string
}

export interface DecryptionOptions {
  keyId?: string
  maxAge?: number // Maximum age in milliseconds
}

export interface KeyDerivationOptions {
  salt?: Uint8Array
  iterations?: number
  keyLength?: number
  algorithm?: string
}

/**
 * Secure cryptographic operations with proper key management
 */
export class CryptoManager {
  private static instance: CryptoManager
  private keyStore = new Map<string, KeyPair>()
  private derivedKeys = new Map<string, Uint8Array>()
  
  // Secure defaults
  private readonly AES_KEY_LENGTH = 32 // 256 bits
  private readonly IV_LENGTH = 12     // 96 bits for GCM
  private readonly TAG_LENGTH = 16    // 128 bits for GCM
  private readonly PBKDF2_ITERATIONS = 100000
  private readonly RSA_KEY_SIZE = 4096 // Strong key size
  
  static getInstance(): CryptoManager {
    if (!CryptoManager.instance) {
      CryptoManager.instance = new CryptoManager()
    }
    return CryptoManager.instance
  }

  /**
   * Generate a secure RSA key pair for digital signatures
   */
  async generateRSAKeyPair(keyId?: string): Promise<KeyPair> {
    const id = keyId || crypto.randomUUID()
    
    try {
      const { publicKey, privateKey } = crypto.generateKeyPairSync('rsa', {
        modulusLength: this.RSA_KEY_SIZE,
        publicKeyEncoding: {
          type: 'spki',
          format: 'pem'
        },
        privateKeyEncoding: {
          type: 'pkcs8',
          format: 'pem'
        }
      })

      const keyPair: KeyPair = {
        publicKey,
        privateKey,
        keyId: id,
        algorithm: 'RSA-4096',
        createdAt: Date.now(),
        expiresAt: Date.now() + (365 * 24 * 60 * 60 * 1000) // 1 year
      }

      this.keyStore.set(id, keyPair)
      return keyPair
    } catch (error) {
      throw new Error(`Failed to generate RSA key pair: ${error.message}`)
    }
  }

  /**
   * Generate Ed25519 key pair for high-performance digital signatures
   */
  async generateEd25519KeyPair(keyId?: string): Promise<KeyPair> {
    const id = keyId || crypto.randomUUID()
    
    try {
      const { publicKey, privateKey } = crypto.generateKeyPairSync('ed25519', {
        publicKeyEncoding: {
          type: 'spki',
          format: 'pem'
        },
        privateKeyEncoding: {
          type: 'pkcs8',
          format: 'pem'
        }
      })

      const keyPair: KeyPair = {
        publicKey,
        privateKey,
        keyId: id,
        algorithm: 'Ed25519',
        createdAt: Date.now(),
        expiresAt: Date.now() + (365 * 24 * 60 * 60 * 1000) // 1 year
      }

      this.keyStore.set(id, keyPair)
      return keyPair
    } catch (error) {
      throw new Error(`Failed to generate Ed25519 key pair: ${error.message}`)
    }
  }

  /**
   * Derive encryption key from password using PBKDF2
   */
  async deriveKey(
    password: string, 
    salt: Uint8Array,
    options: KeyDerivationOptions = {}
  ): Promise<Uint8Array> {
    const iterations = options.iterations || this.PBKDF2_ITERATIONS
    const keyLength = options.keyLength || this.AES_KEY_LENGTH
    const algorithm = options.algorithm || 'sha256'

    if (password.length < 8) {
      throw new Error('Password must be at least 8 characters long')
    }

    if (salt.length < 16) {
      throw new Error('Salt must be at least 16 bytes long')
    }

    try {
      return new Promise((resolve, reject) => {
        crypto.pbkdf2(password, salt, iterations, keyLength, algorithm, (err, derivedKey) => {
          if (err) {
            reject(new Error(`Key derivation failed: ${err.message}`))
          } else {
            resolve(new Uint8Array(derivedKey))
          }
        })
      })
    } catch (error) {
      throw new Error(`Key derivation error: ${error.message}`)
    }
  }

  /**
   * Generate cryptographically secure random salt
   */
  generateSalt(length = 32): Uint8Array {
    return new Uint8Array(crypto.randomBytes(length))
  }

  /**
   * Encrypt data using AES-256-GCM
   */
  async encryptData(
    data: Uint8Array, 
    key: Uint8Array, 
    keyId?: string
  ): Promise<EncryptionResult> {
    if (key.length !== this.AES_KEY_LENGTH) {
      throw new Error(`Key must be ${this.AES_KEY_LENGTH} bytes long`)
    }

    const iv = new Uint8Array(crypto.randomBytes(this.IV_LENGTH))
    const cipher = crypto.createCipher('aes-256-gcm', key)
    cipher.setAAD(Buffer.from(keyId || 'communitas'))

    try {
      const encrypted = cipher.update(data)
      cipher.final()
      const authTag = cipher.getAuthTag()

      return {
        ciphertext: new Uint8Array(encrypted),
        iv,
        authTag: new Uint8Array(authTag),
        keyId: keyId || 'default'
      }
    } catch (error) {
      throw new Error(`Encryption failed: ${error.message}`)
    }
  }

  /**
   * Decrypt data using AES-256-GCM
   */
  async decryptData(
    encryptedData: EncryptionResult,
    key: Uint8Array,
    options: DecryptionOptions = {}
  ): Promise<Uint8Array> {
    if (key.length !== this.AES_KEY_LENGTH) {
      throw new Error(`Key must be ${this.AES_KEY_LENGTH} bytes long`)
    }

    const decipher = crypto.createDecipher('aes-256-gcm', key)
    decipher.setAuthTag(Buffer.from(encryptedData.authTag))
    decipher.setAAD(Buffer.from(encryptedData.keyId))

    try {
      const decrypted = decipher.update(Buffer.from(encryptedData.ciphertext))
      decipher.final()
      
      return new Uint8Array(decrypted)
    } catch (error) {
      throw new Error(`Decryption failed: ${error.message}`)
    }
  }

  /**
   * Sign data using private key
   */
  async signData(data: Uint8Array, keyId: string): Promise<Uint8Array> {
    const keyPair = this.keyStore.get(keyId)
    if (!keyPair) {
      throw new Error(`Key pair with ID ${keyId} not found`)
    }

    // Check if key is expired
    if (keyPair.expiresAt && Date.now() > keyPair.expiresAt) {
      throw new Error(`Key pair ${keyId} has expired`)
    }

    try {
      const signature = crypto.sign(null, data, {
        key: keyPair.privateKey,
        padding: keyPair.algorithm.startsWith('RSA') ? crypto.constants.RSA_PKCS1_PSS_PADDING : undefined,
        saltLength: keyPair.algorithm.startsWith('RSA') ? crypto.constants.RSA_PSS_SALTLEN_DIGEST : undefined
      })

      return new Uint8Array(signature)
    } catch (error) {
      throw new Error(`Signing failed: ${error.message}`)
    }
  }

  /**
   * Verify signature using public key
   */
  async verifySignature(
    data: Uint8Array, 
    signature: Uint8Array, 
    publicKey: string,
    algorithm: string
  ): Promise<boolean> {
    try {
      return crypto.verify(null, data, {
        key: publicKey,
        padding: algorithm.startsWith('RSA') ? crypto.constants.RSA_PKCS1_PSS_PADDING : undefined,
        saltLength: algorithm.startsWith('RSA') ? crypto.constants.RSA_PSS_SALTLEN_DIGEST : undefined
      }, signature)
    } catch (error) {
      console.error('Signature verification failed:', error)
      return false
    }
  }

  /**
   * Generate secure hash using SHA-256
   */
  hash(data: Uint8Array, algorithm = 'sha256'): Uint8Array {
    const hash = crypto.createHash(algorithm)
    hash.update(data)
    return new Uint8Array(hash.digest())
  }

  /**
   * Generate secure HMAC
   */
  hmac(data: Uint8Array, key: Uint8Array, algorithm = 'sha256'): Uint8Array {
    const hmac = crypto.createHmac(algorithm, key)
    hmac.update(data)
    return new Uint8Array(hmac.digest())
  }

  /**
   * Secure random number generation
   */
  randomBytes(length: number): Uint8Array {
    return new Uint8Array(crypto.randomBytes(length))
  }

  /**
   * Generate cryptographically secure UUID
   */
  generateSecureId(): string {
    return crypto.randomUUID()
  }

  /**
   * Get key pair by ID
   */
  getKeyPair(keyId: string): KeyPair | undefined {
    const keyPair = this.keyStore.get(keyId)
    
    // Check if key is expired
    if (keyPair?.expiresAt && Date.now() > keyPair.expiresAt) {
      this.keyStore.delete(keyId)
      return undefined
    }
    
    return keyPair
  }

  /**
   * Remove key pair from storage
   */
  revokeKeyPair(keyId: string): boolean {
    return this.keyStore.delete(keyId)
  }

  /**
   * List all active key pairs (public info only)
   */
  listKeyPairs(): Array<Omit<KeyPair, 'privateKey'>> {
    const result: Array<Omit<KeyPair, 'privateKey'>> = []
    
    for (const [id, keyPair] of this.keyStore.entries()) {
      // Skip expired keys
      if (keyPair.expiresAt && Date.now() > keyPair.expiresAt) {
        this.keyStore.delete(id)
        continue
      }
      
      result.push({
        publicKey: keyPair.publicKey,
        keyId: keyPair.keyId,
        algorithm: keyPair.algorithm,
        createdAt: keyPair.createdAt,
        expiresAt: keyPair.expiresAt
      })
    }
    
    return result
  }

  /**
   * Clear all keys and derived keys (for security)
   */
  clearAll(): void {
    this.keyStore.clear()
    this.derivedKeys.clear()
  }

  /**
   * Validate cryptographic parameters
   */
  validateCryptoParams(params: {
    keyLength?: number
    ivLength?: number
    iterations?: number
    algorithm?: string
  }): boolean {
    const { keyLength, ivLength, iterations, algorithm } = params

    if (keyLength !== undefined && keyLength < 16) {
      throw new Error('Key length must be at least 16 bytes')
    }

    if (ivLength !== undefined && ivLength < 12) {
      throw new Error('IV length must be at least 12 bytes')
    }

    if (iterations !== undefined && iterations < 10000) {
      throw new Error('PBKDF2 iterations must be at least 10,000')
    }

    if (algorithm && !['sha256', 'sha384', 'sha512'].includes(algorithm)) {
      throw new Error('Algorithm must be one of: sha256, sha384, sha512')
    }

    return true
  }

  /**
   * Get cryptographic strength assessment
   */
  assessStrength(keyPair: KeyPair): {
    level: 'weak' | 'moderate' | 'strong'
    issues: string[]
    recommendations: string[]
  } {
    const issues: string[] = []
    const recommendations: string[] = []

    // Check key age
    const ageInDays = (Date.now() - keyPair.createdAt) / (24 * 60 * 60 * 1000)
    if (ageInDays > 365) {
      issues.push('Key is over 1 year old')
      recommendations.push('Consider rotating to a new key pair')
    }

    // Check algorithm strength
    if (keyPair.algorithm.includes('RSA-2048')) {
      issues.push('RSA-2048 is becoming weak')
      recommendations.push('Upgrade to RSA-4096 or Ed25519')
    }

    // Check expiration
    if (!keyPair.expiresAt) {
      issues.push('Key has no expiration date')
      recommendations.push('Set an expiration date for better security')
    }

    const level = issues.length === 0 ? 'strong' : 
                  issues.length <= 2 ? 'moderate' : 'weak'

    return { level, issues, recommendations }
  }
}

// Export singleton instance
export const cryptoManager = CryptoManager.getInstance()