#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Enthropic Trading Platform - Dockerfile Optimizer        ║${NC}"
echo -e "${BLUE}║  Best Practices for npm Workspace & Multi-stage Builds    ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -f "docker-compose.yml" ]; then
    echo -e "${RED}❌ Error: This script must be run from the project root directory${NC}"
    echo -e "${YELLOW}   Please cd to the directory containing package.json and docker-compose.yml${NC}"
    exit 1
fi

echo -e "${YELLOW}📋 This script will:${NC}"
echo -e "   1. Backup your current Dockerfiles"
echo -e "   2. Deploy optimized Dockerfiles with:"
echo -e "      • Proper npm workspace support"
echo -e "      • Multi-stage builds for minimal image size"
echo -e "      • Better layer caching"
echo -e "      • Security best practices (non-root users)"
echo -e "      • Health checks"
echo -e "   3. Create/update .dockerignore for faster builds"
echo ""

read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Cancelled.${NC}"
    exit 0
fi

echo ""
echo -e "${GREEN}🚀 Starting deployment...${NC}"
echo ""

# Create backup directory
BACKUP_DIR="infra/docker/backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

echo -e "${BLUE}📦 Backing up current Dockerfiles...${NC}"

# Backup existing Dockerfiles
for dockerfile in infra/docker/*.Dockerfile; do
    if [ -f "$dockerfile" ]; then
        filename=$(basename "$dockerfile")
        cp "$dockerfile" "$BACKUP_DIR/$filename"
        echo -e "   ✓ Backed up: $filename"
    fi
done

echo -e "${GREEN}   ✅ Backup completed: $BACKUP_DIR${NC}"
echo ""

# Check if optimized Dockerfiles exist
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo -e "${BLUE}📝 Deploying optimized Dockerfiles...${NC}"

# Copy new Dockerfiles
declare -A dockerfiles=(
    ["dashboard.Dockerfile"]="Dashboard (React + Vite + Nginx)"
    ["nats-gateway.Dockerfile"]="NATS Gateway (Node.js)"
    ["risk-service.Dockerfile"]="Risk Service (NestJS + Prisma)"
    ["execution-core.Dockerfile"]="Execution Core (Rust)"
    ["strategy-service.Dockerfile"]="Strategy Service (Python)"
)

for dockerfile in "${!dockerfiles[@]}"; do
    source_file="$SCRIPT_DIR/$dockerfile"
    dest_file="infra/docker/$dockerfile"
    
    if [ -f "$source_file" ]; then
        cp "$source_file" "$dest_file"
        echo -e "   ✓ Deployed: ${dockerfiles[$dockerfile]}"
    else
        echo -e "   ${YELLOW}⚠ Skipped: $dockerfile (not found in script directory)${NC}"
    fi
done

echo -e "${GREEN}   ✅ Dockerfiles deployed${NC}"
echo ""

# Deploy .dockerignore
echo -e "${BLUE}📝 Deploying .dockerignore...${NC}"
if [ -f "$SCRIPT_DIR/.dockerignore" ]; then
    cp "$SCRIPT_DIR/.dockerignore" ".dockerignore"
    echo -e "${GREEN}   ✅ .dockerignore deployed${NC}"
else
    echo -e "${YELLOW}   ⚠ .dockerignore not found, skipping${NC}"
fi
echo ""

# Check for package-lock.json
echo -e "${BLUE}🔍 Checking npm workspace setup...${NC}"
if [ ! -f "package-lock.json" ]; then
    echo -e "${YELLOW}   ⚠ package-lock.json not found${NC}"
    echo -e "${YELLOW}   ℹ️  Generating package-lock.json...${NC}"
    npm install
    echo -e "${GREEN}   ✅ package-lock.json generated${NC}"
else
    echo -e "${GREEN}   ✅ package-lock.json exists${NC}"
fi
echo ""

echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✨ Deployment Complete!                                   ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}📊 What changed:${NC}"
echo ""
echo -e "${BLUE}Dashboard:${NC}"
echo -e "   • 3-stage build: deps → builder → nginx"
echo -e "   • Workspace-aware dependency installation"
echo -e "   • Optimized nginx config with gzip & caching"
echo -e "   • Health check endpoint"
echo ""
echo -e "${BLUE}NATS Gateway & Risk Service:${NC}"
echo -e "   • 4-stage build: deps → builder → prod-deps → runtime"
echo -e "   • Separate production dependencies"
echo -e "   • Non-root user for security"
echo -e "   • Proper signal handling with dumb-init"
echo ""
echo -e "${BLUE}Execution Core (Rust):${NC}"
echo -e "   • Cached dependency builds"
echo -e "   • Minimal Debian runtime"
echo -e "   • Non-root user"
echo ""
echo -e "${BLUE}Strategy Service (Python):${NC}"
echo -e "   • 2-stage build: deps → runtime"
echo -e "   • Minimal runtime dependencies"
echo -e "   • Non-root user"
echo ""
echo -e "${YELLOW}🎯 Next steps:${NC}"
echo ""
echo -e "   1. Review the changes:"
echo -e "      ${BLUE}diff -r infra/docker $BACKUP_DIR${NC}"
echo ""
echo -e "   2. Build and test:"
echo -e "      ${BLUE}docker-compose build${NC}"
echo ""
echo -e "   3. Start services:"
echo -e "      ${BLUE}docker-compose up -d${NC}"
echo ""
echo -e "   4. Check logs:"
echo -e "      ${BLUE}docker-compose logs -f${NC}"
echo ""
echo -e "${GREEN}💡 Tips:${NC}"
echo -e "   • First build will take longer (downloading layers)"
echo -e "   • Subsequent builds will be faster (layer caching)"
echo -e "   • Images will be smaller (multi-stage builds)"
echo -e "   • More secure (non-root users, minimal attack surface)"
echo ""
echo -e "${YELLOW}📁 Backup location: $BACKUP_DIR${NC}"
echo -e "${YELLOW}   (You can restore from here if needed)${NC}"
echo ""
