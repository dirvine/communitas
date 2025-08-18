/**
 * Post-Quantum Cryptography Manager for Frontend
 * Uses Tauri commands to access ML-KEM-768 and ML-DSA-65 operations
 */

import { invoke } from '@tauri-apps/api/core';

// PQC types matching Rust backend
export interface PqcKeyPair {
  public_key: number[];
  secret_key: number[];
  algorithm: string;
  created_at: number;
}

export interface PqcSignature {
  signature: number[];
  algorithm: string;
  context: string;
}

export interface PqcEncryptionResult {
  ciphertext: number[];
  nonce: number[];
  ml_kem_ciphertext: number[];
  algorithm: string;
}

export interface PqcVerificationResult {
  is_valid: boolean;
  algorithm: string;
  details: Record<string, string>;
}

export interface PqcEncapsulationResult {
  ciphertext: number[];
  shared_secret: number[];
  algorithm: string;
}

export interface PqcInfo {
  ml_kem_768_public_key_size: string;
  ml_kem_768_secret_key_size: string;
  ml_kem_768_ciphertext_size: string;
  ml_dsa_65_public_key_size: string;
  ml_dsa_65_secret_key_size: string;
  ml_dsa_65_signature_size: string;
  chacha20poly1305_key_size: string;
  chacha20poly1305_nonce_size: string;
  algorithms: string;
  version: string;
}

/**
 * Post-Quantum Cryptography Manager
 * Provides ML-KEM-768 and ML-DSA-65 operations via Tauri bridge
 */
export class PqcCryptoManager {
  private static instance: PqcCryptoManager;
  private keyPairCache: Map<string, PqcKeyPair> = new Map();
  private pqcInfo: PqcInfo | null = null;

  static getInstance(): PqcCryptoManager {
    if (!PqcCryptoManager.instance) {
      PqcCryptoManager.instance = new PqcCryptoManager();
    }
    return PqcCryptoManager.instance;
  }

  /**
   * Get PQC algorithm information
   */
  async getPqcInfo(): Promise<PqcInfo> {
    if (this.pqcInfo) {
      return this.pqcInfo;
    }

    try {
      this.pqcInfo = await invoke<PqcInfo>('get_pqc_info');
      return this.pqcInfo;
    } catch (error) {
      throw new Error(`Failed to get PQC info: ${error}`);
    }
  }

  /**
   * Generate ML-DSA-65 key pair for digital signatures
   */
  async generateSigningKeyPair(): Promise<PqcKeyPair> {
    try {
      const keyPair = await invoke<PqcKeyPair>('generate_ml_dsa_keypair');
      const keyId = this.generateKeyId();
      this.keyPairCache.set(keyId, keyPair);
      return keyPair;
    } catch (error) {
      throw new Error(`Failed to generate ML-DSA-65 key pair: ${error}`);
    }
  }

  /**
   * Generate ML-KEM-768 key pair for key encapsulation
   */
  async generateEncryptionKeyPair(): Promise<PqcKeyPair> {
    try {
      const keyPair = await invoke<PqcKeyPair>('generate_ml_kem_keypair');
      const keyId = this.generateKeyId();
      this.keyPairCache.set(keyId, keyPair);
      return keyPair;
    } catch (error) {
      throw new Error(`Failed to generate ML-KEM-768 key pair: ${error}`);
    }
  }

  /**
   * Sign data with ML-DSA-65
   */
  async signData(
    data: Uint8Array,
    secretKey: number[],
    context: string = 'default'
  ): Promise<PqcSignature> {
    try {
      return await invoke<PqcSignature>('ml_dsa_sign', {
        data: Array.from(data),
        secretKeyBytes: secretKey,
        context,
      });
    } catch (error) {
      throw new Error(`Failed to sign data with ML-DSA-65: ${error}`);
    }
  }

  /**
   * Verify ML-DSA-65 signature
   */
  async verifySignature(
    data: Uint8Array,
    signature: number[],
    publicKey: number[],
    context: string = 'default'
  ): Promise<PqcVerificationResult> {
    try {
      return await invoke<PqcVerificationResult>('ml_dsa_verify', {
        data: Array.from(data),
        signatureBytes: signature,
        publicKeyBytes: publicKey,
        context,
      });
    } catch (error) {
      throw new Error(`Failed to verify ML-DSA-65 signature: ${error}`);
    }
  }

  /**
   * Encapsulate shared secret with ML-KEM-768
   */
  async encapsulateKey(publicKey: number[]): Promise<PqcEncapsulationResult> {
    try {
      return await invoke<PqcEncapsulationResult>('ml_kem_encapsulate', {
        publicKeyBytes: publicKey,
      });
    } catch (error) {
      throw new Error(`Failed to encapsulate key with ML-KEM-768: ${error}`);
    }
  }

  /**
   * Decapsulate shared secret with ML-KEM-768
   */
  async decapsulateKey(
    ciphertext: number[],
    secretKey: number[]
  ): Promise<Uint8Array> {
    try {
      const result = await invoke<number[]>('ml_kem_decapsulate', {
        ciphertextBytes: ciphertext,
        secretKeyBytes: secretKey,
      });
      return new Uint8Array(result);
    } catch (error) {
      throw new Error(`Failed to decapsulate key with ML-KEM-768: ${error}`);
    }
  }

  /**
   * Encrypt data using hybrid PQC (ML-KEM-768 + ChaCha20Poly1305)
   */
  async encryptData(
    data: Uint8Array,
    publicKey: number[]
  ): Promise<PqcEncryptionResult> {
    try {
      return await invoke<PqcEncryptionResult>('pqc_encrypt', {
        data: Array.from(data),
        publicKeyBytes: publicKey,
      });
    } catch (error) {
      throw new Error(`Failed to encrypt data with PQC: ${error}`);
    }
  }

  /**
   * Decrypt data using hybrid PQC (ML-KEM-768 + ChaCha20Poly1305)
   */
  async decryptData(
    ciphertext: number[],
    nonce: number[],
    mlKemCiphertext: number[],
    secretKey: number[]
  ): Promise<Uint8Array> {
    try {
      const result = await invoke<number[]>('pqc_decrypt', {
        ciphertext,
        nonce,
        mlKemCiphertext,
        secretKeyBytes: secretKey,
      });
      return new Uint8Array(result);
    } catch (error) {
      throw new Error(`Failed to decrypt data with PQC: ${error}`);
    }
  }

  /**
   * Encrypt file with PQC
   */
  async encryptFile(
    file: File,
    publicKey: number[]
  ): Promise<{
    encryptedData: PqcEncryptionResult;
    originalName: string;
    originalSize: number;
    mimeType: string;
  }> {
    const arrayBuffer = await file.arrayBuffer();
    const data = new Uint8Array(arrayBuffer);
    
    const encryptedData = await this.encryptData(data, publicKey);
    
    return {
      encryptedData,
      originalName: file.name,
      originalSize: file.size,
      mimeType: file.type,
    };
  }

  /**
   * Decrypt file with PQC
   */
  async decryptFile(
    encryptedData: PqcEncryptionResult,
    secretKey: number[],
    originalName: string,
    mimeType: string
  ): Promise<File> {
    const decryptedData = await this.decryptData(
      encryptedData.ciphertext,
      encryptedData.nonce,
      encryptedData.ml_kem_ciphertext,
      secretKey
    );
    
    return new File([decryptedData], originalName, { type: mimeType });
  }

  /**
   * Sign and encrypt data (authenticated encryption)
   */
  async signAndEncrypt(
    data: Uint8Array,
    encryptionPublicKey: number[],
    signingSecretKey: number[],
    context: string = 'authenticated-encryption'
  ): Promise<{
    encryptedData: PqcEncryptionResult;
    signature: PqcSignature;
  }> {
    // First encrypt the data
    const encryptedData = await this.encryptData(data, encryptionPublicKey);
    
    // Then sign the encrypted data for authenticity
    const dataToSign = new Uint8Array([
      ...encryptedData.ciphertext,
      ...encryptedData.nonce,
      ...encryptedData.ml_kem_ciphertext,
    ]);
    
    const signature = await this.signData(dataToSign, signingSecretKey, context);
    
    return {
      encryptedData,
      signature,
    };
  }

  /**
   * Verify and decrypt data (authenticated decryption)
   */
  async verifyAndDecrypt(
    encryptedData: PqcEncryptionResult,
    signature: PqcSignature,
    decryptionSecretKey: number[],
    verificationPublicKey: number[],
    context: string = 'authenticated-encryption'
  ): Promise<Uint8Array> {
    // First verify the signature
    const dataToVerify = new Uint8Array([
      ...encryptedData.ciphertext,
      ...encryptedData.nonce,
      ...encryptedData.ml_kem_ciphertext,
    ]);
    
    const verificationResult = await this.verifySignature(
      dataToVerify,
      signature.signature,
      verificationPublicKey,
      context
    );
    
    if (!verificationResult.is_valid) {
      throw new Error('Signature verification failed');
    }
    
    // Then decrypt the data
    return await this.decryptData(
      encryptedData.ciphertext,
      encryptedData.nonce,
      encryptedData.ml_kem_ciphertext,
      decryptionSecretKey
    );
  }

  /**
   * Generate a unique key identifier
   */
  private generateKeyId(): string {
    return `pqc-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Clear key cache (for security)
   */
  clearCache(): void {
    this.keyPairCache.clear();
  }

  /**
   * Convert between different data formats
   */
  utils = {
    /**
     * Convert string to Uint8Array
     */
    stringToBytes(str: string): Uint8Array {
      return new TextEncoder().encode(str);
    },

    /**
     * Convert Uint8Array to string
     */
    bytesToString(bytes: Uint8Array): string {
      return new TextDecoder().decode(bytes);
    },

    /**
     * Convert ArrayBuffer to number array
     */
    arrayBufferToNumbers(buffer: ArrayBuffer): number[] {
      return Array.from(new Uint8Array(buffer));
    },

    /**
     * Convert number array to Uint8Array
     */
    numbersToBytes(numbers: number[]): Uint8Array {
      return new Uint8Array(numbers);
    },

    /**
     * Convert Uint8Array to base64 string
     */
    bytesToBase64(bytes: Uint8Array): string {
      return btoa(String.fromCharCode(...bytes));
    },

    /**
     * Convert base64 string to Uint8Array
     */
    base64ToBytes(base64: string): Uint8Array {
      const binaryString = atob(base64);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      return bytes;
    },
  };
}

// Export singleton instance
export const pqcCrypto = PqcCryptoManager.getInstance();

// Export default for convenience
export default pqcCrypto;