/**
 * Local-first DocumentService implementation
 * Provides fully functional document editing without requiring Tauri commands.
 */

import {
  Document,
  DocumentStorageMode,
  DocumentWithState,
  createDocId,
} from '../types/documents';

const STORAGE_KEY = 'communitas.docs.v1';

type StoredDocument = {
  meta: Document;
  content: string;
  createdAt: number;
  updatedAt: number;
};

type DocumentStore = Record<string, StoredDocument>;

const loadStore = (): DocumentStore => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as DocumentStore;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
};

const persistStore = (store: DocumentStore) => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
};

const ensureStore = (): DocumentStore => {
  const store = loadStore();
  if (!Object.keys(store).length) {
    persistStore(store);
  }
  return store;
};

export class DocumentService {
  private getStore(): DocumentStore {
    return ensureStore();
  }

  private writeStore(next: DocumentStore) {
    persistStore(next);
  }

  private upsert(doc: StoredDocument) {
    const store = this.getStore();
    store[doc.meta.docId] = doc;
    this.writeStore(store);
  }

  private remove(docId: string) {
    const store = this.getStore();
    delete store[docId];
    this.writeStore(store);
  }

  async createDocument(entityId: string, name: string, storageMode: DocumentStorageMode): Promise<Document> {
    const safeName = name.trim().replace(/\s+/g, '-');
    const docId = createDocId(entityId, safeName);
    const now = Date.now();

    const existing = this.getStore();
    if (existing[docId]) {
      throw new Error(`Document ${safeName} already exists for ${entityId}`);
    }

    const meta: Document = {
      docId,
      entityId,
      name: safeName,
      storageMode,
    };

    const stored: StoredDocument = {
      meta,
      content: '',
      createdAt: now,
      updatedAt: now,
    };

    this.upsert(stored);
    return meta;
  }

  async insertText(docId: string, position: number, text: string): Promise<void> {
    const store = this.getStore();
    const current = store[docId];
    if (!current) {
      throw new Error(`Document ${docId} not found`);
    }

    const safePos = Math.max(0, Math.min(position, current.content.length));
    current.content =
      current.content.slice(0, safePos) + text + current.content.slice(safePos);
    current.updatedAt = Date.now();

    this.upsert(current);
  }

  async deleteText(docId: string, position: number, length: number): Promise<void> {
    const store = this.getStore();
    const current = store[docId];
    if (!current) {
      throw new Error(`Document ${docId} not found`);
    }

    const start = Math.max(0, Math.min(position, current.content.length));
    const end = Math.max(start, Math.min(start + length, current.content.length));
    current.content = current.content.slice(0, start) + current.content.slice(end);
    current.updatedAt = Date.now();

    this.upsert(current);
  }

  async getText(docId: string): Promise<string> {
    const store = this.getStore();
    const current = store[docId];
    if (!current) {
      throw new Error(`Document ${docId} not found`);
    }
    return current.content;
  }

  async getCRDTUpdate(docId: string): Promise<number[]> {
    const text = await this.getText(docId);
    return Array.from(text).map(char => char.charCodeAt(0));
  }

  async applyCRDTUpdate(docId: string, update: number[]): Promise<void> {
    const payload = update.map(code => String.fromCharCode(code)).join('');
    const store = this.getStore();
    const current = store[docId];

    if (!current) {
      this.upsert({
        meta: {
          docId,
          entityId: docId.split('/')[0],
          name: docId.split('/')[1] ?? docId,
          storageMode: 'files',
        },
        content: payload,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      });
      return;
    }

    current.content = payload;
    current.updatedAt = Date.now();
    this.upsert(current);
  }

  async listDocuments(entityId: string, storageMode: DocumentStorageMode): Promise<Document[]> {
    const store = this.getStore();
    return Object.values(store)
      .filter(doc => doc.meta.entityId === entityId && (storageMode === 'both' || doc.meta.storageMode === storageMode))
      .map(doc => doc.meta);
  }

  async deleteDocument(docId: string): Promise<void> {
    this.remove(docId);
  }

  async getDocumentWithContent(docId: string): Promise<DocumentWithState> {
    const store = this.getStore();
    const current = store[docId];
    if (!current) {
      throw new Error(`Document ${docId} not found`);
    }

    return {
      ...current.meta,
      content: current.content,
      modified: new Date(current.updatedAt).toISOString(),
      size: current.content.length,
      localState: {
        isEditing: false,
        isDirty: false,
        syncStatus: 'synced',
        lastSyncTime: current.updatedAt,
      },
    };
  }

  async renameDocument(oldDocId: string, newName: string): Promise<Document> {
    const store = this.getStore();
    const current = store[oldDocId];
    if (!current) {
      throw new Error(`Document ${oldDocId} not found`);
    }

    const { entityId } = current.meta;
    const newDoc = await this.createDocument(entityId, newName, current.meta.storageMode);
    await this.insertText(newDoc.docId, 0, current.content);
    await this.deleteDocument(oldDocId);
    return newDoc;
  }

  async documentExists(docId: string): Promise<boolean> {
    const store = this.getStore();
    return docId in store;
  }
}

export const documentService = new DocumentService();
