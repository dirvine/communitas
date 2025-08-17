import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig(({ mode }) => {
  // Load env file based on mode
  const env = loadEnv(mode, process.cwd(), '')
  
  // Use browser env file if in browser mode
  if (mode === 'browser') {
    Object.assign(process.env, loadEnv('browser', process.cwd(), ''))
  }

  return {
  plugins: [react()],
  test: {
    setupFiles: ['src/setupTests.ts'],
    // Default to jsdom for UI-ish tests; override heavy suites to Node env
    environment: 'jsdom',
    // Constrain worker pool to minimize memory usage on CI/Node 24
    pool: 'forks',
    maxThreads: 1,
    minThreads: 1,
    isolate: true,
    testTimeout: 60000,
    hookTimeout: 60000,
    // Run storage tests under Node to avoid jsdom overhead
    environmentMatchGlobs: [
      ['src/services/storage/**', 'node'],
    ],
    include: [
      'src/services/__tests__/featureFlags.test.ts',
      'src/utils/__tests__/identity.test.ts',
      'src/services/storage/__tests__/dhtStorage.test.ts',
      'src/services/storage/__tests__/reedSolomon.test.ts',
      'src/services/storage/__tests__/markdownPublisher.test.ts',
    ],
    // Sprint 1: exclude heavy/integration/E2E and complex UI suites
    exclude: [
      'node_modules/**',
      'dist/**',
      'src/services/storage/__tests__/integration/**',
      'src/services/storage/__tests__/e2e/**',
      'src/services/storage/__tests__/yjsCollaboration.test.*',
      // Keep other storage suites disabled for now
      'src/services/storage/__tests__/**',
      'src/components/unified/__tests__/**',
      'src/components/identity/__tests__/**',
    ],
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  }
}})
