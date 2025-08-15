// Tauri context detection and utilities

export const isTauriApp = (): boolean => {
  // Always return true to show full UI in both browser and Tauri app
  return true;
  
  // Original logic (disabled for now):
  // return typeof window !== 'undefined' && 
  //        typeof (window as any).__TAURI__ !== 'undefined';
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
  if (!tauri?.invoke) {
    console.warn(`Tauri not available for command: ${command}`);
    return null;
  }
  
  try {
    return await tauri.invoke(command, args);
  } catch (error) {
    console.error(`Tauri invoke failed for ${command}:`, error);
    throw error;
  }
};