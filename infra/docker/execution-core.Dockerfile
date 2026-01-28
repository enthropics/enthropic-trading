# ===========================================
# Stage 1: Build dependencies
# ===========================================
FROM rust:bookworm AS deps

WORKDIR /app

# Copy manifest and build script
COPY apps/execution-core/Cargo.toml apps/execution-core/build.rs ./

# Copy source code
COPY apps/execution-core/src ./src

# Copy benchmarks directory
COPY apps/execution-core/benches ./benches

# Create dummy main to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release && rm -rf src

# ===========================================
# Stage 2: Build application
# ===========================================
FROM deps AS builder

# Copy actual source code
COPY apps/execution-core/src ./src

# Build the actual application
RUN cargo build --release

# ===========================================
# Stage 3: Production image
# ===========================================
FROM debian:bookworm-slim AS production

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        libpq5 \
        curl \
        && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/execution-core /app/execution-core

# Set ownership
RUN chown -R appuser:appuser /app

USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9100/health/live || exit 1

EXPOSE 9100

CMD ["./execution-core"]