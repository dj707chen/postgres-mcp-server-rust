#!/usr/bin/env bash

# based on README.md section: 3. Run the MCP Server

export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/testdb"
export DANGEROUSLY_ALLOW_WRITE_OPS=true
cargo run

echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query","arguments":{"sql":"SELECT * FROM users WHERE age > 30"}}}' \
    | $HOME/AI/MCP/postgres-mcp-server-rust/target/release/postgres_mcp_server \
    | jq