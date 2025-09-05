// Browser-compatible DHT Storage implementation
// Simplified for frontend use
import { NetworkIdentity } from '../../types/collaboration'
import { cryptoManager } from '../security/cryptoManager'

export interface DHTConfig {
  identity: NetworkIdentity
  bootstrapNodes: string[]
  replicationFactor: number
}

export interface EncryptedBlock {
  encryptedData: Uint8Array
  iv: Uint8Array
  authTag: Uint8Array
  signature: Uint8Array
  keyId: string
  publicKey: string
  metadata?: BlockMetadata
  // Test/compatibility helpers
  nonce?: Uint8Array
  __returnType?: 'buffer' | 'uint8array'
}

export interface BlockMetadata {
  size: number
  createdAt: number
  mimeType: string
  erasureIndex?: {
    shardIndex: number
    totalShards: number
  }
  forwardIdentity?: NetworkIdentity
}

export interface DHTNode {
  nodeId: string
  address: string
  publicKey: string
  isOnline: boolean
}

export interface QueryResult {
  blockId: string
  nodes: DHTNode[]
  metadata?: BlockMetadata
}

export class DHTStorage {
  private config: DHTConfig
  private encryptionKey: Uint8Array
  private keyId: string
  private isConnected = false
  private connectedNodes: Map<string, DHTNode> = new Map()
  private cryptoReady: Promise<void>
  private bootstrapNodes: string[]
  private replicationFactor: number
  private storedBlocks: Map<string, EncryptedBlock> = new Map()
  private metadataIndex: Map<string, BlockMetadata> = new Map()
  private identity: NetworkIdentity
  private eventListeners: Map<string, ((data: any) => void)[]> = new Map()

  constructor(config: DHTConfig) {
    this.config = config
    this.encryptionKey = new Uint8Array(32)
    this.keyId = config.identity.publicKey // Use the real public key from identity
    this.bootstrapNodes = config.bootstrapNodes
    this.replicationFactor = config.replicationFactor
    this.identity = config.identity
    this.cryptoReady = this.initialize()
    this.initialize()
  }

  private async initialize(): Promise<void> {
    // Simple initialization for browser compatibility
    crypto.getRandomValues(this.encryptionKey)
    console.log(`DHT initialized with key ID: ${this.keyId}`)
  }

  private async ensureReady(): Promise<void> {
    return this.cryptoReady
  }

  private async initializeCrypto(): Promise<void> {
    // SECURITY: Generate secure encryption key using Web Crypto API
    this.encryptionKey = new Uint8Array(32)
    crypto.getRandomValues(this.encryptionKey)

    // SECURITY: Generate key ID using Web Crypto API
    const hashBuffer = await crypto.subtle.digest('SHA-256', new Uint8Array(this.encryptionKey))
    const hashArray = new Uint8Array(hashBuffer)
    this.keyId = Array.from(hashArray.slice(0, 16))
      .map(b => b.toString(16).padStart(2, '0'))
      .join('')

    console.log(`DHT initialized with key ID: ${this.keyId}`)
  }

  async connect(): Promise<void> {
    await this.ensureReady()
    // Simulate connection to bootstrap nodes
    for (const nodeAddress of this.bootstrapNodes) {
      const node: DHTNode = {
        nodeId: await this.generateNodeId(nodeAddress),
        address: nodeAddress,
        publicKey: this.generatePublicKey(),
        isOnline: true
      }
      this.connectedNodes.set(node.nodeId, node)
    }
    // Ensure we have at least replicationFactor nodes available
    let index = 0
    while (this.connectedNodes.size < this.replicationFactor) {
      const addr = `sim://node-${index++}`
      const node: DHTNode = {
        nodeId: await this.generateNodeId(addr),
        address: addr,
        publicKey: this.generatePublicKey(),
        isOnline: true,
      }
      this.connectedNodes.set(node.nodeId, node)
    }
    
    this.isConnected = true
    console.log('DHT connected')
  }

  async disconnect(): Promise<void> {
    this.connectedNodes.clear()
    this.isConnected = false
    console.log('DHT disconnected')
  }

  /**
    * Generate a new encryption key using the PQC crypto manager
    */
  async generateEncryptionKey(): Promise<Uint8Array> {
    await this.ensureReady()

    // If we don't have a real key pair yet, generate one using PQC
    if (this.keyId === this.config.identity.publicKey) {
      // Generate a real PQC key pair using crypto manager
      const keyPair = await cryptoManager.generateKeyPair() // Now uses PQC ML-DSA-65
      this.keyId = keyPair.keyId
    }

    // Generate a secure encryption key
    const encryptionKey = new Uint8Array(32)
    crypto.getRandomValues(encryptionKey)

    return encryptionKey
  }

  // Core DHT operations
  async put(data: Uint8Array, encryptionKey?: Uint8Array): Promise<string> {
    if (!this.isConnected) {
      throw new Error('DHT not connected')
    }
    await this.ensureReady()
    
    const key = encryptionKey || this.encryptionKey
    const encrypted = await this.encrypt(data, key)
    const signed = await this.createSignedBlock(encrypted)
    const blockId = await this.computeHash(data)
    
    // Store locally and replicate
    this.storedBlocks.set(blockId, signed)
    await this.replicateToNodes(blockId, signed)
    
    return blockId
  }

  async get(blockId: string, decryptionKey?: Uint8Array): Promise<Uint8Array> {
    await this.ensureReady()
    const block = await this.retrieveBlock(blockId)
    if (!block) {
      throw new Error(`Block ${blockId} not found`)
    }
    
    // Verify signature
    const isValid = await this.verifySignature(block)
    if (!isValid) {
      throw new Error(`Block ${blockId} signature verification failed`)
    }
    
    const key = decryptionKey || this.encryptionKey
    return await this.decrypt(block, key)
  }

  async putWithMetadata(data: Uint8Array, metadata: BlockMetadata): Promise<string> {
    const blockId = await this.put(data)
    const block = this.storedBlocks.get(blockId)
    
    if (block) {
      block.metadata = metadata
      this.storedBlocks.set(blockId, block)
    }
    this.metadataIndex.set(blockId, metadata)
    
    return blockId
  }

  async getWithMetadata(blockId: string): Promise<{ data: Uint8Array; metadata?: BlockMetadata }> {
    const data = await this.get(blockId)
    const block = this.storedBlocks.get(blockId)
    
    return {
      data,
      metadata: block?.metadata
    }
  }

  // Node discovery
  async findNodes(blockId: string): Promise<DHTNode[]> {
    const ranked = Array.from(this.connectedNodes.entries())
      .map(([nodeId, node]) => ({ node, distance: this.computeDistance(blockId, nodeId) }))
      .sort((a, b) => a.distance - b.distance)
    return ranked.slice(0, this.replicationFactor).map(r => r.node)
  }

  // SECURITY: Secure encryption methods using cryptoManager
  async encrypt(data: Uint8Array, key?: Uint8Array): Promise<EncryptedBlock> {
    await this.ensureReady()
    const encKey = key || this.encryptionKey
    
    // Use secure crypto manager for encryption
    const encryptionResult = await cryptoManager.encryptData(data, encKey, this.keyId)
    
    // Create signature using crypto manager
    const dataToSign = new Uint8Array([
      ...encryptionResult.ciphertext,
      ...encryptionResult.iv,
      ...encryptionResult.authTag
    ])
    
    const signature = await cryptoManager.signData(dataToSign, this.keyId)
    const keyPair = cryptoManager.getKeyPair(this.keyId)
    
    if (!keyPair) {
      throw new Error('Key pair not found for encryption')
    }
    
    const result: EncryptedBlock = {
      encryptedData: encryptionResult.ciphertext,
      iv: encryptionResult.iv,
      authTag: encryptionResult.authTag,
      signature,
      keyId: this.keyId,
      publicKey: keyPair.publicKey
    }
    result.nonce = encryptionResult.iv
    // Track original input type to match test expectations on decrypt
    result.__returnType = Buffer.isBuffer(data) ? 'buffer' : 'uint8array'
    return result
  }

  async decrypt(block: EncryptedBlock, key?: Uint8Array): Promise<Uint8Array> {
    await this.ensureReady()
    const decKey = key || this.encryptionKey
    
    // Verify signature first
    const dataToVerify = new Uint8Array([
      ...block.encryptedData,
      ...block.iv,
      ...block.authTag
    ])
    
    const keyPair = cryptoManager.getKeyPair(block.keyId)
    if (!keyPair) {
      throw new Error('Key pair not found for signature verification')
    }
    
    const isValidSignature = await cryptoManager.verifySignature(
      dataToVerify,
      block.signature,
      keyPair.publicKey,
      keyPair.algorithm
    )
    
    if (!isValidSignature) {
      throw new Error('Block signature verification failed')
    }
    
    // Use secure crypto manager for decryption
    const encryptionResult = {
      ciphertext: block.encryptedData,
      iv: block.iv,
      authTag: block.authTag,
      keyId: block.keyId
    }
    
    const plain = await cryptoManager.decryptData(encryptionResult, decKey)
    // Return Buffer when original input to encrypt was a Buffer
    return (block as any).__returnType === 'buffer' ? Buffer.from(plain) : plain
  }

  async createSignedBlock(data: EncryptedBlock | Uint8Array): Promise<EncryptedBlock> {
    if (data instanceof Uint8Array) {
      return await this.encrypt(data)
    }
    return data
  }

  async verifySignature(block: EncryptedBlock): Promise<boolean> {
    try {
      const dataToVerify = new Uint8Array([
        ...block.encryptedData,
        ...block.iv,
        ...block.authTag
      ])
      
      const keyPair = cryptoManager.getKeyPair(block.keyId)
      if (!keyPair) {
        console.warn(`Key pair not found for verification: ${block.keyId}`)
        return false
      }
      
      return await cryptoManager.verifySignature(
        dataToVerify,
        block.signature,
        keyPair.publicKey,
        keyPair.algorithm
      )
    } catch (error) {
      console.error('Signature verification error:', error)
      return false
    }
  }

  // Utility methods

  async computeHash(data: Uint8Array): Promise<string> {
    const hashBuffer = await crypto.subtle.digest('SHA-256', data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) as ArrayBuffer)
    const hashArray = new Uint8Array(hashBuffer)
    return Array.from(hashArray, byte => byte.toString(16).padStart(2, '0')).join('')
  }

  private computeDistance(hash1: string, hash2: string): number {
    // XOR distance for DHT routing
    const buf1 = Buffer.from(hash1, 'hex')
    const buf2 = Buffer.from(hash2, 'hex')

    let distance = 0
    const minLength = Math.min(buf1.length, buf2.length)

    for (let i = 0; i < minLength; i++) {
      distance += this.hammingWeight(buf1[i] ^ buf2[i])
    }

    return distance
  }

  /**
   * Find blocks by MIME type
   */
  async findByMimeType(mimeType: string): Promise<EncryptedBlock[]> {
    const results: EncryptedBlock[] = []

    for (const [blockId, block] of this.storedBlocks.entries()) {
      const metadata = this.metadataIndex.get(blockId)
      if (metadata && metadata.mimeType === mimeType) {
        results.push(block)
      }
    }

    return results
  }

  private hammingWeight(n: number): number {
    let count = 0
    while (n) {
      count += n & 1
      n >>>= 1
    }
    return count
  }



  // Replication and self-healing
  private async replicateToNodes(blockId: string, block: EncryptedBlock): Promise<void> {
    const targetNodes = await this.findNodes(blockId)
    
    for (const node of targetNodes) {
      // Simulate replication to node
      console.log(`Replicating block ${blockId} to node ${node.nodeId}`)
    }
  }

  async simulateNodeFailure(nodeAddress: string): Promise<void> {
    for (const [nodeId, node] of this.connectedNodes) {
      if (node.address === nodeAddress) {
        node.isOnline = false
        this.emit('nodeFailure', node)
        break
      }
    }
  }

  async simulateNetworkPartition(partition1: string[], partition2: string[]): Promise<void> {
    // Simulate network partition by marking cross-partition connections as offline
    this.emit('networkPartition', { partition1, partition2 })
  }

  async healNetworkPartition(): Promise<void> {
    // Restore all connections
    for (const [, node] of this.connectedNodes) {
      node.isOnline = true
    }
    this.emit('networkHealed', {})
  }

  // Batch operations
  async putBatch(blocks: Array<{ data: Uint8Array; key: Uint8Array }>): Promise<string[]> {
    const blockIds: string[] = []
    
    const operations = blocks.map(async ({ data, key }) => {
      const blockId = await this.put(data, key)
      blockIds.push(blockId)
      return blockId
    })
    
    await Promise.all(operations)
    return blockIds
  }

  // Raw block access (for testing)
  async getRawBlock(blockId: string): Promise<EncryptedBlock> {
    const block = this.storedBlocks.get(blockId)
    if (!block) {
      throw new Error(`Block ${blockId} not found`)
    }
    return block
  }

  private async retrieveBlock(blockId: string): Promise<EncryptedBlock | null> {
    // Try local storage first
    let block = this.storedBlocks.get(blockId)
    if (block) {
      return block
    }
    
    // Try to retrieve from network nodes
    const nodes = await this.findNodes(blockId)
    for (const node of nodes) {
      if (node.isOnline) {
        // Simulate network retrieval
        block = this.storedBlocks.get(blockId)
        if (block) {
          return block
        }
      }
    }
    
    return null
  }

  private async generateNodeId(address: string): Promise<string> {
    const hashBuffer = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(address))
    const hashArray = new Uint8Array(hashBuffer)
    return Array.from(hashArray, byte => byte.toString(16).padStart(2, '0')).join('')
  }

  private generatePublicKey(): string {
    const key = new Uint8Array(32)
    crypto.getRandomValues(key)
    return Array.from(key, byte => byte.toString(16).padStart(2, '0')).join('')
  }

  private emit(event: string, data: any): void {
    const listeners = this.eventListeners.get(event)
    if (listeners) {
      listeners.forEach(listener => listener(data))
    }
  }

  // Performance and monitoring
  getStats(): any {
    return {
      connectedNodes: this.connectedNodes.size,
      storedBlocks: this.storedBlocks.size,
      replicationFactor: this.replicationFactor,
      isConnected: this.isConnected,
      identity: this.identity.fourWords
    }
  }

  // Testing utilities
  async waitForReplication(): Promise<void> {
    // Simulate waiting for replication to complete
    await new Promise(resolve => setTimeout(resolve, 100))
  }

  async clearStorage(): Promise<void> {
    this.storedBlocks.clear()
  }
}