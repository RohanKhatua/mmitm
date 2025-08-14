#!/bin/bash

# Test script for the MMITM API

BASE_URL="http://localhost:3000"

echo "Testing MMITM API..."

# Test health endpoint
echo "1. Testing health endpoint..."
curl -s "$BASE_URL/health" | jq '.'

echo -e "\n\n2. Testing recommendations endpoint..."
curl -s -X POST "$BASE_URL/recommendations" \
  -H "Content-Type: application/json" \
  -d @test_request.json | jq '.'

echo -e "\n\nTest completed!"
