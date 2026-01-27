# Observability Guide

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Services  │────▶│ OTEL        │────▶│   Jaeger    │
│             │     │ Collector   │     │  (Traces)   │
└─────────────┘     └──────┬──────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐     ┌─────────────┐
                    │ Prometheus  │────▶│  Grafana    │
                    │  (Metrics)  │     │ (Dashboard) │
                    └─────────────┘     └─────────────┘
```

## Metrics

### Custom Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `enthropic_orders_processed_total` | Counter | status, side, symbol | Total orders |
| `enthropic_order_processing_duration_seconds` | Histogram | operation | Latency |
| `enthropic_active_positions` | Gauge | - | Open positions |
| `enthropic_circuit_breaker_state` | Gauge | name | 0=closed, 0.5=half, 1=open |

### Prometheus Queries

```promql
# Order rate per second
sum(rate(enthropic_orders_processed_total[1m]))

# P99 latency
histogram_quantile(0.99, rate(enthropic_order_processing_duration_seconds_bucket[5m]))

# Error rate
sum(rate(enthropic_orders_processed_total{status="error"}[5m])) / sum(rate(enthropic_orders_processed_total[5m]))
```

## Alerts

### Critical Alerts

- High error rate (> 5%)
- Circuit breaker open
- Service down
- High latency (P99 > 100ms)

### Warning Alerts

- Memory usage > 80%
- Order backlog growing
- Database connection pool exhausted

## Troubleshooting

### Find Slow Traces
1. Open Jaeger: http://localhost:16686
2. Select service
3. Set min duration to 100ms
4. Search

### Debug Circuit Breaker
```bash
# Check Prometheus
curl http://localhost:9090/api/v1/query?query=enthropic_circuit_breaker_state

# Check service logs
docker-compose logs execution-core | grep "circuit_breaker"
```
