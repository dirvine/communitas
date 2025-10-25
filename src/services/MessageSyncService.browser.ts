/**
 * Browser-compatible MessageSyncService Mock
 * Provides a lightweight in-memory implementation with persistence via localStorage
 * and cross-tab propagation through BroadcastChannel. Mirrors camelCase CRDT types.
 */

import type {
    CRDTMessage, EntitySyncState,
    SyncRequest,
    SyncResponse, VectorClock
} from '../types/crdt'

export interface ReceiveResult {
  accepted: boolean
  outOfOrder: boolean
  missingRanges?: Array<{
    peerId: string
    fromTimestamp: number
    toTimestamp: number
  }>
}

export interface SyncResult {
  messagesAdded: number
  messagesRejected: number
}

type EntityType = CRDTMessage['metadata']['entityType']

type Mutable<T> = {
  -readonly [K in keyof T]: T[K]
}

const STORAGE_PREFIX = 'crdt:messages:'

const emptyClock = (): VectorClock => ({})

const normalizeVectorClock = (raw: unknown): VectorClock => {
  if (!raw || typeof raw !== 'object') {
    return emptyClock()
  }
  const normalized: VectorClock = {}
  for (const [peerId, value] of Object.entries(raw as Record<string, unknown>)) {
    const numeric = typeof value === 'number' ? value : Number(value)
    if (!Number.isNaN(numeric)) {
      normalized[peerId] = numeric
    }
  }
  return normalized
}

const normalizeMessage = (raw: any): CRDTMessage | null => {
  if (!raw || typeof raw !== 'object') {
    return null
  }

  const metadataSource = raw.metadata ?? {}
  const contentSource = raw.content ?? {}
  const localStateSource = raw.localState ?? raw.local_state

  const metadata: Mutable<CRDTMessage['metadata']> = {
    id: metadataSource.id ?? metadataSource.ID ?? `msg-${Date.now()}`,
    entityId: metadataSource.entityId ?? metadataSource.entity_id ?? 'unknown-entity',
    entityType: metadataSource.entityType ?? metadataSource.entity_type ?? 'group',
    authorPeerId: metadataSource.authorPeerId ?? metadataSource.author_peer_id ?? 'unknown-peer',
    vectorClock: normalizeVectorClock(metadataSource.vectorClock ?? metadataSource.vector_clock),
    lamportClock: metadataSource.lamportClock ?? metadataSource.lamport_clock ?? 0,
    timestamp: metadataSource.timestamp ?? Date.now(),
    previousMessageId: metadataSource.previousMessageId ?? metadataSource.previous_message_id,
    replyToId: metadataSource.replyToId ?? metadataSource.reply_to_id,
  }

  const reactions = (localStateSource?.reactions ?? []).map((reaction: any) => ({
    emoji: reaction.emoji,
    count: reaction.count ?? 0,
    userReacted: reaction.userReacted ?? reaction.user_reacted ?? false,
    peerIds: reaction.peerIds ?? reaction.peer_ids ?? [],
  }))

  const localState = localStateSource
    ? {
        status: localStateSource.status,
        reactions,
        threadCount: localStateSource.threadCount ?? localStateSource.thread_count,
        latestReplyBy: localStateSource.latestReplyBy ?? localStateSource.latest_reply_by,
      }
    : undefined

  return {
    content: {
      text: contentSource.text ?? '',
      author: contentSource.author ?? 'Unknown',
      attachments: contentSource.attachments,
    },
    metadata,
    localState,
  }
}

export class MessageSyncService {
  private initialized = false
  private peerId = ''
  private messages: Map<string, CRDTMessage[]> = new Map()
  private vectorClocks: Map<string, VectorClock> = new Map()
  private broadcastChannel: BroadcastChannel | null = null
  private lamportClock = 0

  async initialize(peerId: string): Promise<void> {
    if (this.initialized) {
      return
    }

    this.peerId = peerId
    this.initialized = true

    this.broadcastChannel = new BroadcastChannel('crdt-sync')
    this.broadcastChannel.onmessage = (event) => {
      if (event?.data?.type !== 'new-message') {
        return
      }
      if (event.data.peerId === this.peerId) {
        return
      }
      const normalized = normalizeMessage(event.data.message)
      if (!normalized) {
        return
      }
      this.appendMessage(normalized)
    }

    this.loadFromLocalStorage()

    console.log(`✅ MessageSyncService (Browser Mock) initialized with peer: ${peerId}`)
  }

  async getAllMessages(entityId: string): Promise<SyncResponse> {
    this.ensureInitialized()
    const messages = this.messages.get(entityId) ?? []
    const vectorClock = this.vectorClocks.get(entityId) ?? emptyClock()
    const entityType: EntityType = messages[0]?.metadata.entityType ?? 'group'

    return {
      entityId,
      entityType,
      messages,
      vectorClock,
    }
  }

  async receiveMessage(message: CRDTMessage): Promise<ReceiveResult> {
    this.ensureInitialized()
    const normalized = normalizeMessage(message)
    if (!normalized) {
      return { accepted: false, outOfOrder: false }
    }

    const added = this.appendMessage(normalized)
    return {
      accepted: added,
      outOfOrder: false,
    }
  }

  async sendMessage(
    entityId: string,
    entityType: EntityType,
    text: string,
    author: string,
    replyToId?: string,
  ): Promise<CRDTMessage> {
    this.ensureInitialized()

    this.lamportClock += 1

    const currentClock = this.vectorClocks.get(entityId) ?? emptyClock()
    const nextClock: VectorClock = {
      ...currentClock,
      [this.peerId]: (currentClock[this.peerId] ?? 0) + 1,
    }
    this.vectorClocks.set(entityId, nextClock)

    const message: CRDTMessage = {
      content: {
        text,
        author,
      },
      metadata: {
        id: `msg-${Date.now()}-${Math.random().toString(16).slice(2)}`,
        entityId,
        entityType,
        authorPeerId: this.peerId,
        vectorClock: nextClock,
        lamportClock: this.lamportClock,
        timestamp: Date.now(),
        previousMessageId: undefined,
        replyToId,
      },
      localState: {
        status: 'sent',
        reactions: [],
        threadCount: undefined,
        latestReplyBy: undefined,
      },
    }

    this.appendMessage(message)

    if (this.broadcastChannel) {
      this.broadcastChannel.postMessage({
        type: 'new-message',
        peerId: this.peerId,
        message,
      })
    }

    return message
  }

  async requestSync(entityId: string): Promise<SyncRequest> {
    this.ensureInitialized()
    const vectorClock = this.vectorClocks.get(entityId) ?? emptyClock()
    const entityType: EntityType = this.messages.get(entityId)?.[0]?.metadata.entityType ?? 'group'

    return {
      entityId,
      entityType,
      requesterPeerId: this.peerId,
      vectorClock,
      missingMessageIds: undefined,
    }
  }

  async handleSyncResponse(response: SyncResponse): Promise<SyncResult> {
    this.ensureInitialized()
    const { entityId } = response
    let added = 0

    response.messages.forEach(message => {
      if (this.appendMessage(message)) {
        added += 1
      }
    })

    this.vectorClocks.set(entityId, response.vectorClock)

    if (added > 0) {
      this.saveToLocalStorage(entityId)
    }

    return {
      messagesAdded: added,
      messagesRejected: 0,
    }
  }

  async getSyncState(entityId: string): Promise<EntitySyncState> {
    this.ensureInitialized()
    const messages = this.messages.get(entityId) ?? []
    const vectorClock = this.vectorClocks.get(entityId) ?? emptyClock()
    const entityType: EntityType = messages[0]?.metadata.entityType ?? 'group'

    return {
      entityId,
      entityType,
      vectorClock,
      lastSyncTime: Date.now(),
      messageCount: messages.length,
      missingMessages: [],
      outOfOrderMessages: [],
    }
  }

  async getMessages(entityId: string): Promise<CRDTMessage[]> {
    this.ensureInitialized()
    this.loadFromLocalStorage()
    const messages = this.messages.get(entityId) ?? []
    return [...messages].sort((a, b) => a.metadata.lamportClock - b.metadata.lamportClock)
  }

  async needsSync(): Promise<boolean> {
    this.ensureInitialized()
    return false
  }

  private appendMessage(message: CRDTMessage): boolean {
    const entityId = message.metadata.entityId
    const list = this.messages.get(entityId) ?? []
    if (list.find(existing => existing.metadata.id === message.metadata.id)) {
      return false
    }

    list.push(message)
    this.messages.set(entityId, list)
    this.vectorClocks.set(entityId, message.metadata.vectorClock)
    this.saveToLocalStorage(entityId)
    return true
  }

  private ensureInitialized() {
    if (!this.initialized) {
      throw new Error('MessageSyncService not initialized. Call initialize() first.')
    }
  }

  private saveToLocalStorage(entityId: string) {
    const messages = this.messages.get(entityId) ?? []
    try {
      localStorage.setItem(`${STORAGE_PREFIX}${entityId}`, JSON.stringify(messages))
    } catch (error) {
      console.warn('Persisting messages failed:', error)
    }
  }

  private loadFromLocalStorage() {
    if (typeof localStorage === 'undefined') {
      return
    }

    for (let i = 0; i < localStorage.length; i += 1) {
      const key = localStorage.key(i)
      if (!key || !key.startsWith(STORAGE_PREFIX)) {
        continue
      }
      const entityId = key.slice(STORAGE_PREFIX.length)
      const raw = localStorage.getItem(key)
      if (!raw) {
        continue
      }
      try {
        const parsed = JSON.parse(raw) as unknown[]
        const normalizedMessages = parsed
          .map(value => normalizeMessage(value))
          .filter((message): message is CRDTMessage => Boolean(message))

        if (normalizedMessages.length > 0) {
          this.messages.set(entityId, normalizedMessages)
          this.vectorClocks.set(entityId, normalizedMessages[normalizedMessages.length - 1].metadata.vectorClock)
        }
      } catch (error) {
        console.warn('Failed to load CRDT messages from storage:', error)
      }
    }
  }
}

let instance: MessageSyncService | null = null

export function getMessageSyncService(): MessageSyncService {
  if (!instance) {
    instance = new MessageSyncService()
  }
  return instance
}
