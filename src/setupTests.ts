// Vitest setup: provide a minimal blake3 shim for tests
// Map 'blake3' import to a simple SHA-256-based placeholder for deterministic test gradients
import crypto from 'crypto'

// @ts-ignore
globalThis.__BLAKE3_SHIM__ = (input: string) => {
  const h = crypto.createHash('sha256')
  h.update(input)
  return h.digest('hex')
}
