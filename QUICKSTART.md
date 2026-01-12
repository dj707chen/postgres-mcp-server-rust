# Quick Start Guide

Get up and running with the PostgreSQL MCP Server in 5 minutes.

## Prerequisites

- Docker installed and running
- Rust installed (if building from source)
- Claude Desktop (or another MCP client)

## Step 1: Start PostgreSQL

```bash
# Make setup script executable
chmod +x setup.sh

# Start Docker, then run:
./setup.sh
```

You should see:
```
PostgreSQL is ready!
Connection string: postgresql://postgres:postgres@localhost:5432/testdb
```

## Step 2: Build the Server

```bash
cargo build --release
```

Binary will be at: `target/release/postgres_mcp_server`

## Step 3: Configure Claude Desktop

**macOS:** Edit `~/Library/Application Support/Claude/claude_desktop_config.json`

**Windows:** Edit `%APPDATA%\Claude\claude_desktop_config.json`

Add this (replace `/path/to/` with your actual project path):

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

**To get your full path:**
```bash
pwd
# Then append: /target/release/postgres_mcp_server
```

## Step 4: Restart Claude Desktop

Completely quit and restart Claude Desktop.

## Step 5: Test It

In Claude Desktop, try:

> "What tables are available in the PostgreSQL database?"

> "Show me all users in the database"

> "Which products have been ordered the most?"

## That's It!

You now have a working PostgreSQL MCP server.

## Next Steps

- Read [USAGE.md](USAGE.md) for detailed usage examples
- See [README.md](README.md) for full documentation
- Check [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md) for implementation details

## Troubleshooting

**PostgreSQL not starting?**
- Make sure Docker is running
- Check if port 5432 is already in use: `lsof -i :5432`

**Server not showing in Claude?**
- Verify the config file path is correct
- Check the binary path in config is absolute (starts with `/`)
- Look at Claude Desktop logs for errors

**Can't connect to database?**
- Verify PostgreSQL is running: `docker ps`
- Test connection: `psql postgresql://postgres:postgres@localhost:5432/testdb -c "SELECT 1"`

## Need Help?

Check the full documentation:
- [README.md](README.md) - Overview and features
- [USAGE.md](USAGE.md) - Detailed usage guide
- [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md) - Technical details
