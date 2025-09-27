
# Communitas Chrome DevTools MCP Test Report
Generated: 2025-09-27T10:49:12.804Z
URL: http://127.0.0.1:5003

## Test Results Summary
- navigation: ✅ PASS
- screenshot: ✅ PASS
- performance: ❌ FAIL
- console: ✅ PASS
- network: ✅ PASS
- react: ❌ FAIL
- authentication: ❌ FAIL
- theme: ❌ FAIL
- memory: ❌ FAIL
- errors: ❌ FAIL

## Detailed Results

### Navigation
✅ Successfully navigated to http://127.0.0.1:5003

### Screenshot
✅ Screenshot saved to /Users/davidirvine/Desktop/Devel/projects/communitas/mcp-artifacts/chrome-devtools/comprehensive-screenshot.png

### Performance
❌ Performance check failed: MCP error -32602: Invalid arguments for tool performance_start_trace: [
  {
    "code": "invalid_type",
    "expected": "boolean",
    "received": "undefined",
    "path": [
      "reload"
    ],
    "message": "Required"
  },
  {
    "code": "invalid_type",
    "expected": "boolean",
    "received": "undefined",
    "path": [
      "autoStop"
    ],
    "message": "Required"
  }
]

### Console Logs
✅ Console logs captured (see console-logs.txt)

### Network Requests
✅ Network requests captured (see network-requests.txt)

### React Components
❌ React check failed: MCP error -32602: Invalid arguments for tool evaluate_script: [
  {
    "code": "invalid_type",
    "expected": "string",
    "received": "undefined",
    "path": [
      "function"
    ],
    "message": "Required"
  }
]

### Authentication State
❌ Auth check failed: MCP error -32602: Invalid arguments for tool evaluate_script: [
  {
    "code": "invalid_type",
    "expected": "string",
    "received": "undefined",
    "path": [
      "function"
    ],
    "message": "Required"
  }
]

### Theme Switching
❌ Theme test failed: MCP error -32602: Invalid arguments for tool evaluate_script: [
  {
    "code": "invalid_type",
    "expected": "string",
    "received": "undefined",
    "path": [
      "function"
    ],
    "message": "Required"
  }
]

### Memory Usage
❌ Memory check failed: MCP error -32602: Invalid arguments for tool evaluate_script: [
  {
    "code": "invalid_type",
    "expected": "string",
    "received": "undefined",
    "path": [
      "function"
    ],
    "message": "Required"
  }
]

## Error Summary
Performance: MCP error -32602: Invalid arguments for tool performance_start_trace: [
  {
    "code": "invalid_type",
    "expected": "boolean",
    "received": "undefined",
    "path": [
      "reload"
    ],
    "message": "Required"
  },
  {
    "code": "invalid_type",
    "expected": "boolean",
    "received": "undefined",
    "path": [
      "autoStop"
    ],
    "message": "Required"
  }
]

## Artifacts Location
All test artifacts saved to: /Users/davidirvine/Desktop/Devel/projects/communitas/mcp-artifacts/chrome-devtools
