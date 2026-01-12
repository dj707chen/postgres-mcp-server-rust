# Usage Guide

This guide provides detailed instructions on how to use the PostgreSQL MCP Server.

## Getting Started

### 1. Start PostgreSQL Database

Start the PostgreSQL Docker container:

```bash
# Start Docker first (if not already running)
# Then run:
./setup.sh
```

This will:
- Start a PostgreSQL 17 container
- Create a test database with sample tables (users, products, orders)
- Expose PostgreSQL on port 5432

### 2. Build the Server

```bash
cargo build --release
```

The binary will be located at `./target/release/postgres_mcp_server`

### 3. Configure MCP Client

For Claude Desktop, edit your config file:
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

Add the server configuration (replace `/path/to/` with your actual path):

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

### 4. Restart Claude Desktop

Restart Claude Desktop to load the new MCP server.

## Using the MCP Server

### Query Tool

The `query` tool allows you to execute SQL queries.

#### Example: Select Data

```sql
SELECT * FROM users WHERE age > 30
```

In Claude:
> "Use the query tool to show me all users older than 30"

#### Example: Join Tables

```sql
SELECT u.name, p.name as product, o.quantity
FROM orders o
JOIN users u ON o.user_id = u.id
JOIN products p ON o.product_id = p.id
```

In Claude:
> "Show me all orders with user names and product names"

#### Example: Aggregate Data

```sql
SELECT p.name, SUM(o.quantity) as total_sold
FROM orders o
JOIN products p ON o.product_id = p.id
GROUP BY p.id, p.name
ORDER BY total_sold DESC
```

In Claude:
> "Which products have been ordered the most?"

### Resources

Database tables are exposed as resources. You can ask Claude to:

1. **List all tables**:
   > "What tables are available in the database?"

2. **View table contents**:
   > "Show me the contents of the users table"

3. **Explore table structure**:
   > "What data is in the products table?"

## Write Operations

By default, write operations (INSERT, UPDATE, DELETE, etc.) are **disabled** for safety.

### Enabling Write Operations

To enable write operations, add the environment variable:

```json
{
  "mcpServers": {
    "postgres": {
      "command": "/path/to/postgres-mcp-server-rust/target/release/postgres_mcp_server",
      "env": {
        "DATABASE_URL": "postgresql://postgres:postgres@localhost:5432/testdb",
        "DANGEROUSLY_ALLOW_WRITE_OPS": "true"
      }
    }
  }
}
```

### Example Write Operations

**Insert:**
```sql
INSERT INTO users (name, email, age, active)
VALUES ('John Doe', 'john@example.com', 32, true)
```

**Update:**
```sql
UPDATE products
SET price = 99.99
WHERE name = 'Wireless Mouse'
```

**Delete:**
```sql
DELETE FROM orders
WHERE id = 7
```

**Create Table:**
```sql
CREATE TABLE categories (
  id SERIAL PRIMARY KEY,
  name VARCHAR(100) NOT NULL
)
```

## Manual Testing

You can test the server manually using stdin/stdout:

```bash
# Set environment variable
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/testdb"

# Run the server
cargo run --release

# Then send JSON-RPC requests (one per line):
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query","arguments":{"sql":"SELECT * FROM users LIMIT 5"}}}
```

Or use the test script:

```bash
./test_mcp.sh
```

## Connecting to Your Own Database

Replace the DATABASE_URL with your own PostgreSQL connection string:

```bash
postgresql://username:password@hostname:port/database_name
```

Examples:

```bash
# Local database
postgresql://postgres:mypassword@localhost:5432/mydb

# Remote database
postgresql://user:pass@db.example.com:5432/production

# With SSL
postgresql://user:pass@db.example.com:5432/mydb?sslmode=require
```

## Troubleshooting

### Server won't start

1. Check DATABASE_URL is set correctly
2. Verify PostgreSQL is running: `docker ps` or `nc -z localhost 5432`
3. Test connection: `psql $DATABASE_URL -c "SELECT 1"`

### Connection refused

- Ensure PostgreSQL container is running: `docker-compose ps`
- Check port 5432 is not blocked by firewall
- Verify DATABASE_URL has correct hostname and port

### Write operations not working

- Ensure `DANGEROUSLY_ALLOW_WRITE_OPS=true` is set in the MCP config
- Restart Claude Desktop after changing config

### No tables listed in resources

- Check that tables exist in the `public` schema
- Run: `psql $DATABASE_URL -c "\dt"`
- The server only shows tables in the `public` schema

## Advanced Usage

### Custom Connection Parameters

Add connection parameters to the DATABASE_URL:

```bash
# Set connection timeout
postgresql://user:pass@host:5432/db?connect_timeout=10

# Set application name
postgresql://user:pass@host:5432/db?application_name=mcp_server

# Multiple parameters
postgresql://user:pass@host:5432/db?connect_timeout=10&application_name=mcp_server
```

### Using Environment Files

Create a `.env` file:

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/testdb
DANGEROUSLY_ALLOW_WRITE_OPS=false
```

Load it before running:

```bash
export $(cat .env | xargs)
cargo run --release
```

### Multiple Database Connections

You can configure multiple MCP server instances for different databases:

```json
{
  "mcpServers": {
    "postgres-dev": {
      "command": "/path/to/postgres_mcp_server",
      "env": {
        "DATABASE_URL": "postgresql://postgres:postgres@localhost:5432/dev_db"
      }
    },
    "postgres-staging": {
      "command": "/path/to/postgres_mcp_server",
      "env": {
        "DATABASE_URL": "postgresql://user:pass@staging.example.com:5432/staging_db"
      }
    }
  }
}
```

## Sample Queries

Here are some useful queries to try:

```sql
-- Count users by active status
SELECT active, COUNT(*) as count
FROM users
GROUP BY active;

-- Products low on stock
SELECT name, stock
FROM products
WHERE stock < 100
ORDER BY stock ASC;

-- Revenue by product
SELECT p.name, SUM(o.total_price) as revenue
FROM orders o
JOIN products p ON o.product_id = p.id
GROUP BY p.id, p.name
ORDER BY revenue DESC;

-- Recent orders
SELECT u.name as customer, p.name as product, o.quantity, o.order_date
FROM orders o
JOIN users u ON o.user_id = u.id
JOIN products p ON o.product_id = p.id
ORDER BY o.order_date DESC
LIMIT 10;
```

## Best Practices

1. **Start with SELECT**: Always test queries with SELECT before enabling writes
2. **Limit Results**: Use LIMIT to avoid overwhelming responses
3. **Backup Data**: Backup your database before enabling write operations
4. **Use Transactions**: For multiple related writes, consider using transactions
5. **Validate Input**: Never directly interpolate user input into SQL queries
6. **Monitor Logs**: Check the server logs for errors and warnings

## Security Notes

- The server does NOT use prepared statements by default
- SQL injection is possible if queries are constructed from untrusted input
- Always validate and sanitize any user-provided data
- Consider using views to restrict access to sensitive columns
- Use PostgreSQL roles and permissions to limit what the database user can do
- Never expose write access to production databases without proper safeguards
