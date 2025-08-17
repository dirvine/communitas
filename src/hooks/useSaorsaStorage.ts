/**
 * React Hook for Saorsa Storage System
 * Provides easy access to storage functionality from React components
 */

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  StoragePolicy,
  StorageInitRequest,
  FrontendStorageRequest,
  StorageResponse,
  FrontendRetrievalRequest,
  RetrievalResponse,
  ContentListRequest,
  StorageAddress,
  StorageEngineStats,
  StorageErrorResponse,
  validateContent,
  inferContentType,
  stringToBytes,
  bytesToString,
} from '../types/saorsa-storage';

interface StorageState {
  isInitialized: boolean;
  isLoading: boolean;
  error: string | null;
  stats: StorageEngineStats | null;
}

interface UseStorageResult {
  // State
  state: StorageState;
  
  // Actions
  initializeStorage: (masterKeyHex: string, configPath?: string) => Promise<boolean>;
  generateMasterKey: () => Promise<string>;
  storeContent: (request: Omit<FrontendStorageRequest, 'content'> & { content: string | Uint8Array }) => Promise<StorageResponse>;
  retrieveContent: (request: FrontendRetrievalRequest) => Promise<{ content: string; metadata: any; source: any; operation_time_ms: number }>;
  listContent: (request: ContentListRequest) => Promise<StorageAddress[]>;
  deleteContent: (address: StorageAddress, userId: string) => Promise<boolean>;
  getStats: () => Promise<StorageEngineStats>;
  transitionPolicy: (address: StorageAddress, newPolicy: StoragePolicy, userId: string) => Promise<StorageAddress>;
  performMaintenance: () => Promise<boolean>;
  validatePolicy: (policy: StoragePolicy, contentSize: number, userId: string, contentType: string) => Promise<boolean>;
  
  // Utility functions
  checkInitialized: () => Promise<boolean>;
  clearError: () => void;
  refreshStats: () => Promise<void>;
}

export function useSaorsaStorage(): UseStorageResult {
  const [state, setState] = useState<StorageState>({
    isInitialized: false,
    isLoading: false,
    error: null,
    stats: null,
  });

  const setLoading = useCallback((loading: boolean) => {
    setState(prev => ({ ...prev, isLoading: loading }));
  }, []);

  const setError = useCallback((error: string | null) => {
    setState(prev => ({ ...prev, error }));
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, [setError]);

  const handleError = useCallback((error: any) => {
    console.error('Storage operation failed:', error);
    
    if (typeof error === 'object' && error.message) {
      setError(error.message);
    } else if (typeof error === 'string') {
      setError(error);
    } else {
      setError('An unknown error occurred');
    }
  }, [setError]);

  const checkInitialized = useCallback(async (): Promise<boolean> => {
    try {
      const initialized = await invoke<boolean>('is_storage_initialized');
      setState(prev => ({ ...prev, isInitialized: initialized }));
      return initialized;
    } catch (error) {
      handleError(error);
      return false;
    }
  }, [handleError]);

  const generateMasterKey = useCallback(async (): Promise<string> => {
    try {
      setLoading(true);
      clearError();
      return await invoke<string>('generate_master_key');
    } catch (error) {
      handleError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const initializeStorage = useCallback(async (masterKeyHex: string, configPath?: string): Promise<boolean> => {
    try {
      setLoading(true);
      clearError();

      const request: StorageInitRequest = {
        master_key_hex: masterKeyHex,
        config_path: configPath || null,
      };

      const result = await invoke<boolean>('init_storage_engine', { request });
      
      if (result) {
        setState(prev => ({ ...prev, isInitialized: true }));
      }

      return result;
    } catch (error) {
      handleError(error);
      return false;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const storeContent = useCallback(async (
    request: Omit<FrontendStorageRequest, 'content'> & { content: string | Uint8Array }
  ): Promise<StorageResponse> => {
    try {
      setLoading(true);
      clearError();

      // Convert content to number array
      let contentBytes: number[];
      if (typeof request.content === 'string') {
        contentBytes = stringToBytes(request.content);
      } else {
        contentBytes = Array.from(request.content);
      }

      // Validate content
      const validation = validateContent(contentBytes, request.content_type, request.policy);
      if (!validation.valid) {
        throw new Error(validation.error);
      }

      const storageRequest: FrontendStorageRequest = {
        ...request,
        content: contentBytes,
      };

      return await invoke<StorageResponse>('store_content', { request: storageRequest });
    } catch (error) {
      handleError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const retrieveContent = useCallback(async (request: FrontendRetrievalRequest) => {
    try {
      setLoading(true);
      clearError();

      const response = await invoke<RetrievalResponse>('retrieve_content', { request });
      
      // Convert content back to string for easier handling
      const contentString = bytesToString(response.content);
      
      return {
        content: contentString,
        metadata: response.metadata,
        source: response.source,
        operation_time_ms: response.operation_time_ms,
      };
    } catch (error) {
      handleError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const listContent = useCallback(async (request: ContentListRequest): Promise<StorageAddress[]> => {
    try {
      setLoading(true);
      clearError();
      return await invoke<StorageAddress[]>('list_content', { request });
    } catch (error) {
      handleError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const deleteContent = useCallback(async (address: StorageAddress, userId: string): Promise<boolean> => {
    try {
      setLoading(true);
      clearError();
      return await invoke<boolean>('delete_content', { address, user_id: userId });
    } catch (error) {
      handleError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const getStats = useCallback(async (): Promise<StorageEngineStats> => {
    try {
      setLoading(true);
      clearError();
      const stats = await invoke<StorageEngineStats>('get_storage_stats');
      setState(prev => ({ ...prev, stats }));
      return stats;
    } catch (error) {
      handleError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const refreshStats = useCallback(async (): Promise<void> => {
    try {
      await getStats();
    } catch (error) {
      // Error already handled by getStats
    }
  }, [getStats]);

  const transitionPolicy = useCallback(async (
    address: StorageAddress, 
    newPolicy: StoragePolicy, 
    userId: string
  ): Promise<StorageAddress> => {
    try {
      setLoading(true);
      clearError();
      return await invoke<StorageAddress>('transition_content_policy', { 
        address, 
        new_policy: newPolicy, 
        user_id: userId 
      });
    } catch (error) {
      handleError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const performMaintenance = useCallback(async (): Promise<boolean> => {
    try {
      setLoading(true);
      clearError();
      return await invoke<boolean>('perform_storage_maintenance');
    } catch (error) {
      handleError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [setLoading, clearError, handleError]);

  const validatePolicy = useCallback(async (
    policy: StoragePolicy, 
    contentSize: number, 
    userId: string, 
    contentType: string
  ): Promise<boolean> => {
    try {
      return await invoke<boolean>('validate_storage_policy', { 
        policy, 
        content_size: contentSize, 
        user_id: userId, 
        content_type: contentType 
      });
    } catch (error) {
      handleError(error);
      return false;
    }
  }, [handleError]);

  // Check initialization status on mount
  useEffect(() => {
    checkInitialized();
  }, [checkInitialized]);

  return {
    state,
    initializeStorage,
    generateMasterKey,
    storeContent,
    retrieveContent,
    listContent,
    deleteContent,
    getStats,
    transitionPolicy,
    performMaintenance,
    validatePolicy,
    checkInitialized,
    clearError,
    refreshStats,
  };
}

// Higher-order hook for specific storage patterns

export function useMarkdownStorage() {
  const storage = useSaorsaStorage();

  const storeMarkdown = useCallback(async (
    content: string,
    policy: StoragePolicy,
    author: string,
    userId: string,
    tags: string[] = [],
    namespace?: string,
    groupId?: string
  ) => {
    return storage.storeContent({
      content,
      content_type: 'text/markdown',
      policy,
      author,
      tags,
      user_id: userId,
      namespace: namespace || null,
      group_id: groupId || null,
    });
  }, [storage]);

  const retrieveMarkdown = useCallback(async (
    address: StorageAddress,
    userId: string
  ) => {
    return storage.retrieveContent({
      address,
      user_id: userId,
      decryption_key_hex: null,
    });
  }, [storage]);

  return {
    ...storage,
    storeMarkdown,
    retrieveMarkdown,
  };
}

export function useFileStorage() {
  const storage = useSaorsaStorage();

  const storeFile = useCallback(async (
    file: File,
    policy: StoragePolicy,
    author: string,
    userId: string,
    tags: string[] = [],
    namespace?: string,
    groupId?: string
  ) => {
    const arrayBuffer = await file.arrayBuffer();
    const content = new Uint8Array(arrayBuffer);
    const contentType = file.type || inferContentType(file.name);

    return storage.storeContent({
      content,
      content_type: contentType,
      policy,
      author,
      tags: [...tags, `filename:${file.name}`],
      user_id: userId,
      namespace: namespace || null,
      group_id: groupId || null,
    });
  }, [storage]);

  const retrieveFile = useCallback(async (
    address: StorageAddress,
    userId: string
  ): Promise<{ blob: Blob; metadata: any; source: any; operation_time_ms: number }> => {
    const result = await storage.retrieveContent({
      address,
      user_id: userId,
      decryption_key_hex: null,
    });

    // Convert string back to blob
    const uint8Array = new TextEncoder().encode(result.content);
    const blob = new Blob([uint8Array], { type: result.metadata.content_type });

    return {
      blob,
      metadata: result.metadata,
      source: result.source,
      operation_time_ms: result.operation_time_ms,
    };
  }, [storage]);

  return {
    ...storage,
    storeFile,
    retrieveFile,
  };
}