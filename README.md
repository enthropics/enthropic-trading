# Enthropic Trading Platform

## Overview

Enthropic is an enterprise-grade trading platform designed for high-performance financial operations. The platform leverages a modern technology stack including Rust, NestJS, Python, and React to deliver robust, scalable, and secure trading capabilities.

## Core Capabilities

### Phase 1: Data Persistence Architecture

The platform implements a comprehensive persistence layer utilizing PostgreSQL with TimescaleDB extensions for optimized time-series data management. Key features include:

- Complete trading schema supporting accounts, orders, positions, and trade records
- Hypertable implementation with automated compression and data retention policies
- ACID-compliant transactions ensuring data integrity and consistency
- Optimized indexing strategies for high-frequency trading operations

### Phase 2: Security and Access Control

A robust authentication and authorization framework has been implemented to ensure secure access to platform resources:

- JSON Web Token (JWT) authentication utilizing HS256 algorithm with 15-minute token expiration
- Role-Based Access Control (RBAC) system encompassing 5 distinct roles and 16 granular permissions
- Token refresh mechanism with 7-day validity period for enhanced security
- Automated account lockout mechanism following failed authentication attempts
- Comprehensive audit logging for security event monitoring and compliance

### Phase 3: System Observability and Resilience

The platform incorporates enterprise-grade observability and fault tolerance mechanisms:

- **Distributed Tracing**: OpenTelemetry integration with Jaeger for end-to-end request tracking
- **Metrics Collection**: Prometheus-based monitoring with Grafana visualization dashboards
- **Structured Logging**: JSON-formatted logs with contextual information for efficient analysis
- **Circuit Breaker Pattern**: Automated fault tolerance for downstream service dependencies
- **Retry Mechanisms**: Exponential backoff strategies for handling transient failures
- **Health Monitoring**: Kubernetes-compatible liveness and readiness probes

### Phase 4: Scalability, High Availability, and Continuous Delivery

Production-ready infrastructure with comprehensive automation and testing:

- **Container Orchestration**: Kubernetes deployment with Helm charts and Horizontal Pod Autoscaling
- **CI/CD Pipeline**: GitHub Actions-based automation for continuous integration and deployment
- **Testing Framework**: Comprehensive test coverage including unit, integration, end-to-end, and load testing
- **High Availability**: Multi-replica deployments with pod disruption budgets
- **Security Scanning**: Automated vulnerability detection with Trivy, network policy enforcement

## System Architecture

### Directory Structure

```
enthropic/
├── apps/
│   ├── dashboard/          # React-based frontend application
│   ├── execution-core/     # Rust-based high-performance execution engine
│   ├── risk-service/       # NestJS risk management service
│   ├── strategy-service/   # Python-based trading strategy engine
│   └── nats-gateway/       # TypeScript WebSocket gateway service
├── libs/shared/            # Shared library components
├── infra/
│   ├── db/init/           # Database initialization scripts and schemas
│   ├── docker/            # Docker container definitions
│   ├── kubernetes/        # Helm charts and Kubernetes manifests
│   └── monitoring/        # Observability configuration (Prometheus, Grafana)
├── tests/
│   ├── e2e/               # End-to-end test suites (Playwright)
│   └── load/              # Load testing scenarios (Locust)
└── .github/workflows/     # CI/CD pipeline definitions
```

## System Requirements

### Development Prerequisites

- Docker Engine 24.0+ with Docker Compose
- Node.js 20.x or higher (LTS recommended)
- Rust 1.75+ with Cargo build system
- Python 3.11+ with pip package manager

## Installation and Configuration

### Local Development Environment

```bash
# Clone the repository
git clone <repository-url>
cd enthropic

# Configure environment variables
cp .env.example .env

# Initialize and start all services
docker-compose up -d

# Monitor service logs
docker-compose logs -f
```

### Service Endpoints

| Service | Endpoint | Purpose |
|---------|----------|---------|
| Dashboard UI | http://localhost:5173 | Web-based trading interface |
| NATS Gateway | ws://localhost:3002 | WebSocket API gateway |
| Risk Service API | http://localhost:3001 | REST API for risk management |
| Grafana | http://localhost:3000 | Metrics visualization (admin/admin123) |
| Prometheus | http://localhost:9090 | Metrics collection and storage |
| Jaeger UI | http://localhost:16686 | Distributed tracing interface |
| Vault | http://localhost:8200 | Secrets management service |

### Default User Accounts

| Username | Password | Role | Access Level |
|----------|----------|------|--------------|
| admin | admin123 | Administrator | Full system access |
| trader1 | trader123 | Trader | Trading operations |
| viewer1 | viewer123 | Viewer | Read-only access |

**Note**: These credentials are for development purposes only and must be changed in production environments.

## Authentication Integration

### Authentication Workflow

```bash
# Request access token
curl -X POST http://localhost:3001/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"trader1","password":"trader123"}'

# Expected response
{
  "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "...",
  "expiresAt": 1704067200,
  "user": {
    "id": "uuid-string",
    "username": "trader1",
    "role": "trader"
  }
}
```

### Authenticated API Request

```bash
curl http://localhost:3001/api/risk/positions \
  -H "Authorization: Bearer <accessToken>"
```

## Monitoring and Observability

### Accessing Grafana Dashboards

1. Navigate to http://localhost:3000
2. Authenticate using credentials (admin/admin123)
3. Select Dashboards → Enthropic from the navigation menu

### Key Performance Indicators

| Metric | Description |
|--------|-------------|
| `enthropic_orders_processed_total` | Cumulative order processing throughput |
| `enthropic_order_processing_duration_seconds` | Order processing latency distribution |
| `enthropic_active_positions` | Current open position count |
| `enthropic_circuit_breaker_state` | Circuit breaker operational status |

### Distributed Tracing

1. Access Jaeger UI at http://localhost:16686
2. Select the desired service from the dropdown menu
3. Configure search parameters and execute trace queries

## Kubernetes Deployment

### Local Deployment (Minikube)

```bash
# Initialize Minikube cluster
minikube start --memory=8192 --cpus=4

# Deploy application using Helm
helm install enthropic ./infra/kubernetes/charts/enthropic \
  --namespace enthropic \
  --create-namespace \
  --set global.environment=development
```

### Production Deployment

```bash
# Create required Kubernetes secrets
kubectl create secret generic enthropic-jwt-secret \
  --from-literal=jwt-secret=$(openssl rand -base64 64)

kubectl create secret generic enthropic-db-credentials \
  --from-literal=url="postgresql://username:password@hostname:5432/database"

# Deploy to production namespace
helm upgrade --install enthropic ./infra/kubernetes/charts/enthropic \
  --namespace production \
  --values values-production.yaml
```

## Testing Procedures

### Unit Testing

```bash
# Rust components
cd apps/execution-core && cargo test

# TypeScript services
cd apps/risk-service && npm test

# Python services
cd apps/strategy-service && pytest
```

### End-to-End Testing

```bash
cd tests/e2e
npm install
npx playwright install
npx playwright test
```

### Load Testing

```bash
cd tests/load
pip install -r requirements.txt
locust -f locustfile.py --headless -u 100 -r 10 --run-time 2m
```

### Performance Benchmarking

```bash
cd apps/execution-core
cargo bench
```

## Continuous Integration and Deployment

The automated CI/CD pipeline implements the following stages:

1. **Code Quality Analysis** - Static analysis using Rust clippy, ESLint, and Black
2. **Unit Testing** - Automated test execution for all service components
3. **Container Build** - Docker image creation for each microservice
4. **Security Scanning** - Vulnerability assessment using Trivy
5. **Staging Deployment** - Automated deployment to staging environment (develop branch)
6. **End-to-End Testing** - Playwright-based integration tests
7. **Load Testing** - Performance validation using Locust
8. **Production Deployment** - Manual approval required for main branch deployment

## Performance Specifications

| Performance Metric | Target Value |
|-------------------|--------------|
| Order Processing Latency (P99) | < 10 milliseconds |
| System Throughput | 10,000 orders per second |
| System Availability | 99.99% uptime |
| Position Update Latency | < 5 milliseconds |

## Security Measures

The platform implements multiple layers of security controls:

- Password hashing using bcrypt algorithm with cost factor 12
- Short-lived JWT tokens with automatic expiration
- Token revocation and blacklisting on user logout
- Automated account lockout following failed authentication attempts
- Comprehensive audit logging for authentication and authorization events
- Kubernetes network policies for service isolation
- Automated container vulnerability scanning with Trivy

## Technical Documentation

- [Authentication and Authorization Guide](docs/AUTH.md)
- [API Reference Documentation](docs/API.md)
- [Deployment and Operations Guide](docs/DEPLOYMENT.md)
- [Observability and Monitoring Guide](docs/OBSERVABILITY.md)

## Contributing Guidelines

Contributors should adhere to the following workflow:

1. Fork the repository to your personal account
2. Create a feature branch from the develop branch
3. Implement changes with appropriate test coverage
4. Submit a pull request against the develop branch for review

## License

This software is proprietary and confidential. All rights reserved. Unauthorized copying, distribution, or modification of this software is strictly prohibited.

---

**Version**: 1.0.0  
**Last Updated**: February 2026  
**Maintained By**: Enthropic Development Team
