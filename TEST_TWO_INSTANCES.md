# Testing Two-Instance P2P Connection

This guide shows how to test peer-to-peer connection between two Communitas instances using Chrome DevTools MCP.

## Prerequisites

- Chrome DevTools MCP configured (`.mcp.json`)
- Two terminal windows for running instances
- Node.js and npm installed

## Current Network Commands (Tauri)

The following commands are available in `communitas-desktop/src/network.rs`:

1. **`get_endpoint_four_words`** - Get local connection ID (four words)
2. **`connect_via_four_words`** - Connect to peer using four-word address
3. **`get_network_status`** - Check connection status
4. **`get_network_info`** - Get detailed network info
5. **`validate_four_words`** - Validate four-word format

## Architecture Notes

- **Saorsa-Core Integration**: Commands currently return stub data; full integration pending
- **LAN Discovery**: Nodes should detect LAN address for local mini-network testing
- **Four-Word Encoding**: Uses `four-word-networking` crate v2.6.0 for IP:port encoding

## Test Workflow

### Step 1: Start Instance 1 (Port 5173)

```bash
# Terminal 1
cd /Users/davidirvine/Desktop/Devel/projects/communitas
npm run tauri dev
```

This starts the first instance on default port 5173.

### Step 2: Start Instance 2 (Port 5174)

```bash
# Terminal 2
cd /Users/davidirvine/Desktop/Devel/projects/communitas
PORT=5174 npm run tauri dev
```

This starts the second instance on port 5174.

### Step 3: Get Connection ID from Instance 1

Using Chrome DevTools MCP, navigate to Instance 1 and execute:

```javascript
// Option 1: Call Tauri command directly
await window.__TAURI_INTERNALS__.invoke('get_endpoint_four_words');

// Option 2: Use console helper (if available)
await window.getEndpointWords();
```

Expected result (currently stub):
```javascript
{
  endpoint_four_words: "alpha-bravo-charlie-delta",
  ip_address: "192.168.1.100",
  port: 12345
}
```

### Step 4: Connect Instance 2 to Instance 1

Navigate to Instance 2 in Chrome DevTools MCP and execute:

```javascript
await window.__TAURI_INTERNALS__.invoke('connect_via_four_words', {
  four_words: "alpha-bravo-charlie-delta",
  user_four_words: "ocean-forest-moon-star"  // Your user identity
});
```

Expected result:
```javascript
{
  success: true,
  message: "Bootstrap node added"
}
```

### Step 5: Verify Connection Status

On both instances, check connection status:

```javascript
await window.__TAURI_INTERNALS__.invoke('get_network_status');
```

Expected result:
```javascript
{
  connected: true,
  peers: 1,
  endpoint_four_words: "alpha-bravo-charlie-delta"
}
```

## Chrome DevTools MCP Test Script

```javascript
// test-two-instances.js
// Run this script using Chrome DevTools MCP

async function testTwoInstanceConnection() {
  console.log("=== Two Instance P2P Connection Test ===");

  // Step 1: Open Instance 1
  console.log("\n1. Opening Instance 1 (port 5173)...");
  await navigatePage("http://localhost:5173/");
  await waitFor("Communitas");

  // Get connection ID from Instance 1
  console.log("\n2. Getting connection ID from Instance 1...");
  const instance1Info = await evaluateScript(async () => {
    return await window.__TAURI_INTERNALS__.invoke('get_endpoint_four_words');
  });
  console.log("Instance 1 connection ID:", instance1Info);

  // Step 2: Open Instance 2 in new tab
  console.log("\n3. Opening Instance 2 (port 5174)...");
  await newPage("http://localhost:5174/");
  await waitFor("Communitas");

  // Get connection ID from Instance 2
  console.log("\n4. Getting connection ID from Instance 2...");
  const instance2Info = await evaluateScript(async () => {
    return await window.__TAURI_INTERNALS__.invoke('get_endpoint_four_words');
  });
  console.log("Instance 2 connection ID:", instance2Info);

  // Step 3: Connect Instance 2 to Instance 1
  console.log("\n5. Connecting Instance 2 to Instance 1...");
  const connectResult = await evaluateScript(async (fourWords) => {
    return await window.__TAURI_INTERNALS__.invoke('connect_via_four_words', {
      four_words: fourWords,
      user_four_words: "ocean-forest-moon-star"
    });
  }, [instance1Info.endpoint_four_words]);
  console.log("Connection result:", connectResult);

  // Step 4: Verify connection on Instance 2
  console.log("\n6. Verifying connection on Instance 2...");
  const status2 = await evaluateScript(async () => {
    return await window.__TAURI_INTERNALS__.invoke('get_network_status');
  });
  console.log("Instance 2 status:", status2);

  // Step 5: Switch back to Instance 1 and verify
  console.log("\n7. Verifying connection on Instance 1...");
  await selectPage(0);  // Switch to Instance 1 tab
  const status1 = await evaluateScript(async () => {
    return await window.__TAURI_INTERNALS__.invoke('get_network_status');
  });
  console.log("Instance 1 status:", status1);

  // Summary
  console.log("\n=== Test Summary ===");
  console.log("Instance 1 Connected:", status1.connected);
  console.log("Instance 1 Peers:", status1.peers);
  console.log("Instance 2 Connected:", status2.connected);
  console.log("Instance 2 Peers:", status2.peers);

  return {
    instance1: { info: instance1Info, status: status1 },
    instance2: { info: instance2Info, status: status2 },
    success: status1.connected && status2.connected && status1.peers > 0 && status2.peers > 0
  };
}

// Run the test
testTwoInstanceConnection()
  .then(result => {
    console.log("\n=== TEST COMPLETED ===");
    console.log("Success:", result.success);
    console.log("Full result:", JSON.stringify(result, null, 2));
  })
  .catch(error => {
    console.error("\n=== TEST FAILED ===");
    console.error("Error:", error);
  });
```

## Manual Testing Steps (Without MCP)

### Using Browser Console

1. **Instance 1 (localhost:5173)**:
   ```javascript
   // Get connection ID
   const info1 = await window.__TAURI_INTERNALS__.invoke('get_endpoint_four_words');
   console.log("My connection ID:", info1.endpoint_four_words);
   ```

2. **Instance 2 (localhost:5174)**:
   ```javascript
   // Connect to Instance 1
   await window.__TAURI_INTERNALS__.invoke('connect_via_four_words', {
     four_words: "alpha-bravo-charlie-delta",  // Replace with actual ID from Instance 1
     user_four_words: "ocean-forest-moon-star"
   });

   // Check status
   const status = await window.__TAURI_INTERNALS__.invoke('get_network_status');
   console.log("Connection status:", status);
   ```

## Known Limitations (Current Stubs)

⚠️ **Note**: The following behaviors are currently stubbed and will be implemented with proper saorsa-core integration:

1. **`get_endpoint_four_words`**: Returns hardcoded "alpha-bravo-charlie-delta" instead of actual local endpoint
2. **Connection establishment**: Bootstrap node is added but no actual P2P connection established
3. **Network status**: Reports connected=false with 0 peers
4. **LAN discovery**: Not yet implemented (nodes should detect LAN IP automatically)

## Next Steps for Full Implementation

1. **Integrate saorsa-core networking**:
   - Use ant-quic for QUIC transport
   - Use DHT for peer discovery
   - Implement proper listener startup with real endpoint

2. **LAN IP detection**:
   - Detect local network interface addresses
   - Prefer LAN IP for local testing scenarios
   - Fall back to localhost if LAN not available

3. **Four-word encoding**:
   - Encode actual local endpoint (IP:port) to four words
   - Use `FourWordAdaptiveEncoder` from four-word-networking crate
   - Handle both IPv4 and IPv6 addresses

4. **Peer connection**:
   - Use saorsa-core's connection establishment
   - Maintain peer list in NetworkRuntime
   - Update connection status in real-time

## Debugging

### Check Tauri Logs

```bash
# View Tauri backend logs
RUST_LOG=debug npm run tauri dev
```

### Verify Network Commands

```javascript
// List all available Tauri commands
console.log(Object.keys(window.__TAURI_INTERNALS__));

// Test command availability
try {
  await window.__TAURI_INTERNALS__.invoke('get_endpoint_four_words');
  console.log("✓ Command available");
} catch (error) {
  console.error("✗ Command error:", error);
}
```

### Common Issues

1. **"Command not found"**: Check that commands are registered in `main.rs`
2. **Port conflicts**: Use different ports for each instance
3. **CORS errors**: Not applicable for Tauri (uses IPC, not HTTP)

## Expected Behavior (When Fully Implemented)

1. **Instance 1 starts**:
   - Detects LAN IP (e.g., 192.168.1.100)
   - Starts QUIC listener on available port (e.g., 12345)
   - Encodes IP:port to four words (e.g., "mountain-river-cloud-fire")
   - Reports connection ID to user

2. **Instance 2 starts**:
   - Same initialization as Instance 1
   - Gets own connection ID (e.g., "ocean-forest-moon-star")

3. **Instance 2 connects to Instance 1**:
   - User enters Instance 1's four words: "mountain-river-cloud-fire"
   - System decodes to 192.168.1.100:12345
   - Establishes QUIC connection via saorsa-core
   - Both instances show connected=true, peers=1

4. **Testing capabilities enabled**:
   - Voice/video calls between instances
   - Message exchange
   - File sharing
   - Screen sharing
