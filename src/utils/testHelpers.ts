/**
 * Comprehensive Test Utilities for Communitas
 *
 * Shared helpers, mocks, and utilities for testing across the entire application.
 * Supports both unit tests (Vitest) and E2E tests (Playwright).
 */

import { vi } from 'vitest'

// Test Data Constants
export const TEST_CONSTANTS = {
  // User data
  TEST_USERS: {
    alice: { id: 'alice-test', username: 'alice_test', password: 'SecurePass123!' },
    bob: { id: 'bob-test', username: 'bob_test', password: 'SecurePass456!' },
    charlie: { id: 'charlie-test', username: 'charlie_test', password: 'SecurePass789!' }
  },

  // PQC Test Data
  PQC_DATA: {
    message: 'This is a test message protected by ML-DSA and Kyber PQC',
    document: 'Test document content for PQC encryption and signing',
    fileName: 'test-document.txt',
    largeData: 'A'.repeat(1024 * 1024), // 1MB test data
  },

  // Network Test Data
  NETWORK_DATA: {
    peerId1: 'peer-001',
    peerId2: 'peer-002',
    dhtKey: 'test-dht-key',
    groupId: 'test-group-123',
  },

  // Timeouts
  TIMEOUTS: {
    SHORT: 1000,    // 1 second
    MEDIUM: 5000,   // 5 seconds
    LONG: 10000,    // 10 seconds
    E2E: 30000,     // 30 seconds for E2E
  },

  // Test Selectors
  SELECTORS: {
    LOGIN_USERNAME: '[data-testid="login-username"]',
    LOGIN_PASSWORD: '[data-testid="login-password"]',
    LOGIN_SUBMIT: '[data-testid="login-submit"]',
    DASHBOARD: '[data-testid="dashboard"]',
    CHAT_INPUT: '[data-testid="message-input"]',
    SEND_BUTTON: '[data-testid="send-message"]',
    MESSAGE_CONTENT: '[data-testid="message-content"]',
  }
}

// Mock Factories
export class MockFactory {
  static createMockPeer(id: string, overrides: Partial<any> = {}) {
    return {
      id,
      address: `mock://${id}`,
      publicKey: `mock-pqc-key-${id}`,
      connected: true,
      latency: Math.floor(Math.random() * 100) + 10,
      storageCapacity: 1000000000, // 1GB
      availableStorage: 500000000, // 500MB
      capabilities: ['storage', 'messaging', 'pqc-encryption'],
      lastSeen: Date.now(),
      ...overrides
    }
  }

  static createMockDHTEntry(key: string, value: Uint8Array, peers: string[] = []) {
    return {
      key,
      value,
      peers,
      timestamp: Date.now(),
      ttl: 3600000, // 1 hour
    }
  }

  static createMockPQCKeyPair(algorithm = 'ML-DSA-65') {
    return {
      publicKey: `mock-pqc-public-key-${algorithm}`,
      privateKey: `mock-pqc-private-key-${algorithm}`,
      keyId: `mock-key-id-${Date.now()}`,
      algorithm,
      createdAt: Date.now(),
      expiresAt: Date.now() + 365 * 24 * 60 * 60 * 1000, // 1 year
    }
  }

  static createMockEncryptedBlock(data: Uint8Array, keyId: string) {
    return {
      encryptedData: data,
      iv: new Uint8Array(12), // 96 bits for GCM
      authTag: new Uint8Array(16), // 128 bits
      signature: new Uint8Array(64), // ML-DSA signature
      keyId,
      publicKey: `mock-pqc-public-key-${keyId}`,
      algorithm: 'ML-DSA-65',
    }
  }
}

// PQC Test Helpers
export class PQCTestHelper {
  static generateMockMLDSAKeyPair() {
    return MockFactory.createMockPQCKeyPair('ML-DSA-65')
  }

  static generateMockKyberKeyPair() {
    return MockFactory.createMockPQCKeyPair('Kyber-768')
  }

  static createMockPQCSignature(data: Uint8Array, keyPair: any) {
    // Simulate ML-DSA signature (would be 3309 bytes in reality)
    return new Uint8Array(64) // Mock signature
  }

  static createMockPQCEncryption(data: Uint8Array, publicKey: string) {
    // Simulate Kyber encryption
    return {
      ciphertext: data, // In reality this would be encrypted
      keyId: `pqc-key-${Date.now()}`,
      algorithm: 'Kyber-768',
    }
  }

  static async simulatePQCLatency(operation: string): Promise<void> {
    // Simulate PQC operation latency (typically 10-50ms)
    const latency = operation.includes('sign') ? 20 :
                   operation.includes('verify') ? 15 :
                   operation.includes('encrypt') ? 30 :
                   operation.includes('decrypt') ? 25 : 10

    await new Promise(resolve => setTimeout(resolve, latency))
  }
}

// Network Test Helpers
export class NetworkTestHelper {
  static createMockPeerNetwork(peerCount: number = 5) {
    const peers = []
    for (let i = 0; i < peerCount; i++) {
      peers.push(MockFactory.createMockPeer(`peer-${i.toString().padStart(3, '0')}`))
    }
    return peers
  }

  static createMockDHTNetwork() {
    const entries = new Map()
    const testKeys = ['key1', 'key2', 'key3']

    testKeys.forEach(key => {
      const value = new TextEncoder().encode(`DHT value for ${key}`)
      entries.set(key, MockFactory.createMockDHTEntry(key, value, ['peer-001', 'peer-002']))
    })

    return entries
  }

  static async simulateNetworkDelay(minMs: number = 10, maxMs: number = 100): Promise<void> {
    const delay = Math.random() * (maxMs - minMs) + minMs
    await new Promise(resolve => setTimeout(resolve, delay))
  }

  static simulateNetworkFailure(): void {
    // Simulate network conditions
    const shouldFail = Math.random() < 0.1 // 10% failure rate
    if (shouldFail) {
      throw new Error('Simulated network failure')
    }
  }
}

// Storage Test Helpers
export class StorageTestHelper {
  static createMockStorageData(size: number = 1024): Uint8Array {
    return new Uint8Array(size).map((_, i) => i % 256)
  }

  static createMockReedSolomonConfig(groupSize: number) {
    // Adaptive configuration based on group size
    let dataShards: number
    let parityShards: number

    if (groupSize <= 3) {
      dataShards = 3
      parityShards = 2
    } else if (groupSize <= 8) {
      dataShards = 8
      parityShards = 4
    } else if (groupSize <= 20) {
      dataShards = 12
      parityShards = 6
    } else {
      dataShards = 16
      parityShards = 8
    }

    return {
      dataShards,
      parityShards,
      totalShards: dataShards + parityShards,
      canLoseMembers: parityShards,
      redundancyFactor: (dataShards + parityShards) / dataShards,
    }
  }

  static async simulateStorageOperation(operation: string): Promise<void> {
    // Simulate storage operation latency
    const latency = operation.includes('write') ? 50 :
                   operation.includes('read') ? 30 :
                   operation.includes('delete') ? 20 : 10

    await new Promise(resolve => setTimeout(resolve, latency))
  }
}

// UI Test Helpers
export class UITestHelper {
  static async waitForElement(selector: string, timeout: number = 5000): Promise<void> {
    // Implementation would depend on testing framework
    await new Promise(resolve => setTimeout(resolve, 100))
  }

  static async simulateUserTyping(input: string, delay: number = 50): Promise<void> {
    // Simulate realistic typing with delays
    for (const char of input) {
      await new Promise(resolve => setTimeout(resolve, delay))
      // Type character (implementation depends on framework)
    }
  }

  static generateTestId(prefix: string, suffix?: string): string {
    return `test-${prefix}${suffix ? `-${suffix}` : ''}-${Date.now()}`
  }
}

// Performance Test Helpers
export class PerformanceTestHelper {
  static async measureExecutionTime<T>(
    operation: () => Promise<T>,
    operationName: string = 'operation'
  ): Promise<{ result: T; duration: number }> {
    const startTime = performance.now()
    const result = await operation()
    const endTime = performance.now()
    const duration = endTime - startTime

    console.log(`${operationName} took ${duration.toFixed(2)}ms`)
    return { result, duration }
  }

  static async runLoadTest(
    operation: () => Promise<any>,
    iterations: number = 100,
    concurrency: number = 10
  ): Promise<{
    totalTime: number
    averageTime: number
    minTime: number
    maxTime: number
    successRate: number
  }> {
    const results: number[] = []
    let successCount = 0

    const startTime = performance.now()

    // Run operations with controlled concurrency
    for (let i = 0; i < iterations; i += concurrency) {
      const batch = Array(Math.min(concurrency, iterations - i))
        .fill(null)
        .map(async () => {
          const batchStart = performance.now()
          try {
            await operation()
            successCount++
            const batchEnd = performance.now()
            results.push(batchEnd - batchStart)
          } catch (error) {
            const batchEnd = performance.now()
            results.push(batchEnd - batchStart)
          }
        })

      await Promise.all(batch)
    }

    const endTime = performance.now()
    const totalTime = endTime - startTime

    return {
      totalTime,
      averageTime: results.reduce((a, b) => a + b, 0) / results.length,
      minTime: Math.min(...results),
      maxTime: Math.max(...results),
      successRate: successCount / iterations,
    }
  }
}

// Security Test Helpers
export class SecurityTestHelper {
  static generateMaliciousPayload(type: 'xss' | 'sql' | 'path-traversal' | 'buffer-overflow'): string {
    switch (type) {
      case 'xss':
        return '<script>alert("XSS")</script>'
      case 'sql':
        return "'; DROP TABLE users; --"
      case 'path-traversal':
        return '../../../etc/passwd'
      case 'buffer-overflow':
        return 'A'.repeat(1000000) // 1MB string
      default:
        return 'malicious payload'
    }
  }

  static async simulateTimingAttack(operation: (input: string) => Promise<any>): Promise<boolean> {
    // Simulate timing attack detection
    const testInputs = ['short', 'A'.repeat(100), 'A'.repeat(1000)]
    const timings: number[] = []

    for (const input of testInputs) {
      const start = performance.now()
      await operation(input)
      const end = performance.now()
      timings.push(end - start)
    }

    // Check for suspicious timing patterns
    const variance = timings.reduce((acc, time, i) => {
      if (i === 0) return acc
      return acc + Math.abs(time - timings[i - 1])
    }, 0) / (timings.length - 1)

    return variance > 10 // High variance might indicate timing leak
  }
}

// Export convenience functions
export const {
  TEST_USERS,
  PQC_DATA,
  NETWORK_DATA,
  TIMEOUTS,
  SELECTORS
} = TEST_CONSTANTS

// All helper classes are exported above