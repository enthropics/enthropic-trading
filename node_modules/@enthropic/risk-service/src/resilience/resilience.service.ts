import { Injectable, Logger } from '@nestjs/common';
import {
  circuitBreaker,
  ConsecutiveBreaker,
  ExponentialBackoff,
  handleAll,
  retry,
  wrap,
  CircuitState,
} from 'cockatiel';
import { MetricsService } from '../observability/metrics.service';

export interface CircuitBreakerOptions {
  name: string;
  halfOpenAfter?: number;
  breaker?: {
    threshold: number;
    duration?: number;
  };
}

@Injectable()
export class ResilienceService {
  private readonly logger = new Logger(ResilienceService.name);
  private circuitBreakers: Map<string, ReturnType<typeof circuitBreaker>> = new Map();

  constructor(private metricsService: MetricsService) {}

  createCircuitBreaker(options: CircuitBreakerOptions) {
    const breaker = circuitBreaker(handleAll, {
      halfOpenAfter: options.halfOpenAfter || 30000,
      breaker: new ConsecutiveBreaker(options.breaker?.threshold || 5),
    });

    breaker.onStateChange((state) => {
      this.logger.log(`Circuit breaker ${options.name} state: ${CircuitState[state]}`);
      const metricValue = state === CircuitState.Closed ? 0 : state === CircuitState.HalfOpen ? 0.5 : 1;
      this.metricsService.circuitBreakerState.labels(options.name).set(metricValue);
    });

    this.circuitBreakers.set(options.name, breaker);
    return breaker;
  }

  async withRetry<T>(
    operation: string,
    fn: () => Promise<T>,
    options?: { maxRetries?: number; maxDelay?: number },
  ): Promise<T> {
    const retryPolicy = retry(handleAll, {
      maxAttempts: options?.maxRetries || 3,
      backoff: new ExponentialBackoff({
        initialDelay: 100,
        maxDelay: options?.maxDelay || 10000,
      }),
    });

    retryPolicy.onRetry((data: any) => {
      this.logger.warn(`Retry attempt ${data.attempt} for operation due to: ${data.error?.message || 'unknown error'}`);
    });

    return retryPolicy.execute(fn);
  }

  async withResilience<T>(
    breakerName: string,
    operation: string,
    fn: () => Promise<T>,
  ): Promise<T> {
    let breaker = this.circuitBreakers.get(breakerName);
    if (!breaker) {
      breaker = this.createCircuitBreaker({ name: breakerName });
    }

    const retryPolicy = retry(handleAll, {
      maxAttempts: 3,
      backoff: new ExponentialBackoff(),
    });

    const policy = wrap(breaker, retryPolicy);
    return policy.execute(fn);
  }
}
