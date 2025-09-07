import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { nodePolyfills } from 'vite-plugin-node-polyfills'

export default defineConfig({
  plugins: [
    react(),
    // Add polyfills for Node.js modules
    nodePolyfills({
      include: ['events', 'crypto', 'stream', 'buffer'],
      globals: {
        Buffer: true,
        global: true,
        process: true,
      },
    }),
  ],
  server: {
    port: 1422,
    strictPort: false,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    chunkSizeWarningLimit: 2000,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules')) {
            if (id.includes('monaco-editor')) return 'monaco';
            if (id.includes('@mui/')) return 'mui';
            if (id.includes('react') || id.includes('react-dom')) return 'react';
            if (id.includes('yjs') || id.includes('y-webrtc')) return 'yjs';
            if (id.includes('@tauri-apps')) return 'tauri';
          }
        },
      },
    },
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
