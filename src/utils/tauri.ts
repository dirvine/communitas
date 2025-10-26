// Tauri context detection and utilities

export const isTauriApp = (): boolean => {
  // Detect real Tauri runtime; tests/browser should be false
  // Check multiple possible Tauri indicators
  if (typeof window === 'undefined') return false;
  
  // Check for _TAURI_ global
  if (typeof (window as any)._TAURI_ !== 'undefined') return true;
  
  // Check for _TAURI_IPC_ global (alternative)
  if (typeof (window as any)._TAURI_IPC_ !== 'undefined') return true;
  
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
  return (window as any)._TAURI_;
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
