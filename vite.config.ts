import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { nodePolyfills } from 'vite-plugin-node-polyfills'

export default defineConfig({
  plugins: [
    react(),
    // Add polyfills for Node.js modules
    nodePolyfills({
      include: ['events', 'crypto', 'stream', 'buffer', 'util', 'string_decoder'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
      protocolImports: true,
    }),
  ],
  server: {
    host: '0.0.0.0',
    port: 5000,
    strictPort: false,
    cors: true,
    allowedHosts: true,
    headers: {
      'Access-Control-Allow-Origin': '*',
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    chunkSizeWarningLimit: 2000,
  },
  optimizeDeps: {
    esbuildOptions: {
      // Enable esbuild polyfill for Node.js modules
      define: {
        global: 'globalThis',
      },
    },
  },
})
