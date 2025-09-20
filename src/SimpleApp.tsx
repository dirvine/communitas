import React from 'react'

function SimpleApp() {
  return (
    <div style={{ padding: '20px', fontFamily: 'system-ui' }}>
      <h1>Communitas Test App</h1>
      <p>✅ React is working!</p>
      <div style={{ marginTop: '20px', padding: '10px', background: '#f0f0f0', borderRadius: '5px' }}>
        <h2>Testnet Status</h2>
        <p>5 nodes running on local testnet</p>
        <ul>
          <li>Node 1: philosophy-truth-prevent-wound (Port 9000)</li>
          <li>Node 2: donna-jewish-scorpion-socrates (Port 9010)</li>
          <li>Node 3: bike-in-porto-napkin (Port 9020)</li>
          <li>Node 4: congratulate-twice-tonga-hurt (Port 9030)</li>
          <li>Node 5: sponsor-biker-simon-leipzig (Port 9040)</li>
        </ul>
      </div>
      <div style={{ marginTop: '20px' }}>
        <button onClick={() => console.log('Testing button click')}>Test Button</button>
      </div>
    </div>
  )
}

export default SimpleApp