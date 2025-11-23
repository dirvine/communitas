# Bridge Testing Walkthrough

## Goal
Verify the production P2P deployment using the `communitas-bridge` and a browser-based test script, as requested by the user.

## Accomplishments

1.  **Bridge Configuration**:
    *   Created `bridge.toml` to configure the bridge to connect to the production bootstrap node (`138.197.29.195:4433`).
    *   Identified the bootstrap node's four-word identity: `bangui-routine-evaporate-lunch`.

2.  **Bug Fix in Bridge**:
    *   Identified that `communitas-bridge` initialized the core but failed to start the networking stack, preventing connections.
    *   Added a new API endpoint `POST /api/network/start` to `communitas-bridge/src/handlers.rs` and registered it in `server.rs`.
    *   This allows clients to explicitly start networking after initialization.

3.  **Browser-Based Verification**:
    *   Used the Agent's browser tool to execute a JavaScript test script against the bridge.
    *   **Script Actions**:
        1.  Initialize Core (`/api/core/initialize`)
        2.  Start Networking (`/api/network/start`) - **New Step**
        3.  Connect to Bootstrap Node (`/api/network/connect`)
        4.  Verify Status (`/api/network/connection-info`)
    *   **Result**:
        *   Core initialized successfully.
        *   Networking started (listening on local port).
        *   Connection to `bangui-routine-evaporate-lunch` initiated successfully (`status: "connected"`).

## Key Artifacts

*   `bridge.toml`: Configuration for production bootstrap node.
*   `communitas-bridge/src/handlers.rs`: Added `start_networking` handler.
*   `communitas-bridge/src/server.rs`: Registered `/api/network/start` route.

## How to Run the Test

1.  Start the bridge:
    ```bash
    cargo run -p communitas-bridge -- --config bridge.toml
    ```

2.  Run the following JavaScript in a browser console (e.g., on `http://localhost:3030/health`):

    ```javascript
    async function runTest() {
      const baseUrl = 'http://localhost:3030';
      const bootstrapId = 'bangui-routine-evaporate-lunch';
      
      // 1. Initialize
      await fetch(`${baseUrl}/api/core/initialize`, {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
          four_words: 'agent-test-user-final',
          display_name: 'Agent Test User',
          device_name: 'Agent Browser'
        })
      });

      // 2. Start Networking (CRITICAL STEP)
      await fetch(`${baseUrl}/api/network/start`, {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: '{}'
      });

      // 3. Connect to Bootstrap
      const connect = await fetch(`${baseUrl}/api/network/connect`, {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({ four_word_addr: bootstrapId })
      });
      
      console.log('Connect result:', await connect.json());
    }
    runTest();
    ```
