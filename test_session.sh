#!/bin/bash

# Test script for session endpoints
BASE_URL="http://localhost:3000"

echo "Testing Session Endpoints"
echo "========================="

# Test 1: Create a session
echo "1. Creating a session..."
SESSION_RESPONSE=$(curl -s -X POST $BASE_URL/sessions \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Meetup",
    "creator_name": "Test Creator",
    "settings": {
      "categories": ["restaurant", "cafe"],
      "transport_mode": "DRIVE",
      "limit": 10,
      "auto_refresh": true,
      "require_all_participants": false
    }
  }')

echo "Session Response: $SESSION_RESPONSE"

# Extract session_id and join_code from response
SESSION_ID=$(echo $SESSION_RESPONSE | jq -r '.session_id')
JOIN_CODE=$(echo $SESSION_RESPONSE | jq -r '.join_code')

echo "Session ID: $SESSION_ID"
echo "Join Code: $JOIN_CODE"

# Test 2: Join the session
echo ""
echo "2. Joining the session..."
JOIN_RESPONSE=$(curl -s -X POST $BASE_URL/sessions/join \
  -H "Content-Type: application/json" \
  -d "{
    \"join_code\": \"$JOIN_CODE\",
    \"participant_name\": \"Test User\",
    \"input\": {
      \"Address\": \"San Francisco, CA\"
    }
  }")

echo "Join Response: $JOIN_RESPONSE"

# Extract user_id
USER_ID=$(echo $JOIN_RESPONSE | jq -r '.user_id')
echo "User ID: $USER_ID"

# Test 3: Get session status
echo ""
echo "3. Getting session status..."
STATUS_RESPONSE=$(curl -s -X GET $BASE_URL/sessions/$SESSION_ID)
echo "Status Response: $STATUS_RESPONSE"

# Test 4: Health check
echo ""
echo "4. Health check..."
HEALTH_RESPONSE=$(curl -s -X GET $BASE_URL/health)
echo "Health Response: $HEALTH_RESPONSE"

echo ""
echo "Tests completed!"
