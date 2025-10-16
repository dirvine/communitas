// Mock for @tauri-apps/api/event
import { vi } from 'vitest';

export type UnlistenFn = () => void;

export const listen = vi.fn();
