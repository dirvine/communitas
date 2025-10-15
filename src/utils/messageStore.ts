import * as Y from 'yjs'
import { Mutex } from 'async-mutex'
import { offlineStorage } from '../services/storage/OfflineStorageService'
import { logger } from '../services/LoggingService'
import { backendService } from '../services/api/BackendService'
import type { Message } from '../components/chat/EntityChatView'

interface MessageDoc {
  messages: Record<string, Message>
  order: string[]
  metadata: {
    updatedAt: number
    version: number
  }
}

interface MessageState {
  doc: Y.Doc
  cachedMessages: Message[] | null
  cachedVersion: number
}

const STORAGE_PREFIX = 'yjs:messages:'
const stateCache = new Map<string, MessageState>()
const loadCache = new Map<string, Promise<MessageState>>()
const mutexCache = new Map<string, Mutex>()

const clone = <T>(value: T): T => {
  if (typeof structuredClone === 'function') {
    return structuredClone(value)
  }
  return JSON.parse(JSON.stringify(value)) as T
}

const ensureDocShape = (doc: Y.Doc): void => {
  if (!doc.getMap('root').has('messages')) {
    doc.getMap('root').set('messages', new Y.Map())
  }
  if (!doc.getMap('root').has('order')) {
    doc.getMap('root').set('order', new Y.Array())
  }
  if (!doc.getMap('root').has('metadata')) {
    const metadata = new Y.Map()
    metadata.set('updatedAt', Date.now())
    metadata.set('version', 1)
    doc.getMap('root').set('metadata', metadata)
  }
}

const storageKey = (entityType: string, entityId: string) => `${STORAGE_PREFIX}${entityType}:${entityId}`

const toUint8Array = (data: unknown): Uint8Array | null => {
  if (!data) return null
  if (data instanceof Uint8Array) return data
  if (Array.isArray(data)) return new Uint8Array(data)
  if (typeof data === 'string') {
    try {
      const parsed = JSON.parse(data)
      if (Array.isArray(parsed)) {
        return new Uint8Array(parsed)
      }
    } catch (error) {
      logger.warn('Failed to decode string-backed Yjs payload', { error })
      return null
    }
  }
  return null
}

const materializeMessages = (state: MessageState): Message[] => {
  const root = state.doc.getMap('root')
  const metadata = root.get('metadata') as Y.Map<any> | undefined
  const currentVersion = metadata?.get('version') ?? 0

  // Return cached if version unchanged (fast path)
  if (state.cachedMessages && state.cachedVersion === currentVersion) {
    return state.cachedMessages
  }

  // Expensive operation - only when document changes
  const messagesMap = root.get('messages') as Y.Map<any>
  const orderArray = root.get('order') as Y.Array<string>

  const messages: Record<string, Message> = {}
  messagesMap?.forEach((value, key) => {
    messages[key] = clone(value)
  })

  const order = orderArray?.toArray() ?? []
  const seen = new Set<string>()
  const ordered: Message[] = []

  for (const id of order) {
    const message = messages[id]
    if (message) {
      ordered.push(clone(message))
      seen.add(id)
    }
  }

  for (const [id, message] of Object.entries(messages)) {
    if (!seen.has(id)) {
      ordered.push(clone(message))
    }
  }

  ordered.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime())

  // Cache results for next call
  state.cachedMessages = ordered
  state.cachedVersion = currentVersion

  return ordered
}

const persistState = async (key: string, state: MessageState) => {
  const binary = Y.encodeStateAsUpdate(state.doc)
  await offlineStorage.store(key, Array.from(binary))
}

const getState = async (entityType: string, entityId: string): Promise<MessageState> => {
  const key = storageKey(entityType, entityId)
  if (stateCache.has(key)) {
    return stateCache.get(key)!
  }
  if (loadCache.has(key)) {
    return loadCache.get(key)!
  }

  const promise = (async () => {
    const stored = await offlineStorage.get<number[] | Uint8Array | null>(key)
    const doc = new Y.Doc()

    if (stored) {
      const binary = toUint8Array(stored)
      if (binary && binary.length > 0) {
        try {
          Y.applyUpdate(doc, binary)
          ensureDocShape(doc)
        } catch (error) {
          // CRITICAL: Backup corrupted document before reset
          const backupKey = `${key}:corrupted:${Date.now()}`
          await offlineStorage.store(backupKey, stored).catch(() => {
            // Silently fail backup if storage unavailable
          })

          logger.error('Yjs document corrupted, backed up', {
            error,
            entityType,
            entityId,
            backupKey,
          })

          // Emit event for UI notification
          if (typeof window !== 'undefined') {
            window.dispatchEvent(
              new CustomEvent('messages:corruption', {
                detail: { entityType, entityId, backupKey },
              }),
            )
          }

          // Attempt backend recovery
          const recovered = await recoverFromBackend(entityType, entityId)
          if (recovered) {
            logger.info('Successfully recovered from backend', { entityType, entityId })
            Y.applyUpdate(doc, Y.encodeStateAsUpdate(recovered))
          } else {
            logger.warn('Backend recovery failed, starting fresh', { entityType, entityId })
            ensureDocShape(doc)
          }
        }
      } else {
        ensureDocShape(doc)
      }
    } else {
      ensureDocShape(doc)
    }

    const state: MessageState = {
      doc,
      cachedMessages: null,
      cachedVersion: 0
    }
    stateCache.set(key, state)
    loadCache.delete(key)
    return state
  })()

  loadCache.set(key, promise)
  return promise
}

/**
 * Attempt to recover messages from backend when local Yjs document is corrupted
 */
async function recoverFromBackend(
  entityType: string,
  entityId: string,
): Promise<Y.Doc | null> {
  try {
    logger.info('Attempting backend recovery', { entityType, entityId })

    const messages = await backendService.getMessages(entityType, entityId)
    if (messages.length === 0) {
      logger.warn('No messages found in backend for recovery', { entityType, entityId })
      return null
    }

    // Create fresh Yjs document with recovered messages
    const doc = new Y.Doc()
    ensureDocShape(doc)

    const root = doc.getMap('root')
    const messagesMap = root.get('messages') as Y.Map<any>
    const metadata = root.get('metadata') as Y.Map<any>

    doc.transact(() => {
      messages.forEach(msg => {
        messagesMap.set(msg.id, clone(msg))
      })
      metadata.set('updatedAt', Date.now())
      metadata.set('version', 1)
    })

    logger.info('Successfully recovered messages from backend', {
      entityType,
      entityId,
      messageCount: messages.length,
    })

    return doc
  } catch (error) {
    logger.error('Backend recovery failed', { error, entityType, entityId })
    return null
  }
}

/**
 * Get or create a mutex for a given entity to prevent concurrent modifications
 */
const getMutex = (key: string): Mutex => {
  let mutex = mutexCache.get(key)
  if (!mutex) {
    mutex = new Mutex()
    mutexCache.set(key, mutex)
  }
  return mutex
}

const applyMessages = async (entityType: string, entityId: string, updater: (root: Y.Map<any>) => void): Promise<Message[]> => {
  const key = storageKey(entityType, entityId)
  const mutex = getMutex(key)

  return mutex.runExclusive(async () => {
    const state = await getState(entityType, entityId)
    const root = state.doc.getMap('root')

    state.doc.transact(() => {
      const metadata = root.get('metadata') as Y.Map<any>
      if (!metadata) {
        const newMetadata = new Y.Map()
        newMetadata.set('updatedAt', Date.now())
        newMetadata.set('version', 0)
        root.set('metadata', newMetadata)
      }

      updater(root)

      const metadataMap = root.get('metadata') as Y.Map<any>
      metadataMap.set('updatedAt', Date.now())
      metadataMap.set('version', (metadataMap.get('version') ?? 0) + 1)

      // Update order array based on timestamp sorting
      const messagesMap = root.get('messages') as Y.Map<any>
      const sortedIds: string[] = []
      messagesMap.forEach((message, id) => {
        sortedIds.push({ id, timestamp: new Date(message.timestamp).getTime() } as any)
      })
      sortedIds.sort((a: any, b: any) => a.timestamp - b.timestamp)

      const orderArray = root.get('order') as Y.Array<string>
      orderArray.delete(0, orderArray.length)
      sortedIds.forEach((entry: any) => orderArray.push([entry.id]))
    })

    await persistState(key, state)
    return materializeMessages(state)
  })
}

export const loadMessages = async (entityType: string, entityId: string): Promise<Message[]> => {
  const state = await getState(entityType, entityId)
  return materializeMessages(state)
}

export const mergeRemoteMessages = async (
  entityType: string,
  entityId: string,
  remoteMessages: Message[],
): Promise<Message[]> => {
  return applyMessages(entityType, entityId, root => {
    const messagesMap = root.get('messages') as Y.Map<any>
    for (const message of remoteMessages) {
      messagesMap.set(message.id, structuredClone(message))
    }
  })
}

export const upsertMessage = async (
  entityType: string,
  entityId: string,
  message: Message,
): Promise<void> => {
  await applyMessages(entityType, entityId, root => {
    const messagesMap = root.get('messages') as Y.Map<any>
    messagesMap.set(message.id, clone(message))
  })
}

export const markMessageStatus = async (
  entityType: string,
  entityId: string,
  messageId: string,
  status: Message['status'],
): Promise<void> => {
  await applyMessages(entityType, entityId, root => {
    const messagesMap = root.get('messages') as Y.Map<any>
    const existing = messagesMap.get(messageId)
    if (existing) {
      messagesMap.set(messageId, {
        ...clone(existing),
        status,
      })
    }
  })
}

export const removeMessage = async (
  entityType: string,
  entityId: string,
  messageId: string,
): Promise<void> => {
  await applyMessages(entityType, entityId, root => {
    const messagesMap = root.get('messages') as Y.Map<any>
    const orderArray = root.get('order') as Y.Array<string>

    messagesMap.delete(messageId)

    const orderArr = orderArray.toArray()
    const index = orderArr.findIndex(id => id === messageId)
    if (index >= 0) {
      orderArray.delete(index, 1)
    }
  })
}
