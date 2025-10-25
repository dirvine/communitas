/**
 * useDocumentCRDT - React hook for real-time CRDT document synchronization
 *
 * This hook provides:
 * - Real-time document content synchronization via Yrs CRDT
 * - Optimistic updates for instant UI feedback
 * - Automatic peer synchronization
 * - Collaborative cursor tracking
 * - Offline-first with sync queue
 *
 * Usage:
 * ```typescript
 * const { document, insertText, deleteText, isSyncing } = useDocumentCRDT(docId);
 * ```
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { documentService } from '../services/DocumentService';
import type { DocumentWithState } from '../types/documents';

interface CollaboratorCursor {
  userId: string;
  userName: string;
  position: number;
  color: string;
}

interface UseDocumentCRDTOptions {
  /** Enable automatic polling for peer updates (default: true) */
  autoSync?: boolean;
  /** Sync interval in milliseconds (default: 2000) */
  syncInterval?: number;
  /** Debounce local edits before syncing (default: 300ms) */
  editDebounce?: number;
}

interface UseDocumentCRDTResult {
  /** Current document with content */
  document: DocumentWithState | null;
  /** Is document loading? */
  isLoading: boolean;
  /** Is currently syncing with peers? */
  isSyncing: boolean;
  /** Has unsaved local changes? */
  isDirty: boolean;
  /** Error message if any */
  error: string | null;
  /** Active collaborators */
  collaborators: CollaboratorCursor[];
  /** Insert text at position */
  insertText: (position: number, text: string) => Promise<void>;
  /** Delete text range */
  deleteText: (position: number, length: number) => Promise<void>;
  /** Get full document text */
  getText: () => Promise<string>;
  /** Manually trigger sync with peers */
  syncWithPeers: () => Promise<void>;
  /** Refresh document from backend */
  refresh: () => Promise<void>;
}

/**
 * Hook for real-time CRDT document collaboration
 */
export function useDocumentCRDT(
  docId: string | null,
  options: UseDocumentCRDTOptions = {}
): UseDocumentCRDTResult {
  const {
    autoSync = true,
    syncInterval = 2000,
    editDebounce = 300,
  } = options;

  // State
  const [document, setDocument] = useState<DocumentWithState | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSyncing, setIsSyncing] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [collaborators, setCollaborators] = useState<CollaboratorCursor[]>([]);

  // Refs for managing timers and state
  const syncIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const editDebounceRef = useRef<NodeJS.Timeout | null>(null);
  const localUpdateQueueRef = useRef<Array<() => Promise<void>>>([]);
  const isMountedRef = useRef(true);

  // Load document initially
  const loadDocument = useCallback(async () => {
    if (!docId) {
      setDocument(null);
      setIsLoading(false);
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      const doc = await documentService.getDocumentWithContent(docId);
      if (isMountedRef.current) {
        setDocument(doc);
        setIsDirty(false);
      }
    } catch (err) {
      console.error('Failed to load document:', err);
      if (isMountedRef.current) {
        setError('Failed to load document');
      }
    } finally {
      if (isMountedRef.current) {
        setIsLoading(false);
      }
    }
  }, [docId]);

  // Initial load
  useEffect(() => {
    loadDocument();
  }, [loadDocument]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      isMountedRef.current = false;
      if (syncIntervalRef.current) {
        clearInterval(syncIntervalRef.current);
      }
      if (editDebounceRef.current) {
        clearTimeout(editDebounceRef.current);
      }
    };
  }, []);

  // Sync with peers (get updates from backend)
  const syncWithPeers = useCallback(async () => {
    if (!docId || isSyncing) return;

    try {
      setIsSyncing(true);

      // Get latest CRDT update from backend
      // This will include all changes from peers
      const update = await documentService.getCRDTUpdate(docId);

      // If we have updates, apply them
      if (update && update.length > 0) {
        // Apply the update (this will merge with our local state)
        await documentService.applyCRDTUpdate(docId, update);

        // Reload document to get updated content
        const updatedDoc = await documentService.getDocumentWithContent(docId);
        if (isMountedRef.current) {
          setDocument(updatedDoc);
        }
      }
    } catch (err) {
      console.error('Failed to sync with peers:', err);
      // Don't set error for sync failures - they're non-critical
    } finally {
      if (isMountedRef.current) {
        setIsSyncing(false);
      }
    }
  }, [docId, isSyncing]);

  // Auto-sync interval
  useEffect(() => {
    if (!docId || !autoSync) return;

    // Start sync interval
    syncIntervalRef.current = setInterval(() => {
      syncWithPeers();
    }, syncInterval);

    return () => {
      if (syncIntervalRef.current) {
        clearInterval(syncIntervalRef.current);
      }
    };
  }, [docId, autoSync, syncInterval, syncWithPeers]);

  // Insert text with debouncing and optimistic update
  const insertText = useCallback(
    async (position: number, text: string) => {
      if (!docId || !document) return;

      // Optimistic update - update UI immediately
      setDocument((prev) => {
        if (!prev) return prev;
        const currentContent = prev.content || '';
        const newContent =
          currentContent.slice(0, position) +
          text +
          currentContent.slice(position);
        return {
          ...prev,
          content: newContent,
          localState: {
            ...prev.localState,
            isDirty: true,
            isEditing: true,
          },
        };
      });
      setIsDirty(true);

      // Queue the actual backend update
      const updateFn = async () => {
        try {
          await documentService.insertText(docId, position, text);
          if (isMountedRef.current) {
            setIsDirty(false);
          }
        } catch (err) {
          console.error('Failed to insert text:', err);
          if (isMountedRef.current) {
            setError('Failed to save changes');
          }
        }
      };

      // Add to queue
      localUpdateQueueRef.current.push(updateFn);

      // Debounce the actual sync
      if (editDebounceRef.current) {
        clearTimeout(editDebounceRef.current);
      }

      editDebounceRef.current = setTimeout(async () => {
        // Process all queued updates
        const queue = [...localUpdateQueueRef.current];
        localUpdateQueueRef.current = [];

        for (const update of queue) {
          await update();
        }

        // Sync with peers after local changes
        await syncWithPeers();
      }, editDebounce);
    },
    [docId, document, editDebounce, syncWithPeers]
  );

  // Delete text with debouncing and optimistic update
  const deleteText = useCallback(
    async (position: number, length: number) => {
      if (!docId || !document) return;

      // Optimistic update
      setDocument((prev) => {
        if (!prev) return prev;
        const currentContent = prev.content || '';
        const newContent =
          currentContent.slice(0, position) +
          currentContent.slice(position + length);
        return {
          ...prev,
          content: newContent,
          localState: {
            ...prev.localState,
            isDirty: true,
            isEditing: true,
          },
        };
      });
      setIsDirty(true);

      // Queue the actual backend update
      const updateFn = async () => {
        try {
          await documentService.deleteText(docId, position, length);
          if (isMountedRef.current) {
            setIsDirty(false);
          }
        } catch (err) {
          console.error('Failed to delete text:', err);
          if (isMountedRef.current) {
            setError('Failed to save changes');
          }
        }
      };

      // Add to queue
      localUpdateQueueRef.current.push(updateFn);

      // Debounce the actual sync
      if (editDebounceRef.current) {
        clearTimeout(editDebounceRef.current);
      }

      editDebounceRef.current = setTimeout(async () => {
        // Process all queued updates
        const queue = [...localUpdateQueueRef.current];
        localUpdateQueueRef.current = [];

        for (const update of queue) {
          await update();
        }

        // Sync with peers after local changes
        await syncWithPeers();
      }, editDebounce);
    },
    [docId, document, editDebounce, syncWithPeers]
  );

  // Get latest text content
  const getText = useCallback(async (): Promise<string> => {
    if (!docId) return '';
    try {
      return await documentService.getText(docId);
    } catch (err) {
      console.error('Failed to get text:', err);
      return document?.content || '';
    }
  }, [docId, document]);

  // Refresh document from backend
  const refresh = useCallback(async () => {
    await loadDocument();
  }, [loadDocument]);

  return {
    document,
    isLoading,
    isSyncing,
    isDirty,
    error,
    collaborators,
    insertText,
    deleteText,
    getText,
    syncWithPeers,
    refresh,
  };
}
