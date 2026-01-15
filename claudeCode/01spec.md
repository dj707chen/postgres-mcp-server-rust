# Spec

- Plan to create a MCP server in Rust, accepting HTTP connections and queries
- Use Rust crates rust-mcp-sdk, tokio, axum, reqwest
- Allow defining DATABASE_URL through MCP env configuration
- Query postgres data through tool
  - By default, make it readonly
  - Allow write ops by setting ENV `DANGEROUSLY_ALLOW_WRITE_OPS=true|1`
- Access tables as `resources`
- Set up PostgreSQL 17.6 server by Docker
- Get the DB URL and set it to shell environment variable when launch MCP inspect running the MCP server
