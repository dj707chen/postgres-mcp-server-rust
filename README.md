# PostgreSQL MCP Server (Rust)

A Model Context Protocol (MCP) server implementation in Rust that provides PostgreSQL database access through tools and resources.

**⚡ Quick Start:** See [QUICKSTART.md](QUICKSTART.md) to get running in 5 minutes!

## Features

- **Query Tool**: Execute SQL queries against PostgreSQL database
  - Read-only by default
  - Optional write operations (INSERT, UPDATE, DELETE, etc.)
- **Resources**: Access database tables as MCP resources
  - List all tables in the public schema
  - Read table contents (up to 100 rows)
- **Environment Configuration**: Configure database connection via `DATABASE_URL`
- **Safety**: Write operations disabled by default, require explicit opt-in

## Prerequisites

- Rust (stable toolchain)
- Docker and Docker Compose
- PostgreSQL client (optional, for testing)

## Quick Start

### 1. Start PostgreSQL Database

First, start Docker, then run:

```bash
./setup.sh
```

Or manually:

```bash
docker-compose up -d
```

This creates a PostgreSQL 17 instance with sample data (users, products, orders tables).

### 2. Build the MCP Server

```bash
cargo build --release
```

### 3. Run the MCP Server

Set the database connection URL:

```bash
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/testdb"
```

For write operations (optional):

```bash
export DANGEROUSLY_ALLOW_WRITE_OPS=true
```

Run the server:

```bash
cargo run
```

## MCP Integration

### Configuration

Add to your MCP client configuration (e.g., Claude Desktop):

```json
{
  "mcpServers": {
    "postgres": {
      "command": "/path/to/postgres-mcp-server-rust/target/release/postgres_mcp_server",
      "env": {
        "DATABASE_URL": "postgresql://postgres:postgres@localhost:5432/testdb"
      }
    }
  }
}
```

### Available Tools

#### `query`

Execute SQL queries against the database.

**Parameters:**
- `sql` (string, required): The SQL query to execute

**Example:**
```json
{
  "name": "query",
  "arguments": {
    "sql": "SELECT * FROM users WHERE age > 30"
  }
}
```

**Read-only mode**: Only SELECT queries are allowed by default.

**Write mode**: Set `DANGEROUSLY_ALLOW_WRITE_OPS=true` to enable INSERT, UPDATE, DELETE, CREATE, DROP, ALTER, and TRUNCATE operations.

### Available Resources

The server exposes database tables as resources with URIs in the format:
- `postgres:///table_name`

**Resource capabilities:**
- List all tables in the public schema
- Read table contents (limited to 100 rows per table)

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | - | PostgreSQL connection string (format: `postgresql://user:password@host:port/database`) |
| `DANGEROUSLY_ALLOW_WRITE_OPS` | No | `false` | Enable write operations (`true` or `1` to enable) |

## Sample Data

The included `init.sql` script creates three tables with sample data:

- **users**: User accounts with name, email, age, and status
- **products**: Product catalog with descriptions and pricing
- **orders**: Order records linking users and products

## Development

### Project Structure

```
postgres-mcp-server-rust/
├── src/
│   └── main.rs           # MCP server implementation
├── Cargo.toml            # Rust dependencies
├── docker-compose.yml    # PostgreSQL container setup
├── init.sql              # Database initialization script
└── setup.sh              # Quick setup script
```

### Testing

1. Start PostgreSQL: `./setup.sh`
2. Build the server: `cargo build`
3. Set environment variables
4. Run the server: `cargo run`
5. Send JSON-RPC requests via stdin

### Example JSON-RPC Requests

**Initialize:**
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

**List Tools:**
```json
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

**Execute Query:**
```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query","arguments":{"sql":"SELECT * FROM users LIMIT 5"}}}
```

**List Resources:**
```json
{"jsonrpc":"2.0","id":4,"method":"resources/list"}
```

**Read Resource:**
```json
{"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"postgres:///users"}}
```

## Security Considerations

- By default, only SELECT queries are permitted
- Write operations require explicit environment variable configuration
- SQL injection protection is the responsibility of the query author
- Use parameterized queries when possible
- Always validate and sanitize user input before constructing SQL queries

## License

MIT

## Contributing

Contributions welcome! Please open an issue or pull request.
