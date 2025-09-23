// Tauri context detection and utilities

export const isTauriApp = (): boolean => {
  // Detect real Tauri runtime; tests/browser should be false
  // Check multiple possible Tauri indicators
  if (typeof window === 'undefined') return false;
  
  // Check for __TAURI__ global
  if (typeof (window as any).__TAURI__ !== 'undefined') return true;
  
  // Check for __TAURI_IPC__ global (alternative)
  if (typeof (window as any).__TAURI_IPC__ !== 'undefined') return true;
  
  // Check for tauri:// protocol in window.location
  if (window.location.protocol === 'tauri:') return true;
  
  // Check for Tauri in user agent
  if (navigator.userAgent.includes('Tauri')) return true;
  
  return false;
};

export const getTauriApi = () => {
  if (!isTauriApp()) {
    return null;
  }
  return (window as any).__TAURI__;
};

// Safe invoke wrapper that handles missing Tauri context
export const safeInvoke = async <T = any>(
  command: string, 
  args?: Record<string, any>
): Promise<T | null> => {
  const tauri = getTauriApi();

  const invoker = tauri?.invoke
    ?? tauri?.core?.invoke
    ?? tauri?.tauri?.invoke;

  try {
    if (invoker) {
      return await invoker(command, args);
    }

    const mod = await import('@tauri-apps/api/core');
    return await mod.invoke<T>(command, args);
  } catch (error) {
    if (import.meta.env.DEV) {
      console.warn(`Tauri invoke failed for ${command}:`, error);
    }
    return null;
  }
};
