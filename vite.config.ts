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
    environment: 'jsdom',
    // Sprint 1: exclude heavy/integration/E2E and complex UI suites
    exclude: [
      'node_modules/**',
      'dist/**',
      'src/services/storage/__tests__/integration/**',
      'src/services/storage/__tests__/e2e/**',
      'src/services/storage/__tests__/yjsCollaboration.test.*',
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
