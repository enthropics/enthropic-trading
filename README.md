# Enthropic Trading Platform

Enterprise-grade trading platform built with Rust, NestJS, Python, and React.

## 🚀 Features

### Phase 1: Persistence Foundation ✅
- PostgreSQL with TimescaleDB for time-series data
- Complete trading schema (accounts, orders, positions, trades)
- Hypertables with automatic compression and retention
- Transaction safety and data integrity

### Phase 2: Authentication & Authorization ✅
- JWT-based authentication (HS256, 15-minute expiry)
- Role-Based Access Control (RBAC) with 5 roles, 16 permissions
- Token refresh mechanism (7-day refresh tokens)
- Account locking after failed attempts
- Audit logging for security events

### Phase 3: Observability & Resilience ✅
- **Distributed Tracing**: OpenTelemetry with Jaeger
- **Metrics**: Prometheus + Grafana dashboards
- **Structured Logging**: JSON format with context
- **Circuit Breakers**: Fault tolerance for downstream services
- **Retry with Backoff**: Handles transient failures
- **Health Checks**: Liveness/Readiness probes

### Phase 4: Scalability, HA, CI/CD, Testing ✅
- **Kubernetes**: Helm charts with HPA auto-scaling
- **CI/CD**: GitHub Actions pipeline (lint → test → build → deploy)
- **Testing**: Unit, integration, E2E (Playwright), Load (Locust)
- **HA**: Pod disruption budgets, multi-replica deployments
- **Security**: Trivy container scanning, network policies

## 📁 Project Structure

```
enthropic/
├── apps/
│   ├── dashboard/          # React + TypeScript frontend
│   ├── execution-core/     # Rust high-performance engine
│   ├── risk-service/       # NestJS risk management
│   ├── strategy-service/   # Python trading strategies
│   └── nats-gateway/       # TypeScript WebSocket gateway
├── libs/shared/            # Shared libraries
├── infra/
│   ├── db/init/           # Database schemas
│   ├── docker/            # Dockerfiles
│   ├── kubernetes/        # Helm charts
│   └── monitoring/        # Prometheus, Grafana configs
├── tests/
│   ├── e2e/               # Playwright E2E tests
│   └── load/              # Locust load tests
└── .github/workflows/     # CI/CD pipelines
```

## 🛠️ Quick Start

### Prerequisites
- Docker & Docker Compose
- Node.js 20+ (for local development)
- Rust 1.75+ (for execution-core)
- Python 3.11+ (for strategy-service)

### Local Development

```bash
# Clone repository
git clone <repo-url>
cd enthropic

# Copy environment file
cp .env.example .env

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f
```

### Access Points
| Service | URL |
|---------|-----|
| Dashboard | http://localhost:5173 |
| NATS Gateway (WebSocket) | ws://localhost:3002 |
| Risk Service API | http://localhost:3001 |
| Grafana | http://localhost:3000 (admin/admin123) |
| Prometheus | http://localhost:9090 |
| Jaeger UI | http://localhost:16686 |
| Vault | http://localhost:8200 |

### Demo Accounts
| Username | Password | Role |
|----------|----------|------|
| admin | admin123 | Full access |
| trader1 | trader123 | Trading |
| viewer1 | viewer123 | Read-only |

## 🔐 Authentication

### Login Flow
```bash
# Login
curl -X POST http://localhost:3001/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"trader1","password":"trader123"}'

# Response
{
  "accessToken": "eyJ...",
  "refreshToken": "...",
  "expiresAt": 1704067200,
  "user": { "id": "...", "username": "trader1", "role": "trader" }
}
```

### Use Token
```bash
curl http://localhost:3001/api/risk/positions \
  -H "Authorization: Bearer <accessToken>"
```

## 📊 Observability

### Grafana Dashboards
1. Open http://localhost:3000
2. Login with admin/admin123
3. Navigate to Dashboards → Enthropic

### Key Metrics
- `enthropic_orders_processed_total` - Order throughput
- `enthropic_order_processing_duration_seconds` - Latency histogram
- `enthropic_active_positions` - Position count
- `enthropic_circuit_breaker_state` - Circuit breaker status

### Tracing
1. Open Jaeger UI: http://localhost:16686
2. Select service (execution-core, risk-service, etc.)
3. Search for traces

## ☸️ Kubernetes Deployment

### Local (Minikube)
```bash
# Start minikube
minikube start --memory=8192 --cpus=4

# Install Helm chart
helm install enthropic ./infra/kubernetes/charts/enthropic \
  --namespace enthropic \
  --create-namespace \
  --set global.environment=development
```

### Production
```bash
# Create secrets
kubectl create secret generic enthropic-jwt-secret \
  --from-literal=jwt-secret=$(openssl rand -base64 64)

kubectl create secret generic enthropic-db-credentials \
  --from-literal=url="postgres://user:pass@host:5432/db"

# Deploy
helm upgrade --install enthropic ./infra/kubernetes/charts/enthropic \
  --namespace production \
  --values values-production.yaml
```

## 🧪 Testing

### Unit Tests
```bash
# Rust
cd apps/execution-core && cargo test

# TypeScript
cd apps/risk-service && npm test

# Python
cd apps/strategy-service && pytest
```

### E2E Tests
```bash
cd tests/e2e
npm install
npx playwright install
npx playwright test
```

### Load Tests
```bash
cd tests/load
pip install -r requirements.txt
locust -f locustfile.py --headless -u 100 -r 10 --run-time 2m
```

### Benchmarks
```bash
cd apps/execution-core
cargo bench
```

## 🔄 CI/CD Pipeline

The GitHub Actions pipeline includes:

1. **Lint** - Code quality checks (Rust clippy, ESLint, Black)
2. **Test** - Unit tests for all services
3. **Build** - Docker images for each service
4. **Security** - Trivy vulnerability scanning
5. **Deploy Staging** - On develop branch
6. **E2E Tests** - Playwright tests against staging
7. **Load Tests** - Locust performance tests
8. **Deploy Production** - On main branch (manual approval)

## 📈 Performance Targets

| Metric | Target |
|--------|--------|
| Order Latency (P99) | < 10ms |
| Throughput | 10,000 orders/sec |
| Uptime | 99.99% |
| Position Update | < 5ms |

## 🛡️ Security

- All passwords hashed with bcrypt (cost 12)
- JWT tokens with short expiry
- Token blacklisting on logout
- Account lockout on failed attempts
- Audit logging for all auth events
- Network policies in Kubernetes
- Container scanning with Trivy

## 📚 Documentation

- [Authentication Guide](docs/AUTH.md)
- [API Reference](docs/API.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Observability Guide](docs/OBSERVABILITY.md)

## 🤝 Contributing

1. Fork the repository
2. Create feature branch
3. Write tests
4. Submit PR against develop branch

## 📜 License

Proprietary - All rights reserved.
