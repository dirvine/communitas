/**
 * LocalVaultService - Password-encrypted local vault for Communitas
 *
 * Provides secure, local-first storage for user identity, friends, organizations,
 * and other critical data. Uses password-based encryption with PBKDF2 for key derivation
 * and AES-GCM for data encryption. All data is stored encrypted in IndexedDB.
 */

import { UserIdentity } from '../../contexts/AuthContext';

// Vault configuration
const VAULT_DB_NAME = 'communitas-vault';
const VAULT_DB_VERSION = 1;
const VAULT_STORE_NAME = 'encrypted-vaults';
const PBKDF2_ITERATIONS = 100000;
const SALT_LENGTH = 32;
const IV_LENGTH = 12;

// Vault data structure
export interface VaultData {
  identity: UserIdentity | null;
  friends: Friend[];
  organizations: Organization[];
  groups: Group[];
  settings: VaultSettings;
  lastSyncedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface Friend {
  fourWordAddress: string;
  name: string;
  displayName?: string;
  avatarUrl?: string;
  publicKey?: string;
  addedAt: string;
  lastSeenAt?: string;
  status: 'pending' | 'accepted' | 'blocked';
}

export interface Organization {
  id: string;
  name: string;
  description?: string;
  fourWordAddress: string;
  role: 'owner' | 'admin' | 'member' | 'viewer';
  joinedAt: string;
  members?: Friend[];
}

export interface Group {
  id: string;
  name: string;
  organizationId?: string;
  members: string[]; // fourWordAddresses
  isPrivate: boolean;
  createdAt: string;
}

export interface VaultSettings {
  autoSync: boolean;
  encryptionEnabled: boolean;
  theme?: 'light' | 'dark' | 'auto';
  notifications: boolean;
}

// Encrypted vault structure stored in IndexedDB
interface EncryptedVault {
  id: string; // fourWordAddress
  salt: ArrayBuffer;
  iv: ArrayBuffer;
  encryptedData: ArrayBuffer;
  checksum: string;
  version: number;
  createdAt: string;
  lastAccessedAt: string;
}

export class LocalVaultService {
  private static instance: LocalVaultService;
  private db: IDBDatabase | null = null;
  private currentVault: VaultData | null = null;
  private encryptionKey: CryptoKey | null = null;
  private currentUserId: string | null = null;

  private constructor() {}

  static getInstance(): LocalVaultService {
    if (!LocalVaultService.instance) {
      LocalVaultService.instance = new LocalVaultService();
    }
    return LocalVaultService.instance;
  }

  /**
   * Initialize the vault database
   */
  async initialize(): Promise<void> {
    if (this.db) return;

    return new Promise((resolve, reject) => {
      const request = indexedDB.open(VAULT_DB_NAME, VAULT_DB_VERSION);

      request.onerror = () => reject(new Error('Failed to open vault database'));

      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        if (!db.objectStoreNames.contains(VAULT_STORE_NAME)) {
          const store = db.createObjectStore(VAULT_STORE_NAME, { keyPath: 'id' });
          store.createIndex('lastAccessedAt', 'lastAccessedAt', { unique: false });
          store.createIndex('createdAt', 'createdAt', { unique: false });
        }
      };
    });
  }

  /**
   * Create or open a vault with password
   */
  async openVault(fourWordAddress: string, password: string): Promise<VaultData> {
    await this.initialize();

    // Check if vault exists
    const existingVault = await this.getEncryptedVault(fourWordAddress);

    if (existingVault) {
      // Decrypt existing vault
      return await this.decryptVault(existingVault, password);
    } else {
      // Create new vault
      return await this.createVault(fourWordAddress, password);
    }
  }

  /**
   * Create a new vault
   */
  private async createVault(fourWordAddress: string, password: string): Promise<VaultData> {
    const now = new Date().toISOString();

    const vaultData: VaultData = {
      identity: null,
      friends: [],
      organizations: [],
      groups: [],
      settings: {
        autoSync: true,
        encryptionEnabled: true,
        notifications: true,
      },
      lastSyncedAt: null,
      createdAt: now,
      updatedAt: now,
    };

    // Generate encryption key from password
    const salt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH));
    this.encryptionKey = await this.deriveKey(password, salt);

    // Save vault
    await this.saveVault(fourWordAddress, vaultData, salt);

    this.currentVault = vaultData;
    this.currentUserId = fourWordAddress;

    return vaultData;
  }

  /**
   * Decrypt an existing vault
   */
  private async decryptVault(encryptedVault: EncryptedVault, password: string): Promise<VaultData> {
    try {
      // Derive key from password and stored salt
      const key = await this.deriveKey(password, new Uint8Array(encryptedVault.salt));

      // Decrypt data
      const decrypted = await crypto.subtle.decrypt(
        {
          name: 'AES-GCM',
          iv: new Uint8Array(encryptedVault.iv),
        },
        key,
        encryptedVault.encryptedData
      );

      // Parse decrypted data
      const decoder = new TextDecoder();
      const jsonStr = decoder.decode(decrypted);
      const vaultData = JSON.parse(jsonStr) as VaultData;

      // Update last accessed time
      await this.updateLastAccessed(encryptedVault.id);

      // Cache for current session
      this.encryptionKey = key;
      this.currentVault = vaultData;
      this.currentUserId = encryptedVault.id;

      return vaultData;
    } catch (error) {
      throw new Error('Invalid password or corrupted vault');
    }
  }

  /**
   * Save vault data (encrypted)
   */
  async saveVault(fourWordAddress: string, vaultData: VaultData, salt?: Uint8Array): Promise<void> {
    if (!this.encryptionKey) {
      throw new Error('No encryption key available');
    }

    // Update timestamp
    vaultData.updatedAt = new Date().toISOString();

    // Serialize data
    const encoder = new TextEncoder();
    const dataStr = JSON.stringify(vaultData);
    const dataBuffer = encoder.encode(dataStr);

    // Generate IV for encryption
    const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH));

    // Encrypt data
    const encrypted = await crypto.subtle.encrypt(
      {
        name: 'AES-GCM',
        iv: iv,
      },
      this.encryptionKey,
      dataBuffer
    );

    // Calculate checksum
    const checksum = await this.calculateChecksum(encrypted);

    // Use existing salt or the one provided
    const vaultSalt = salt || await this.getVaultSalt(fourWordAddress);
    if (!vaultSalt) {
      throw new Error('No salt available for vault encryption');
    }

    // Store encrypted vault
    const encryptedVault: EncryptedVault = {
      id: fourWordAddress,
      salt: vaultSalt.buffer.slice(0) as ArrayBuffer,
      iv: iv.buffer.slice(0) as ArrayBuffer,
      encryptedData: encrypted,
      checksum,
      version: 1,
      createdAt: vaultData.createdAt,
      lastAccessedAt: new Date().toISOString(),
    };

    await this.storeEncryptedVault(encryptedVault);
    this.currentVault = vaultData;
  }

  /**
   * Update identity in vault
   */
  async updateIdentity(identity: UserIdentity): Promise<void> {
    if (!this.currentVault || !this.currentUserId) {
      throw new Error('No vault is currently open');
    }

    this.currentVault.identity = identity;
    await this.saveVault(this.currentUserId, this.currentVault);
  }

  /**
   * Add or update a friend
   */
  async addFriend(friend: Friend): Promise<void> {
    if (!this.currentVault || !this.currentUserId) {
      throw new Error('No vault is currently open');
    }

    const existingIndex = this.currentVault.friends.findIndex(
      f => f.fourWordAddress === friend.fourWordAddress
    );

    if (existingIndex >= 0) {
      this.currentVault.friends[existingIndex] = friend;
    } else {
      this.currentVault.friends.push(friend);
    }

    await this.saveVault(this.currentUserId, this.currentVault);
  }

  /**
   * Add or update an organization
   */
  async addOrganization(org: Organization): Promise<void> {
    if (!this.currentVault || !this.currentUserId) {
      throw new Error('No vault is currently open');
    }

    const existingIndex = this.currentVault.organizations.findIndex(
      o => o.id === org.id
    );

    if (existingIndex >= 0) {
      this.currentVault.organizations[existingIndex] = org;
    } else {
      this.currentVault.organizations.push(org);
    }

    await this.saveVault(this.currentUserId, this.currentVault);
  }

  /**
   * Get current vault data
   */
  getCurrentVault(): VaultData | null {
    return this.currentVault;
  }

  /**
   * Check if a vault exists for a user
   */
  async vaultExists(fourWordAddress: string): Promise<boolean> {
    await this.initialize();
    const vault = await this.getEncryptedVault(fourWordAddress);
    return vault !== null;
  }

  /**
   * Get all vault IDs (four-word addresses) stored on this device
   */
  async getAllVaultIds(): Promise<string[]> {
    await this.initialize();

    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([VAULT_STORE_NAME], 'readonly');
      const store = transaction.objectStore(VAULT_STORE_NAME);
      const request = store.getAllKeys();

      request.onsuccess = () => {
        const keys = request.result as string[];
        resolve(keys);
      };

      request.onerror = () => reject(new Error('Failed to get vault IDs'));
    });
  }

  /**
   * Try to decrypt any vault with the given password
   * Returns the first successful match with four-word address
   */
  async tryDecryptWithPassword(password: string): Promise<{ fourWords: string; vault: VaultData } | null> {
    await this.initialize();

    // Get all vault IDs
    const vaultIds = await this.getAllVaultIds();

    // Try each vault with the provided password
    for (const fourWords of vaultIds) {
      try {
        const encryptedVault = await this.getEncryptedVault(fourWords);
        if (!encryptedVault) continue;

        // Try to decrypt with the given password
        const vaultData = await this.decryptVault(encryptedVault, password);

        // If successful, return the match
        return { fourWords, vault: vaultData };
      } catch (error) {
        // Password didn't match this vault, continue to next
        continue;
      }
    }

    // No matching vault found
    return null;
  }

  /**
   * Close current vault (clear from memory)
   */
  closeVault(): void {
    this.currentVault = null;
    this.encryptionKey = null;
    this.currentUserId = null;
  }

  /**
   * Delete a vault permanently
   */
  async deleteVault(fourWordAddress: string, password: string): Promise<void> {
    // Verify password first
    await this.openVault(fourWordAddress, password);

    // Delete from IndexedDB
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([VAULT_STORE_NAME], 'readwrite');
      const store = transaction.objectStore(VAULT_STORE_NAME);
      const request = store.delete(fourWordAddress);

      request.onsuccess = () => {
        this.closeVault();
        resolve();
      };

      request.onerror = () => reject(new Error('Failed to delete vault'));
    });
  }

  /**
   * Change vault password
   */
  async changePassword(fourWordAddress: string, oldPassword: string, newPassword: string): Promise<void> {
    // Open vault with old password to verify
    const vaultData = await this.openVault(fourWordAddress, oldPassword);

    // Generate new salt and derive new key
    const newSalt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH));
    this.encryptionKey = await this.deriveKey(newPassword, newSalt);

    // Re-encrypt and save with new key
    await this.saveVault(fourWordAddress, vaultData, newSalt);
  }

  /**
   * Export vault data (decrypted, for backup)
   */
  async exportVault(fourWordAddress: string, password: string): Promise<string> {
    const vaultData = await this.openVault(fourWordAddress, password);
    return JSON.stringify(vaultData, null, 2);
  }

  /**
   * Import vault data
   */
  async importVault(fourWordAddress: string, password: string, jsonData: string): Promise<void> {
    const vaultData = JSON.parse(jsonData) as VaultData;

    // Create new vault with imported data
    const salt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH));
    this.encryptionKey = await this.deriveKey(password, salt);

    await this.saveVault(fourWordAddress, vaultData, salt);
  }

  // Private helper methods

  private async deriveKey(password: string, salt: Uint8Array): Promise<CryptoKey> {
    const encoder = new TextEncoder();
    const passwordBuffer = encoder.encode(password);

    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      passwordBuffer,
      'PBKDF2',
      false,
      ['deriveBits', 'deriveKey']
    );

    return crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: salt.buffer.slice(salt.byteOffset, salt.byteOffset + salt.byteLength) as ArrayBuffer,
        iterations: PBKDF2_ITERATIONS,
        hash: 'SHA-256',
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );
  }

  private async calculateChecksum(data: ArrayBuffer): Promise<string> {
    const hash = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hash));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }

  private async getEncryptedVault(fourWordAddress: string): Promise<EncryptedVault | null> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([VAULT_STORE_NAME], 'readonly');
      const store = transaction.objectStore(VAULT_STORE_NAME);
      const request = store.get(fourWordAddress);

      request.onsuccess = () => {
        resolve(request.result || null);
      };

      request.onerror = () => reject(new Error('Failed to get vault'));
    });
  }

  private async storeEncryptedVault(vault: EncryptedVault): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([VAULT_STORE_NAME], 'readwrite');
      const store = transaction.objectStore(VAULT_STORE_NAME);
      const request = store.put(vault);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error('Failed to store vault'));
    });
  }

  private async getVaultSalt(fourWordAddress: string): Promise<Uint8Array | null> {
    const vault = await this.getEncryptedVault(fourWordAddress);
    return vault ? new Uint8Array(vault.salt) : null;
  }

  private async updateLastAccessed(fourWordAddress: string): Promise<void> {
    const vault = await this.getEncryptedVault(fourWordAddress);
    if (vault) {
      vault.lastAccessedAt = new Date().toISOString();
      await this.storeEncryptedVault(vault);
    }
  }
}

// Export singleton instance
export const localVault = LocalVaultService.getInstance();