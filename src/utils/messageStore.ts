import * as Automerge from '@automerge/automerge'
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
  doc: Automerge.Doc<MessageDoc>
  cachedMessages: Message[] | null
  cachedVersion: number
}

const STORAGE_PREFIX = 'automerge:messages:'
const stateCache = new Map<string, MessageState>()
const loadCache = new Map<string, Promise<MessageState>>()
const mutexCache = new Map<string, Mutex>()

const clone = <T>(value: T): T => {
  if (typeof structuredClone === 'function') {
    return structuredClone(value)
  }
  return JSON.parse(JSON.stringify(value)) as T
}

const ensureDocShape = (doc: Automerge.Doc<MessageDoc>): Automerge.Doc<MessageDoc> => {
  if ((doc as any).messages && (doc as any).order) {
    return doc
  }

  return Automerge.change(doc, { time: Date.now() }, draft => {
    draft.messages = {}
    draft.order = []
    draft.metadata = {
      updatedAt: Date.now(),
      version: 1,
    }
  })
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
      logger.warn('Failed to decode string-backed automerge payload', { error })
      return null
    }
  }
  return null
}

const materializeMessages = (state: MessageState): Message[] => {
  const currentVersion = state.doc.metadata?.version ?? 0

  // Return cached if version unchanged (fast path)
  if (state.cachedMessages && state.cachedVersion === currentVersion) {
    return state.cachedMessages
  }

  // Expensive operation - only when document changes
  const snapshot = Automerge.toJS(state.doc) as MessageDoc
  const order = snapshot.order ?? []
  const messages = snapshot.messages ?? {}
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
  const binary = Automerge.save(state.doc)
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
    let doc: Automerge.Doc<MessageDoc>

    if (stored) {
      const binary = toUint8Array(stored)
      if (binary && binary.length > 0) {
        try {
          doc = ensureDocShape(Automerge.load<MessageDoc>(binary))
        } catch (error) {
          // CRITICAL: Backup corrupted document before reset
          const backupKey = `${key}:corrupted:${Date.now()}`
          await offlineStorage.store(backupKey, stored).catch(() => {
            // Silently fail backup if storage unavailable
          })

          logger.error('Automerge document corrupted, backed up', {
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

          // Attempt DHT recovery
          const recovered = await recoverFromDHT(entityType, entityId)
          if (recovered) {
            logger.info('Successfully recovered from DHT', { entityType, entityId })
            doc = recovered
          } else {
            logger.warn('DHT recovery failed, starting fresh', { entityType, entityId })
            doc = ensureDocShape(Automerge.init<MessageDoc>())
          }
        }
      } else {
        doc = ensureDocShape(Automerge.init<MessageDoc>())
      }
    } else {
      doc = ensureDocShape(Automerge.init<MessageDoc>())
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
 * Attempt to recover messages from DHT when local Automerge document is corrupted
 */
async function recoverFromDHT(
  entityType: string,
  entityId: string,
): Promise<Automerge.Doc<MessageDoc> | null> {
  try {
    logger.info('Attempting DHT recovery', { entityType, entityId })

    const messages = await backendService.getMessages(entityType, entityId)
    if (messages.length === 0) {
      logger.warn('No messages found in DHT for recovery', { entityType, entityId })
      return null
    }

    // Create fresh Automerge document with recovered messages
    let doc = ensureDocShape(Automerge.init<MessageDoc>())
    doc = Automerge.change(doc, { time: Date.now() }, draft => {
      messages.forEach(msg => {
        draft.messages[msg.id] = clone(msg)
      })
      draft.metadata.updatedAt = Date.now()
      draft.metadata.version = 1
    })

    logger.info('Successfully recovered messages from DHT', {
      entityType,
      entityId,
      messageCount: messages.length,
    })

    return doc
  } catch (error) {
    logger.error('DHT recovery failed', { error, entityType, entityId })
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

const applyMessages = async (entityType: string, entityId: string, updater: (draft: MessageDoc) => void): Promise<Message[]> => {
  const key = storageKey(entityType, entityId)
  const mutex = getMutex(key)

  return mutex.runExclusive(async () => {
    const state = await getState(entityType, entityId)

    state.doc = Automerge.change(state.doc, draft => {
      if (!draft.metadata) {
        draft.metadata = {
          updatedAt: Date.now(),
          version: 0,
        }
      }

      updater(draft)
      draft.metadata.updatedAt = Date.now()
      draft.metadata.version = (draft.metadata.version ?? 0) + 1

      const sortedIds = Object.values(draft.messages)
        .map(message => ({ id: message.id, timestamp: new Date(message.timestamp).getTime() }))
        .sort((a, b) => a.timestamp - b.timestamp)
        .map(entry => entry.id)

      draft.order.length = 0
      sortedIds.forEach(id => draft.order.push(id))
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
  return applyMessages(entityType, entityId, draft => {
    for (const message of remoteMessages) {
      draft.messages[message.id] = structuredClone(message)
    }
  })
}

export const upsertMessage = async (
  entityType: string,
  entityId: string,
  message: Message,
): Promise<void> => {
  await applyMessages(entityType, entityId, draft => {
    draft.messages[message.id] = clone(message)
  })
}

export const markMessageStatus = async (
  entityType: string,
  entityId: string,
  messageId: string,
  status: Message['status'],
): Promise<void> => {
  await applyMessages(entityType, entityId, draft => {
    const existing = draft.messages[messageId]
    if (existing) {
      draft.messages[messageId] = {
        ...clone(existing),
        status,
      }
    }
  })
}

export const removeMessage = async (
  entityType: string,
  entityId: string,
  messageId: string,
): Promise<void> => {
  await applyMessages(entityType, entityId, draft => {
    delete draft.messages[messageId]
    const index = draft.order.findIndex(id => id === messageId)
    if (index >= 0) {
      draft.order.splice(index, 1)
    }
  })
}
