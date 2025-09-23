#!/usr/bin/env python3
"""
Direct MCP Server Testing for Communitas
Tests core functionality via the MCP server
"""

import socket
import json
import time
import os
import sys
from typing import Dict, Any, Optional

class MCPClient:
    def __init__(self, socket_path: str = '/tmp/tauri-mcp-communitas-94743.sock'):
        self.socket_path = socket_path
        self.socket = None
        self.id_counter = 0

    def connect(self):
        """Connect to the MCP server via Unix domain socket"""
        try:
            self.socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            self.socket.connect(self.socket_path)
            print(f"✅ Connected to MCP server at {self.socket_path}")
            return True
        except Exception as e:
            print(f"❌ Failed to connect: {e}")
            return False

    def call(self, command: str, payload: Dict[str, Any] = None) -> Optional[Any]:
        """Call an MCP command and get the response"""
        if not self.socket:
            print("❌ Not connected to MCP server")
            return None

        self.id_counter += 1
        request = {
            "command": command,
            "payload": payload or {}
        }

        try:
            # Send request
            request_str = json.dumps(request) + '\n'
            self.socket.send(request_str.encode())

            # Receive response
            response_data = b""
            while True:
                chunk = self.socket.recv(4096)
                response_data += chunk
                if b'\n' in response_data:
                    break

            # Parse response
            response = json.loads(response_data.decode().strip())

            if 'error' in response:
                print(f"❌ Error: {response['error']}")
                return None

            # MCP returns raw result, not wrapped in 'result' field
            return response

        except Exception as e:
            print(f"❌ Call failed: {e}")
            return None

    def close(self):
        """Close the connection"""
        if self.socket:
            self.socket.close()
            print("👋 Disconnected from MCP server")

def test_app_status(mcp: MCPClient):
    """Test 1: Check app status"""
    print("\n🔍 Test 1: App Status")
    print("-" * 40)

    script = """
        JSON.stringify({
            tauriAvailable: !!window.__TAURI__,
            location: window.location.href,
            title: document.title,
            hasUser: !!window.__COMMUNITAS_USER__,
            appReady: !!window.__APP_READY__
        })
    """

    result = mcp.call('execute_js', {'window_label': 'main', 'code': script})
    if result:
        status = json.loads(result)
        print(f"✅ Tauri Available: {status['tauriAvailable']}")
        print(f"📍 Location: {status['location']}")
        print(f"📄 Title: {status['title']}")
        print(f"👤 Has User: {status['hasUser']}")
        print(f"🚀 App Ready: {status['appReady']}")
        return True
    return False

def test_claim_identity(mcp: MCPClient):
    """Test 2: Claim Four-Word identity"""
    print("\n🔍 Test 2: Claim Identity")
    print("-" * 40)

    script = """
        (async () => {
            const words = ['ocean', 'forest', 'moon', 'star'];
            try {
                const idHex = await window.__TAURI__.invoke('core_claim', { words });
                return JSON.stringify({ success: true, idHex, words });
            } catch (error) {
                return JSON.stringify({ success: false, error: error.toString() });
            }
        })()
    """

    result = mcp.call('execute_js', {'window_label': 'main', 'code': script, 'await_promise': True})
    if result:
        data = json.loads(result)
        if data['success']:
            print(f"✅ Identity claimed: {data['idHex'][:32]}...")
            print(f"🔤 Four Words: {'-'.join(data['words'])}")
            return True
        else:
            print(f"❌ Claim failed: {data['error']}")
    return False

def test_initialize_core(mcp: MCPClient):
    """Test 3: Initialize CoreContext"""
    print("\n🔍 Test 3: Initialize CoreContext")
    print("-" * 40)

    script = """
        (async () => {
            try {
                await window.__TAURI__.invoke('core_initialize', {
                    fourWords: 'ocean-forest-moon-star',
                    displayName: 'Test User Alice',
                    deviceName: 'Test Device',
                    deviceType: 'Desktop'
                });
                return JSON.stringify({ success: true });
            } catch (error) {
                return JSON.stringify({ success: false, error: error.toString() });
            }
        })()
    """

    result = mcp.call('execute_js', {'window_label': 'main', 'code': script, 'await_promise': True})
    if result:
        data = json.loads(result)
        if data['success']:
            print("✅ CoreContext initialized successfully")
            return True
        else:
            print(f"❌ Initialize failed: {data['error']}")
    return False

def test_create_channel(mcp: MCPClient):
    """Test 4: Create a channel"""
    print("\n🔍 Test 4: Create Channel")
    print("-" * 40)

    script = """
        (async () => {
            try {
                const channel = await window.__TAURI__.invoke('core_create_channel', {
                    name: 'test-general',
                    description: 'General test channel for MCP testing'
                });
                window.__TEST_CHANNEL__ = channel;
                return JSON.stringify({ success: true, channel });
            } catch (error) {
                return JSON.stringify({ success: false, error: error.toString() });
            }
        })()
    """

    result = mcp.call('execute_js', {'window_label': 'main', 'code': script, 'await_promise': True})
    if result:
        data = json.loads(result)
        if data['success']:
            channel = data['channel']
            print(f"✅ Channel created: {channel.get('name', 'Unknown')}")
            print(f"📍 Channel ID: {channel.get('id', 'Unknown')}")
            return True
        else:
            print(f"❌ Create channel failed: {data['error']}")
    return False

def test_storage(mcp: MCPClient):
    """Test 5: Container storage"""
    print("\n🔍 Test 5: Storage Operations")
    print("-" * 40)

    # Initialize container
    script_init = """
        (async () => {
            try {
                await window.__TAURI__.invoke('container_init');
                return JSON.stringify({ success: true });
            } catch (error) {
                return JSON.stringify({ success: false, error: error.toString() });
            }
        })()
    """

    result = mcp.call('execute_js', {'script': script_init, 'await_promise': True})
    if result:
        data = json.loads(result)
        if data['success']:
            print("✅ Container initialized")
        else:
            print(f"⚠️  Container init: {data['error']}")

    # Store an object
    script_store = """
        (async () => {
            try {
                const encoder = new TextEncoder();
                const data = encoder.encode('Test content from MCP direct testing');
                const handle = await window.__TAURI__.invoke('container_put_object', {
                    bytes: Array.from(data)
                });
                window.__TEST_HANDLE__ = handle;
                return JSON.stringify({ success: true, handle });
            } catch (error) {
                return JSON.stringify({ success: false, error: error.toString() });
            }
        })()
    """

    result = mcp.call('execute_js', {'script': script_store, 'await_promise': True})
    if result:
        data = json.loads(result)
        if data['success']:
            print(f"✅ Object stored: {data['handle'][:32]}...")
            return True
        else:
            print(f"❌ Store failed: {data['error']}")
    return False

def test_network_status(mcp: MCPClient):
    """Test 6: Network status"""
    print("\n🔍 Test 6: Network Status")
    print("-" * 40)

    script = """
        JSON.stringify({
            hasNetworkService: !!window.testNetwork,
            status: window.testNetwork?.status?.() || 'unknown',
            isConnected: window.testNetwork?.isConnected?.() || false
        })
    """

    result = mcp.call('execute_js', {'window_label': 'main', 'code': script})
    if result:
        data = json.loads(result)
        print(f"✅ Network Service: {data['hasNetworkService']}")
        print(f"📡 Status: {data['status']}")
        print(f"🔌 Connected: {data['isConnected']}")
        return True
    return False

def take_screenshot(mcp: MCPClient, name: str = "test"):
    """Take a screenshot"""
    print(f"\n📸 Taking screenshot: {name}")

    result = mcp.call('take_screenshot', {'window_label': 'main', 'format': 'png'})
    if result:
        # Save to file
        filename = f"screenshot-{name}-{int(time.time())}.png"
        try:
            import base64
            with open(filename, 'wb') as f:
                f.write(base64.b64decode(result))
            print(f"✅ Screenshot saved: {filename}")
            return True
        except Exception as e:
            print(f"❌ Failed to save screenshot: {e}")
    return False

def main():
    """Main test runner"""
    print("🚀 Communitas MCP Direct Testing")
    print("=" * 50)

    # Find the MCP socket (try the most recent known socket first)
    socket_path = '/tmp/tauri-mcp-communitas-94743.sock'

    # If that doesn't exist, search for any available socket
    if not os.path.exists(socket_path):
        socket_path = None
        for pid in ['94743', '93041', '95938', '96895', '9439', '9787']:
            test_path = f'/tmp/tauri-mcp-communitas-{pid}.sock'
            if os.path.exists(test_path):
                socket_path = test_path
                break

        if not socket_path:
            print("❌ No MCP socket found. Is Tauri running?")
            sys.exit(1)

    print(f"📍 Using socket: {socket_path}")

    # Create client
    mcp = MCPClient(socket_path)

    # Connect
    if not mcp.connect():
        sys.exit(1)

    # Run tests
    tests = [
        ("App Status", test_app_status),
        ("Claim Identity", test_claim_identity),
        ("Initialize Core", test_initialize_core),
        ("Create Channel", test_create_channel),
        ("Storage", test_storage),
        ("Network Status", test_network_status),
    ]

    results = []

    # Take initial screenshot
    take_screenshot(mcp, "initial")

    for name, test_func in tests:
        try:
            success = test_func(mcp)
            results.append((name, success))
            time.sleep(0.5)  # Small delay between tests
        except Exception as e:
            print(f"❌ Test '{name}' crashed: {e}")
            results.append((name, False))

    # Take final screenshot
    take_screenshot(mcp, "final")

    # Print summary
    print("\n" + "=" * 50)
    print("📊 TEST SUMMARY")
    print("=" * 50)

    passed = sum(1 for _, success in results if success)
    failed = len(results) - passed

    for name, success in results:
        status = "✅" if success else "❌"
        print(f"{status} {name}")

    print("-" * 50)
    print(f"Total: {passed} passed, {failed} failed")

    # Close connection
    mcp.close()

    # Exit code based on results
    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()