import React, { useEffect, useState } from 'react'
import { subscribeMessages, DecryptedMessagePayload } from '../../services/messagingSubscription'

export const MessageConsole: React.FC<{ channelId?: string }> = ({ channelId }) => {
  const [messages, setMessages] = useState<DecryptedMessagePayload[]>([])

  useEffect(() => {
    let unlisten: (() => void) | undefined
    subscribeMessages((payload) => {
      setMessages((prev) => [payload, ...prev].slice(0, 200))
    }, { channelId })
      .then((fn) => { unlisten = fn })
      .catch((e) => console.error('subscribeMessages failed', e))
    return () => { if (unlisten) unlisten() }
  }, [channelId])

  return (
    <div style={{ padding: 12, fontFamily: 'monospace', background: '#0b0f14', color: '#cde' }}>
      <h3>Message Console {channelId ? `(Channel ${channelId})` : ''}</h3>
      <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
        {messages.map((m, i) => (
          <li key={i} style={{ marginBottom: 8 }}>
            {m.encrypted ? (
              <span style={{ color: '#f5a623' }}>[encrypted] receivedAt={m.receivedAt}</span>
            ) : (
              <span>
                <strong>{m.sender}</strong> → {m.channel_id} :: {JSON.stringify(m.content)}
              </span>
            )}
          </li>
        ))}
      </ul>
    </div>
  )
}

