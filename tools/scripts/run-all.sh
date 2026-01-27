#!/bin/bash
set -e

echo "=== Starting All Services ==="

docker-compose up -d

echo "Waiting for services to be healthy..."
sleep 10

echo ""
echo "=== All Services Running ==="
echo "  - Dashboard: http://localhost:5173"
echo "  - NATS Gateway: ws://localhost:3002"
echo "  - Risk Service: http://localhost:3001"
echo "  - Vault UI: http://localhost:8200"
