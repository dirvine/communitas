import React from 'react'

export default function AppMinimal() {
  return (
    <div style={{ padding: '40px', fontFamily: 'system-ui, sans-serif' }}>
      <h1>Communitas</h1>
      <p>App is loading successfully!</p>
      <p>TextDecoder status: {typeof TextDecoder !== 'undefined' ? 'Available' : 'Not available'}</p>
      <p>TextEncoder status: {typeof TextEncoder !== 'undefined' ? 'Available' : 'Not available'}</p>
      {typeof TextDecoder === 'undefined' && (
        <div style={{ color: 'red', marginTop: '20px' }}>
          <strong>Error: TextDecoder is not available</strong>
          <p>This is causing the app to fail during initialization.</p>
        </div>
      )}
    </div>
  )
}