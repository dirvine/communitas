/**
 * Browser Vault - Web Crypto API based vault for browser mode
 *
 * Provides encrypted storage for identities when Tauri backend is not available.
 * Uses Web Crypto API for encryption and localStorage for persistence.
 */

export interface VaultData {
  fourWords: string;
  displayName: string;
  encryptedData: string;
  salt: string;
  iv: string;
  createdAt: string;
}

export interface CreateVaultOptions {
  fourWords: string;
  password: string;
  displayName: string;
}

class BrowserVault {
  private readonly STORAGE_KEY = 'communitas_vault';

  /**
   * Derive encryption key from password using PBKDF2
   */
  private async deriveKey(password: string, salt: Uint8Array): Promise<CryptoKey> {
    const encoder = new TextEncoder();
    const passwordKey = await crypto.subtle.importKey(
      'raw',
      encoder.encode(password),
      'PBKDF2',
      false,
      ['deriveKey']
    );

    return crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations: 100000,
        hash: 'SHA-256'
      },
      passwordKey,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
  }

  /**
   * Create a new encrypted vault
   */
  async createVault(options: CreateVaultOptions): Promise<void> {
    try {
      // Generate random salt and IV
      const salt = crypto.getRandomValues(new Uint8Array(16));
      const iv = crypto.getRandomValues(new Uint8Array(12));

      // Derive encryption key from password
      const key = await this.deriveKey(options.password, salt);

      // Encrypt the four-word identity
      const encoder = new TextEncoder();
      const data = encoder.encode(JSON.stringify({
        fourWords: options.fourWords,
        displayName: options.displayName,
        createdAt: new Date().toISOString()
      }));

      const encryptedData = await crypto.subtle.encrypt(
        { name: 'AES-GCM', iv },
        key,
        data
      );

      // Store vault data
      const vault: VaultData = {
        fourWords: options.fourWords, // Store in plaintext for identity lookup
        displayName: options.displayName,
        encryptedData: this.arrayBufferToBase64(encryptedData),
        salt: this.arrayBufferToBase64(salt),
        iv: this.arrayBufferToBase64(iv),
        createdAt: new Date().toISOString()
      };

      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(vault));
      console.log('✅ Browser vault created successfully');
    } catch (error) {
      console.error('❌ Vault creation failed:', error);
      throw new Error('Failed to create vault: ' + (error as Error).message);
    }
  }

  /**
   * Unlock vault with password
   */
  async unlockVault(password: string): Promise<{ fourWords: string; displayName: string }> {
    try {
      const vaultJson = localStorage.getItem(this.STORAGE_KEY);
      if (!vaultJson) {
        throw new Error('No vault found');
      }

      const vault: VaultData = JSON.parse(vaultJson);

      // Derive key from password
      const salt = this.base64ToArrayBuffer(vault.salt);
      const key = await this.deriveKey(password, new Uint8Array(salt));

      // Decrypt data
      const iv = this.base64ToArrayBuffer(vault.iv);
      const encryptedData = this.base64ToArrayBuffer(vault.encryptedData);

      const decryptedData = await crypto.subtle.decrypt(
        { name: 'AES-GCM', iv: new Uint8Array(iv) },
        key,
        encryptedData
      );

      const decoder = new TextDecoder();
      const data = JSON.parse(decoder.decode(decryptedData));

      return {
        fourWords: data.fourWords,
        displayName: data.displayName
      };
    } catch (error) {
      console.error('❌ Vault unlock failed:', error);
      throw new Error('Invalid password or corrupted vault');
    }
  }

  /**
   * Check if a vault exists
   */
  hasVault(): boolean {
    return localStorage.getItem(this.STORAGE_KEY) !== null;
  }

  /**
   * Get vault info (without decrypting)
   */
  getVaultInfo(): { fourWords: string; displayName: string; createdAt: string } | null {
    const vaultJson = localStorage.getItem(this.STORAGE_KEY);
    if (!vaultJson) return null;

    const vault: VaultData = JSON.parse(vaultJson);
    return {
      fourWords: vault.fourWords,
      displayName: vault.displayName,
      createdAt: vault.createdAt
    };
  }

  /**
   * Delete vault
   */
  deleteVault(): void {
    localStorage.removeItem(this.STORAGE_KEY);
  }

  /**
   * Convert ArrayBuffer to Base64
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
   * Convert Base64 to ArrayBuffer
   */
  private base64ToArrayBuffer(base64: string): ArrayBuffer {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
  }
}

// Singleton instance
export const browserVault = new BrowserVault();

// Export for testing
export { BrowserVault };
