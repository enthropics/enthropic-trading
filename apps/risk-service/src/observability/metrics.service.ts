import { Injectable, OnModuleInit } from '@nestjs/common';
import { Counter, Histogram, Gauge, Registry, collectDefaultMetrics } from 'prom-client';

@Injectable()
export class MetricsService implements OnModuleInit {
  private registry: Registry;

  // Custom metrics
  public riskChecks: Counter<string>;
  public riskCheckDuration: Histogram<string>;
  public activeConnections: Gauge<string>;
  public circuitBreakerState: Gauge<string>;

  constructor() {
    this.registry = new Registry();
  }

  onModuleInit() {
    // Collect default metrics (CPU, memory, etc.)
    collectDefaultMetrics({ register: this.registry, prefix: 'enthropic_' });

    // Risk checks counter
    this.riskChecks = new Counter({
      name: 'enthropic_risk_checks_total',
      help: 'Total number of risk checks performed',
      labelNames: ['result', 'type'],
      registers: [this.registry],
    });

    // Risk check latency
    this.riskCheckDuration = new Histogram({
      name: 'enthropic_risk_check_duration_seconds',
      help: 'Risk check duration in seconds',
      labelNames: ['type'],
      buckets: [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1],
      registers: [this.registry],
    });

    // Active WebSocket connections
    this.activeConnections = new Gauge({
      name: 'enthropic_active_connections',
      help: 'Number of active connections',
      labelNames: ['type'],
      registers: [this.registry],
    });

    // Circuit breaker state
    this.circuitBreakerState = new Gauge({
      name: 'enthropic_circuit_breaker_state',
      help: 'Circuit breaker state (0=closed, 0.5=half-open, 1=open)',
      labelNames: ['name'],
      registers: [this.registry],
    });
  }

  getMetrics(): Promise<string> {
    return this.registry.metrics();
  }

  getRegistry(): Registry {
    return this.registry;
  }
}
