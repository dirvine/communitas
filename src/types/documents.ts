/**
 * Document CRDT Types for Communitas
 *
 * These types match the Rust doc_commands API in communitas-desktop/src/doc_commands.rs
 * Documents use Yrs CRDT for collaborative text editing with entity-scoped storage.
 *
 * Storage Modes:
 * - Files: Encrypted, accessible to entity members only (threshold encryption)
 * - Web: Public, unencrypted markdown for website publishing
 * - Both: Document stored in both Files and Web storage simultaneously
 */

/**
 * Storage mode for documents
 * Maps to StorageMode enum in doc_replicator.rs
 */
export type DocumentStorageMode = 'files' | 'web' | 'both';

/**
 * Document metadata returned from backend
 * Corresponds to DocResponse in doc_commands.rs
 */
export interface Document {
  /** Entity-scoped document ID: {entity_id}/{doc_name} */
  docId: string;
  /** Entity this document belongs to */
  entityId: string;
  /** Document name (without entity prefix) */
  name: string;
  /** Storage mode: "files" | "web" | "both" */
  storageMode: DocumentStorageMode;
}

/**
 * Request to create a new document
 */
export interface CreateDocumentRequest {
  /** Entity ID this document belongs to */
  entityId: string;
  /** Document name (will be prefixed with entity_id/) */
  name: string;
  /** Storage mode */
  storageMode: DocumentStorageMode;
}

/**
 * Request to insert text into document
 */
export interface InsertTextRequest {
  /** Entity-scoped document ID */
  docId: string;
  /** Character position to insert at (0-based) */
  position: number;
  /** Text to insert */
  text: string;
}

/**
 * Request to delete text from document
 */
export interface DeleteTextRequest {
  /** Entity-scoped document ID */
  docId: string;
  /** Character position to delete from (0-based) */
  position: number;
  /** Number of characters to delete */
  length: number;
}

/**
 * Request to get document text
 */
export interface GetTextRequest {
  /** Entity-scoped document ID */
  docId: string;
}

/**
 * Request to get CRDT update for synchronization
 */
export interface GetUpdateRequest {
  /** Entity-scoped document ID */
  docId: string;
}

/**
 * Request to apply CRDT update from peer
 */
export interface ApplyUpdateRequest {
  /** Entity-scoped document ID */
  docId: string;
  /** CRDT update bytes (Yrs encoded update) */
  update: number[]; // Vec<u8> in Rust maps to number[] in TypeScript
}

/**
 * Request to list documents for an entity
 */
export interface ListDocumentsRequest {
  /** Entity ID to list documents for */
  entityId: string;
  /** Storage mode to filter by */
  storageMode: DocumentStorageMode;
}

/**
 * Request to delete a document
 */
export interface DeleteDocumentRequest {
  /** Entity-scoped document ID */
  docId: string;
}

/**
 * Local document state (UI-only, not synced)
 */
export interface DocumentLocalState {
  /** Is document currently being edited? */
  isEditing: boolean;
  /** Cursor position in document */
  cursorPosition?: number;
  /** Has unsaved changes? */
  isDirty: boolean;
  /** Last sync timestamp */
  lastSyncTime?: number;
  /** Sync status */
  syncStatus?: 'synced' | 'syncing' | 'offline' | 'error';
  /** Error message if sync failed */
  syncError?: string;
}

/**
 * Complete document with local state
 */
export interface DocumentWithState extends Document {
  /** Document text content (cached locally) */
  content?: string;
  /** Local UI state */
  localState?: DocumentLocalState;
  /** File size in bytes */
  size?: number;
  /** Last modified timestamp */
  modified?: string;
}

/**
 * Document sync state for entity
 */
export interface EntityDocumentSyncState {
  /** Entity ID */
  entityId: string;
  /** Number of documents in Files storage */
  filesDocCount: number;
  /** Number of documents in Web storage */
  webDocCount: number;
  /** Last successful sync time */
  lastSyncTime?: number;
  /** Pending updates to sync */
  pendingUpdates: number;
}

/**
 * Helper to create entity-scoped document ID
 */
export function createDocId(entityId: string, docName: string): string {
  return `${entityId}/${docName}`;
}

/**
 * Helper to parse entity-scoped document ID
 */
export function parseDocId(docId: string): { entityId: string; docName: string } | null {
  const parts = docId.split('/');
  if (parts.length !== 2) {
    return null;
  }
  return {
    entityId: parts[0],
    docName: parts[1],
  };
}

/**
 * Helper to get storage mode label for UI
 */
export function getStorageModeLabel(mode: DocumentStorageMode): string {
  switch (mode) {
    case 'files':
      return 'Private (Encrypted)';
    case 'web':
      return 'Public (Website)';
    case 'both':
      return 'Both (Private + Public)';
  }
}

/**
 * Helper to get storage mode description
 */
export function getStorageModeDescription(mode: DocumentStorageMode): string {
  switch (mode) {
    case 'files':
      return 'Encrypted with threshold encryption. Only accessible to entity members.';
    case 'web':
      return 'Plain markdown for public website. Not encrypted.';
    case 'both':
      return 'Stored in both private encrypted storage and public website storage.';
  }
}

/**
 * Validate document name
 */
export function isValidDocumentName(name: string): boolean {
  // No slashes (would break entity-scoped ID format)
  if (name.includes('/')) return false;
  // No empty names
  if (!name.trim()) return false;
  // Reasonable length limit
  if (name.length > 255) return false;
  return true;
}

/**
 * Get file extension from document name
 */
export function getDocumentExtension(name: string): string {
  const parts = name.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
}

/**
 * Check if document is markdown
 */
export function isMarkdownDocument(name: string): boolean {
  const ext = getDocumentExtension(name);
  return ext === 'md' || ext === 'markdown';
}
