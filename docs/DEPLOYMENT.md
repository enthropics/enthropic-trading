# Deployment Guide

## Prerequisites

- Kubernetes cluster (1.25+)
- Helm 3.x
- kubectl configured
- PostgreSQL (managed or self-hosted)
- Redis (managed or self-hosted)

## Production Checklist

### Security
- [ ] Generate new JWT secret
- [ ] Use managed PostgreSQL with SSL
- [ ] Use managed Redis with auth
- [ ] Enable network policies
- [ ] Configure ingress with TLS
- [ ] Set up Vault for secrets

### Monitoring
- [ ] Deploy Prometheus Operator
- [ ] Configure alerting rules
- [ ] Set up Grafana dashboards
- [ ] Configure log aggregation

### Scaling
- [ ] Configure HPA thresholds
- [ ] Set resource requests/limits
- [ ] Enable pod disruption budgets
- [ ] Test failover scenarios

## Helm Values (Production)

```yaml
# values-production.yaml
global:
  environment: production

executionCore:
  replicaCount: 3
  resources:
    requests:
      cpu: 1000m
      memory: 1Gi
    limits:
      cpu: 4000m
      memory: 4Gi
  autoscaling:
    enabled: true
    minReplicas: 3
    maxReplicas: 20

postgresql:
  enabled: false
  external:
    host: prod-postgres.xxx.rds.amazonaws.com
    existingSecret: enthropic-db-credentials
```

## Rolling Updates

```bash
# Update image
helm upgrade enthropic ./infra/kubernetes/charts/enthropic \
  --set executionCore.image.tag=v1.2.0

# Monitor rollout
kubectl rollout status deployment/enthropic-execution-core
```

## Rollback

```bash
# Rollback to previous
helm rollback enthropic

# Rollback to specific revision
helm rollback enthropic 3
```
