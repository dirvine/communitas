import { describe, test, expect, beforeAll, afterAll } from 'vitest'
// Heavy integration; skip in unit env and handle in Sprint 2
describe.skip('Storage Pipeline Integration', () => {})
import { StoragePipeline } from '../../storagePipeline'
import { YjsMarkdownEditor } from '../../yjsCollaboration'
import { ReedSolomonEncoder } from '../../reedSolomon'
import { DHTStorage } from '../../dhtStorage'
import { MarkdownWebPublisher } from '../../markdownPublisher'
import { FourWordIdentity } from '../../../../types/collaboration'

describe('Storage Pipeline Integration', () => {
  let pipeline: StoragePipeline
  let groupMembers: FourWordIdentity[]
  
  beforeAll(async () => {
    // Set up group with 10 members
    groupMembers = []
    for (let i = 0; i < 10; i++) {
      groupMembers.push({
        fourWords: `user${i}-ocean-forest-star`,
        publicKey: `pk_user_${i}`,
        dhtAddress: `dht://user${i}-ocean-forest-star`
      })
    }
    
    pipeline = new StoragePipeline({
      groupMembers,
      reedSolomonConfig: {
        dataShards: 10,
        parityShards: 6
      },
      dhtBootstrapNodes: ['localhost:5001', 'localhost:5002', 'localhost:5003']
    })
    
    await pipeline.initialize()
  })

  afterAll(async () => {
    await pipeline.shutdown()
  })

  test('should handle complete file upload flow', async () => {
    // Given: User uploads 50MB file to group storage
    const fileSize = 50 * 1024 * 1024
    const fileData = new Uint8Array(fileSize)
    for (let i = 0; i < fileSize; i++) {
      fileData[i] = i % 256
    }
    
    const file = {
      name: 'large-document.pdf',
      data: fileData,
      mimeType: 'application/pdf'
    }
    
    // When: Process through pipeline
    const result = await pipeline.uploadFile(file, groupMembers[0])
    
    // Then:
    // 1. File chunked appropriately
    expect(result.chunks).toBeDefined()
    expect(result.chunks.length).toBeGreaterThan(1)
    
    // 2. Reed-Solomon encoding applied
    expect(result.shards).toBeDefined()
    expect(result.shards.length).toBe(16) // 10 data + 6 parity
    
    // 3. Shards encrypted
    expect(result.encryptedShards).toBeDefined()
    result.encryptedShards.forEach(shard => {
      expect(shard.encryptedData).toBeDefined()
      expect(shard.nonce).toHaveLength(12)
    })
    
    // 4. Distributed to group members
    expect(result.distribution).toBeDefined()
    expect(Object.keys(result.distribution)).toHaveLength(10)
    
    // Each member should have ~1.6 shards on average
    Object.values(result.distribution).forEach(memberShards => {
      expect(memberShards.length).toBeGreaterThanOrEqual(1)
      expect(memberShards.length).toBeLessThanOrEqual(3)
    })
    
    // 5. DHT entries created
    expect(result.dhtEntries).toBeDefined()
    expect(result.dhtEntries.length).toBeGreaterThan(0)
    
    // 6. Manifest updated
    expect(result.manifest).toBeDefined()
    expect(result.manifest.fileId).toBe(result.fileId)
    expect(result.manifest.fileName).toBe('large-document.pdf')
    expect(result.manifest.shardLocations).toHaveLength(16)
  })

  test('should recover from partial node failure', async () => {
    // Given: File distributed across 10 nodes
    const fileData = new Uint8Array(10 * 1024 * 1024) // 10MB
    const file = {
      name: 'important.md',
      data: fileData,
      mimeType: 'text/markdown'
    }
    
    const uploadResult = await pipeline.uploadFile(file, groupMembers[0])
    
    // When: 4 nodes go offline
    const failedNodes = groupMembers.slice(0, 4)
    await pipeline.simulateNodeFailures(failedNodes)
    
    // Then:
    // 1. File still retrievable
    const retrieved = await pipeline.downloadFile(uploadResult.fileId)
    expect(retrieved.data).toEqual(fileData)
    
    // 2. Self-healing initiates
    const healingStatus = await pipeline.getHealingStatus()
    expect(healingStatus.inProgress).toBe(true)
    expect(healingStatus.targetReplication).toBe(16)
    
    // Wait for healing to complete
    await pipeline.waitForHealing()
    
    // 3. New shards created on available nodes
    const newDistribution = await pipeline.getShardDistribution(uploadResult.fileId)
    const activeNodes = groupMembers.slice(4)
    
    activeNodes.forEach(node => {
      const nodeShards = newDistribution[node.fourWords]
      expect(nodeShards.length).toBeGreaterThanOrEqual(2) // Higher load on remaining nodes
    })
  })

  test('should sync Yjs changes to DHT backup', async () => {
    // Given: Active Yjs editing session
    const editor = new YjsMarkdownEditor('user1', 'test-doc')
    await editor.connect()
    
    pipeline.attachYjsEditor(editor)
    
    // Make some edits
    editor.insertText(0, '# Document Title\n\nThis is content.')
    
    // When: Checkpoint triggered
    const checkpoint = await pipeline.createYjsCheckpoint(editor)
    
    // Then:
    // 1. Yjs snapshot created
    expect(checkpoint.snapshot).toBeDefined()
    expect(checkpoint.snapshot.version).toBeGreaterThan(0)
    
    // 2. Snapshot encrypted
    expect(checkpoint.encryptedSnapshot).toBeDefined()
    expect(checkpoint.encryptedSnapshot).not.toContain('Document Title')
    
    // 3. Stored in DHT
    expect(checkpoint.dhtBlockId).toBeDefined()
    
    // 4. Retrievable by other nodes
    const retrieved = await pipeline.retrieveYjsCheckpoint(checkpoint.checkpointId)
    expect(retrieved.content).toBe('# Document Title\n\nThis is content.')
  })

  test('should handle concurrent file operations', async () => {
    // Multiple users uploading files simultaneously
    const uploads = []
    
    for (let i = 0; i < 5; i++) {
      const file = {
        name: `file${i}.txt`,
        data: new Uint8Array(1024 * 1024), // 1MB each
        mimeType: 'text/plain'
      }
      uploads.push(pipeline.uploadFile(file, groupMembers[i]))
    }
    
    const results = await Promise.all(uploads)
    
    // All uploads should succeed
    results.forEach((result, index) => {
      expect(result.success).toBe(true)
      expect(result.manifest.fileName).toBe(`file${index}.txt`)
    })
    
    // No conflicts in shard distribution
    const allShardIds = new Set()
    results.forEach(result => {
      result.shards.forEach(shard => {
        expect(allShardIds.has(shard.id)).toBe(false)
        allShardIds.add(shard.id)
      })
    })
  })

  test('should maintain consistency during network partition', async () => {
    // Create network partition
    const partition1 = groupMembers.slice(0, 5)
    const partition2 = groupMembers.slice(5)
    
    await pipeline.createNetworkPartition(partition1, partition2)
    
    // Upload file in partition 1
    const file1 = {
      name: 'partition1.md',
      data: new Uint8Array(1024),
      mimeType: 'text/markdown'
    }
    const result1 = await pipeline.uploadFile(file1, partition1[0], { partition: 1 })
    
    // Upload file in partition 2
    const file2 = {
      name: 'partition2.md',
      data: new Uint8Array(1024),
      mimeType: 'text/markdown'
    }
    const result2 = await pipeline.uploadFile(file2, partition2[0], { partition: 2 })
    
    // Heal partition
    await pipeline.healNetworkPartition()
    
    // Both files should be accessible from all nodes
    const file1FromPartition2 = await pipeline.downloadFile(result1.fileId, { 
      requestingNode: partition2[0] 
    })
    expect(file1FromPartition2).toBeDefined()
    
    const file2FromPartition1 = await pipeline.downloadFile(result2.fileId, {
      requestingNode: partition1[0]
    })
    expect(file2FromPartition1).toBeDefined()
  })

  test('should optimize storage for different group sizes', async () => {
    // Test with 2-person group (should use full replication)
    const twoPersonPipeline = new StoragePipeline({
      groupMembers: groupMembers.slice(0, 2),
      reedSolomonConfig: 'auto' // Should detect and use full replication
    })
    await twoPersonPipeline.initialize()
    
    const file = {
      name: 'shared.md',
      data: new Uint8Array(1024),
      mimeType: 'text/markdown'
    }
    
    const result = await twoPersonPipeline.uploadFile(file, groupMembers[0])
    
    // Should use full replication
    expect(result.storageMode).toBe('full-replication')
    expect(result.distribution[groupMembers[0].fourWords]).toHaveLength(1)
    expect(result.distribution[groupMembers[1].fourWords]).toHaveLength(1)
    
    // Both copies should be complete
    const copy1 = await twoPersonPipeline.downloadFile(result.fileId, {
      requestingNode: groupMembers[0]
    })
    const copy2 = await twoPersonPipeline.downloadFile(result.fileId, {
      requestingNode: groupMembers[1]
    })
    
    expect(copy1.data).toEqual(file.data)
    expect(copy2.data).toEqual(file.data)
    
    await twoPersonPipeline.shutdown()
  })

  test('should handle progressive file streaming', async () => {
    // Large file that should be streamed
    const largeFile = {
      name: 'video.mp4',
      data: new Uint8Array(100 * 1024 * 1024), // 100MB
      mimeType: 'video/mp4'
    }
    
    const uploadResult = await pipeline.uploadFile(largeFile, groupMembers[0])
    
    // Stream download
    const stream = await pipeline.streamFile(uploadResult.fileId)
    
    let receivedBytes = 0
    const chunks: Uint8Array[] = []
    
    stream.on('data', (chunk: Uint8Array) => {
      chunks.push(chunk)
      receivedBytes += chunk.length
      
      // Should receive data progressively
      expect(receivedBytes).toBeLessThanOrEqual(largeFile.data.length)
    })
    
    await new Promise<void>((resolve, reject) => {
      stream.on('end', () => {
        // Verify complete file received
        expect(receivedBytes).toBe(largeFile.data.length)
        
        // Concatenate chunks and verify
        const reconstructed = new Uint8Array(receivedBytes)
        let offset = 0
        chunks.forEach(chunk => {
          reconstructed.set(chunk, offset)
          offset += chunk.length
        })
        
        expect(reconstructed).toEqual(largeFile.data)
        resolve()
      })
      
      stream.on('error', reject)
    })
  })

  test('should maintain file integrity across operations', async () => {
    const file = {
      name: 'integrity-test.md',
      data: new Uint8Array(1024 * 1024),
      mimeType: 'text/markdown'
    }
    
    // Fill with non-random pattern for verification
    for (let i = 0; i < file.data.length; i++) {
      file.data[i] = (i * 7 + 13) % 256
    }
    
    // Upload
    const uploadResult = await pipeline.uploadFile(file, groupMembers[0])
    
    // Simulate various operations
    await pipeline.simulateNodeFailures(groupMembers.slice(0, 2))
    await pipeline.waitForHealing()
    
    // Download and verify
    const downloaded = await pipeline.downloadFile(uploadResult.fileId)
    
    // Verify every byte
    for (let i = 0; i < file.data.length; i++) {
      expect(downloaded.data[i]).toBe((i * 7 + 13) % 256)
    }
    
    // Verify checksum
    const checksum = await pipeline.computeChecksum(downloaded.data)
    expect(checksum).toBe(uploadResult.manifest.checksum)
  })
})