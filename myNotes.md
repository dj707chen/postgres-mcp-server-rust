# 

Based on https://cursor.com/docs/cookbook/building-mcp-server#how-to-build-the-mcp-server

```shell
cd ~/AI/MCP
gh repo clone dj707chen/postgres-mcp-server-rust
cd postgres-mcp-server-rust

# Initial prompt
cat > 00prompt.txt <<'END'
Read the following and follow @spec.md to understand what we want. All necessary dependencies are installed
- @https://raw.githubusercontent.com/modelcontextprotocol/typescript-sdk/refs/heads/main/README.md
- @https://raw.githubusercontent.com/porsager/postgres/refs/heads/master/README.md
END

# execute
export DATABASE_URL=postgres://localhost/test
npx @modelcontextprotocol/inspector bun run index.ts
```

