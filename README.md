# PostgreSQL MCP Server (HTTP-based)

An HTTP-based Model Context Protocol (MCP) server that provides access to PostgreSQL databases.

## Features

- **HTTP-based MCP Server** - JSON-RPC 2.0 over HTTP
- **Query Tool** - Execute SQL queries against PostgreSQL
- **Table Resources** - Access database tables as MCP resources
- **Read-only by default** - Write operations require explicit opt-in
- **Type-safe** - Comprehensive PostgreSQL type support

## Architecture

```
[MCP Client] <--HTTP/JSON-RPC--> [Axum Server] <--TCP--> [PostgreSQL 17.6]
```

## Quick Start

### 1. Prerequisites

- Rust (1.80+)
- Docker and Docker Compose
- PostgreSQL 17.6 (via Docker)

### 2. Start PostgreSQL

```bash
docker-compose up -d
```

This starts PostgreSQL 17.6 with sample data (users, products, orders tables).

### 3. Set Environment Variables

```bash
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/testdb
export MCP_SERVER_HOST=127.0.0.1
export MCP_SERVER_PORT=8080
export RUST_LOG=info
```

For write operations:
```bash
export DANGEROUSLY_ALLOW_WRITE_OPS=true
```

### 4. Run the Server

```bash
cargo run --release
```

The server will start on `http://127.0.0.1:8080`.

### 5. Test with Health Check

```bash
curl http://localhost:8080/health
```

Expected response: `OK`

## MCP Protocol Usage

### Initialize

```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {}
  }'
```

Response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "tools": {},
      "resources": {}
    },
    "serverInfo": {
      "name": "postgres-mcp-server",
      "version": "0.2.0"
    }
  }
}
```

### List Tools

```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
  }'
```

### Call Query Tool

```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "query",
      "arguments": {
        "sql": "SELECT * FROM users LIMIT 5"
      }
    }
  }'
```

### List Resources (Tables)

```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "resources/list",
    "params": {}
  }'
```

### Read Resource (Table Data)

```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 5,
    "method": "resources/read",
    "params": {
      "uri": "postgres:///users"
    }
  }'
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `DANGEROUSLY_ALLOW_WRITE_OPS` | Enable write operations (`true` or `1`) | `false` |
| `MCP_SERVER_HOST` | Server bind address | `127.0.0.1` |
| `MCP_SERVER_PORT` | Server port | `8080` |
| `RUST_LOG` | Logging level (`trace`, `debug`, `info`, `warn`, `error`) | `info` |

### Using .env File

Copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

Then edit `.env` with your configuration.

## Sample Database

The Docker setup includes sample tables:

- **users** - User accounts (id, name, email, age, active, created_at)
- **products** - Product catalog (id, name, description, price, stock, created_at)
- **orders** - Order history (id, user_id, product_id, quantity, total_price, order_date)

## Security Considerations

- **Localhost only** - Server binds to `127.0.0.1` by default
- **Read-only by default** - Write operations require explicit opt-in
- **No authentication** - Suitable for local development only
- **SQL injection risk** - Do not expose to untrusted input

## Type Support

The server supports the following PostgreSQL types:

- Integers: `int2`, `int4`, `int8`
- Text: `text`, `varchar`, `bpchar`, `name`
- Boolean: `bool`
- Floats: `float4`, `float8`
- Numeric: `numeric`
- Timestamps: `timestamp`, `timestamptz`, `date`, `time`, `timetz`
- UUID: `uuid`
- JSON: `json`, `jsonb`

## Testing

### Manual Testing

```bash
# Terminal 1: Start PostgreSQL
docker-compose up -d

# Terminal 2: Start MCP server
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/testdb
cargo run

# Terminal 3: Test queries
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
```

### With MCP Inspector

```bash
npx @modelcontextprotocol/inspector
```

Then configure the inspector to use `http://localhost:8080` with HTTP transport.

## Project Structure

```
postgres-mcp-server-rust/
├── src/
│   ├── main.rs           # HTTP server with Axum
│   ├── handler.rs        # MCP protocol handler
│   ├── db.rs             # PostgreSQL client wrapper
│   ├── tools.rs          # Query tool implementation
│   └── resources.rs      # Table resources implementation
├── Cargo.toml            # Rust dependencies
├── docker-compose.yml    # PostgreSQL 17.6 setup
├── init.sql              # Sample database schema
├── .env.example          # Environment variables template
└── README.md             # This file
```

## Building for Production

```bash
cargo build --release
```

The optimized binary will be in `target/release/postgres_mcp_server`.

## Limitations

- Single database connection (no connection pooling)
- No authentication mechanism
- No TLS/SSL support
- 100-row limit for table resources
- Not suitable for production use without additional hardening

## Future Enhancements

- [ ] Connection pooling (deadpool-postgres)
- [ ] Authentication (API keys, OAuth)
- [ ] TLS/SSL support
- [ ] Prepared statements
- [ ] Query pagination
- [ ] Schema support (non-public schemas)
- [ ] Transaction support
- [ ] Health/metrics endpoints
- [ ] Rate limiting

## License

MIT

## Version

Current version: 0.2.0

Protocol version: 2025-11-25 (MCP)
