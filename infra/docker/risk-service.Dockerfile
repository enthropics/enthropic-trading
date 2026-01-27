# ===========================================
# Stage 1: Dependencies
# Install all workspace dependencies
# ===========================================
FROM node:20-alpine AS deps

WORKDIR /app

# Install OpenSSL for Prisma
RUN apk add --no-cache openssl

# Copy workspace configuration
COPY package.json package-lock.json ./
COPY nx.json tsconfig.base.json ./

# Copy all workspace package.json files for dependency resolution
COPY apps/risk-service/package.json ./apps/risk-service/
COPY libs/shared ./libs/shared

# Install dependencies using npm ci for reproducible builds
RUN npm ci --workspace=apps/risk-service

# ===========================================
# Stage 2: Builder
# Build the application
# ===========================================
FROM node:20-alpine AS builder

WORKDIR /app

# Install OpenSSL for Prisma
RUN apk add --no-cache openssl

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/package.json ./package.json

# Copy workspace configuration
COPY nx.json tsconfig.base.json ./

# Copy shared libraries
COPY libs ./libs

# Copy risk-service source code and configs
COPY apps/risk-service ./apps/risk-service

# Generate Prisma Client
WORKDIR /app/apps/risk-service
RUN npx prisma generate

# Build the service
RUN npm run build

# ===========================================
# Stage 3: Production Dependencies
# Install only production dependencies
# ===========================================
FROM node:20-alpine AS prod-deps

WORKDIR /app

# Install OpenSSL for Prisma
RUN apk add --no-cache openssl

# Copy workspace configuration
COPY package.json package-lock.json ./

# Copy app package.json
COPY apps/risk-service/package.json ./apps/risk-service/

# Copy Prisma schema to generate client
COPY apps/risk-service/prisma ./apps/risk-service/prisma

# Copy all libs/shared
COPY libs/shared ./libs/shared

# Install production dependencies only
RUN npm ci --workspace=apps/risk-service --omit=dev

# Generate Prisma Client for production
WORKDIR /app/apps/risk-service
RUN npx prisma generate

# ===========================================
# Stage 4: Production Runtime
# ===========================================
FROM node:20-alpine AS production

# Install required runtime dependencies
RUN apk add --no-cache dumb-init openssl

WORKDIR /app

# Copy production dependencies
COPY --from=prod-deps /app/node_modules ./node_modules

# Copy built application
COPY --from=builder /app/apps/risk-service/dist ./dist
COPY --from=builder /app/apps/risk-service/package.json ./package.json

# Copy Prisma files for migrations
COPY --from=builder /app/apps/risk-service/prisma ./prisma

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001 && \
    chown -R nodejs:nodejs /app

USER nodejs

EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD node -e "require('http').get('http://localhost:3001/api/health', (r) => {process.exit(r.statusCode === 200 ? 0 : 1)})"

# Use dumb-init to handle signals properly
ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "dist/main.js"]
