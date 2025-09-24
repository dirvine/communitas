export const isTauriApp = (): boolean => {
  if (typeof window === 'undefined') return false
  if (typeof (window as any).__TAURI__ !== 'undefined') return true
  if (typeof (window as any).__TAURI_IPC__ !== 'undefined') return true
  if (window.location?.protocol === 'tauri:') return true
  if (typeof navigator !== 'undefined' && navigator.userAgent.includes('Tauri')) return true
  return false
}

export const getTauriApi = () => {
  if (!isTauriApp()) return null
  return (window as any).__TAURI__ ?? null
}

export const safeInvoke = async <T = any>(command: string, args?: Record<string, unknown>): Promise<T | null> => {
  try {
    const tauri = getTauriApi()
    const invoker = tauri?.invoke ?? tauri?.core?.invoke ?? tauri?.tauri?.invoke

    if (typeof invoker === 'function') {
      return await invoker(command, args)
    }

    const mod = await import('@tauri-apps/api/core')
    if (typeof mod.invoke === 'function') {
      return await mod.invoke<T>(command, args)
    }
  } catch (error) {
    if (import.meta?.env?.DEV) {
      console.warn(`[Communitas][safeInvoke] ${command} failed`, error)
    }
  }
  return null
}
