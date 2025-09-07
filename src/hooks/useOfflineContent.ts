/**
 * Hook for managing offline content
 * 
 * Provides:
 * - Automatic caching of fetched content
 * - Offline-first data retrieval
 * - Background sync when online
 * - Progress tracking for downloads
 */

import { useState, useEffect, useCallback } from 'react';
import { offlineStorage } from '../services/storage/OfflineStorageService';

interface OfflineContentOptions {
  cacheKey: string;
  ttl?: number; // Time to live in milliseconds
  autoSync?: boolean;
  onSync?: () => void;
}

interface OfflineContentState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  isOffline: boolean;
  lastSynced: Date | null;
}

export function useOfflineContent<T = any>(
  fetchFn: () => Promise<T>,
  options: OfflineContentOptions
) {
  const [state, setState] = useState<OfflineContentState<T>>({
    data: null,
    loading: true,
    error: null,
    isOffline: !navigator.onLine,
    lastSynced: null,
  });

  // Load cached data immediately
  useEffect(() => {
    loadCachedData();
  }, [options.cacheKey]);

  // Setup network listeners
  useEffect(() => {
    const handleOnline = () => {
      setState(prev => ({ ...prev, isOffline: false }));
      if (options.autoSync) {
        refresh();
      }
    };

    const handleOffline = () => {
      setState(prev => ({ ...prev, isOffline: true }));
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, [options.autoSync]);

  const loadCachedData = async () => {
    try {
      setState(prev => ({ ...prev, loading: true }));
      
      // Try to get from cache first
      const cached = await offlineStorage.get<{
        data: T;
        timestamp: number;
      }>(options.cacheKey);

      if (cached) {
        setState(prev => ({
          ...prev,
          data: cached.data,
          lastSynced: new Date(cached.timestamp),
          loading: false,
        }));

        // If online and autoSync, fetch fresh data in background
        if (navigator.onLine && options.autoSync) {
          fetchAndCache();
        }
      } else if (navigator.onLine) {
        // No cache and online, fetch fresh
        await fetchAndCache();
      } else {
        // No cache and offline
        setState(prev => ({
          ...prev,
          loading: false,
          error: 'No offline data available',
        }));
      }
    } catch (error) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : 'Failed to load data',
      }));
    }
  };

  const fetchAndCache = async () => {
    try {
      const freshData = await fetchFn();
      
      // Store in cache
      await offlineStorage.store(
        options.cacheKey,
        {
          data: freshData,
          timestamp: Date.now(),
        },
        {
          ttl: options.ttl,
          syncOnline: true,
        }
      );

      setState(prev => ({
        ...prev,
        data: freshData,
        lastSynced: new Date(),
        loading: false,
        error: null,
      }));

      options.onSync?.();
    } catch (error) {
      // If fetch fails but we have cached data, keep using it
      if (state.data) {
        console.warn('Failed to fetch fresh data, using cache:', error);
      } else {
        setState(prev => ({
          ...prev,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to fetch data',
        }));
      }
    }
  };

  const refresh = useCallback(() => {
    if (navigator.onLine) {
      fetchAndCache();
    } else {
      loadCachedData();
    }
  }, [options.cacheKey]);

  const clearCache = useCallback(async () => {
    await offlineStorage.store(options.cacheKey, null);
    setState(prev => ({
      ...prev,
      data: null,
      lastSynced: null,
    }));
  }, [options.cacheKey]);

  return {
    ...state,
    refresh,
    clearCache,
  };
}

/**
 * Hook for downloading and caching files for offline access
 */
export function useOfflineFile(url: string, filename: string) {
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [cached, setCached] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkCached();
  }, [filename]);

  const checkCached = async () => {
    try {
      const file = await offlineStorage.get(`file:${filename}`);
      setCached(!!file);
    } catch {
      setCached(false);
    }
  };

  const download = async () => {
    if (cached || downloading) return;

    setDownloading(true);
    setProgress(0);
    setError(null);

    try {
      const response = await fetch(url);
      
      if (!response.ok) {
        throw new Error(`Failed to download: ${response.statusText}`);
      }

      const contentLength = response.headers.get('content-length');
      const total = contentLength ? parseInt(contentLength, 10) : 0;
      
      const reader = response.body?.getReader();
      if (!reader) throw new Error('No response body');

      const chunks: Uint8Array[] = [];
      let receivedLength = 0;

      while (true) {
        const { done, value } = await reader.read();

        if (done) break;

        chunks.push(value);
        receivedLength += value.length;

        if (total > 0) {
          setProgress((receivedLength / total) * 100);
        }
      }

      // Combine chunks
      const content = new Uint8Array(receivedLength);
      let position = 0;
      for (const chunk of chunks) {
        content.set(chunk, position);
        position += chunk.length;
      }

      // Store file
      await offlineStorage.storeFile(filename, content.buffer, {
        url,
        downloadedAt: Date.now(),
        size: receivedLength,
      });

      setCached(true);
      setProgress(100);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Download failed');
    } finally {
      setDownloading(false);
    }
  };

  const getFile = async (): Promise<ArrayBuffer | null> => {
    try {
      const file = await offlineStorage.get<ArrayBuffer>(`file:${filename}`);
      return file;
    } catch {
      return null;
    }
  };

  const removeFile = async () => {
    await offlineStorage.store(`file:${filename}`, null);
    setCached(false);
  };

  return {
    download,
    getFile,
    removeFile,
    downloading,
    progress,
    cached,
    error,
  };
}

/**
 * Hook for managing a collection of offline content
 */
export function useOfflineCollection<T = any>(
  collectionName: string,
  fetchFn?: () => Promise<T[]>
) {
  const [items, setItems] = useState<T[]>([]);
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState(false);

  useEffect(() => {
    loadCollection();
  }, [collectionName]);

  const loadCollection = async () => {
    setLoading(true);
    try {
      const cached = await offlineStorage.getContentByType(collectionName);
      setItems(cached.map(item => item.content));
    } catch (error) {
      console.error('Failed to load collection:', error);
    } finally {
      setLoading(false);
    }
  };

  const addItem = async (item: T) => {
    const id = await offlineStorage.storeContent(collectionName, item);
    setItems(prev => [...prev, { ...item, _id: id } as T]);
    return id;
  };

  const removeItem = async (id: string) => {
    // Implementation would need delete support in OfflineStorageService
    setItems(prev => prev.filter((item: any) => item._id !== id));
  };

  const sync = async () => {
    if (!fetchFn || !navigator.onLine) return;

    setSyncing(true);
    try {
      const freshData = await fetchFn();
      
      // Clear old data
      for (const item of items) {
        if ((item as any)._id) {
          // Would need delete implementation
        }
      }

      // Store new data
      const newItems = [];
      for (const item of freshData) {
        const id = await offlineStorage.storeContent(collectionName, item);
        newItems.push({ ...item, _id: id } as T);
      }

      setItems(newItems);
    } catch (error) {
      console.error('Sync failed:', error);
    } finally {
      setSyncing(false);
    }
  };

  return {
    items,
    loading,
    syncing,
    addItem,
    removeItem,
    sync,
    refresh: loadCollection,
  };
}