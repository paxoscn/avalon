#!/bin/bash

# Test dashboard statistics endpoint

BASE_URL="http://localhost:3000/api"

echo "Testing Dashboard Statistics Endpoint"
echo "======================================"
echo ""

# First, login to get a token
echo "1. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }')

TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
  echo "❌ Login failed"
  echo "Response: $LOGIN_RESPONSE"
  exit 1
fi

echo "✅ Login successful"
echo ""

# Test dashboard stats endpoint
echo "2. Fetching dashboard statistics..."
STATS_RESPONSE=$(curl -s -X GET "$BASE_URL/dashboard/stats" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json")

echo "Response:"
echo "$STATS_RESPONSE" | jq '.' 2>/dev/null || echo "$STATS_RESPONSE"
echo ""

# Check if response contains expected fields
if echo "$STATS_RESPONSE" | grep -q "agents_count"; then
  echo "✅ Dashboard stats endpoint working correctly"
  
  # Extract and display counts
  AGENTS=$(echo "$STATS_RESPONSE" | grep -o '"agents_count":[0-9]*' | cut -d':' -f2)
  FLOWS=$(echo "$STATS_RESPONSE" | grep -o '"flows_count":[0-9]*' | cut -d':' -f2)
  MCP=$(echo "$STATS_RESPONSE" | grep -o '"mcp_tools_count":[0-9]*' | cut -d':' -f2)
  KB=$(echo "$STATS_RESPONSE" | grep -o '"knowledge_bases_count":[0-9]*' | cut -d':' -f2)
  SESSIONS=$(echo "$STATS_RESPONSE" | grep -o '"sessions_count":[0-9]*' | cut -d':' -f2)
  
  echo ""
  echo "Statistics Summary:"
  echo "  - Agents: $AGENTS"
  echo "  - Flows: $FLOWS"
  echo "  - MCP Tools: $MCP"
  echo "  - Knowledge Bases: $KB"
  echo "  - Sessions: $SESSIONS"
else
  echo "❌ Dashboard stats endpoint returned unexpected response"
  exit 1
fi

echo ""
echo "======================================"
echo "✅ All tests passed!"
