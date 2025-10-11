/**
 * DocumentService - Wrapper for Tauri doc_commands
 *
 * Provides a clean TypeScript API for interacting with the CRDT document system.
 * Maps to Rust commands in communitas-desktop/src/doc_commands.rs
 *
 * Storage Modes:
 * - files: Encrypted, entity members only
 * - web: Public, unencrypted markdown
 * - both: Stored in both modes
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  Document,
  DocumentStorageMode,
  DocumentWithState,
  createDocId,
} from '../types/documents';

/**
 * DocumentService provides methods to interact with entity-scoped documents
 */
export class DocumentService {
  /**
   * Create a new document in entity-scoped storage
   *
   * @param entityId - Entity this document belongs to
   * @param name - Document name (will be prefixed with entity_id/)
   * @param storageMode - Storage mode: "files" | "web" | "both"
   * @returns Created document metadata
   *
   * @example
   * ```typescript
   * const doc = await documentService.createDocument(
   *   'channel-123',
   *   'meeting-notes',
   *   'files'
   * );
   * console.log(doc.docId); // "channel-123/meeting-notes"
   * ```
   */
  async createDocument(
    entityId: string,
    name: string,
    storageMode: DocumentStorageMode
  ): Promise<Document> {
    return await invoke('doc_create', {
      entityId,
      name,
      storageMode,
    });
  }

  /**
   * Insert text into document at specific position
   *
   * @param docId - Entity-scoped document ID
   * @param position - Character position (0-based)
   * @param text - Text to insert
   *
   * @example
   * ```typescript
   * await documentService.insertText('channel-123/notes', 0, 'Hello World!');
   * ```
   */
  async insertText(docId: string, position: number, text: string): Promise<void> {
    return await invoke('doc_insert_text', {
      docId,
      position,
      text,
    });
  }

  /**
   * Delete text from document
   *
   * @param docId - Entity-scoped document ID
   * @param position - Character position to delete from (0-based)
   * @param length - Number of characters to delete
   *
   * @example
   * ```typescript
   * // Delete 5 characters starting at position 7
   * await documentService.deleteText('channel-123/notes', 7, 5);
   * ```
   */
  async deleteText(docId: string, position: number, length: number): Promise<void> {
    return await invoke('doc_delete_text', {
      docId,
      position,
      length,
    });
  }

  /**
   * Get full text content of document
   *
   * @param docId - Entity-scoped document ID
   * @returns Document text content
   *
   * @example
   * ```typescript
   * const text = await documentService.getText('channel-123/notes');
   * console.log(text); // "Hello World!"
   * ```
   */
  async getText(docId: string): Promise<string> {
    return await invoke('doc_get_text', {
      docId,
    });
  }

  /**
   * Get CRDT update for synchronization (full document state)
   *
   * This encodes the full document state from the beginning.
   * Use this to sync to a new peer that doesn't have any prior state.
   *
   * @param docId - Entity-scoped document ID
   * @returns CRDT update bytes (Yrs encoded)
   *
   * @example
   * ```typescript
   * const update = await documentService.getCRDTUpdate('channel-123/notes');
   * // Send update to peer via network
   * ```
   */
  async getCRDTUpdate(docId: string): Promise<number[]> {
    return await invoke('doc_get_update', {
      docId,
    });
  }

  /**
   * Apply CRDT update from peer
   *
   * Creates the document if it doesn't exist, then applies the update.
   * This enables peer-to-peer document synchronization.
   *
   * @param docId - Entity-scoped document ID
   * @param update - CRDT update bytes from peer
   *
   * @example
   * ```typescript
   * // Receive update from peer
   * await documentService.applyCRDTUpdate('channel-123/notes', updateBytes);
   * ```
   */
  async applyCRDTUpdate(docId: string, update: number[]): Promise<void> {
    return await invoke('doc_apply_update', {
      docId,
      update,
    });
  }

  /**
   * List all documents for an entity in a specific storage mode
   *
   * @param entityId - Entity ID to list documents for
   * @param storageMode - Storage mode to filter by
   * @returns Array of documents
   *
   * @example
   * ```typescript
   * const docs = await documentService.listDocuments('channel-123', 'files');
   * docs.forEach(doc => console.log(doc.name));
   * ```
   */
  async listDocuments(
    entityId: string,
    storageMode: DocumentStorageMode
  ): Promise<Document[]> {
    return await invoke('doc_list', {
      entityId,
      storageMode,
    });
  }

  /**
   * Delete a document and all associated data
   *
   * @param docId - Entity-scoped document ID
   *
   * @example
   * ```typescript
   * await documentService.deleteDocument('channel-123/old-notes');
   * ```
   */
  async deleteDocument(docId: string): Promise<void> {
    return await invoke('doc_delete', {
      docId,
    });
  }

  /**
   * Get document with content (convenience method)
   *
   * Combines document metadata with text content.
   *
   * @param docId - Entity-scoped document ID
   * @returns Document with content
   */
  async getDocumentWithContent(docId: string): Promise<DocumentWithState> {
    // Parse doc ID to get entity and name
    const parts = docId.split('/');
    if (parts.length !== 2) {
      throw new Error(`Invalid document ID format: ${docId}`);
    }

    const [entityId, name] = parts;
    const content = await this.getText(docId);

    return {
      docId,
      entityId,
      name,
      storageMode: 'files', // We don't know the actual mode, default to files
      content,
      localState: {
        isEditing: false,
        isDirty: false,
        syncStatus: 'synced',
      },
    };
  }

  /**
   * Rename document (convenience method)
   *
   * Note: This requires creating a new document with the new name,
   * copying content, and deleting the old document.
   *
   * @param oldDocId - Current document ID
   * @param newName - New document name
   * @returns New document metadata
   */
  async renameDocument(oldDocId: string, newName: string): Promise<Document> {
    // Parse old doc ID
    const parts = oldDocId.split('/');
    if (parts.length !== 2) {
      throw new Error(`Invalid document ID format: ${oldDocId}`);
    }

    const [entityId] = parts;

    // Get current content
    const content = await this.getText(oldDocId);

    // Create new document with same storage mode
    // Note: We assume 'files' here; ideally we'd fetch the actual storage mode
    const newDoc = await this.createDocument(entityId, newName, 'files');

    // Copy content
    if (content) {
      await this.insertText(newDoc.docId, 0, content);
    }

    // Delete old document
    await this.deleteDocument(oldDocId);

    return newDoc;
  }

  /**
   * Check if document exists
   *
   * @param docId - Entity-scoped document ID
   * @returns True if document exists
   */
  async documentExists(docId: string): Promise<boolean> {
    try {
      await this.getText(docId);
      return true;
    } catch {
      return false;
    }
  }
}

/**
 * Singleton instance for easy access
 */
export const documentService = new DocumentService();
