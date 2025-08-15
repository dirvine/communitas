import { DHTMapper } from '../DHTMapper'
import { blake3 } from '@noble/hashes/blake3'
import { MockDHTNetwork } from '../__mocks__/MockDHTNetwork'

describe('DHTMapper', () => {
  let mapper: DHTMapper
  let mockNetwork: MockDHTNetwork

  beforeEach(() => {
    mockNetwork = new MockDHTNetwork()
    mapper = new DHTMapper(mockNetwork)
  })

  afterEach(() => {
    mockNetwork.cleanup()
  })

  describe('hashing', () => {
    test('generates consistent DHT IDs', () => {
      const fourWords = 'ocean-azure-stone-dream'
      const id1 = mapper.calculateDHTId(fourWords)
      const id2 = mapper.calculateDHTId(fourWords)
      
      expect(id1).toEqual(id2)
      expect(id1).toBeInstanceOf(Uint8Array)
      expect(id1.length).toBe(32) // 256 bits
    })

    test('uses BLAKE3 correctly', () => {
      const fourWords = 'ocean-azure-stone-dream'
      const id = mapper.calculateDHTId(fourWords)
      const expected = blake3(fourWords)
      
      expect(id).toEqual(expected)
    })

    test('handles different input formats', () => {
      const variations = [
        'ocean-azure-stone-dream',
        'Ocean-Azure-Stone-Dream',
        'OCEAN-AZURE-STONE-DREAM',
        '  ocean-azure-stone-dream  '
      ]
      
      const normalized = mapper.calculateDHTId(variations[0])
      
      // All should normalize to same ID
      variations.slice(1).forEach(variant => {
        expect(mapper.calculateDHTId(variant)).toEqual(normalized)
      })
    })

    test('produces 256-bit output', () => {
      const batch = Array(100).fill(null).map((_, i) => 
        `word${i}-word${i}-word${i}-word${i}`
      )
      
      batch.forEach(fourWords => {
        const id = mapper.calculateDHTId(fourWords)
        expect(id.length).toBe(32) // 32 bytes = 256 bits
        expect(id.every(byte => byte >= 0 && byte <= 255)).toBe(true)
      })
    })

    test('generates unique IDs for different inputs', () => {
      const ids = new Set()
      
      for (let i = 0; i < 1000; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const id = Buffer.from(mapper.calculateDHTId(fourWords)).toString('hex')
        ids.add(id)
      }
      
      expect(ids.size).toBe(1000)
    })
  })

  describe('node mapping', () => {
    test('maps four-words to correct node', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const dhtId = mapper.calculateDHTId(fourWords)
      
      const node = await mapper.mapToNode(fourWords)
      expect(node.id).toEqual(dhtId)
      expect(node.fourWords).toBe(fourWords)
    })

    test('handles network topology changes', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      
      // Initial mapping
      const node1 = await mapper.mapToNode(fourWords)
      
      // Simulate topology change
      mockNetwork.simulateChurn(0.2)
      
      // Should still find the node
      const node2 = await mapper.mapToNode(fourWords)
      expect(node2.id).toEqual(node1.id)
    })

    test('maintains mapping consistency', async () => {
      const mappings = new Map()
      
      // Create multiple mappings
      for (let i = 0; i < 10; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const node = await mapper.mapToNode(fourWords)
        mappings.set(fourWords, node.id)
      }
      
      // Verify consistency after network activity
      mockNetwork.simulateActivity()
      
      for (const [fourWords, expectedId] of mappings) {
        const node = await mapper.mapToNode(fourWords)
        expect(node.id).toEqual(expectedId)
      }
    })

    test('finds closest node for unclaimed identity', async () => {
      const fourWords = 'unclaimed-identity-test-case'
      const dhtId = mapper.calculateDHTId(fourWords)
      
      // Add some nodes to network
      await mockNetwork.addNodes(10)
      
      const closestNode = await mapper.findClosestNode(fourWords)
      expect(closestNode).toBeDefined()
      
      // Verify it's actually the closest by XOR distance
      const distance = mapper.calculateXorDistance(dhtId, closestNode.id)
      const otherNodes = await mockNetwork.getAllNodes()
      
      otherNodes.forEach(node => {
        const nodeDistance = mapper.calculateXorDistance(dhtId, node.id)
        expect(distance).toBeLessThanOrEqual(nodeDistance)
      })
    })

    test('handles K-bucket routing', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const kBucket = await mapper.getKBucket(fourWords)
      
      expect(kBucket).toHaveLength(8) // K=8 for replication
      expect(kBucket.every(node => node.id)).toBe(true)
      
      // All nodes should be reachable
      const reachable = await Promise.all(
        kBucket.map(node => mockNetwork.ping(node.id))
      )
      expect(reachable.every(r => r)).toBe(true)
    })
  })

  describe('verification', () => {
    test('verifies ownership proofs', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      // Register identity
      const proof = await mapper.createOwnershipProof(
        fourWords,
        privateKey
      )
      
      await mapper.registerIdentity(fourWords, publicKey, proof)
      
      // Verify ownership
      const isValid = await mapper.verifyOwnership(fourWords)
      expect(isValid).toBe(true)
    })

    test('validates ML-DSA signatures', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateMLDSAKeypair()
      
      const message = Buffer.from(fourWords)
      const signature = await mapper.signMLDSA(message, privateKey)
      
      const isValid = await mapper.verifyMLDSA(
        message,
        signature,
        publicKey
      )
      expect(isValid).toBe(true)
      
      // Invalid signature should fail
      const tamperedSignature = new Uint8Array(signature)
      tamperedSignature[0] ^= 1
      
      const isInvalid = await mapper.verifyMLDSA(
        message,
        tamperedSignature,
        publicKey
      )
      expect(isInvalid).toBe(false)
    })

    test('handles invalid proofs', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const fakeProof = new Uint8Array(64).fill(0)
      const fakePublicKey = new Uint8Array(32).fill(0)
      
      await mapper.registerIdentity(fourWords, fakePublicKey, fakeProof)
      
      const isValid = await mapper.verifyOwnership(fourWords)
      expect(isValid).toBe(false)
    })

    test('performs challenge-response verification', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      // Register and get node
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Challenge-response
      const challenge = crypto.getRandomValues(new Uint8Array(32))
      const response = await node.respondToChallenge(challenge)
      
      const isValid = await mapper.verifyChallengeResponse(
        challenge,
        response,
        publicKey
      )
      expect(isValid).toBe(true)
    })

    test('prevents replay attacks', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // First challenge
      const challenge1 = crypto.getRandomValues(new Uint8Array(32))
      const response1 = await node.respondToChallenge(challenge1)
      
      // Verify first response
      expect(await mapper.verifyChallengeResponse(
        challenge1,
        response1,
        publicKey
      )).toBe(true)
      
      // Replay should fail
      expect(await mapper.verifyChallengeResponse(
        challenge1,
        response1,
        publicKey
      )).toBe(false)
    })
  })

  describe('network operations', () => {
    test('handles node discovery', async () => {
      // Register multiple identities
      const identities = []
      for (let i = 0; i < 10; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const { publicKey } = await mapper.generateKeypair()
        await mapper.registerIdentity(fourWords, publicKey)
        identities.push(fourWords)
      }
      
      // Discover all nodes
      for (const fourWords of identities) {
        const node = await mapper.discoverNode(fourWords)
        expect(node).toBeDefined()
        expect(node.fourWords).toBe(fourWords)
      }
    })

    test('handles network partitions', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey } = await mapper.generateKeypair()
      await mapper.registerIdentity(fourWords, publicKey)
      
      // Simulate partition
      mockNetwork.simulatePartition()
      
      // Should handle gracefully with fallback
      const node = await mapper.mapToNode(fourWords)
      expect(node).toBeDefined()
      
      // Heal partition
      mockNetwork.healPartition()
      
      // Should work normally
      const nodeAfter = await mapper.mapToNode(fourWords)
      expect(nodeAfter.id).toEqual(node.id)
    })

    test('manages DHT churn', async () => {
      const identities = new Map()
      
      // Register identities
      for (let i = 0; i < 20; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const { publicKey } = await mapper.generateKeypair()
        await mapper.registerIdentity(fourWords, publicKey)
        identities.set(fourWords, publicKey)
      }
      
      // Simulate 50% churn
      mockNetwork.simulateChurn(0.5)
      
      // All identities should still be discoverable
      for (const [fourWords] of identities) {
        const node = await mapper.discoverNode(fourWords)
        expect(node).toBeDefined()
      }
    })

    test('implements redundancy for reliability', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey } = await mapper.generateKeypair()
      
      // Register with replication factor
      await mapper.registerIdentity(fourWords, publicKey, null, {
        replicationFactor: 3
      })
      
      // Verify replicas exist
      const replicas = await mapper.getReplicas(fourWords)
      expect(replicas.length).toBe(3)
      
      // All replicas should have the same data
      const data = await replicas[0].getData()
      for (const replica of replicas.slice(1)) {
        const replicaData = await replica.getData()
        expect(replicaData).toEqual(data)
      }
    })
  })

  describe('performance', () => {
    test('caches DHT lookups', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      
      // First lookup - cache miss
      const start1 = performance.now()
      await mapper.mapToNode(fourWords)
      const time1 = performance.now() - start1
      
      // Second lookup - cache hit
      const start2 = performance.now()
      await mapper.mapToNode(fourWords)
      const time2 = performance.now() - start2
      
      // Cached lookup should be much faster
      expect(time2).toBeLessThan(time1 * 0.1)
    })

    test('batches DHT operations', async () => {
      const batch = Array(100).fill(null).map((_, i) => 
        `word${i}-color${i}-object${i}-concept${i}`
      )
      
      const start = performance.now()
      const results = await mapper.batchMapToNodes(batch)
      const time = performance.now() - start
      
      expect(results).toHaveLength(100)
      expect(time).toBeLessThan(1000) // Should complete in < 1s
      
      // Verify batching is more efficient than individual
      const start2 = performance.now()
      for (const fourWords of batch.slice(0, 10)) {
        await mapper.mapToNode(fourWords)
      }
      const time2 = performance.now() - start2
      
      // Batch of 100 should be faster than 10 individual
      expect(time).toBeLessThan(time2 * 10)
    })

    test('maintains lookup performance at scale', async () => {
      // Add many nodes
      await mockNetwork.addNodes(10000)
      
      // Register many identities
      for (let i = 0; i < 1000; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const { publicKey } = await mapper.generateKeypair()
        await mapper.registerIdentity(fourWords, publicKey)
      }
      
      // Lookups should still be fast
      const lookupTimes = []
      for (let i = 0; i < 100; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const start = performance.now()
        await mapper.mapToNode(fourWords)
        lookupTimes.push(performance.now() - start)
      }
      
      const avgTime = lookupTimes.reduce((a, b) => a + b) / lookupTimes.length
      expect(avgTime).toBeLessThan(100) // < 100ms average
    })
  })

  describe('error handling', () => {
    test('handles network timeouts', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      
      // Simulate network timeout
      mockNetwork.simulateTimeout()
      
      await expect(mapper.mapToNode(fourWords))
        .rejects.toThrow('Network timeout')
    })

    test('handles invalid four-word format', async () => {
      await expect(mapper.mapToNode('invalid-format'))
        .rejects.toThrow('Invalid four-word format')
      
      await expect(mapper.mapToNode(''))
        .rejects.toThrow('Four-words cannot be empty')
    })

    test('handles node unavailability', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey } = await mapper.generateKeypair()
      await mapper.registerIdentity(fourWords, publicKey)
      
      // Make node unavailable
      const node = await mapper.mapToNode(fourWords)
      mockNetwork.makeUnavailable(node.id)
      
      // Should fallback to closest available
      const result = await mapper.mapToNode(fourWords)
      expect(result).toBeDefined()
      expect(result.available).toBe(true)
    })

    test('validates cryptographic parameters', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      
      await expect(mapper.registerIdentity(
        fourWords,
        new Uint8Array(10), // Wrong key size
        null
      )).rejects.toThrow('Invalid public key size')
      
      await expect(mapper.verifyMLDSA(
        Buffer.from('message'),
        new Uint8Array(10), // Wrong signature size
        new Uint8Array(32)
      )).rejects.toThrow('Invalid signature size')
    })
  })
})