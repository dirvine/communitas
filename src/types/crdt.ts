/**
 * CRDT (Conflict-free Replicated Data Type) Types for Communitas
 *
 * This implements a causal consistency model for distributed messaging where:
 * - Messages can arrive out of order
 * - Peers can request missing messages
 * - Each peer maintains a vector clock for causal ordering
 * - All entities (contacts, groups, projects, orgs, channels) follow the same pattern
 */

/**
 * Vector Clock - Tracks logical time for each peer
 * Maps peer ID -> logical timestamp
 */
export type VectorClock = Record<string, number>

/**
 * Lamport Timestamp - Simple logical clock
 */
export type LamportTimestamp = number

/**
 * Message Metadata for CRDT synchronization
 */
export interface MessageMetadata {
  id: string                    // Unique message ID (UUID)
  entityId: string              // Entity this message belongs to (contact/group/project/org/channel)
  entityType: 'person' | 'group' | 'project' | 'channel' | 'organisation'
  authorPeerId: string          // Four-word address of sender
  vectorClock: VectorClock      // Causal ordering information
  lamportClock: LamportTimestamp // Total ordering fallback
  timestamp: number             // Unix timestamp (wallclock time, for reference only)
  previousMessageId?: string    // Link to previous message in this entity's thread (causal parent)
  replyToId?: string           // If replying to a specific message
}

/**
 * Complete message with CRDT metadata
 */
export interface CRDTMessage {
  // Core message content
  content: {
    text: string
    author: string              // Display name
    attachments?: Array<{
      type: 'file' | 'image' | 'video'
      url: string
      name: string
      size: number
    }>
  }

  // CRDT metadata
  metadata: MessageMetadata

  // UI state (local only, not synced)
  localState?: {
    status?: 'sent' | 'delivered' | 'read'
    reactions?: Array<{
      emoji: string
      count: number
      userReacted?: boolean
      peerIds: string[]         // Which peers reacted
    }>
    threadCount?: number
    latestReplyBy?: string
  }
}

/**
 * Sync Request - Ask a peer for messages we're missing
 */
export interface SyncRequest {
  entityId: string              // Which entity to sync
  entityType: 'person' | 'group' | 'project' | 'channel' | 'organisation'
  requesterPeerId: string       // Who's asking
  vectorClock: VectorClock      // What we already have
  missingMessageIds?: string[]  // Specific messages we detected as missing
}

/**
 * Sync Response - Reply with messages
 */
export interface SyncResponse {
  entityId: string
  entityType: 'person' | 'group' | 'project' | 'channel' | 'organisation'
  messages: CRDTMessage[]       // All messages in causal order
  vectorClock: VectorClock      // Sender's current clock
}

/**
 * Entity Sync State - Track sync status per entity
 */
export interface EntitySyncState {
  entityId: string
  entityType: 'person' | 'group' | 'project' | 'channel' | 'organisation'
  vectorClock: VectorClock      // Our current knowledge
  lastSyncTime: number          // Last successful sync (wallclock)
  messageCount: number          // Total messages we have
  missingMessages: string[]     // Messages we know exist but don't have yet
  outOfOrderMessages: string[]  // Messages waiting for causal parents
}

/**
 * Compare vector clocks to determine causal relationship
 * Returns:
 *  - 'concurrent': Neither happened before the other (conflict)
 *  - 'before': a happened before b
 *  - 'after': b happened before a
 *  - 'equal': Same logical time
 */
export function compareVectorClocks(a: VectorClock, b: VectorClock): 'concurrent' | 'before' | 'after' | 'equal' {
  const allPeers = new Set([...Object.keys(a), ...Object.keys(b)])

  let aLess = false
  let bLess = false

  for (const peer of allPeers) {
    const aVal = a[peer] ?? 0
    const bVal = b[peer] ?? 0

    if (aVal < bVal) bLess = true
    if (aVal > bVal) aLess = true
  }

  if (aLess && bLess) return 'concurrent'
  if (aLess) return 'after'  // a is after b
  if (bLess) return 'before' // a is before b
  return 'equal'
}

/**
 * Merge vector clocks (take max for each peer)
 */
export function mergeVectorClocks(a: VectorClock, b: VectorClock): VectorClock {
  const allPeers = new Set([...Object.keys(a), ...Object.keys(b)])
  const merged: VectorClock = {}

  for (const peer of allPeers) {
    merged[peer] = Math.max(a[peer] ?? 0, b[peer] ?? 0)
  }

  return merged
}

/**
 * Increment our clock for a new event
 */
export function incrementVectorClock(clock: VectorClock, peerId: string): VectorClock {
  return {
    ...clock,
    [peerId]: (clock[peerId] ?? 0) + 1,
  }
}

/**
 * Check if we have all causal dependencies for a message
 */
export function hasCausalDependencies(
  message: CRDTMessage,
  localClock: VectorClock
): boolean {
  const messageClock = message.metadata.vectorClock

  // Check if we've seen all events that causally precede this message
  for (const [peer, timestamp] of Object.entries(messageClock)) {
    const ourTimestamp = localClock[peer] ?? 0

    // If message has timestamp N for peer P, we must have seen timestamps 0..N-1
    if (ourTimestamp < timestamp - 1) {
      return false // We're missing events from this peer
    }
  }

  return true
}

/**
 * Get missing message IDs by comparing vector clocks
 */
export function getMissingMessageRange(
  localClock: VectorClock,
  remoteClock: VectorClock
): { peerId: string; fromTimestamp: number; toTimestamp: number }[] {
  const missing: { peerId: string; fromTimestamp: number; toTimestamp: number }[] = []

  for (const [peerId, remoteTimestamp] of Object.entries(remoteClock)) {
    const localTimestamp = localClock[peerId] ?? 0

    if (remoteTimestamp > localTimestamp) {
      missing.push({
        peerId,
        fromTimestamp: localTimestamp + 1,
        toTimestamp: remoteTimestamp,
      })
    }
  }

  return missing
}

/**
 * Sort messages in causal order
 * Uses vector clocks first, falls back to Lamport clock for concurrent messages
 */
export function sortMessagesCausally(messages: CRDTMessage[]): CRDTMessage[] {
  return messages.sort((a, b) => {
    const comparison = compareVectorClocks(
      a.metadata.vectorClock,
      b.metadata.vectorClock
    )

    if (comparison === 'before') return -1
    if (comparison === 'after') return 1

    // Concurrent or equal - use Lamport clock as tiebreaker
    if (a.metadata.lamportClock !== b.metadata.lamportClock) {
      return a.metadata.lamportClock - b.metadata.lamportClock
    }

    // Still tied - use message ID for deterministic ordering
    return a.metadata.id.localeCompare(b.metadata.id)
  })
}
