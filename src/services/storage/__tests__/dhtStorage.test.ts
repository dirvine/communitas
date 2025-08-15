import { describe, test, expect, beforeEach, afterEach } from 'vitest'
import { DHTStorage, EncryptedBlock, BlockMetadata } from '../dhtStorage'
import { FourWordIdentity } from '../../../types/collaboration'
import * as crypto from 'crypto'

describe('DHTStorage', () => {
  let dht: DHTStorage
  let testIdentity: FourWordIdentity
  
  beforeEach(async () => {
    testIdentity = {
      fourWords: 'ocean-forest-moon-star',
      publicKey: 'pk_test_123',
      dhtAddress: 'dht://ocean-forest-moon-star'
    }
    
    dht = new DHTStorage({
      identity: testIdentity,
      bootstrapNodes: ['localhost:5001', 'localhost:5002'],
      replicationFactor: 3
    })
    
    await dht.connect()
  })

  afterEach(async () => {
    await dht.disconnect()
  })

  describe('Basic DHT operations', () => {
    test('should store and retrieve encrypted blocks', async () => {
      // Given: Data block with encryption
      const plaintext = new Uint8Array([1, 2, 3, 4, 5])
      const key = await dht.generateEncryptionKey()
      
      // When: PUT to DHT, then GET
      const blockId = await dht.put(plaintext, key)
      const retrieved = await dht.get(blockId, key)
      
      // Then: Retrieved data matches original
      expect(retrieved).toEqual(plaintext)
    })

    test('should find nodes storing specific content', async () => {
      // Given: Content hash
      const data = new Uint8Array([1, 2, 3])
      const key = await dht.generateEncryptionKey()
      const blockId = await dht.put(data, key)
      
      // When: FIND_NODE query
      const nodes = await dht.findNodes(blockId)
      
      // Then: Returns list of storing nodes
      expect(nodes.length).toBeGreaterThanOrEqual(1)
      expect(nodes[0]).toHaveProperty('nodeId')
      expect(nodes[0]).toHaveProperty('address')
    })

    test('should replicate to maintain redundancy', async () => {
      // Given: Replication factor = 3
      const data = new Uint8Array([1, 2, 3])
      const key = await dht.generateEncryptionKey()
      
      // When: Store block
      const blockId = await dht.put(data, key)
      
      // Then: Block exists on 3+ nodes
      const nodes = await dht.findNodes(blockId)
      expect(nodes.length).toBeGreaterThanOrEqual(3)
    })
  })

  describe('Encryption tests', () => {
    test('should encrypt with AES-256-GCM', async () => {
      // Given: Plaintext data
      const plaintext = Buffer.from('Secret message')
      
      // When: Encrypt for storage
      const encrypted = await dht.encrypt(plaintext)
      
      // Then: Ciphertext passes validation
      expect(encrypted.encryptedData).not.toEqual(plaintext)
      expect(encrypted.nonce).toHaveLength(12)
      expect(encrypted.authTag).toHaveLength(16)
      
      // Can decrypt back
      const decrypted = await dht.decrypt(encrypted)
      expect(decrypted).toEqual(plaintext)
    })

    test('should verify signatures on blocks', async () => {
      // Given: Signed block
      const data = new Uint8Array([1, 2, 3])
      const block = await dht.createSignedBlock(data)
      
      // When: Verify with public key
      const isValid = await dht.verifySignature(block)
      
      // Then: Signature valid
      expect(isValid).toBe(true)
      
      // Tamper with data
      block.encryptedData[0] = (block.encryptedData[0] + 1) % 256
      
      // Should detect tampering
      const isTampered = await dht.verifySignature(block)
      expect(isTampered).toBe(false)
    })

    test('should use unique nonces for each encryption', async () => {
      const data = Buffer.from('Same data')
      
      const encrypted1 = await dht.encrypt(data)
      const encrypted2 = await dht.encrypt(data)
      
      expect(encrypted1.nonce).not.toEqual(encrypted2.nonce)
      expect(encrypted1.encryptedData).not.toEqual(encrypted2.encryptedData)
    })
  })

  describe('Block metadata', () => {
    test('should attach metadata to blocks', async () => {
      const data = new Uint8Array(1024)
      const metadata: BlockMetadata = {
        size: 1024,
        createdAt: Date.now(),
        mimeType: 'text/markdown',
        erasureIndex: { shardIndex: 3, totalShards: 16 },
        forwardIdentity: {
          fourWords: 'river-mountain-sun-cloud',
          publicKey: 'pk_forward_456',
          dhtAddress: 'dht://river-mountain-sun-cloud'
        }
      }
      
      const blockId = await dht.putWithMetadata(data, metadata)
      const retrieved = await dht.getWithMetadata(blockId)
      
      expect(retrieved.metadata).toEqual(metadata)
    })

    test('should index blocks by metadata properties', async () => {
      // Store multiple blocks with different mime types
      await dht.putWithMetadata(new Uint8Array(10), {
        size: 10,
        createdAt: Date.now(),
        mimeType: 'text/markdown'
      })
      
      await dht.putWithMetadata(new Uint8Array(20), {
        size: 20,
        createdAt: Date.now(),
        mimeType: 'image/png'
      })
      
      // Query by mime type
      const markdownBlocks = await dht.findByMimeType('text/markdown')
      const imageBlocks = await dht.findByMimeType('image/png')
      
      expect(markdownBlocks.length).toBeGreaterThanOrEqual(1)
      expect(imageBlocks.length).toBeGreaterThanOrEqual(1)
    })
  })

  describe('Content addressing', () => {
    test('should use BLAKE3 for content addressing', async () => {
      const data = new Uint8Array([1, 2, 3, 4, 5])
      const hash = await dht.computeHash(data)
      
      expect(hash).toHaveLength(64) // 32 bytes hex encoded
      
      // Same data produces same hash
      const hash2 = await dht.computeHash(data)
      expect(hash2).toBe(hash)
      
      // Different data produces different hash
      const differentData = new Uint8Array([5, 4, 3, 2, 1])
      const hash3 = await dht.computeHash(differentData)
      expect(hash3).not.toBe(hash)
    })

    test('should handle content deduplication', async () => {
      const data = new Uint8Array([1, 2, 3])
      const key = await dht.generateEncryptionKey()
      
      // Store same content twice
      const blockId1 = await dht.put(data, key)
      const blockId2 = await dht.put(data, key)
      
      // Should return same block ID (content addressed)
      expect(blockId2).toBe(blockId1)
    })
  })

  describe('Network resilience', () => {
    test('should handle node failures gracefully', async () => {
      const data = new Uint8Array([1, 2, 3])
      const key = await dht.generateEncryptionKey()
      const blockId = await dht.put(data, key)
      
      // Simulate node failure
      await dht.simulateNodeFailure('localhost:5001')
      
      // Should still retrieve data from other replicas
      const retrieved = await dht.get(blockId, key)
      expect(retrieved).toEqual(data)
    })

    test('should self-heal after node failures', async () => {
      const data = new Uint8Array([1, 2, 3])
      const key = await dht.generateEncryptionKey()
      const blockId = await dht.put(data, key)
      
      // Simulate node failure
      await dht.simulateNodeFailure('localhost:5001')
      
      // Wait for self-healing
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // Check replication factor is maintained
      const nodes = await dht.findNodes(blockId)
      expect(nodes.length).toBeGreaterThanOrEqual(3)
    })

    test('should handle network partitions', async () => {
      // Create partition
      await dht.simulateNetworkPartition(['localhost:5001'], ['localhost:5002', 'localhost:5003'])
      
      const data = new Uint8Array([1, 2, 3])
      const key = await dht.generateEncryptionKey()
      
      // Store during partition
      const blockId = await dht.put(data, key)
      
      // Heal partition
      await dht.healNetworkPartition()
      
      // Data should be available on all nodes
      const nodes = await dht.findNodes(blockId)
      expect(nodes.length).toBeGreaterThanOrEqual(3)
    })
  })

  describe('Performance', () => {
    test('should handle batch operations efficiently', async () => {
      const blocks: Array<{ data: Uint8Array, key: Uint8Array }> = []
      
      // Prepare 100 blocks
      for (let i = 0; i < 100; i++) {
        blocks.push({
          data: new Uint8Array([i]),
          key: await dht.generateEncryptionKey()
        })
      }
      
      const startTime = performance.now()
      
      // Batch put
      const blockIds = await dht.putBatch(blocks)
      
      const endTime = performance.now()
      
      expect(blockIds).toHaveLength(100)
      expect(endTime - startTime).toBeLessThan(1000) // Should complete in under 1 second
    })

    test('should efficiently query large datasets', async () => {
      // Store 1000 blocks
      for (let i = 0; i < 1000; i++) {
        await dht.putWithMetadata(new Uint8Array([i]), {
          size: 1,
          createdAt: Date.now(),
          mimeType: i % 2 === 0 ? 'text/markdown' : 'image/png'
        })
      }
      
      const startTime = performance.now()
      
      // Query subset
      const markdownBlocks = await dht.findByMimeType('text/markdown')
      
      const endTime = performance.now()
      
      expect(markdownBlocks.length).toBeGreaterThanOrEqual(500)
      expect(endTime - startTime).toBeLessThan(100) // Query should be fast
    })
  })
})