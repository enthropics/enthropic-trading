# ===========================================
# Stage 1: Dependencies
# Install all workspace dependencies
# ===========================================
FROM node:20-alpine AS deps

WORKDIR /app

# Copy workspace configuration
COPY package.json package-lock.json ./
COPY nx.json tsconfig.base.json ./

# Copy all workspace package.json files for dependency resolution
COPY apps/dashboard/package.json ./apps/dashboard/
COPY libs/shared ./libs/shared

# Install dependencies using npm ci for reproducible builds
# This will install all workspace dependencies
RUN npm install --workspace=apps/dashboard

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

# Copy shared libraries (if any are needed for build)
COPY libs ./libs

# Copy dashboard source code
COPY apps/dashboard ./apps/dashboard

# Build the dashboard
WORKDIR /app/apps/dashboard
RUN npm run build

# ===========================================
# Stage 3: Production
# Serve with nginx
# ===========================================
FROM nginx:alpine AS production

# Copy built static files
COPY --from=builder /app/apps/dashboard/dist /usr/share/nginx/html

# Configure nginx for SPA routing
RUN echo 'server { \
    listen 80; \
    server_name _; \
    root /usr/share/nginx/html; \
    index index.html; \
    \
    # Enable gzip compression \
    gzip on; \
    gzip_vary on; \
    gzip_min_length 1024; \
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json; \
    \
    # Cache static assets \
    location ~* \\.(?:css|js|jpg|jpeg|gif|png|ico|svg|woff|woff2|ttf|eot)$ { \
        expires 1y; \
        add_header Cache-Control "public, immutable"; \
    } \
    \
    # SPA routing - serve index.html for all routes \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    \
    # Health check endpoint \
    location /health { \
        access_log off; \
        return 200 "healthy\\n"; \
        add_header Content-Type text/plain; \
    } \
}' > /etc/nginx/conf.d/default.conf

# Remove default nginx config
RUN rm -f /etc/nginx/conf.d/default.conf.default

EXPOSE 80

# Add healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost/health || exit 1

CMD ["nginx", "-g", "daemon off;"]
