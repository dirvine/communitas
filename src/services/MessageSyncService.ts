/**
 * Message Synchronization Service
 *
 * TypeScript wrapper for Rust backend CRDT message synchronization.
 * Handles CRDT-based message synchronization across peers:
 * - get_all_messages() for full sync requests
 * - Out-of-order message detection
 * - Missing message reply mechanism
 * - Causal consistency enforcement
 */

import { invoke } from '@tauri-apps/api/core'
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

export class MessageSyncService {
  private initialized = false

  /**
   * Initialize the message sync service with our peer ID
   */
  async initialize(peerId: string): Promise<void> {
    await invoke('message_sync_initialize', { peerId })
    this.initialized = true
    console.log(`🔄 MessageSyncService initialized for peer: ${peerId}`)
  }

  /**
   * Get all messages for an entity (contact, group, project, org, channel)
   * This is the entry point for sync requests from other peers
   */
  async getAllMessages(entityId: string): Promise<SyncResponse> {
    this.ensureInitialized()
    return await invoke<SyncResponse>('message_sync_get_all_messages', { entityId })
  }

  /**
   * Handle incoming message - detect out-of-order and missing dependencies
   */
  async receiveMessage(message: CRDTMessage): Promise<ReceiveResult> {
    this.ensureInitialized()
    return await invoke<ReceiveResult>('message_sync_receive_message', { message })
  }

  /**
   * Send a new message - assigns vector clock and Lamport timestamp
   */
  async sendMessage(
    entityId: string,
    entityType: 'person' | 'group' | 'project' | 'channel' | 'organisation',
    text: string,
    author: string,
    replyToId?: string
  ): Promise<CRDTMessage> {
    this.ensureInitialized()
    return await invoke<CRDTMessage>('message_sync_send_message', {
      entityId,
      entityType,
      text,
      author,
      replyToId,
    })
  }

  /**
   * Request sync from a peer - send our vector clock and get missing messages
   */
  async requestSync(entityId: string, fromPeerId: string): Promise<SyncRequest> {
    this.ensureInitialized()
    return await invoke<SyncRequest>('message_sync_request_sync', {
      entityId,
      fromPeerId,
    })
  }

  /**
   * Handle sync response - integrate received messages
   */
  async handleSyncResponse(response: SyncResponse): Promise<SyncResult> {
    this.ensureInitialized()
    return await invoke<SyncResult>('message_sync_handle_sync_response', { response })
  }

  /**
   * Get sync state for an entity
   */
  async getSyncState(entityId: string): Promise<EntitySyncState> {
    this.ensureInitialized()
    return await invoke<EntitySyncState>('message_sync_get_sync_state', { entityId })
  }

  /**
   * Get all messages in causal order for an entity
   */
  async getMessages(entityId: string): Promise<CRDTMessage[]> {
    this.ensureInitialized()
    return await invoke<CRDTMessage[]>('message_sync_get_messages', { entityId })
  }

  /**
   * Detect if we need to request a sync (missing messages)
   */
  async needsSync(entityId: string, remoteClock: VectorClock): Promise<boolean> {
    this.ensureInitialized()
    return await invoke<boolean>('message_sync_needs_sync', {
      entityId,
      remoteClock,
    })
  }

  private ensureInitialized() {
    if (!this.initialized) {
      throw new Error('MessageSyncService not initialized. Call initialize() first.')
    }
  }
}

// Singleton instance
let instance: MessageSyncService | null = null

export function getMessageSyncService(): MessageSyncService {
  if (!instance) {
    instance = new MessageSyncService()
  }
  return instance
}
