#!/bin/bash

# Setup script for PostgreSQL MCP Server

echo "Starting PostgreSQL Docker container..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Error: Docker is not running. Please start Docker and try again."
    exit 1
fi

# Start PostgreSQL container
docker-compose up -d

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
sleep 5

# Check if container is healthy
docker-compose ps

echo ""
echo "PostgreSQL is ready!"
echo "Connection string: postgresql://postgres:postgres@localhost:5432/testdb"
echo ""
echo "To use with MCP server, set:"
echo "export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/testdb"
echo ""
echo "To allow write operations:"
echo "export DANGEROUSLY_ALLOW_WRITE_OPS=true"
