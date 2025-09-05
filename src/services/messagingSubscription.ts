import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

export interface DecryptedMessagePayload {
  id?: string
  channel_id?: string
  sender?: string
  content?: any
  receivedAt?: string
  encrypted?: boolean
  error?: string
}

/**
 * Subscribe to backend message events and receive decrypted payloads when possible.
 * Returns an unlisten function to cancel the subscription.
 */
export async function subscribeMessages(
  onMessage: (payload: DecryptedMessagePayload) => void,
  opts?: { channelId?: string }
): Promise<UnlistenFn> {
  await invoke('core_subscribe_messages', { channelId: opts?.channelId })
  const unlisten = await listen<DecryptedMessagePayload>('message-received', (evt) => {
    onMessage(evt.payload)
  })
  return unlisten
}

/**
 * Example usage:
 *
 * const unlisten = await subscribeMessages((msg) => {
 *   if (msg.encrypted) console.log('Encrypted message received')
 *   else console.log('Message:', msg)
 * }, { channelId })
 *
 * // Later to stop
 * unlisten()
 */

