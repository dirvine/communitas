/**
 * LocalStorageProvider - Local-first storage operations
 *
 * Provides a local-first abstraction layer for all storage operations.
 * Operations work immediately using local cache/simulation, then sync
 * to the network when available.
 */

import { invoke } from '@tauri-apps/api/core';
import React, { createContext, ReactNode, useCallback, useContext, useEffect, useState } from 'react';
import { networkService } from '../services/network/NetworkConnectionService';
import { offlineStorage } from '../services/storage/OfflineStorageService';

// Storage types
export interface StorageFile {
  name: string;
  path: string;
  size: number;
  contentType: string;
  createdAt: string;
  modifiedAt: string;
  isDirectory: boolean;
  content?: string | ArrayBuffer;
  checksum?: string;
}

export interface StorageOperation {
  id: string;
  type: 'list' | 'read' | 'write' | 'delete' | 'mkdir';
  entityId: string;
  path: string;
  data?: any;
  timestamp: string;
  status: 'pending' | 'syncing' | 'completed' | 'failed';
  error?: string;
}

// Local storage context type
export interface LocalStorageContextType {
  // Storage operations
  list: (entityId: string, path: string) => Promise<StorageFile[]>;
  read: (entityId: string, path: string) => Promise<StorageFile>;
  write: (entityId: string, path: string, content: string | ArrayBuffer) => Promise<void>;
  delete: (entityId: string, path: string) => Promise<void>;
  mkdir: (entityId: string, path: string) => Promise<void>;

  // Sync management
  getSyncQueue: () => StorageOperation[];
  forceSyncNow: () => Promise<void>;
  clearSyncQueue: () => void;

  // Status
  isOnline: boolean;
  syncInProgress: boolean;
  lastSyncError: string | null;
}

const LocalStorageContext = createContext<LocalStorageContextType | undefined>(undefined);

interface LocalStorageProviderProps {
  children: ReactNode;
}

// In-memory virtual file system for local operations
class VirtualFileSystem {
  private files: Map<string, Map<string, StorageFile>> = new Map();

  constructor() {
    // Initialize with some default structure
    this.initializeDefaults();
  }

  private initializeDefaults(): void {
    // Create default directories for common entity types
    const _defaultDirs = [
      '/documents',
      '/images',
      '/videos',
      '/shared',
      '/private',
    ];

    // These would be created for each entity
    // For now, just structure setup
  }

  private getEntityStorage(entityId: string): Map<string, StorageFile> {
    if (!this.files.has(entityId)) {
      this.files.set(entityId, new Map());
      this.createDefaultStructure(entityId);
    }
    return this.files.get(entityId)!;
  }

  private createDefaultStructure(entityId: string): void {
    const storage = this.files.get(entityId)!;
    const now = new Date().toISOString();

    // Create default directories
    const dirs = [
      { path: '/', name: 'root' },
      { path: '/documents', name: 'documents' },
      { path: '/images', name: 'images' },
      { path: '/shared', name: 'shared' },
      { path: '/private', name: 'private' },
    ];

    dirs.forEach(dir => {
      storage.set(dir.path, {
        name: dir.name,
        path: dir.path,
        size: 0,
        contentType: 'directory',
        createdAt: now,
        modifiedAt: now,
        isDirectory: true,
      });
    });
  }

  list(entityId: string, path: string): StorageFile[] {
    const storage = this.getEntityStorage(entityId);
    const files: StorageFile[] = [];

    // Normalize path
    const normalizedPath = path.endsWith('/') ? path.slice(0, -1) : path;
    const searchPath = normalizedPath || '/';

    storage.forEach((file, filePath) => {
      // Get files in the specified directory
      if (filePath === searchPath) {
        // Skip the directory itself
        return;
      }

      // Check if file is direct child of the path
      if (filePath.startsWith(searchPath)) {
        const relativePath = filePath.slice(searchPath.length);
        const parts = relativePath.split('/').filter(p => p);

        // Only include direct children (one level deep)
        if (parts.length === 1 || (searchPath === '/' && parts.length === 1)) {
          files.push(file);
        }
      }
    });

    return files.sort((a, b) => {
      // Directories first, then alphabetical
      if (a.isDirectory && !b.isDirectory) return -1;
      if (!a.isDirectory && b.isDirectory) return 1;
      return a.name.localeCompare(b.name);
    });
  }

  read(entityId: string, path: string): StorageFile | null {
    const storage = this.getEntityStorage(entityId);
    return storage.get(path) || null;
  }

  write(entityId: string, path: string, content: string | ArrayBuffer): void {
    const storage = this.getEntityStorage(entityId);
    const now = new Date().toISOString();

    // Extract filename from path
    const parts = path.split('/');
    const name = parts[parts.length - 1];

    // Determine content type from extension
    const ext = name.split('.').pop()?.toLowerCase() || '';
    const contentType = this.getContentType(ext);

    // Calculate size
    const size = typeof content === 'string'
      ? new TextEncoder().encode(content).length
      : content.byteLength;

    storage.set(path, {
      name,
      path,
      size,
      contentType,
      createdAt: storage.get(path)?.createdAt || now,
      modifiedAt: now,
      isDirectory: false,
      content,
    });

    // Ensure parent directories exist
    this.ensureParentDirs(entityId, path);
  }

  delete(entityId: string, path: string): boolean {
    const storage = this.getEntityStorage(entityId);

    // If it's a directory, delete all children
    if (storage.get(path)?.isDirectory) {
      const toDelete: string[] = [];
      storage.forEach((_file, filePath) => {
        if (filePath.startsWith(path) && filePath !== path) {
          toDelete.push(filePath);
        }
      });
      toDelete.forEach(p => storage.delete(p));
    }

    return storage.delete(path);
  }

  mkdir(entityId: string, path: string): void {
    const storage = this.getEntityStorage(entityId);
    const now = new Date().toISOString();

    const parts = path.split('/');
    const name = parts[parts.length - 1];

    storage.set(path, {
      name,
      path,
      size: 0,
      contentType: 'directory',
      createdAt: now,
      modifiedAt: now,
      isDirectory: true,
    });

    this.ensureParentDirs(entityId, path);
  }

  private ensureParentDirs(entityId: string, path: string): void {
    const storage = this.getEntityStorage(entityId);
    const parts = path.split('/').filter(p => p);
    let currentPath = '';

    for (let i = 0; i < parts.length - 1; i++) {
      currentPath += '/' + parts[i];
      if (!storage.has(currentPath)) {
        const now = new Date().toISOString();
        storage.set(currentPath, {
          name: parts[i],
          path: currentPath,
          size: 0,
          contentType: 'directory',
          createdAt: now,
          modifiedAt: now,
          isDirectory: true,
        });
      }
    }
  }

  private getContentType(extension: string): string {
    const types: { [key: string]: string } = {
      // Text
      'txt': 'text/plain',
      'md': 'text/markdown',
      'html': 'text/html',
      'css': 'text/css',
      'js': 'application/javascript',
      'json': 'application/json',

      // Images
      'jpg': 'image/jpeg',
      'jpeg': 'image/jpeg',
      'png': 'image/png',
      'gif': 'image/gif',
      'svg': 'image/svg+xml',
      'webp': 'image/webp',

      // Documents
      'pdf': 'application/pdf',
      'doc': 'application/msword',
      'docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',

      // Archives
      'zip': 'application/zip',
      'tar': 'application/x-tar',
      'gz': 'application/gzip',
    };

    return types[extension] || 'application/octet-stream';
  }

  // Export/import for persistence
  export(): string {
    const data: { [entityId: string]: { [path: string]: StorageFile } } = {};

    this.files.forEach((storage, entityId) => {
      data[entityId] = {};
      storage.forEach((file, path) => {
        // Don't export file content to keep size manageable
        const { content, ...metadata } = file;
        data[entityId][path] = metadata as StorageFile;
      });
    });

    return JSON.stringify(data);
  }

  import(jsonData: string): void {
    try {
      const data = JSON.parse(jsonData);

      Object.entries(data).forEach(([entityId, files]) => {
        const storage = new Map<string, StorageFile>();
        Object.entries(files as any).forEach(([path, file]) => {
          storage.set(path, file as StorageFile);
        });
        this.files.set(entityId, storage);
      });
    } catch (error) {
      console.error('[VFS] Import failed:', error);
    }
  }
}

export const LocalStorageProvider: React.FC<LocalStorageProviderProps> = ({ children }) => {
  const [isConnected, setIsConnected] = useState(false);
  const [vfs] = useState(() => new VirtualFileSystem());
  const [syncQueue, setSyncQueue] = useState<StorageOperation[]>([]);
  const [syncInProgress, setSyncInProgress] = useState(false);
  const [lastSyncError, setLastSyncError] = useState<string | null>(null);

  // Subscribe to network status changes
  useEffect(() => {
    const unsubscribe = networkService.subscribe((state) => {
      setIsConnected(state.status === 'connected');
    });

    // Get initial state
    const currentState = networkService.getState();
    setIsConnected(currentState.status === 'connected');

    return unsubscribe;
  }, []);

  // Load VFS from IndexedDB on mount
  useEffect(() => {
    const loadVFS = async () => {
      try {
        const stored = await offlineStorage.get('vfs-data');
        if (stored) {
          vfs.import(stored);
        }
      } catch (error) {
        console.error('[LocalStorage] Failed to load VFS:', error);
      }
    };

    loadVFS();
  }, [vfs]);

  // Save VFS to IndexedDB on changes
  const persistVFS = useCallback(async () => {
    try {
      const data = vfs.export();
      await offlineStorage.store('vfs-data', data, {
        encrypt: true,
        syncOnline: false,
      });
    } catch (error) {
      console.error('[LocalStorage] Failed to persist VFS:', error);
    }
  }, [vfs]);

  // List files/directories
  const list = useCallback(async (entityId: string, path: string): Promise<StorageFile[]> => {
    // Always return local data immediately
    const localFiles = vfs.list(entityId, path);

    // If online, try to fetch from network and update local
    if (isConnected) {
      try {
        const result = await invoke('core_storage_list', {
          entityId,
          path,
        });

        // Update local VFS with network data
        if (result && Array.isArray(result)) {
          // Merge network results with local
          // This would update the VFS with fresh data
          console.log('[LocalStorage] Updated from network:', result);
        }
      } catch (error) {
        console.warn('[LocalStorage] Network list failed, using local:', error);
        // Continue with local data
      }
    }

    return localFiles;
  }, [vfs, isConnected]);

  // Read file
  const read = useCallback(async (entityId: string, path: string): Promise<StorageFile> => {
    // Check local first
    const localFile = vfs.read(entityId, path);

    if (localFile) {
      // If we have content locally, return it
      if (localFile.content) {
        return localFile;
      }

      // If online, try to fetch content
      if (isConnected) {
        try {
          const content = await invoke('core_storage_read', {
            entityId,
            path,
          });

          // Update local with content
          vfs.write(entityId, path, content as string);
          await persistVFS();

          return vfs.read(entityId, path)!;
        } catch (error) {
          console.warn('[LocalStorage] Network read failed:', error);
        }
      }
    }

    // If not found locally and can't fetch, throw error
    throw new Error(`File not found: ${path}`);
  }, [vfs, isConnected, persistVFS]);

  // Write file
  const write = useCallback(async (entityId: string, path: string, content: string | ArrayBuffer): Promise<void> => {
    // Write locally immediately
    vfs.write(entityId, path, content);
    await persistVFS();

    // Queue for network sync
    const operation: StorageOperation = {
      id: crypto.randomUUID(),
      type: 'write',
      entityId,
      path,
      data: content,
      timestamp: new Date().toISOString(),
      status: 'pending',
    };

    setSyncQueue(prev => [...prev, operation]);

    // If online, try to sync immediately
    if (isConnected) {
      await syncOperation(operation);
    }
  }, [vfs, isConnected, persistVFS]);

  // Delete file/directory
  const deleteFile = useCallback(async (entityId: string, path: string): Promise<void> => {
    // Delete locally immediately
    vfs.delete(entityId, path);
    await persistVFS();

    // Queue for network sync
    const operation: StorageOperation = {
      id: crypto.randomUUID(),
      type: 'delete',
      entityId,
      path,
      timestamp: new Date().toISOString(),
      status: 'pending',
    };

    setSyncQueue(prev => [...prev, operation]);

    // If online, try to sync immediately
    if (isConnected) {
      await syncOperation(operation);
    }
  }, [vfs, isConnected, persistVFS]);

  // Create directory
  const mkdir = useCallback(async (entityId: string, path: string): Promise<void> => {
    // Create locally immediately
    vfs.mkdir(entityId, path);
    await persistVFS();

    // Queue for network sync
    const operation: StorageOperation = {
      id: crypto.randomUUID(),
      type: 'mkdir',
      entityId,
      path,
      timestamp: new Date().toISOString(),
      status: 'pending',
    };

    setSyncQueue(prev => [...prev, operation]);

    // If online, try to sync immediately
    if (isConnected) {
      await syncOperation(operation);
    }
  }, [vfs, isConnected, persistVFS]);

  // Sync a single operation
  const syncOperation = async (operation: StorageOperation): Promise<void> => {
    try {
      setSyncQueue(prev =>
        prev.map(op => op.id === operation.id
          ? { ...op, status: 'syncing' }
          : op
        )
      );

      switch (operation.type) {
        case 'write':
          await invoke('core_storage_write', {
            entityId: operation.entityId,
            path: operation.path,
            content: operation.data,
          });
          break;

        case 'delete':
          await invoke('core_storage_delete', {
            entityId: operation.entityId,
            path: operation.path,
          });
          break;

        case 'mkdir':
          await invoke('core_storage_mkdir', {
            entityId: operation.entityId,
            path: operation.path,
          });
          break;
      }

      // Mark as completed and remove from queue
      setSyncQueue(prev => prev.filter(op => op.id !== operation.id));
      setLastSyncError(null);
    } catch (error) {
      console.error('[LocalStorage] Sync failed:', operation, error);

      setSyncQueue(prev =>
        prev.map(op => op.id === operation.id
          ? { ...op, status: 'failed', error: (error as Error).message }
          : op
        )
      );

      setLastSyncError((error as Error).message);
    }
  };

  // Force sync all pending operations
  const forceSyncNow = useCallback(async (): Promise<void> => {
    if (syncInProgress || !isConnected) return;

    setSyncInProgress(true);

    try {
      const pending = syncQueue.filter(op => op.status === 'pending' || op.status === 'failed');

      for (const operation of pending) {
        await syncOperation(operation);
      }
    } finally {
      setSyncInProgress(false);
    }
  }, [syncQueue, syncInProgress, isConnected]);

  // Auto-sync when coming online
  useEffect(() => {
    if (isConnected && syncQueue.length > 0) {
      forceSyncNow();
    }
  }, [isConnected, syncQueue.length, forceSyncNow]);

  // Clear sync queue
  const clearSyncQueue = useCallback((): void => {
    setSyncQueue([]);
    setLastSyncError(null);
  }, []);

  const value: LocalStorageContextType = {
    list,
    read,
    write,
    delete: deleteFile,
    mkdir,
    getSyncQueue: () => syncQueue,
    forceSyncNow,
    clearSyncQueue,
    isOnline: isConnected,
    syncInProgress,
    lastSyncError,
  };

  return (
    <LocalStorageContext.Provider value={value}>
      {children}
    </LocalStorageContext.Provider>
  );
};

export const useLocalStorage = (): LocalStorageContextType => {
  const context = useContext(LocalStorageContext);
  if (!context) {
    throw new Error('useLocalStorage must be used within LocalStorageProvider');
  }
  return context;
};