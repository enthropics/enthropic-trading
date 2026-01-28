# ===========================================
# Stage 1: Dependencies
# ===========================================
FROM node:20-alpine AS deps

WORKDIR /app

RUN apk add --no-cache openssl

# Copy package files
COPY apps/risk-service/package.json ./package.json
COPY apps/risk-service/package-lock.json* ./

# Install ALL dependencies (dev + prod)
RUN npm install

# ===========================================
# Stage 2: Builder
# ===========================================
FROM node:20-alpine AS builder

WORKDIR /app

RUN apk add --no-cache openssl

# Copy dependencies
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/package.json ./package.json

# Copy source code
COPY apps/risk-service/src ./src
COPY apps/risk-service/prisma ./prisma
COPY apps/risk-service/tsconfig.json ./tsconfig.json
COPY apps/risk-service/tsconfig.build.json* ./
COPY apps/risk-service/nest-cli.json ./nest-cli.json

# Generate Prisma Client
RUN npx prisma generate

# Build
RUN npm run build

# ===========================================
# Stage 3: Production
# ===========================================
FROM node:20-alpine AS production

RUN apk add --no-cache dumb-init openssl curl

WORKDIR /app

# Copy package.json
COPY apps/risk-service/package.json ./package.json
COPY apps/risk-service/package-lock.json* ./

# Install production dependencies only
RUN npm install --omit=dev

# Copy Prisma and generate client
COPY apps/risk-service/prisma ./prisma
RUN npx prisma generate

# Copy built application
COPY --from=builder /app/dist ./dist

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001 && \
    chown -R nodejs:nodejs /app

USER nodejs

EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3001/api/health || exit 1

ENTRYPOINT ["dumb-init", "--"]
CMD ["node", "dist/main.js"]