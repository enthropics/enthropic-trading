#!/bin/bash
set -e

echo "=== Enthropic Trading Platform - Development Setup ==="

# Check prerequisites
command -v docker >/dev/null 2>&1 || { echo "Docker required"; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "Docker Compose required"; exit 1; }

# Create .env from example if not exists
if [ ! -f .env ]; then
    cp .env.example .env
    echo "Created .env from .env.example"
fi

# Start infrastructure
echo "Starting infrastructure services..."
docker-compose up -d postgres redis nats vault

# Wait for Postgres
echo "Waiting for PostgreSQL..."
until docker-compose exec -T postgres pg_isready -U postgres; do
    sleep 2
done

# Initialize Vault (dev mode)
echo "Vault is running in dev mode with token: dev-token-123"

echo ""
echo "=== Setup Complete ==="
echo "Services running:"
echo "  - PostgreSQL: localhost:5432"
echo "  - Redis: localhost:6379"
echo "  - NATS: localhost:4222"
echo "  - Vault: http://localhost:8200"
echo ""
echo "Demo accounts:"
echo "  - admin / admin123"
echo "  - trader1 / trader123"
echo "  - viewer1 / viewer123"
