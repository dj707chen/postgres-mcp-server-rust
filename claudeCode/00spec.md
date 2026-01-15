# Spec

- Plan to create a MCP server in Rust
- Use Rust crates tokio, axum, reqwest
- Allow defining DATABASE_URL through MCP env configuration
- Query postgres data through tool
  - By default, make it readonly
  - Allow write ops by setting ENV `DANGEROUSLY_ALLOW_WRITE_OPS=true|1`
- Access tables as `resources`
- Set up PostgreSQL 17.7 server by Docker
- Get the DB URL and set it to shell environment variable when launch MCP inspect running the MCP server
