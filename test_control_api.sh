#!/bin/bash

# Test script for HTTP Control API

API_BASE="http://localhost:3040"

echo "============================================"
echo "Testing HTTP Control API"
echo "============================================"
echo

# Test 1: Health check
echo "1. Testing /health endpoint:"
curl -s ${API_BASE}/health | jq . || echo "FAILED"
echo

# Test 2: Get current identity (should be not logged in initially)
echo "2. Testing /api/identity/current (should be logged out):"
curl -s ${API_BASE}/api/identity/current | jq . || echo "FAILED"
echo

# Test 3: Create new vault with auto-generated identity
echo "3. Testing /api/auth/vault (create vault with auto-generated identity):"
VAULT_RESPONSE=$(curl -s -X POST ${API_BASE}/api/auth/vault \
  -H "Content-Type: application/json" \
  -d '{"password":"test123","display_name":"Test User"}')
echo "$VAULT_RESPONSE" | jq .
FOUR_WORDS=$(echo "$VAULT_RESPONSE" | jq -r '.four_words')
echo "Created vault with identity: $FOUR_WORDS"
echo

# Test 4: Verify logged in after vault creation
echo "4. Testing /api/identity/current (should be logged in):"
curl -s ${API_BASE}/api/identity/current | jq . || echo "FAILED"
echo

# Test 5: Get network status
echo "5. Testing /api/network/status:"
curl -s ${API_BASE}/api/network/status | jq . || echo "FAILED"
echo

# Test 6: List entities (should be empty)
echo "6. Testing /api/entities (should be empty):"
curl -s ${API_BASE}/api/entities | jq . || echo "FAILED"
echo

# Test 7: Logout
echo "7. Testing /api/auth/logout:"
curl -s -X POST ${API_BASE}/api/auth/logout | jq . || echo "FAILED"
echo

# Test 8: Verify logged out
echo "8. Testing /api/identity/current (should be logged out):"
curl -s ${API_BASE}/api/identity/current | jq . || echo "FAILED"
echo

# Test 9: Login with existing vault
echo "9. Testing /api/auth/login (with identity: $FOUR_WORDS):"
curl -s -X POST ${API_BASE}/api/auth/login \
  -H "Content-Type: application/json" \
  -d "{\"four_words\":\"$FOUR_WORDS\",\"password\":\"test123\"}" | jq . || echo "FAILED"
echo

# Test 10: Verify logged in again
echo "10. Testing /api/identity/current (should be logged in):"
curl -s ${API_BASE}/api/identity/current | jq . || echo "FAILED"
echo

echo "============================================"
echo "All authentication tests completed!"
echo "============================================"
echo
echo "Summary of available endpoints:"
echo "  - GET  /health                       Health check"
echo "  - POST /api/auth/vault               Create vault & login"
echo "  - POST /api/auth/login               Login with existing vault"
echo "  - POST /api/auth/logout              Logout current session"
echo "  - GET  /api/identity/current         Get current identity"
echo "  - GET  /api/network/status           Network connection status"
echo "  - POST /api/entities                 Create entity"
echo "  - GET  /api/entities                 List all entities"
echo "  - POST /api/messages/send            Send message"
echo "  - GET  /api/entities/:id/messages    Get entity messages"
echo
