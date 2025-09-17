/**
 * Offline Storage Service
 * 
 * Ensures complete offline functionality by managing:
 * - Local caching of all network data
 * - Persistent storage of user content
 * - Sync queue for when network returns
 * - Automatic fallback to cached data
 */

import { invoke } from '@tauri-apps/api/core';

interface CacheEntry {
  key: string;
  data: any;
  timestamp: number;
  expiresAt?: number;
  source: 'network' | 'local';
}

interface SyncQueueItem {
  id: string;
  operation: 'create' | 'update' | 'delete';
  resource: string;
  data: any;
  timestamp: number;
  retryCount: number;
}

export class OfflineStorageService {
  private static instance: OfflineStorageService;
  private cache: Map<string, CacheEntry> = new Map();
  private syncQueue: SyncQueueItem[] = [];
  private isOnline: boolean = navigator.onLine;
  private db: IDBDatabase | null = null;
  private readonly DB_NAME = 'communitas-offline';
  private readonly DB_VERSION = 1;

  private constructor() {
    this.initializeDB();
    this.setupNetworkListeners();
    this.loadCacheFromDB();
  }

  static getInstance(): OfflineStorageService {
    if (!OfflineStorageService.instance) {
      OfflineStorageService.instance = new OfflineStorageService();
    }
    return OfflineStorageService.instance;
  }

  /**
   * Initialize IndexedDB for persistent storage
   */
  private async initializeDB(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.DB_NAME, this.DB_VERSION);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Store for cached data
        if (!db.objectStoreNames.contains('cache')) {
          const cacheStore = db.createObjectStore('cache', { keyPath: 'key' });
          cacheStore.createIndex('timestamp', 'timestamp', { unique: false });
          cacheStore.createIndex('source', 'source', { unique: false });
        }

        // Store for sync queue
        if (!db.objectStoreNames.contains('syncQueue')) {
          const syncStore = db.createObjectStore('syncQueue', { keyPath: 'id' });
          syncStore.createIndex('timestamp', 'timestamp', { unique: false });
        }

        // Store for user content
        if (!db.objectStoreNames.contains('userContent')) {
          const contentStore = db.createObjectStore('userContent', { keyPath: 'id' });
          contentStore.createIndex('type', 'type', { unique: false });
          contentStore.createIndex('createdAt', 'createdAt', { unique: false });
        }

        // Store for downloaded files
        if (!db.objectStoreNames.contains('files')) {
          const filesStore = db.createObjectStore('files', { keyPath: 'path' });
          filesStore.createIndex('size', 'size', { unique: false });
        }
      };
    });
  }

  /**
   * Setup network status listeners
   */
  private setupNetworkListeners(): void {
    window.addEventListener('online', () => {
      console.log('🌐 Network connection restored');
      this.isOnline = true;
      this.processSyncQueue();
    });

    window.addEventListener('offline', () => {
      console.log('📵 Network connection lost - switching to offline mode');
      this.isOnline = false;
    });
  }

  /**
   * Load cached data from IndexedDB on startup
   */
  private async loadCacheFromDB(): Promise<void> {
    if (!this.db) return;

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(['cache'], 'readonly');
      const store = transaction.objectStore('cache');
      const request = store.getAll();

      request.onsuccess = () => {
        const entries = request.result as CacheEntry[];
        entries.forEach(entry => {
          // Only load non-expired entries
          if (!entry.expiresAt || entry.expiresAt > Date.now()) {
            this.cache.set(entry.key, entry);
          }
        });
        console.log(`📦 Loaded ${this.cache.size} cached entries from storage`);
        resolve();
      };

      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Store data with automatic caching
   */
  async store(key: string, data: any, options?: {
    ttl?: number; // Time to live in milliseconds
    encrypt?: boolean;
    syncOnline?: boolean;
  }): Promise<void> {
    const entry: CacheEntry = {
      key,
      data,
      timestamp: Date.now(),
      expiresAt: options?.ttl ? Date.now() + options.ttl : undefined,
      source: this.isOnline ? 'network' : 'local'
    };

    // Store in memory cache
    this.cache.set(key, entry);

    // Store in IndexedDB
    await this.persistToDB('cache', entry);

    // If encrypted storage requested, also store in Tauri backend
    if (options?.encrypt) {
      try {
        await invoke('core_private_put', {
          key,
          content: new TextEncoder().encode(JSON.stringify(data))
        });
      } catch (error) {
        console.warn('Failed to store encrypted, using local only:', error);
      }
    }

    // Queue for sync if offline and sync requested
    if (!this.isOnline && options?.syncOnline) {
      this.addToSyncQueue('create', key, data);
    }
  }

  /**
   * Retrieve data with automatic fallback
   */
  async get<T = any>(key: string): Promise<T | null> {
    // Check memory cache first
    if (this.cache.has(key)) {
      const entry = this.cache.get(key)!;
      if (!entry.expiresAt || entry.expiresAt > Date.now()) {
        return entry.data as T;
      }
      // Remove expired entry
      this.cache.delete(key);
    }

    // Try to get from encrypted storage if online
    if (this.isOnline) {
      try {
        const encrypted = await invoke<number[]>('core_private_get', { key });
        const decoded = new TextDecoder().decode(new Uint8Array(encrypted));

        // Check if decoded string is valid before parsing
        if (!decoded || decoded.trim() === '') {
          console.debug('Empty response from network storage');
          return undefined;
        }

        const data = JSON.parse(decoded);

        // Update cache
        await this.store(key, data);
        return data as T;
      } catch (error) {
        console.warn('Failed to retrieve from network, using cache:', error);
      }
    }

    // Fallback to IndexedDB
    return this.getFromDB<T>('cache', key);
  }

  /**
   * Store user-generated content
   */
  async storeContent(type: string, content: any): Promise<string> {
    const id = `content_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const contentEntry = {
      id,
      type,
      content,
      createdAt: Date.now(),
      modifiedAt: Date.now(),
      synced: false
    };

    await this.persistToDB('userContent', contentEntry);

    // Queue for sync if offline
    if (!this.isOnline) {
      this.addToSyncQueue('create', `content/${type}`, contentEntry);
    }

    return id;
  }

  /**
   * Store downloaded files for offline access
   */
  async storeFile(path: string, content: ArrayBuffer, metadata?: any): Promise<void> {
    const fileEntry = {
      path,
      content,
      size: content.byteLength,
      metadata,
      downloadedAt: Date.now()
    };

    await this.persistToDB('files', fileEntry);
    console.log(`💾 Stored file for offline: ${path} (${this.formatBytes(content.byteLength)})`);
  }

  /**
   * Get all content of a specific type
   */
  async getContentByType(type: string): Promise<any[]> {
    if (!this.db) return [];

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(['userContent'], 'readonly');
      const store = transaction.objectStore('userContent');
      const index = store.index('type');
      const request = index.getAll(type);

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Add operation to sync queue
   */
  private addToSyncQueue(operation: SyncQueueItem['operation'], resource: string, data: any): void {
    const item: SyncQueueItem = {
      id: `sync_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      operation,
      resource,
      data,
      timestamp: Date.now(),
      retryCount: 0
    };

    this.syncQueue.push(item);
    this.persistToDB('syncQueue', item);
    console.log(`📝 Added to sync queue: ${operation} ${resource}`);
  }

  /**
   * Process sync queue when online
   */
  private async processSyncQueue(): Promise<void> {
    if (!this.isOnline || this.syncQueue.length === 0) return;

    console.log(`🔄 Processing ${this.syncQueue.length} queued operations`);

    const processed: string[] = [];

    for (const item of this.syncQueue) {
      try {
        // Process based on operation type
        switch (item.operation) {
          case 'create':
          case 'update':
            await invoke('core_private_put', {
              key: item.resource,
              content: new TextEncoder().encode(JSON.stringify(item.data))
            });
            break;
          case 'delete':
            // Implement delete if needed
            break;
        }

        processed.push(item.id);
        console.log(`✅ Synced: ${item.operation} ${item.resource}`);
      } catch (error) {
        console.error(`Failed to sync ${item.resource}:`, error);
        item.retryCount++;
        
        // Remove from queue if too many retries
        if (item.retryCount > 3) {
          processed.push(item.id);
          console.warn(`⚠️ Giving up on sync after 3 retries: ${item.resource}`);
        }
      }
    }

    // Remove processed items
    this.syncQueue = this.syncQueue.filter(item => !processed.includes(item.id));
    
    // Clean up IndexedDB
    for (const id of processed) {
      await this.deleteFromDB('syncQueue', id);
    }
  }

  /**
   * Persist data to IndexedDB
   */
  private async persistToDB(storeName: string, data: any): Promise<void> {
    if (!this.db) return;

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readwrite');
      const store = transaction.objectStore(storeName);
      const request = store.put(data);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Get data from IndexedDB
   */
  private async getFromDB<T>(storeName: string, key: string): Promise<T | null> {
    if (!this.db) return null;

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readonly');
      const store = transaction.objectStore(storeName);
      const request = store.get(key);

      request.onsuccess = () => {
        const result = request.result;
        resolve(result ? result.data || result : null);
      };
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Delete from IndexedDB
   */
  private async deleteFromDB(storeName: string, key: string): Promise<void> {
    if (!this.db) return;

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readwrite');
      const store = transaction.objectStore(storeName);
      const request = store.delete(key);

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Clear all cached data (use with caution)
   */
  async clearCache(): Promise<void> {
    this.cache.clear();
    
    if (!this.db) return;

    const stores = ['cache', 'syncQueue'];
    for (const storeName of stores) {
      await new Promise<void>((resolve, reject) => {
        const transaction = this.db!.transaction([storeName], 'readwrite');
        const store = transaction.objectStore(storeName);
        const request = store.clear();

        request.onsuccess = () => resolve();
        request.onerror = () => reject(request.error);
      });
    }

    console.log('🧹 Cache cleared');
  }

  /**
   * Get storage statistics
   */
  async getStats(): Promise<{
    cacheSize: number;
    syncQueueLength: number;
    isOnline: boolean;
    contentCount: number;
    fileCount: number;
  }> {
    let contentCount = 0;
    let fileCount = 0;

    if (this.db) {
      contentCount = await this.countInStore('userContent');
      fileCount = await this.countInStore('files');
    }

    return {
      cacheSize: this.cache.size,
      syncQueueLength: this.syncQueue.length,
      isOnline: this.isOnline,
      contentCount,
      fileCount
    };
  }

  /**
   * Count items in a store
   */
  private async countInStore(storeName: string): Promise<number> {
    if (!this.db) return 0;

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readonly');
      const store = transaction.objectStore(storeName);
      const request = store.count();

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Format bytes to human readable
   */
  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
  }

  /**
   * Export all offline data (for backup)
   */
  async exportData(): Promise<Blob> {
    const data = {
      cache: Array.from(this.cache.entries()),
      syncQueue: this.syncQueue,
      userContent: await this.getAllFromStore('userContent'),
      timestamp: Date.now()
    };

    return new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
  }

  /**
   * Get all data from a store
   */
  private async getAllFromStore(storeName: string): Promise<any[]> {
    if (!this.db) return [];

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readonly');
      const store = transaction.objectStore(storeName);
      const request = store.getAll();

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }
}

// Export singleton instance
export const offlineStorage = OfflineStorageService.getInstance();