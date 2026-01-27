# ===========================================
# Stage 1: Dependencies
# Install Python dependencies
# ===========================================
FROM python:3.11-slim AS deps

WORKDIR /app

# Install system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        gcc \
        libpq-dev \
        && \
    rm -rf /var/lib/apt/lists/*

# Copy dependency files
COPY apps/strategy-service/pyproject.toml ./

# Install Python dependencies
RUN pip install --no-cache-dir --upgrade pip setuptools wheel && \
    pip install --no-cache-dir .

# ===========================================
# Stage 2: Production Runtime
# Minimal runtime image
# ===========================================
FROM python:3.11-slim AS production

WORKDIR /app

# Install only runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libpq5 \
        curl \
        && \
    rm -rf /var/lib/apt/lists/*

# Copy installed packages from deps stage
COPY --from=deps /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --from=deps /usr/local/bin /usr/local/bin

# Copy application source
COPY apps/strategy-service/src ./src
COPY apps/strategy-service/alembic ./alembic
COPY apps/strategy-service/pyproject.toml ./

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash appuser && \
    chown -R appuser:appuser /app

USER appuser

# Environment variables
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    METRICS_PORT=9102

EXPOSE 9102

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:9102/health || exit 1

CMD ["python", "-m", "src.main"]
