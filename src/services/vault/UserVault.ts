/**
 * UserVault - CRDT-based user data structure using Yjs
 *
 * Provides conflict-free replicated data types for user data that can be
 * safely merged when reconnecting to the network. Uses Yjs for CRDT operations
 * and IndexedDB persistence for offline support.
 */

import { IndexeddbPersistence } from 'y-indexeddb';
import * as Y from 'yjs';
import { UserIdentity } from '../../contexts/AuthContext';
import { Friend, Group, localVault, Organization, VaultData } from './LocalVaultService';

// Event types for subscription
export type VaultEventType = 'identity' | 'friends' | 'organizations' | 'groups' | 'sync';

export interface VaultEvent {
  type: VaultEventType;
  data: any;
  timestamp: string;
}

// Sync status
export interface SyncStatus {
  isOnline: boolean;
  lastSyncedAt: string | null;
  pendingChanges: number;
  syncInProgress: boolean;
  error: string | null;
}

export class UserVault {
  private doc: Y.Doc;
  private persistence: IndexeddbPersistence | null = null;
  private userId: string;
  private isInitialized = false;

  // Yjs shared types
  private identityMap: Y.Map<any>;
  private friendsArray: Y.Array<Friend>;
  private organizationsArray: Y.Array<Organization>;
  private groupsArray: Y.Array<Group>;
  private settingsMap: Y.Map<any>;
  private syncStatusMap: Y.Map<any>;

  // Event listeners
  private listeners: Map<string, Set<(event: VaultEvent) => void>> = new Map();

  // Sync queue for network operations
  private syncQueue: Array<{ action: string; data: any; timestamp: string }> = [];
  private syncTimer: NodeJS.Timeout | null = null;

  constructor(userId: string) {
    this.userId = userId;
    this.doc = new Y.Doc();

    // Initialize shared types
    this.identityMap = this.doc.getMap('identity');
    this.friendsArray = this.doc.getArray('friends');
    this.organizationsArray = this.doc.getArray('organizations');
    this.groupsArray = this.doc.getArray('groups');
    this.settingsMap = this.doc.getMap('settings');
    this.syncStatusMap = this.doc.getMap('syncStatus');

    this.setupObservers();
  }

  /**
   * Initialize the vault with persistence
   */
  async initialize(vaultData?: VaultData): Promise<void> {
    if (this.isInitialized) return;

    // Set up IndexedDB persistence for CRDT document
    this.persistence = new IndexeddbPersistence(`vault-crdt-${this.userId}`, this.doc);

    // Wait for initial load from IndexedDB
    await new Promise<void>((resolve) => {
      this.persistence!.on('synced', () => {
        resolve();
      });

      // Timeout fallback
      setTimeout(() => resolve(), 1000);
    });

    // If vault data provided, merge it with CRDT document
    if (vaultData) {
      await this.mergeVaultData(vaultData);
    }

    this.isInitialized = true;
    this.emit('sync', { status: 'initialized' });
  }

  /**
   * Set up observers for CRDT changes
   */
  private setupObservers(): void {
    // Observe identity changes
    this.identityMap.observe((event) => {
      this.emit('identity', { changes: event.changes });
      this.queueSync('identity', this.getIdentity());
    });

    // Observe friends changes
    this.friendsArray.observe((event) => {
      this.emit('friends', { changes: event.changes });
      this.queueSync('friends', this.getFriends());
    });

    // Observe organizations changes
    this.organizationsArray.observe((event) => {
      this.emit('organizations', { changes: event.changes });
      this.queueSync('organizations', this.getOrganizations());
    });

    // Observe groups changes
    this.groupsArray.observe((event) => {
      this.emit('groups', { changes: event.changes });
      this.queueSync('groups', this.getGroups());
    });
  }

  /**
   * Merge vault data into CRDT document
   */
  private async mergeVaultData(vaultData: VaultData): Promise<void> {
    this.doc.transact(() => {
      // Merge identity
      if (vaultData.identity) {
        Object.entries(vaultData.identity).forEach(([key, value]) => {
          this.identityMap.set(key, value);
        });
      }

      // Merge friends (deduplicate by fourWordAddress)
      const existingFriends = new Set(
        this.friendsArray.toArray().map(f => f.fourWordAddress)
      );

      vaultData.friends.forEach(friend => {
        if (!existingFriends.has(friend.fourWordAddress)) {
          this.friendsArray.push([friend]);
        }
      });

      // Merge organizations (deduplicate by id)
      const existingOrgs = new Set(
        this.organizationsArray.toArray().map(o => o.id)
      );

      vaultData.organizations.forEach(org => {
        if (!existingOrgs.has(org.id)) {
          this.organizationsArray.push([org]);
        }
      });

      // Merge groups (deduplicate by id)
      const existingGroups = new Set(
        this.groupsArray.toArray().map(g => g.id)
      );

      vaultData.groups.forEach(group => {
        if (!existingGroups.has(group.id)) {
          this.groupsArray.push([group]);
        }
      });

      // Merge settings
      Object.entries(vaultData.settings).forEach(([key, value]) => {
        this.settingsMap.set(key, value);
      });

      // Update sync status
      this.syncStatusMap.set('lastSyncedAt', vaultData.lastSyncedAt);
      this.syncStatusMap.set('updatedAt', new Date().toISOString());
    });
  }

  /**
   * Update identity
   */
  async updateIdentity(identity: UserIdentity): Promise<void> {
    this.doc.transact(() => {
      Object.entries(identity).forEach(([key, value]) => {
        this.identityMap.set(key, value);
      });
    });

    // Also update in encrypted vault
    const vault = localVault.getCurrentVault();
    if (vault) {
      await localVault.updateIdentity(identity);
    }
  }

  /**
   * Add or update a friend
   */
  async addFriend(friend: Friend): Promise<void> {
    this.doc.transact(() => {
      const friends = this.friendsArray.toArray();
      const existingIndex = friends.findIndex(
        f => f.fourWordAddress === friend.fourWordAddress
      );

      if (existingIndex >= 0) {
        // Update existing friend
        this.friendsArray.delete(existingIndex, 1);
        this.friendsArray.insert(existingIndex, [friend]);
      } else {
        // Add new friend
        this.friendsArray.push([friend]);
      }
    });

    // Also update in encrypted vault
    await localVault.addFriend(friend);
  }

  /**
   * Add or update an organization
   */
  async addOrganization(org: Organization): Promise<void> {
    this.doc.transact(() => {
      const orgs = this.organizationsArray.toArray();
      const existingIndex = orgs.findIndex(o => o.id === org.id);

      if (existingIndex >= 0) {
        // Update existing organization
        this.organizationsArray.delete(existingIndex, 1);
        this.organizationsArray.insert(existingIndex, [org]);
      } else {
        // Add new organization
        this.organizationsArray.push([org]);
      }
    });

    // Also update in encrypted vault
    await localVault.addOrganization(org);
  }

  /**
   * Add or update a group
   */
  async addGroup(group: Group): Promise<void> {
    this.doc.transact(() => {
      const groups = this.groupsArray.toArray();
      const existingIndex = groups.findIndex(g => g.id === group.id);

      if (existingIndex >= 0) {
        // Update existing group
        this.groupsArray.delete(existingIndex, 1);
        this.groupsArray.insert(existingIndex, [group]);
      } else {
        // Add new group
        this.groupsArray.push([group]);
      }
    });
  }

  /**
   * Get current identity
   */
  getIdentity(): UserIdentity | null {
    const identity: any = {};
    this.identityMap.forEach((value, key) => {
      identity[key] = value;
    });

    return Object.keys(identity).length > 0 ? (identity as UserIdentity) : null;
  }

  /**
   * Get all friends
   */
  getFriends(): Friend[] {
    return this.friendsArray.toArray();
  }

  /**
   * Get all organizations
   */
  getOrganizations(): Organization[] {
    return this.organizationsArray.toArray();
  }

  /**
   * Get all groups
   */
  getGroups(): Group[] {
    return this.groupsArray.toArray();
  }

  /**
   * Get settings
   */
  getSettings(): any {
    const settings: any = {};
    this.settingsMap.forEach((value, key) => {
      settings[key] = value;
    });
    return settings;
  }

  /**
   * Get sync status
   */
  getSyncStatus(): SyncStatus {
    return {
      isOnline: this.syncStatusMap.get('isOnline') || false,
      lastSyncedAt: this.syncStatusMap.get('lastSyncedAt') || null,
      pendingChanges: this.syncQueue.length,
      syncInProgress: this.syncStatusMap.get('syncInProgress') || false,
      error: this.syncStatusMap.get('error') || null,
    };
  }

  /**
   * Queue changes for sync when online
   */
  private queueSync(action: string, data: any): void {
    this.syncQueue.push({
      action,
      data,
      timestamp: new Date().toISOString(),
    });

    // Debounce sync attempts
    if (this.syncTimer) {
      clearTimeout(this.syncTimer);
    }

    this.syncTimer = setTimeout(() => {
      this.attemptSync();
    }, 1000);
  }

  /**
   * Attempt to sync queued changes with network
   */
  private async attemptSync(): Promise<void> {
    if (this.syncQueue.length === 0) return;

    const status = this.getSyncStatus();
    if (!status.isOnline || status.syncInProgress) return;

    this.syncStatusMap.set('syncInProgress', true);
    this.emit('sync', { status: 'syncing', pendingChanges: this.syncQueue.length });

    try {
      // Process sync queue
      while (this.syncQueue.length > 0) {
        const item = this.syncQueue.shift()!;

        // Here we would normally sync with the DHT/network
        // For now, just mark as synced
        console.log('[UserVault] Would sync:', item.action, item.data);
      }

      this.syncStatusMap.set('lastSyncedAt', new Date().toISOString());
      this.syncStatusMap.set('error', null);
      this.emit('sync', { status: 'completed' });
    } catch (error) {
      this.syncStatusMap.set('error', (error as Error).message);
      this.emit('sync', { status: 'error', error });

      // Re-queue failed items
      // Items remain in queue for retry
    } finally {
      this.syncStatusMap.set('syncInProgress', false);
    }
  }

  /**
   * Force sync with network
   */
  async forceSync(): Promise<void> {
    this.syncStatusMap.set('isOnline', true);
    await this.attemptSync();
  }

  /**
   * Set online status
   */
  setOnlineStatus(isOnline: boolean): void {
    this.syncStatusMap.set('isOnline', isOnline);

    if (isOnline) {
      this.attemptSync();
    }
  }

  /**
   * Subscribe to vault events
   */
  subscribe(eventType: VaultEventType, callback: (event: VaultEvent) => void): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }

    this.listeners.get(eventType)!.add(callback);

    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(eventType);
      if (listeners) {
        listeners.delete(callback);
      }
    };
  }

  /**
   * Emit an event to listeners
   */
  private emit(type: VaultEventType, data: any): void {
    const listeners = this.listeners.get(type);
    if (!listeners) return;

    const event: VaultEvent = {
      type,
      data,
      timestamp: new Date().toISOString(),
    };

    listeners.forEach(callback => {
      try {
        callback(event);
      } catch (error) {
        console.error('[UserVault] Listener error:', error);
      }
    });
  }

  /**
   * Get CRDT update for network sync
   */
  getUpdate(): Uint8Array {
    return Y.encodeStateAsUpdate(this.doc);
  }

  /**
   * Apply CRDT update from network
   */
  applyUpdate(update: Uint8Array): void {
    Y.applyUpdate(this.doc, update);
  }

  /**
   * Export vault data as JSON
   */
  exportData(): VaultData {
    return {
      identity: this.getIdentity(),
      friends: this.getFriends(),
      organizations: this.getOrganizations(),
      groups: this.getGroups(),
      settings: this.getSettings(),
      lastSyncedAt: this.syncStatusMap.get('lastSyncedAt') || null,
      createdAt: this.syncStatusMap.get('createdAt') || new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  }

  /**
   * Destroy the vault and clean up resources
   */
  destroy(): void {
    if (this.syncTimer) {
      clearTimeout(this.syncTimer);
    }

    if (this.persistence) {
      this.persistence.destroy();
    }

    this.doc.destroy();
    this.listeners.clear();
    this.syncQueue = [];
  }
}

// Vault manager for managing multiple vaults
export class VaultManager {
  private static instance: VaultManager;
  private vaults: Map<string, UserVault> = new Map();
  private activeVault: UserVault | null = null;

  private constructor() {}

  static getInstance(): VaultManager {
    if (!VaultManager.instance) {
      VaultManager.instance = new VaultManager();
    }
    return VaultManager.instance;
  }

  /**
   * Open or create a user vault
   */
  async openVault(fourWordAddress: string, password: string): Promise<UserVault> {
    // Check if vault already open
    if (this.vaults.has(fourWordAddress)) {
      this.activeVault = this.vaults.get(fourWordAddress)!;
      return this.activeVault;
    }

    // Open encrypted vault
    const vaultData = await localVault.openVault(fourWordAddress, password);

    // Create CRDT vault
    const userVault = new UserVault(fourWordAddress);
    await userVault.initialize(vaultData);

    // Cache and set as active
    this.vaults.set(fourWordAddress, userVault);
    this.activeVault = userVault;

    return userVault;
  }

  /**
   * Get active vault
   */
  getActiveVault(): UserVault | null {
    return this.activeVault;
  }

  /**
   * Close a vault
   */
  closeVault(fourWordAddress: string): void {
    const vault = this.vaults.get(fourWordAddress);
    if (vault) {
      vault.destroy();
      this.vaults.delete(fourWordAddress);

      if (this.activeVault === vault) {
        this.activeVault = null;
      }
    }

    localVault.closeVault();
  }

  /**
   * Close all vaults
   */
  closeAllVaults(): void {
    this.vaults.forEach(vault => vault.destroy());
    this.vaults.clear();
    this.activeVault = null;
    localVault.closeVault();
  }
}

// Export singleton instance
export const vaultManager = VaultManager.getInstance();