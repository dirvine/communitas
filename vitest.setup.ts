import '@testing-library/jest-dom'
import { vi } from 'vitest'
import crypto from 'crypto'

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
