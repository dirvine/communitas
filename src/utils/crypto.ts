import { UserIdentity } from '../contexts/AuthContext';

// Encryption algorithms and configurations
export const ENCRYPTION_CONFIG = {
  ALGORITHM: 'AES-GCM',
  KEY_LENGTH: 256,
  IV_LENGTH: 12,
  TAG_LENGTH: 128,
  SALT_LENGTH: 32,
  ITERATIONS: 100000,
} as const;

// Key derivation configuration
export const KEY_DERIVATION_CONFIG = {
  NAME: 'PBKDF2',
  HASH: 'SHA-256',
  ITERATIONS: ENCRYPTION_CONFIG.ITERATIONS,
} as const;

// Message types for encrypted data
export interface EncryptedData {
  data: ArrayBuffer;
  iv: ArrayBuffer;
  salt?: ArrayBuffer;
  algorithm: string;
  keyId?: string;
  timestamp: number;
  version: number;
}

export interface KeyPair {
  publicKey: CryptoKey;
  privateKey: CryptoKey;
  keyId: string;
  createdAt: string;
  purpose: 'signing' | 'encryption' | 'general';
}

export interface DerivedKey {
  key: CryptoKey;
  salt: ArrayBuffer;
  keyId: string;
  purpose: string;
}

/**
 * Modern cryptographic utilities for end-to-end encryption
 * Uses Web Crypto API for secure, standards-based encryption
 */
export class CryptoManager {
  private static instance: CryptoManager;
  private keyCache: Map<string, CryptoKey> = new Map();
  private keyPairCache: Map<string, KeyPair> = new Map();

  static getInstance(): CryptoManager {
    if (!CryptoManager.instance) {
      CryptoManager.instance = new CryptoManager();
    }
    return CryptoManager.instance;
  }

  /**
   * Generate a cryptographically secure random key for encryption
   */
  async generateEncryptionKey(): Promise<CryptoKey> {
    return await crypto.subtle.generateKey(
      {
        name: ENCRYPTION_CONFIG.ALGORITHM,
        length: ENCRYPTION_CONFIG.KEY_LENGTH,
      },
      true, // extractable
      ['encrypt', 'decrypt']
    );
  }

  /**
   * Generate an ECDH key pair for key exchange
   */
  async generateKeyPair(purpose: KeyPair['purpose'] = 'general'): Promise<KeyPair> {
    const keyPair = await crypto.subtle.generateKey(
      {
        name: 'ECDH',
        namedCurve: 'P-384',
      },
      true, // extractable
      ['deriveKey', 'deriveBits']
    );

    const keyId = await this.generateKeyId();
    const keyPairObj: KeyPair = {
      publicKey: keyPair.publicKey,
      privateKey: keyPair.privateKey,
      keyId,
      createdAt: new Date().toISOString(),
      purpose,
    };

    this.keyPairCache.set(keyId, keyPairObj);
    return keyPairObj;
  }

  /**
   * Derive a shared encryption key from two ECDH key pairs
   */
  async deriveSharedKey(
    privateKey: CryptoKey,
    publicKey: CryptoKey,
    purpose: string = 'encryption'
  ): Promise<DerivedKey> {
    const sharedBits = await crypto.subtle.deriveBits(
      {
        name: 'ECDH',
        public: publicKey,
      },
      privateKey,
      256
    );

    const key = await crypto.subtle.importKey(
      'raw',
      sharedBits,
      { name: ENCRYPTION_CONFIG.ALGORITHM },
      false,
      ['encrypt', 'decrypt']
    );

    const keyId = await this.generateKeyId();
    const salt = crypto.getRandomValues(new Uint8Array(ENCRYPTION_CONFIG.SALT_LENGTH));

    return {
      key,
      salt: salt.buffer,
      keyId,
      purpose,
    };
  }

  /**
   * Derive a key from a password using PBKDF2
   */
  async deriveKeyFromPassword(password: string, salt?: ArrayBuffer): Promise<DerivedKey> {
    const encoder = new TextEncoder();
    const passwordBuffer = encoder.encode(password);

    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      passwordBuffer,
      { name: KEY_DERIVATION_CONFIG.NAME },
      false,
      ['deriveKey']
    );

    const actualSalt = salt || crypto.getRandomValues(new Uint8Array(ENCRYPTION_CONFIG.SALT_LENGTH)).buffer;

    const key = await crypto.subtle.deriveKey(
      {
        name: KEY_DERIVATION_CONFIG.NAME,
        salt: actualSalt,
        iterations: KEY_DERIVATION_CONFIG.ITERATIONS,
        hash: KEY_DERIVATION_CONFIG.HASH,
      },
      keyMaterial,
      {
        name: ENCRYPTION_CONFIG.ALGORITHM,
        length: ENCRYPTION_CONFIG.KEY_LENGTH,
      },
      false,
      ['encrypt', 'decrypt']
    );

    const keyId = await this.generateKeyId();
    
    return {
      key,
      salt: actualSalt,
      keyId,
      purpose: 'password-derived',
    };
  }

  /**
   * Encrypt data with AES-GCM
   */
  async encrypt(data: ArrayBuffer | string, key: CryptoKey, keyId?: string): Promise<EncryptedData> {
    const iv = crypto.getRandomValues(new Uint8Array(ENCRYPTION_CONFIG.IV_LENGTH));
    
    let dataBuffer: ArrayBuffer;
    if (typeof data === 'string') {
      const encoder = new TextEncoder();
      dataBuffer = encoder.encode(data).buffer;
    } else {
      dataBuffer = data;
    }

    const encryptedBuffer = await crypto.subtle.encrypt(
      {
        name: ENCRYPTION_CONFIG.ALGORITHM,
        iv: iv,
      },
      key,
      dataBuffer
    );

    return {
      data: encryptedBuffer,
      iv: iv.buffer,
      algorithm: ENCRYPTION_CONFIG.ALGORITHM,
      keyId,
      timestamp: Date.now(),
      version: 1,
    };
  }

  /**
   * Decrypt data with AES-GCM
   */
  async decrypt(encryptedData: EncryptedData, key: CryptoKey): Promise<ArrayBuffer> {
    return await crypto.subtle.decrypt(
      {
        name: encryptedData.algorithm,
        iv: encryptedData.iv,
      },
      key,
      encryptedData.data
    );
  }

  /**
   * Encrypt text and return as base64 string
   */
  async encryptText(text: string, key: CryptoKey, keyId?: string): Promise<string> {
    const encrypted = await this.encrypt(text, key, keyId);
    return this.encryptedDataToBase64(encrypted);
  }

  /**
   * Decrypt base64 encrypted data to text
   */
  async decryptText(encryptedBase64: string, key: CryptoKey): Promise<string> {
    const encrypted = this.base64ToEncryptedData(encryptedBase64);
    const decrypted = await this.decrypt(encrypted, key);
    const decoder = new TextDecoder();
    return decoder.decode(decrypted);
  }

  /**
   * Encrypt a file
   */
  async encryptFile(file: File, key: CryptoKey): Promise<EncryptedData> {
    const buffer = await file.arrayBuffer();
    return await this.encrypt(buffer, key);
  }

  /**
   * Create a signature for data integrity
   */
  async signData(data: ArrayBuffer, privateKey: CryptoKey): Promise<ArrayBuffer> {
    return await crypto.subtle.sign(
      {
        name: 'ECDSA',
        hash: { name: 'SHA-384' },
      },
      privateKey,
      data
    );
  }

  /**
   * Verify a signature
   */
  async verifySignature(
    signature: ArrayBuffer,
    data: ArrayBuffer,
    publicKey: CryptoKey
  ): Promise<boolean> {
    return await crypto.subtle.verify(
      {
        name: 'ECDSA',
        hash: { name: 'SHA-384' },
      },
      publicKey,
      signature,
      data
    );
  }

  /**
   * Generate a unique key ID
   */
  async generateKeyId(): Promise<string> {
    const randomBytes = crypto.getRandomValues(new Uint8Array(16));
    const hashBuffer = await crypto.subtle.digest('SHA-256', randomBytes);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('').slice(0, 32);
  }

  /**
   * Convert encrypted data to base64 for storage/transmission
   */
  encryptedDataToBase64(encrypted: EncryptedData): string {
    const combined = {
      data: this.arrayBufferToBase64(encrypted.data),
      iv: this.arrayBufferToBase64(encrypted.iv),
      salt: encrypted.salt ? this.arrayBufferToBase64(encrypted.salt) : undefined,
      algorithm: encrypted.algorithm,
      keyId: encrypted.keyId,
      timestamp: encrypted.timestamp,
      version: encrypted.version,
    };

    return btoa(JSON.stringify(combined));
  }

  /**
   * Convert base64 string back to encrypted data
   */
  base64ToEncryptedData(base64: string): EncryptedData {
    const combined = JSON.parse(atob(base64));
    
    return {
      data: this.base64ToArrayBuffer(combined.data),
      iv: this.base64ToArrayBuffer(combined.iv),
      salt: combined.salt ? this.base64ToArrayBuffer(combined.salt) : undefined,
      algorithm: combined.algorithm,
      keyId: combined.keyId,
      timestamp: combined.timestamp,
      version: combined.version,
    };
  }

  /**
   * Export a key for storage
   */
  async exportKey(key: CryptoKey, format: 'raw' | 'pkcs8' | 'spki' | 'jwk' = 'raw'): Promise<ArrayBuffer | JsonWebKey> {
    return await crypto.subtle.exportKey(format, key);
  }

  /**
   * Import a key from storage
   */
  async importKey(
    keyData: ArrayBuffer | JsonWebKey,
    algorithm: string = ENCRYPTION_CONFIG.ALGORITHM,
    usages: KeyUsage[] = ['encrypt', 'decrypt']
  ): Promise<CryptoKey> {
    if (keyData instanceof ArrayBuffer) {
      return await crypto.subtle.importKey('raw', keyData, { name: algorithm }, false, usages)
    }
    // JWK import requires algorithm-specific params; PQC algorithms use raw key format, not JWK
    // Limit JWK to symmetric AES-GCM keys
    return await crypto.subtle.importKey('jwk' as any, keyData as JsonWebKey, { name: algorithm }, false, usages)
  }

  /**
   * Generate a secure hash of data
   */
  async hash(data: ArrayBuffer | string, algorithm: 'SHA-256' | 'SHA-384' | 'SHA-512' = 'SHA-256'): Promise<ArrayBuffer> {
    let buffer: ArrayBuffer;
    if (typeof data === 'string') {
      const encoder = new TextEncoder();
      buffer = encoder.encode(data).buffer;
    } else {
      buffer = data;
    }

    return await crypto.subtle.digest(algorithm, buffer);
  }

  /**
   * Generate a content hash for deduplication
   */
  async generateContentHash(content: ArrayBuffer | string): Promise<string> {
    const hash = await this.hash(content, 'SHA-256');
    return this.arrayBufferToHex(hash);
  }

  /**
   * Utility: Convert ArrayBuffer to base64
   */
  private arrayBufferToBase64(buffer: ArrayBuffer): string {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
  }

  /**
   * Utility: Convert base64 to ArrayBuffer
   */
  private base64ToArrayBuffer(base64: string): ArrayBuffer {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
  }

  /**
   * Utility: Convert ArrayBuffer to hex string
   */
  private arrayBufferToHex(buffer: ArrayBuffer): string {
    const bytes = new Uint8Array(buffer);
    return Array.from(bytes)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
  }

  /**
   * Clear cached keys for security
   */
  clearCache(): void {
    this.keyCache.clear();
    this.keyPairCache.clear();
  }

  /**
   * Generate a random four-word passphrase for easy key exchange
   */
  generatePassphrase(): string {
    const words = [
      'swift', 'river', 'calm', 'peak', 'bright', 'moon', 'soft', 'cloud',
      'deep', 'forest', 'quiet', 'stream', 'warm', 'light', 'gentle', 'wind',
      'clear', 'sky', 'still', 'lake', 'fresh', 'dawn', 'peaceful', 'valley',
      'green', 'meadow', 'cool', 'breeze', 'golden', 'sunset', 'silver', 'star',
      'blue', 'ocean', 'white', 'snow', 'red', 'flame', 'purple', 'flower',
      'orange', 'leaf', 'yellow', 'sun', 'pink', 'rose', 'gray', 'stone',
    ];

    const selected = [];
    for (let i = 0; i < 4; i++) {
      const randomIndex = Math.floor(Math.random() * words.length);
      selected.push(words[randomIndex]);
    }

    return selected.join('-');
  }

  /**
   * Create encryption context for a user
   */
  async createUserEncryptionContext(user: UserIdentity): Promise<{
    masterKey: CryptoKey;
    keyPair: KeyPair;
    passphrase: string;
  }> {
    const masterKey = await this.generateEncryptionKey();
    const keyPair = await this.generateKeyPair('encryption');
    const passphrase = this.generatePassphrase();

    // Cache the keys for this session
    this.keyCache.set(`user:${user.id}:master`, masterKey);
    this.keyPairCache.set(`user:${user.id}:keypair`, keyPair);

    return {
      masterKey,
      keyPair,
      passphrase,
    };
  }
}

// Create singleton instance
export const cryptoManager = CryptoManager.getInstance();

// Utility functions for common encryption tasks
export const encryptUserData = async (
  data: any,
  userKey: CryptoKey
): Promise<string> => {
  const jsonString = JSON.stringify(data);
  return await cryptoManager.encryptText(jsonString, userKey);
};

export const decryptUserData = async <T = any>(
  encryptedData: string,
  userKey: CryptoKey
): Promise<T> => {
  const jsonString = await cryptoManager.decryptText(encryptedData, userKey);
  return JSON.parse(jsonString);
};

export const generateSecureId = async (): Promise<string> => {
  return await cryptoManager.generateKeyId();
};

export const hashPassword = async (password: string): Promise<string> => {
  const hash = await cryptoManager.hash(password, 'SHA-256');
  return cryptoManager['arrayBufferToHex'](hash);
};

export default cryptoManager;