#!/bin/bash

# Test script for MCP server
# This script sends JSON-RPC requests to the MCP server via stdin

set -e

echo "Testing PostgreSQL MCP Server"
echo "=============================="
echo ""

# Set environment variables
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/testdb"

# Build the server
echo "Building server..."
cargo build --release
echo ""

# Start the server in the background
echo "Starting MCP server..."
SERVER_PID=""

# Function to cleanup on exit
cleanup() {
    if [ ! -z "$SERVER_PID" ]; then
        echo "Stopping server..."
        kill $SERVER_PID 2>/dev/null || true
    fi
}

trap cleanup EXIT

# Test 1: Initialize
echo "Test 1: Initialize"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run --release 2>/dev/null | head -1 | jq .
echo ""

# Test 2: List tools
echo "Test 2: List tools"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | cargo run --release 2>/dev/null | head -1 | jq .
echo ""

# Test 3: Execute query (if PostgreSQL is running)
if nc -z localhost 5432 2>/dev/null; then
    echo "Test 3: Execute query (SELECT from users)"
    echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query","arguments":{"sql":"SELECT * FROM users LIMIT 3"}}}' | cargo run --release 2>/dev/null | head -1 | jq .
    echo ""

    echo "Test 4: List resources (tables)"
    echo '{"jsonrpc":"2.0","id":4,"method":"resources/list"}' | cargo run --release 2>/dev/null | head -1 | jq .
    echo ""

    echo "Test 5: Read resource (users table)"
    echo '{"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"postgres:///users"}}' | cargo run --release 2>/dev/null | head -1 | jq .
    echo ""
else
    echo "PostgreSQL is not running on localhost:5432"
    echo "Start it with: ./setup.sh"
    echo ""
fi

echo "Tests complete!"
