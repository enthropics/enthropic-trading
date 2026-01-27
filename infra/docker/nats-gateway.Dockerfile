# ===========================================
# Stage 1: Dependencies
# Install all workspace dependencies
# ===========================================
FROM node:20-alpine AS deps

WORKDIR /app

# Copy workspace configuration
COPY package.json package-lock.json ./
COPY nx.json tsconfig.base.json ./

COPY apps/nats-gateway/package.json ./apps/nats-gateway/
COPY libs/shared ./libs/shared

# Install dependencies using npm ci for reproducible builds
RUN npm ci --workspace=apps/nats-gateway

# ===========================================
# Stage 2: Builder
# Build the application
# ===========================================
FROM node:20-alpine AS builder

WORKDIR /app

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/package.json ./package.json

# Copy workspace configuration
COPY nx.json tsconfig.base.json ./

# Copy shared libraries
COPY libs ./libs

# Copy nats-gateway source code
COPY apps/nats-gateway ./apps/nats-gateway

# Build the service
WORKDIR /app/apps/nats-gateway
RUN npm run build

# ===========================================
# Stage 3: Production Dependencies
# Install only production dependencies
# ===========================================
FROM node:20-alpine AS prod-deps

WORKDIR /app

# Copy workspace configuration
COPY package.json package-lock.json ./

# Copy app package.json
COPY apps/nats-gateway/package.json ./apps/nats-gateway/

# Copy all libs/shared
COPY libs/shared ./libs/shared

# Install production dependencies only
RUN npm ci --workspace=apps/nats-gateway --omit=dev

# ===========================================
# Stage 4: Production Runtime
# ===========================================
FROM node:20-alpine AS production

# Install dumb-init for proper signal handling
RUN apk add --no-cache dumb-init

WORKDIR /app

# Copy production dependencies
COPY --from=prod-deps /app/node_modules ./node_modules

# Copy built application
COPY --from=builder /app/apps/nats-gateway/dist ./dist
COPY --from=builder /app/apps/nats-gateway/package.json ./package.json

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001 && \
    chown -R nodejs:nodejs /app

USER nodejs

EXPOSE 3002

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD node -e "require('http').get('http://localhost:3002/health', (r) => {process.exit(r.statusCode === 200 ? 0 : 1)})"

# Use dumb-init to handle signals properly
ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "dist/index.js"]
