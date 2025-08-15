import { describe, test, expect, beforeEach, afterEach } from '@jest/globals'
import { ReedSolomonEncoder } from '../reedSolomon'

describe('ReedSolomonEncoder', () => {
  let encoder: ReedSolomonEncoder
  
  beforeEach(() => {
    encoder = new ReedSolomonEncoder({
      dataShards: 10,
      parityShards: 6,
    })
  })

  afterEach(() => {
    encoder.cleanup()
  })

  describe('Basic encoding/decoding', () => {
    test('should encode file into k+m shards', async () => {
      // Given: 10MB file
      const fileSize = 10 * 1024 * 1024 // 10MB
      const originalData = new Uint8Array(fileSize)
      for (let i = 0; i < fileSize; i++) {
        originalData[i] = Math.floor(Math.random() * 256)
      }
      
      // When: Encode with k=10, m=6
      const shards = await encoder.encode(originalData)
      
      // Then: Produces 16 shards, each ~1MB
      expect(shards).toHaveLength(16)
      const expectedShardSize = Math.ceil(fileSize / 10)
      shards.forEach(shard => {
        expect(shard.data.length).toBeCloseTo(expectedShardSize, -3)
        expect(shard.index).toBeGreaterThanOrEqual(0)
        expect(shard.index).toBeLessThan(16)
      })
    })

    test('should decode file from any k shards', async () => {
      // Given: Original data and 16 shards with 6 missing
      const originalData = new Uint8Array(1024 * 1024) // 1MB for faster test
      for (let i = 0; i < originalData.length; i++) {
        originalData[i] = i % 256
      }
      
      const allShards = await encoder.encode(originalData)
      
      // When: Decode with any 10 shards (remove 6)
      const availableShards = allShards.filter((_, index) => index !== 3 && index !== 7 && index !== 9 && index !== 12 && index !== 14 && index !== 15)
      expect(availableShards).toHaveLength(10)
      
      const decoded = await encoder.decode(availableShards)
      
      // Then: Original file reconstructed perfectly
      expect(decoded).toEqual(originalData)
    })

    test('should fail decoding with less than k shards', async () => {
      // Given: Only 9 shards available (k=10)
      const originalData = new Uint8Array(1024)
      const allShards = await encoder.encode(originalData)
      const insufficientShards = allShards.slice(0, 9)
      
      // When: Attempt decode
      // Then: Throws InsufficientShardsError
      await expect(encoder.decode(insufficientShards)).rejects.toThrow('InsufficientShardsError')
    })
  })

  describe('Edge cases', () => {
    test('should handle empty files', async () => {
      // Given: 0-byte file
      const emptyData = new Uint8Array(0)
      
      // When: Encode/decode
      const shards = await encoder.encode(emptyData)
      const decoded = await encoder.decode(shards)
      
      // Then: Returns empty file
      expect(decoded).toEqual(emptyData)
    })

    test('should handle files not divisible by k', async () => {
      // Given: 10MB + 7 bytes file
      const size = 10 * 1024 * 1024 + 7
      const originalData = new Uint8Array(size)
      for (let i = 0; i < size; i++) {
        originalData[i] = i % 256
      }
      
      // When: Encode with padding
      const shards = await encoder.encode(originalData)
      
      // Then: Decode removes padding correctly
      const decoded = await encoder.decode(shards)
      expect(decoded).toEqual(originalData)
    })
  })

  describe('Performance boundaries', () => {
    test('should encode 100MB file in under 2 seconds', async () => {
      // Performance benchmark test
      const size = 100 * 1024 * 1024
      const data = new Uint8Array(size)
      
      const startTime = performance.now()
      await encoder.encode(data)
      const endTime = performance.now()
      
      expect(endTime - startTime).toBeLessThan(2000)
    }, 5000) // 5 second timeout for this test
  })

  describe('Shard management', () => {
    test('should generate unique shard identifiers', async () => {
      const data = new Uint8Array(1024)
      const shards = await encoder.encode(data)
      
      const ids = shards.map(s => s.id)
      const uniqueIds = new Set(ids)
      expect(uniqueIds.size).toBe(16)
    })

    test('should verify shard integrity with checksums', async () => {
      const data = new Uint8Array(1024)
      const shards = await encoder.encode(data)
      
      // Corrupt one shard
      shards[0].data[0] = (shards[0].data[0] + 1) % 256
      
      // Should detect corruption
      expect(() => encoder.verifyShard(shards[0])).toThrow('CorruptedShardError')
    })

    test('should handle mixed shard versions gracefully', async () => {
      const dataV1 = new Uint8Array(1024)
      const dataV2 = new Uint8Array(1024)
      dataV2[0] = 255
      
      const shardsV1 = await encoder.encode(dataV1)
      const shardsV2 = await encoder.encode(dataV2)
      
      // Mix shards from different versions
      const mixedShards = [...shardsV1.slice(0, 5), ...shardsV2.slice(5, 10)]
      
      await expect(encoder.decode(mixedShards)).rejects.toThrow('VersionMismatchError')
    })
  })

  describe('2-person sharing mode', () => {
    test('should use full replication for 2-person sharing', async () => {
      const encoder2Person = new ReedSolomonEncoder({
        dataShards: 1,
        parityShards: 1,
        mode: 'two-person'
      })
      
      const data = new Uint8Array(1024 * 1024)
      const shards = await encoder2Person.encode(data)
      
      // Should create full replicas
      expect(shards).toHaveLength(2)
      expect(shards[0].data).toEqual(data)
      expect(shards[1].data).toEqual(data)
    })
  })
})