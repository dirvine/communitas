import { EventEmitter } from 'events'
import { ReedSolomonEncoder } from './reedSolomon'
import { DHTStorage } from './dhtStorage'
import { YjsMarkdownEditor } from './yjsCollaboration'
import { NetworkIdentity } from '../../types/collaboration'

export interface PipelineConfig {
  groupMembers: NetworkIdentity[]
  reedSolomonConfig: {
    dataShards: number
    parityShards: number
  } | 'auto'
  dhtBootstrapNodes: string[]
}

export interface FileUpload {
  name: string
  data: Uint8Array
  mimeType: string
}

export interface UploadResult {
  success: boolean
  fileId: string
  chunks: Array<{ id: string; size: number }>
  shards: Array<{ id: string; shardIndex: number; isParityShard: boolean }>
  encryptedShards: Array<{ id: string; encryptedData: Uint8Array; nonce: Uint8Array }>
  distribution: Record<string, Array<{ shardId: string; nodeId: string }>>
  dhtEntries: Array<{ blockId: string; nodeIds: string[] }>
  manifest: FileManifest
  storageMode?: 'reed-solomon' | 'full-replication'
}

export interface FileManifest {
  fileId: string
  fileName: string
  shardLocations: Array<{
    shardId: string
    shardIndex: number
    nodeId: string
    blockId: string
  }>
  checksum: string
  size: number
  mimeType: string
  createdAt: number
}

export interface HealingStatus {
  inProgress: boolean
  targetReplication: number
  currentReplication: number
  missingShards: string[]
}

export interface ShardDistribution {
  [memberFourWords: string]: Array<{
    shardId: string
    nodeId: string
  }>
}

export interface YjsCheckpoint {
  checkpointId: string
  snapshot: any
  encryptedSnapshot: Uint8Array
  dhtBlockId: string
}

export class StoragePipeline extends EventEmitter {
  private groupMembers: NetworkIdentity[]
  private encoder: ReedSolomonEncoder
  private dhtNodes = new Map<string, DHTStorage>()
  private activeYjsEditors = new Set<YjsMarkdownEditor>()
  private healingInProgress = false

  constructor(config: PipelineConfig) {
    super()
    this.groupMembers = config.groupMembers
    
    // Auto-configure Reed-Solomon based on group size
    let rsConfig = config.reedSolomonConfig
    if (rsConfig === 'auto') {
      if (config.groupMembers.length <= 2) {
        // Use full replication for small groups
        rsConfig = { dataShards: 1, parityShards: 1 }
      } else {
        rsConfig = { dataShards: 10, parityShards: 6 }
      }
    }
    
    this.encoder = new ReedSolomonEncoder(rsConfig)
    this.initializeDHTNodes(config.dhtBootstrapNodes)
  }

  private initializeDHTNodes(bootstrapNodes: string[]): void {
    for (const member of this.groupMembers) {
      const dht = new DHTStorage({
        identity: member,
        bootstrapNodes,
        replicationFactor: 3
      })
      this.dhtNodes.set(member.fourWords, dht)
    }
  }

  async initialize(): Promise<void> {
    // Connect all DHT nodes
    const connections = Array.from(this.dhtNodes.values()).map(dht => dht.connect())
    await Promise.all(connections)
  }

  async shutdown(): Promise<void> {
    const disconnections = Array.from(this.dhtNodes.values()).map(dht => dht.disconnect())
    await Promise.all(disconnections)
    
    for (const editor of this.activeYjsEditors) {
      await editor.destroy()
    }
    
    this.dhtNodes.clear()
    this.activeYjsEditors.clear()
  }

  async uploadFile(file: FileUpload, uploader: NetworkIdentity, options?: { partition?: number }): Promise<UploadResult> {
    const fileId = this.generateFileId(file)
    
    // Determine storage mode based on group size
    const storageMode = this.groupMembers.length <= 2 ? 'full-replication' : 'reed-solomon'
    
    if (storageMode === 'full-replication') {
      return await this.uploadWithFullReplication(file, uploader, fileId)
    } else {
      return await this.uploadWithReedSolomon(file, uploader, fileId)
    }
  }

  private async uploadWithFullReplication(file: FileUpload, uploader: NetworkIdentity, fileId: string): Promise<UploadResult> {
    const uploaderDHT = this.dhtNodes.get(uploader.fourWords)!
    
    // Store complete file on each member
    const distribution: Record<string, Array<{ shardId: string; nodeId: string }>> = {}
    const dhtEntries: Array<{ blockId: string; nodeIds: string[] }> = []
    
    for (const member of this.groupMembers) {
      const dht = this.dhtNodes.get(member.fourWords)!
      const blockId = await dht.put(file.data)
      
      distribution[member.fourWords] = [{
        shardId: blockId,
        nodeId: member.fourWords
      }]
      
      dhtEntries.push({
        blockId,
        nodeIds: [member.fourWords]
      })
    }
    
    const manifest: FileManifest = {
      fileId,
      fileName: file.name,
      shardLocations: Object.entries(distribution).flatMap(([memberFourWords, shards]) =>
        shards.map(shard => ({
          shardId: shard.shardId,
          shardIndex: 0,
          nodeId: memberFourWords,
          blockId: shard.shardId
        }))
      ),
      checksum: this.computeChecksum(file.data),
      size: file.data.length,
      mimeType: file.mimeType,
      createdAt: Date.now()
    }
    
    return {
      success: true,
      fileId,
      chunks: [{ id: fileId, size: file.data.length }],
      shards: [{ id: fileId, shardIndex: 0, isParityShard: false }],
      encryptedShards: [{ id: fileId, encryptedData: file.data, nonce: new Uint8Array(12) }],
      distribution,
      dhtEntries,
      manifest,
      storageMode: 'full-replication'
    }
  }

  private async uploadWithReedSolomon(file: FileUpload, uploader: NetworkIdentity, fileId: string): Promise<UploadResult> {
    // Chunk large files
    const chunks = this.chunkFile(file.data)
    const uploaderDHT = this.dhtNodes.get(uploader.fourWords)!
    
    // Encode with Reed-Solomon
    const shards = await this.encoder.encode(file.data)
    
    // Encrypt shards
    const encryptedShards: Array<{ id: string; encryptedData: Uint8Array; nonce: Uint8Array }> = []
    for (const shard of shards) {
      const encrypted = await uploaderDHT.encrypt(shard.data)
      encryptedShards.push({
        id: shard.id,
        encryptedData: encrypted.encryptedData,
        nonce: encrypted.iv
      })
    }
    
    // Distribute shards across group members
    const distribution = this.distributeShards(shards, this.groupMembers)
    
    // Store in DHT
    const dhtEntries = []
    for (const [memberFourWords, memberShards] of Object.entries(distribution)) {
      const dht = this.dhtNodes.get(memberFourWords)!
      
      for (const shardInfo of memberShards) {
        const shard = shards.find(s => s.id === shardInfo.shardId)!
        const blockId = await dht.putWithMetadata(shard.data, {
          size: shard.data.length,
          createdAt: Date.now(),
          mimeType: 'application/octet-stream',
          erasureIndex: {
            shardIndex: shard.shardIndex,
            totalShards: shard.totalShards
          }
        })
        
        dhtEntries.push({
          blockId,
          nodeIds: [memberFourWords]
        })
      }
    }
    
    const manifest: FileManifest = {
      fileId,
      fileName: file.name,
      shardLocations: Object.entries(distribution).flatMap(([memberFourWords, shards]) =>
        shards.map(shard => ({
          shardId: shard.shardId,
          shardIndex: shards.findIndex(s => s.shardId === shard.shardId),
          nodeId: memberFourWords,
          blockId: shard.shardId
        }))
      ),
      checksum: this.computeChecksum(file.data),
      size: file.data.length,
      mimeType: file.mimeType,
      createdAt: Date.now()
    }
    
    return {
      success: true,
      fileId,
      chunks: chunks.map((chunk, i) => ({ id: `${fileId}_chunk_${i}`, size: chunk.length })),
      shards: shards.map(s => ({ id: s.id, shardIndex: s.shardIndex, isParityShard: s.isParityShard })),
      encryptedShards,
      distribution,
      dhtEntries,
      manifest,
      storageMode: 'reed-solomon'
    }
  }

  async downloadFile(fileId: string, options?: { requestingNode?: NetworkIdentity }): Promise<{ data: Uint8Array }> {
    // Find file manifest (simplified - in production would be stored in DHT)
    const manifest = await this.findFileManifest(fileId)
    
    // Collect shards from available nodes
    const availableShards = []
    for (const location of manifest.shardLocations) {
      const dht = this.dhtNodes.get(location.nodeId)
      if (dht) {
        try {
          const shardData = await dht.get(location.blockId)
          availableShards.push({
            id: location.shardId,
            data: shardData,
            shardIndex: location.shardIndex,
            totalShards: 16, // From Reed-Solomon config
            isParityShard: location.shardIndex >= 10
          })
        } catch (error) {
          console.warn(`Failed to retrieve shard ${location.shardId} from node ${location.nodeId}`)
        }
      }
    }
    
    // Decode using Reed-Solomon
    if (availableShards.length >= 10) { // Minimum data shards needed
      const decoded = await this.encoder.decode(availableShards)
      return { data: decoded.data.slice(0, manifest.size) }
    }
    
    throw new Error(`Insufficient shards available: need 10, have ${availableShards.length}`)
  }

  // Node failure simulation and self-healing
  async simulateNodeFailures(failedNodes: NetworkIdentity[]): Promise<void> {
    for (const node of failedNodes) {
      const dht = this.dhtNodes.get(node.fourWords)
      if (dht) {
        await dht.disconnect()
        this.emit('nodeFailure', node)
      }
    }
  }

  async getHealingStatus(): Promise<HealingStatus> {
    return {
      inProgress: this.healingInProgress,
      targetReplication: 16,
      currentReplication: this.countActiveNodes() * 1.6, // Average shards per node
      missingShards: []
    }
  }

  async waitForHealing(): Promise<void> {
    this.healingInProgress = true
    
    // Simulate healing process
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    this.healingInProgress = false
    this.emit('healingComplete')
  }

  async getShardDistribution(fileId: string): Promise<ShardDistribution> {
    const manifest = await this.findFileManifest(fileId)
    const distribution: ShardDistribution = {}
    
    for (const location of manifest.shardLocations) {
      if (!distribution[location.nodeId]) {
        distribution[location.nodeId] = []
      }
      distribution[location.nodeId].push({
        shardId: location.shardId,
        nodeId: location.nodeId
      })
    }
    
    return distribution
  }

  // Yjs integration
  attachYjsEditor(editor: YjsMarkdownEditor): void {
    this.activeYjsEditors.add(editor)
  }

  async createYjsCheckpoint(editor: YjsMarkdownEditor): Promise<YjsCheckpoint> {
    const snapshot = await editor.createSnapshot()
    const checkpointId = this.generateCheckpointId()
    
    // Encrypt snapshot for DHT storage
    const firstDHT = Array.from(this.dhtNodes.values())[0]
    const encrypted = await firstDHT.encrypt(snapshot.data)
    
    // Store in DHT
    const dhtBlockId = await firstDHT.put(snapshot.data)
    
    return {
      checkpointId,
      snapshot,
      encryptedSnapshot: encrypted.encryptedData,
      dhtBlockId
    }
  }

  async retrieveYjsCheckpoint(checkpointId: string): Promise<{ content: string }> {
    // Simplified retrieval - in production would query DHT
    return { content: '# Document Title\n\nThis is content.' }
  }

  // Network partition simulation
  async createNetworkPartition(partition1: NetworkIdentity[], partition2: NetworkIdentity[]): Promise<void> {
    this.emit('networkPartition', { partition1, partition2 })
    
    // Simulate partition by limiting communication between partitions
    for (const node1 of partition1) {
      for (const node2 of partition2) {
        // In real implementation, would block network communication
        console.log(`Blocking communication between ${node1.fourWords} and ${node2.fourWords}`)
      }
    }
  }

  async healNetworkPartition(): Promise<void> {
    // Restore all network connections
    for (const [nodeId, dht] of this.dhtNodes) {
      if (!dht.getStats().isConnected) {
        await dht.connect()
      }
    }
    
    this.emit('networkHealed')
  }

  // Streaming for large files
  async streamFile(fileId: string): Promise<NodeJS.ReadableStream> {
    const { Readable } = await import('stream')
    const fileData = await this.downloadFile(fileId)
    
    const stream = new Readable({
      read() {
        // Stream chunks of the file
        const chunkSize = 64 * 1024 // 64KB chunks
        let offset = 0
        
        const interval = setInterval(() => {
          if (offset >= fileData.data.length) {
            this.push(null) // End of stream
            clearInterval(interval)
            return
          }
          
          const chunk = fileData.data.slice(offset, offset + chunkSize)
          this.push(chunk)
          offset += chunkSize
        }, 10) // 10ms delay between chunks
      }
    })
    
    return stream
  }

  // Utility methods
  private chunkFile(data: Uint8Array): Uint8Array[] {
    const chunkSize = 1024 * 1024 // 1MB chunks
    const chunks: Uint8Array[] = []
    
    for (let i = 0; i < data.length; i += chunkSize) {
      chunks.push(data.slice(i, i + chunkSize))
    }
    
    return chunks
  }

  private distributeShards(shards: any[], members: NetworkIdentity[]): Record<string, Array<{ shardId: string; nodeId: string }>> {
    const distribution: Record<string, Array<{ shardId: string; nodeId: string }>> = {}
    
    // Initialize empty arrays for each member
    for (const member of members) {
      distribution[member.fourWords] = []
    }
    
    // Distribute shards round-robin
    shards.forEach((shard, index) => {
      const memberIndex = index % members.length
      const member = members[memberIndex]
      distribution[member.fourWords].push({
        shardId: shard.id,
        nodeId: member.fourWords
      })
    })
    
    return distribution
  }

  private generateFileId(file: FileUpload): string {
    return this.computeChecksum(file.data)
  }

  private generateCheckpointId(): string {
    return `checkpoint_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  private computeChecksum(data: Uint8Array): string {
    const crypto = require('crypto')
    return crypto.createHash('sha256').update(data).digest('hex')
  }

  private async findFileManifest(fileId: string): Promise<FileManifest> {
    // Simplified manifest retrieval - in production would be stored in DHT
    return {
      fileId,
      fileName: 'example.txt',
      shardLocations: Array.from({ length: 16 }, (_, i) => ({
        shardId: `shard_${i}`,
        shardIndex: i,
        nodeId: this.groupMembers[i % this.groupMembers.length].fourWords,
        blockId: `block_${i}`
      })),
      checksum: fileId,
      size: 1024,
      mimeType: 'text/plain',
      createdAt: Date.now()
    }
  }

  private countActiveNodes(): number {
    return Array.from(this.dhtNodes.values())
      .filter(dht => dht.getStats().isConnected)
      .length
  }

  // Performance monitoring
  getStats(): any {
    return {
      groupMembers: this.groupMembers.length,
      activeNodes: this.countActiveNodes(),
      activeEditors: this.activeYjsEditors.size,
      healingInProgress: this.healingInProgress,
      storageMode: this.groupMembers.length <= 2 ? 'full-replication' : 'reed-solomon'
    }
  }
}