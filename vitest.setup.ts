import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Mock Tauri core API invoked by pqcCrypto utilities (no TS types in callback)
// '@tauri-apps/api/core' is aliased to a local stub in vitest.config.ts

// Jest compatibility globals
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(globalThis as any).jest = {
  fn: vi.fn,
  mock: vi.mock,
}
