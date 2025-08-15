import * as Y from 'yjs'
import { WebrtcProvider } from 'y-webrtc'
import { IndexeddbPersistence } from 'y-indexeddb'
import { Awareness } from 'y-protocols/awareness'

export interface YjsSnapshot {
  version: number
  data: Uint8Array
  timestamp: number
  checksum: string
}

export interface YjsUserInfo {
  name: string
  color: string
  cursor?: number
}

export interface YjsCollaborator {
  id: string
  name: string
  color: string
  isOnline: boolean
  cursor?: number
  selection?: { start: number; end: number }
}

export interface YjsDiff {
  additions: string[]
  deletions: string[]
  modifications: string[]
}

export class YjsMarkdownEditor {
  private doc: Y.Doc
  private text: Y.Text
  private webrtcProvider: WebrtcProvider
  private persistence: IndexeddbPersistence
  private awareness: Awareness
  private userId: string
  private roomId: string
  private userInfo: YjsUserInfo | null = null
  private isConnected = false

  constructor(userId: string, roomId: string) {
    this.userId = userId
    this.roomId = roomId
    
    this.doc = new Y.Doc()
    this.text = this.doc.getText('content')
    
    // Set up persistence
    this.persistence = new IndexeddbPersistence(roomId, this.doc)
    
    // Set up WebRTC provider for real-time collaboration
    this.webrtcProvider = new WebrtcProvider(roomId, this.doc, {
      signaling: ['ws://localhost:4444'],
      maxConns: 20,
      filterBcConns: false
    })
    
    this.awareness = this.webrtcProvider.awareness
    this.setupAwareness()
  }

  async connect(): Promise<void> {
    return new Promise((resolve) => {
      if (this.isConnected) {
        resolve()
        return
      }

      // Wait for initial sync
      const syncHandler = () => {
        this.isConnected = true
        this.doc.off('sync', syncHandler)
        resolve()
      }
      
      this.doc.on('sync', syncHandler)
      
      // Set timeout fallback
      setTimeout(() => {
        if (!this.isConnected) {
          this.isConnected = true
          this.doc.off('sync', syncHandler)
          resolve()
        }
      }, 1000)
    })
  }

  async destroy(): Promise<void> {
    this.webrtcProvider.destroy()
    this.persistence.destroy()
    this.doc.destroy()
    this.isConnected = false
  }

  // Text manipulation methods
  insertText(position: number, text: string): void {
    this.doc.transact(() => {
      this.text.insert(position, text)
    })
    this.updateCursorPosition(position + text.length)
  }

  deleteText(position: number, length: number): void {
    this.doc.transact(() => {
      this.text.delete(position, length)
    })
    this.updateCursorPosition(position)
  }

  replaceText(oldText: string, newText: string): void {
    const content = this.text.toString()
    const index = content.indexOf(oldText)
    
    if (index !== -1) {
      this.doc.transact(() => {
        this.text.delete(index, oldText.length)
        this.text.insert(index, newText)
      })
      this.updateCursorPosition(index + newText.length)
    }
  }

  getContent(): string {
    return this.text.toString()
  }

  // Cursor and selection management
  setCursorPosition(position: number): void {
    this.updateCursorPosition(position)
  }

  getCursorPosition(): number {
    const localState = this.awareness.getLocalState()
    return localState?.cursor || 0
  }

  setSelection(start: number, end: number): void {
    this.awareness.setLocalStateField('selection', { start, end })
  }

  getSelection(): { start: number; end: number } | null {
    const localState = this.awareness.getLocalState()
    return localState?.selection || null
  }

  private updateCursorPosition(position: number): void {
    this.awareness.setLocalStateField('cursor', position)
  }

  // User management and awareness
  setUserInfo(userInfo: YjsUserInfo): void {
    this.userInfo = userInfo
    this.awareness.setLocalStateField('user', {
      name: userInfo.name,
      color: userInfo.color,
      id: this.userId
    })
  }

  getOnlineUsers(): YjsCollaborator[] {
    const users: YjsCollaborator[] = []
    
    this.awareness.getStates().forEach((state: any, clientId: number) => {
      if (state.user) {
        users.push({
          id: state.user.id || clientId.toString(),
          name: state.user.name || 'Unknown',
          color: state.user.color || '#333333',
          isOnline: true,
          cursor: state.cursor,
          selection: state.selection
        })
      }
    })
    
    return users
  }

  private setupAwareness(): void {
    this.awareness.on('change', ({ added, removed }: { added: number[], updated: number[], removed: number[] }) => {
      // Handle user awareness changes
      for (const clientId of added) {
        const state = this.awareness.getStates().get(clientId)
        console.log(`User joined: ${state?.user?.name || clientId}`)
      }
      
      for (const clientId of removed) {
        console.log(`User left: ${clientId}`)
      }
    })
  }

  // Snapshots and history
  async createSnapshot(): Promise<YjsSnapshot> {
    const stateVector = Y.encodeStateVector(this.doc)
    const docUpdate = Y.encodeStateAsUpdate(this.doc, stateVector)
    
    const snapshot: YjsSnapshot = {
      version: this.doc.clientID,
      data: docUpdate,
      timestamp: Date.now(),
      checksum: this.computeChecksum(docUpdate)
    }
    
    return snapshot
  }

  async restoreSnapshot(snapshot: YjsSnapshot): Promise<void> {
    // Verify checksum
    const computedChecksum = this.computeChecksum(snapshot.data)
    if (computedChecksum !== snapshot.checksum) {
      throw new Error('Snapshot checksum verification failed')
    }
    
    // Create new document from snapshot
    const newDoc = new Y.Doc()
    Y.applyUpdate(newDoc, snapshot.data)
    
    // Replace current document content
    this.doc.transact(() => {
      this.text.delete(0, this.text.length)
      this.text.insert(0, newDoc.getText('content').toString())
    })
  }

  async diff(snapshot1: YjsSnapshot, snapshot2: YjsSnapshot): Promise<YjsDiff> {
    // Create temporary documents for comparison
    const doc1 = new Y.Doc()
    const doc2 = new Y.Doc()
    
    Y.applyUpdate(doc1, snapshot1.data)
    Y.applyUpdate(doc2, snapshot2.data)
    
    const content1 = doc1.getText('content').toString()
    const content2 = doc2.getText('content').toString()
    
    // Simple line-based diff (in production, use a proper diff algorithm)
    const lines1 = content1.split('\n')
    const lines2 = content2.split('\n')
    
    const additions: string[] = []
    const deletions: string[] = []
    const modifications: string[] = []
    
    const maxLength = Math.max(lines1.length, lines2.length)
    
    for (let i = 0; i < maxLength; i++) {
      const line1 = lines1[i]
      const line2 = lines2[i]
      
      if (line1 === undefined) {
        additions.push(line2)
      } else if (line2 === undefined) {
        deletions.push(line1)
      } else if (line1 !== line2) {
        modifications.push(`-${line1}`)
        modifications.push(`+${line2}`)
      }
    }
    
    return { additions, deletions, modifications }
  }

  // Synchronization methods for testing
  async syncWithPeer(peer: YjsMarkdownEditor): Promise<void> {
    // Exchange state vectors and updates
    const myStateVector = Y.encodeStateVector(this.doc)
    const peerStateVector = Y.encodeStateVector(peer.doc)
    
    const myUpdate = Y.encodeStateAsUpdate(this.doc, peerStateVector)
    const peerUpdate = Y.encodeStateAsUpdate(peer.doc, myStateVector)
    
    // Apply updates
    if (myUpdate.length > 0) {
      Y.applyUpdate(peer.doc, myUpdate)
    }
    
    if (peerUpdate.length > 0) {
      Y.applyUpdate(this.doc, peerUpdate)
    }
    
    // Sync awareness states
    const myAwarenessState = this.awareness.getLocalState()
    const peerAwarenessState = peer.awareness.getLocalState()
    
    if (myAwarenessState) {
      peer.awareness.setLocalState(myAwarenessState)
    }
    
    if (peerAwarenessState) {
      this.awareness.setLocalState(peerAwarenessState)
    }
  }

  // Utility methods
  private computeChecksum(data: Uint8Array): string {
    // Simple checksum for demo - in production use crypto.createHash
    let checksum = 0
    for (let i = 0; i < data.length; i++) {
      checksum = (checksum + data[i]) % 256
    }
    return checksum.toString(16).padStart(2, '0')
  }

  // Real-time collaboration events
  onContentChange(callback: (content: string) => void): () => void {
    const handler = () => {
      callback(this.text.toString())
    }
    
    this.text.observe(handler)
    
    // Return unsubscribe function
    return () => {
      this.text.unobserve(handler)
    }
  }

  onUserJoin(callback: (user: YjsCollaborator) => void): () => void {
    const handler = ({ added }: any) => {
      for (const clientId of added) {
        const state = this.awareness.getStates().get(clientId)
        if (state?.user) {
          callback({
            id: state.user.id || clientId.toString(),
            name: state.user.name || 'Unknown',
            color: state.user.color || '#333333',
            isOnline: true,
            cursor: state.cursor,
            selection: state.selection
          })
        }
      }
    }
    
    this.awareness.on('change', handler)
    
    return () => {
      this.awareness.off('change', handler)
    }
  }

  onUserLeave(callback: (userId: string) => void): () => void {
    const handler = ({ removed }: any) => {
      for (const clientId of removed) {
        callback(clientId.toString())
      }
    }
    
    this.awareness.on('change', handler)
    
    return () => {
      this.awareness.off('change', handler)
    }
  }

  // Performance monitoring
  getStats(): any {
    return {
      docSize: Y.encodeStateAsUpdate(this.doc).length,
      clientsConnected: this.awareness.getStates().size,
      textLength: this.text.length,
      transactionCount: (this.doc as any)._transactionCleanups?.size || 0,
      isConnected: this.isConnected,
      roomId: this.roomId,
      userId: this.userId
    }
  }

  // Advanced collaboration features
  async inviteCollaborator(userId: string): Promise<void> {
    // In a real implementation, this would send an invitation
    // For now, just simulate the invitation
    console.log(`Inviting user ${userId} to collaborate on room ${this.roomId}`)
  }

  async save(): Promise<void> {
    // Force persistence to IndexedDB
    await this.persistence.whenSynced
  }

  // Network simulation for testing
  async simulateNetworkDelay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms))
  }

  async simulateDisconnection(): Promise<void> {
    this.webrtcProvider.disconnect()
    this.isConnected = false
  }

  async simulateReconnection(): Promise<void> {
    this.webrtcProvider.connect()
    await this.connect()
  }
}