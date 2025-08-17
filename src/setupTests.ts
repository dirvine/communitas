// Vitest setup: provide a minimal blake3 shim for tests
// Map 'blake3' import to a simple SHA-256-based placeholder for deterministic test gradients
import crypto from 'crypto'
import { expect } from 'vitest'

// @ts-ignore
globalThis.__BLAKE3_SHIM__ = (input: string) => {
  const h = crypto.createHash('sha256')
  h.update(input)
  return h.digest('hex')
}

// Add minimal matcher used by some Jest-era tests
expect.extend({
  toStartWith(received: string, prefix: string) {
    const pass = typeof received === 'string' && received.startsWith(prefix)
    return {
      pass,
      message: () =>
        pass
          ? `Expected string not to start with ${prefix}`
          : `Expected ${received} to start with ${prefix}`,
    }
  },
} as any)
