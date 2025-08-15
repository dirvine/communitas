import { IdentityVerifier } from '../IdentityVerifier'
import { DHTMapper } from '../DHTMapper'
import { MockDHTNetwork } from '../__mocks__/MockDHTNetwork'
import { MockCache } from '../__mocks__/MockCache'

describe('IdentityVerifier', () => {
  let verifier: IdentityVerifier
  let mapper: DHTMapper
  let mockNetwork: MockDHTNetwork
  let mockCache: MockCache

  beforeEach(() => {
    mockNetwork = new MockDHTNetwork()
    mockCache = new MockCache()
    mapper = new DHTMapper(mockNetwork)
    verifier = new IdentityVerifier(mapper, mockCache)
  })

  afterEach(() => {
    mockNetwork.cleanup()
    mockCache.clear()
  })

  describe('verification flow', () => {
    test('performs complete challenge-response verification', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      // Register identity
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Verify identity
      const result = await verifier.verify(fourWords)
      
      expect(result.valid).toBe(true)
      expect(result.fourWords).toBe(fourWords)
      expect(result.publicKey).toEqual(publicKey)
      expect(result.timestamp).toBeDefined()
      expect(result.cached).toBe(false)
    })

    test('validates ML-DSA signatures', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateMLDSAKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setMLDSAKeys(publicKey, privateKey)
      
      const result = await verifier.verifyWithMLDSA(fourWords)
      
      expect(result.valid).toBe(true)
      expect(result.algorithm).toBe('ML-DSA')
      expect(result.quantumResistant).toBe(true)
    })

    test('handles timeout scenarios', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey } = await mapper.generateKeypair()
      await mapper.registerIdentity(fourWords, publicKey)
      
      // Configure short timeout
      verifier.setTimeout(100) // 100ms
      
      // Simulate slow response
      mockNetwork.setLatency(200)
      
      const result = await verifier.verify(fourWords)
      expect(result.valid).toBe(false)
      expect(result.error).toBe('Verification timeout')
    })

    test('manages verification cache', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // First verification - not cached
      const result1 = await verifier.verify(fourWords)
      expect(result1.valid).toBe(true)
      expect(result1.cached).toBe(false)
      
      // Second verification - cached
      const result2 = await verifier.verify(fourWords)
      expect(result2.valid).toBe(true)
      expect(result2.cached).toBe(true)
      expect(result2.publicKey).toEqual(result1.publicKey)
    })

    test('respects cache TTL', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Set short TTL
      verifier.setCacheTTL(100) // 100ms
      
      // First verification
      await verifier.verify(fourWords)
      
      // Wait for cache to expire
      await new Promise(resolve => setTimeout(resolve, 150))
      
      // Should perform fresh verification
      const result = await verifier.verify(fourWords)
      expect(result.cached).toBe(false)
    })

    test('handles batch verification efficiently', async () => {
      const identities = []
      
      // Register multiple identities
      for (let i = 0; i < 10; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const { publicKey, privateKey } = await mapper.generateKeypair()
        await mapper.registerIdentity(fourWords, publicKey)
        const node = await mapper.mapToNode(fourWords)
        node.setPrivateKey(privateKey)
        identities.push(fourWords)
      }
      
      // Batch verify
      const start = performance.now()
      const results = await verifier.verifyBatch(identities)
      const time = performance.now() - start
      
      expect(results).toHaveLength(10)
      expect(results.every(r => r.valid)).toBe(true)
      expect(time).toBeLessThan(500) // Should be fast with parallelization
    })
  })

  describe('security', () => {
    test('prevents replay attacks', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Intercept challenge-response
      let capturedChallenge: Uint8Array
      let capturedResponse: Uint8Array
      
      verifier.onChallenge((challenge, response) => {
        capturedChallenge = challenge
        capturedResponse = response
      })
      
      await verifier.verify(fourWords)
      
      // Try to replay
      const replayResult = await verifier.verifyWithResponse(
        fourWords,
        capturedChallenge!,
        capturedResponse!
      )
      
      expect(replayResult.valid).toBe(false)
      expect(replayResult.error).toContain('replay')
    })

    test('validates key ownership', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey: realKey, privateKey: realPrivate } = await mapper.generateKeypair()
      const { publicKey: fakeKey, privateKey: fakePrivate } = await mapper.generateKeypair()
      
      // Register with real key
      await mapper.registerIdentity(fourWords, realKey)
      const node = await mapper.mapToNode(fourWords)
      
      // Try to respond with fake key
      node.setPrivateKey(fakePrivate)
      
      const result = await verifier.verify(fourWords)
      expect(result.valid).toBe(false)
      expect(result.error).toContain('ownership')
    })

    test('detects impersonation attempts', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey: realKey } = await mapper.generateKeypair()
      const { publicKey: attackerKey, privateKey: attackerPrivate } = await mapper.generateKeypair()
      
      // Register legitimate identity
      await mapper.registerIdentity(fourWords, realKey)
      
      // Attacker tries to claim same identity
      const attackerNode = await mockNetwork.createNode(attackerKey)
      attackerNode.setPrivateKey(attackerPrivate)
      attackerNode.claimIdentity(fourWords)
      
      // Verification should detect mismatch
      const result = await verifier.verify(fourWords)
      expect(result.valid).toBe(false)
      expect(result.error).toContain('impersonation')
    })

    test('validates signature timestamps', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Create old signature
      const oldTimestamp = Date.now() - 3600000 // 1 hour old
      const oldSignature = await node.createTimestampedSignature(oldTimestamp)
      
      // Should reject old signatures
      const result = await verifier.verifySignature(
        fourWords,
        oldSignature,
        publicKey
      )
      
      expect(result.valid).toBe(false)
      expect(result.error).toContain('timestamp')
    })

    test('implements rate limiting', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      
      // Configure rate limit
      verifier.setRateLimit(5, 1000) // 5 requests per second
      
      // Rapid requests
      const promises = Array(10).fill(null).map(() => 
        verifier.verify(fourWords)
      )
      
      const results = await Promise.all(promises)
      const rateLimited = results.filter(r => r.error === 'Rate limited')
      
      expect(rateLimited.length).toBeGreaterThan(0)
    })

    test('validates challenge uniqueness', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Track challenges
      const challenges = new Set<string>()
      
      verifier.onChallenge((challenge) => {
        const hex = Buffer.from(challenge).toString('hex')
        expect(challenges.has(hex)).toBe(false)
        challenges.add(hex)
      })
      
      // Multiple verifications should use unique challenges
      for (let i = 0; i < 10; i++) {
        await verifier.verify(fourWords)
        // Clear cache to force new verification
        mockCache.delete(fourWords)
      }
      
      expect(challenges.size).toBe(10)
    })
  })

  describe('cache management', () => {
    test('implements LRU cache eviction', async () => {
      // Set small cache size
      verifier.setCacheSize(5)
      
      // Verify more than cache size
      for (let i = 0; i < 10; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const { publicKey, privateKey } = await mapper.generateKeypair()
        await mapper.registerIdentity(fourWords, publicKey)
        const node = await mapper.mapToNode(fourWords)
        node.setPrivateKey(privateKey)
        await verifier.verify(fourWords)
      }
      
      // Cache should only have last 5
      const cacheStats = verifier.getCacheStats()
      expect(cacheStats.size).toBe(5)
      
      // First entries should be evicted
      expect(await verifier.isCached('word0-color0-object0-concept0')).toBe(false)
      expect(await verifier.isCached('word9-color9-object9-concept9')).toBe(true)
    })

    test('differentiates positive and negative cache TTL', async () => {
      const validFourWords = 'ocean-azure-stone-dream'
      const invalidFourWords = 'invalid-fake-test-words'
      
      // Set different TTLs
      verifier.setCacheTTL(3600000, 300000) // 1 hour positive, 5 min negative
      
      // Valid identity
      const { publicKey, privateKey } = await mapper.generateKeypair()
      await mapper.registerIdentity(validFourWords, publicKey)
      const node = await mapper.mapToNode(validFourWords)
      node.setPrivateKey(privateKey)
      
      // Verify both
      await verifier.verify(validFourWords)
      await verifier.verify(invalidFourWords)
      
      // Check cache entries
      const validEntry = await verifier.getCacheEntry(validFourWords)
      const invalidEntry = await verifier.getCacheEntry(invalidFourWords)
      
      expect(validEntry.ttl).toBe(3600000)
      expect(invalidEntry.ttl).toBe(300000)
    })

    test('supports cache warming', async () => {
      const identities = []
      
      // Prepare identities
      for (let i = 0; i < 10; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        const { publicKey, privateKey } = await mapper.generateKeypair()
        await mapper.registerIdentity(fourWords, publicKey)
        const node = await mapper.mapToNode(fourWords)
        node.setPrivateKey(privateKey)
        identities.push({ fourWords, publicKey })
      }
      
      // Warm cache
      await verifier.warmCache(identities)
      
      // All should be cached
      for (const { fourWords } of identities) {
        expect(await verifier.isCached(fourWords)).toBe(true)
      }
      
      // Verification should be instant
      const start = performance.now()
      await verifier.verify(identities[0].fourWords)
      const time = performance.now() - start
      
      expect(time).toBeLessThan(1) // Sub-millisecond from cache
    })

    test('handles cache invalidation', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Verify and cache
      await verifier.verify(fourWords)
      expect(await verifier.isCached(fourWords)).toBe(true)
      
      // Invalidate
      await verifier.invalidate(fourWords)
      expect(await verifier.isCached(fourWords)).toBe(false)
      
      // Next verification should be fresh
      const result = await verifier.verify(fourWords)
      expect(result.cached).toBe(false)
    })

    test('provides cache statistics', async () => {
      // Perform various operations
      for (let i = 0; i < 20; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        
        if (i < 10) {
          // Valid identities
          const { publicKey, privateKey } = await mapper.generateKeypair()
          await mapper.registerIdentity(fourWords, publicKey)
          const node = await mapper.mapToNode(fourWords)
          node.setPrivateKey(privateKey)
        }
        
        await verifier.verify(fourWords)
        
        // Verify some multiple times for hits
        if (i < 5) {
          await verifier.verify(fourWords)
          await verifier.verify(fourWords)
        }
      }
      
      const stats = verifier.getCacheStats()
      
      expect(stats.size).toBeGreaterThan(0)
      expect(stats.hits).toBeGreaterThan(0)
      expect(stats.misses).toBeGreaterThan(0)
      expect(stats.hitRate).toBeGreaterThan(0)
      expect(stats.hitRate).toBeLessThanOrEqual(1)
    })
  })

  describe('error handling', () => {
    test('handles network failures gracefully', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      
      // Simulate network failure
      mockNetwork.simulateFailure()
      
      const result = await verifier.verify(fourWords)
      expect(result.valid).toBe(false)
      expect(result.error).toContain('network')
    })

    test('validates input format', async () => {
      const invalidInputs = [
        '',
        'not-enough-words',
        'too-many-words-here-now',
        'UPPERCASE-NOT-ALLOWED-HERE-NOW',
        'special@chars-not-allowed-here',
        '123-numbers-not-allowed-here'
      ]
      
      for (const invalid of invalidInputs) {
        const result = await verifier.verify(invalid)
        expect(result.valid).toBe(false)
        expect(result.error).toContain('format')
      }
    })

    test('handles missing identity gracefully', async () => {
      const fourWords = 'nonexistent-identity-test-case'
      
      const result = await verifier.verify(fourWords)
      expect(result.valid).toBe(false)
      expect(result.error).toContain('not found')
    })

    test('handles corrupted cache data', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Verify to populate cache
      await verifier.verify(fourWords)
      
      // Corrupt cache data
      mockCache.corrupt(fourWords)
      
      // Should fallback to fresh verification
      const result = await verifier.verify(fourWords)
      expect(result.valid).toBe(true)
      expect(result.cached).toBe(false)
    })

    test('provides detailed error information', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey } = await mapper.generateKeypair()
      const { privateKey: wrongKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(wrongKey) // Wrong key
      
      const result = await verifier.verify(fourWords)
      
      expect(result.valid).toBe(false)
      expect(result.error).toBeDefined()
      expect(result.errorCode).toBeDefined()
      expect(result.errorDetails).toBeDefined()
      expect(result.errorDetails).toHaveProperty('expected')
      expect(result.errorDetails).toHaveProperty('actual')
    })
  })

  describe('monitoring and metrics', () => {
    test('tracks verification latency', async () => {
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      // Enable metrics
      verifier.enableMetrics()
      
      // Perform verifications
      for (let i = 0; i < 10; i++) {
        await verifier.verify(fourWords)
        if (i < 5) mockCache.delete(fourWords) // Force some fresh verifications
      }
      
      const metrics = verifier.getMetrics()
      
      expect(metrics.averageLatency).toBeDefined()
      expect(metrics.p50Latency).toBeDefined()
      expect(metrics.p95Latency).toBeDefined()
      expect(metrics.p99Latency).toBeDefined()
      expect(metrics.totalVerifications).toBe(10)
    })

    test('tracks verification outcomes', async () => {
      verifier.enableMetrics()
      
      // Mix of valid and invalid
      for (let i = 0; i < 10; i++) {
        const fourWords = `word${i}-color${i}-object${i}-concept${i}`
        
        if (i < 6) {
          // Valid
          const { publicKey, privateKey } = await mapper.generateKeypair()
          await mapper.registerIdentity(fourWords, publicKey)
          const node = await mapper.mapToNode(fourWords)
          node.setPrivateKey(privateKey)
        }
        
        await verifier.verify(fourWords)
      }
      
      const metrics = verifier.getMetrics()
      
      expect(metrics.successCount).toBe(6)
      expect(metrics.failureCount).toBe(4)
      expect(metrics.successRate).toBe(0.6)
    })

    test('provides real-time monitoring hooks', async () => {
      const events: any[] = []
      
      verifier.onVerification((event) => {
        events.push(event)
      })
      
      const fourWords = 'ocean-azure-stone-dream'
      const { publicKey, privateKey } = await mapper.generateKeypair()
      
      await mapper.registerIdentity(fourWords, publicKey)
      const node = await mapper.mapToNode(fourWords)
      node.setPrivateKey(privateKey)
      
      await verifier.verify(fourWords)
      
      expect(events).toHaveLength(1)
      expect(events[0]).toHaveProperty('fourWords', fourWords)
      expect(events[0]).toHaveProperty('success', true)
      expect(events[0]).toHaveProperty('latency')
      expect(events[0]).toHaveProperty('cached', false)
    })
  })
})