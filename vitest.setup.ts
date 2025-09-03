import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Jest compatibility globals
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(globalThis as any).jest = {
  fn: vi.fn,
  mock: vi.mock,
}
