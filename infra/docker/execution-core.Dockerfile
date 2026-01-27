# ===========================================
# Stage 1: Dependencies Cacher
# Cache Rust dependencies separately
# ===========================================
FROM rust:1.84-bookworm AS deps

WORKDIR /app

# Copy only dependency files for caching
COPY apps/execution-core/Cargo.toml apps/execution-core/build.rs ./
COPY apps/execution-core/src ./src

# Create dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release && rm -rf src

# ===========================================
# Stage 2: Builder
# Build the actual application
# ===========================================
FROM rust:1.84-bookworm AS builder

WORKDIR /app

# Copy cached dependencies
COPY --from=deps /app/target ./target
COPY --from=deps /app/Cargo.lock ./Cargo.lock

# Copy source files
COPY apps/execution-core/Cargo.toml apps/execution-core/build.rs ./
COPY apps/execution-core/src ./src

# Build with release optimizations
RUN cargo build --release

# ===========================================
# Stage 3: Production Runtime
# Minimal runtime image
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

# Copy binary from builder
COPY --from=builder /app/target/release/execution-core /usr/local/bin/execution-core

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash appuser && \
    chown appuser:appuser /usr/local/bin/execution-core

USER appuser

# Environment variables
ENV RUST_LOG=info,execution_core=debug
ENV METRICS_PORT=9100

EXPOSE 9100

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:9100/health || exit 1

CMD ["execution-core"]