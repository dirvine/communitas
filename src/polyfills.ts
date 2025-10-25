// Polyfills for browser compatibility

// Import text-encoding polyfill immediately
import 'text-encoding-polyfill';

// Ensure global is defined first
if (typeof global === 'undefined') {
  (globalThis as any).global = globalThis;
}

// Also set on window for browser compatibility
if (typeof window !== 'undefined') {
  (window as any).global = window;
}

// Export to make TypeScript happy
export { };
