import '@testing-library/jest-dom'
import { vi } from 'vitest'
import crypto from 'crypto'
import 'fake-indexeddb/auto'

// Mock Tauri core API invoked by pqcCrypto utilities (no TS types in callback)
// '@tauri-apps/api/core' is aliased to a local stub in vitest.config.ts

// Jest compatibility globals
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(globalThis as any).jest = {
  fn: vi.fn,
  mock: vi.mock,
}

// Polyfill crypto.subtle for Node.js test environment
if (!globalThis.crypto) {
  // @ts-ignore
  globalThis.crypto = crypto.webcrypto
}

// Ensure crypto.subtle is available
if (globalThis.crypto && !globalThis.crypto.subtle) {
  // @ts-ignore
  globalThis.crypto.subtle = crypto.webcrypto.subtle
}

// Enhanced crypto.subtle.digest to handle various input types
const originalDigest = globalThis.crypto.subtle.digest
globalThis.crypto.subtle.digest = async function(algorithm: AlgorithmIdentifier, data: BufferSource) {
  // Convert string to ArrayBuffer if needed (common test case)
  let buffer: ArrayBuffer
  if (typeof data === 'string') {
    const encoder = new TextEncoder()
    buffer = encoder.encode(data).buffer
  } else if (data instanceof Uint8Array) {
    buffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength)
  } else if (data instanceof ArrayBuffer) {
    buffer = data
  } else if (ArrayBuffer.isView(data)) {
    buffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength)
  } else {
    // Fallback: try to convert to Uint8Array
    try {
      const uint8 = new Uint8Array(data as any)
      buffer = uint8.buffer
    } catch {
      throw new TypeError('Failed to execute digest: Invalid data type')
    }
  }
  
  return originalDigest.call(this, algorithm, buffer)
}
