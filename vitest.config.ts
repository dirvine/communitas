import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

const coverageThresholds = process.env.STRICT_COVERAGE === 'true'
  ? {
      lines: Number(process.env.COVERAGE_MIN_LINES ?? '85'),
      functions: Number(process.env.COVERAGE_MIN_FUNCTIONS ?? '85'),
      branches: Number(process.env.COVERAGE_MIN_BRANCHES ?? '80'),
      statements: Number(process.env.COVERAGE_MIN_STATEMENTS ?? '85'),
    }
  : undefined

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@tauri-apps/api/core': '/src/test-mocks/tauri_core_mock.ts',
      'monaco-editor': '/src/test-mocks/monaco.ts',
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/setupTests.ts', './vitest.setup.ts'],
    include: ['src/**/*.test.{ts,tsx}', 'src/**/__tests__/**/*.{ts,tsx}'],
    // Memory management for CI
    pool: 'forks',
    poolOptions: {
      forks: {
        singleFork: true,
        isolate: true,
      }
    },
    maxWorkers: process.env.CI ? 1 : undefined,
    minWorkers: 1,
    testTimeout: 10000, // 10 seconds to prevent hanging
    hookTimeout: 5000,  // 5 seconds for hooks
    teardownTimeout: 5000, // 5 seconds for cleanup
    coverage: {
      reporter: ['text', 'lcov', 'html'],
      reportsDirectory: './coverage',
      provider: 'v8',
      thresholds: coverageThresholds,
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/__tests__/**',
        'src/setupTests.ts',
        'vitest.setup.ts',
        'src/test-mocks/**',
        '**/*.d.ts',
      ],
    },
  },
})
