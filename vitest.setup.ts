import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Basic Jest compatibility shim for tests using jest.* APIs
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(globalThis as any).jest = {
  fn: vi.fn,
  mock: vi.mock,
}
