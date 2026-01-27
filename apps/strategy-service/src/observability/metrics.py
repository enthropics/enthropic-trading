"""Prometheus Metrics for Strategy Service."""

import os
from prometheus_client import Counter, Histogram, Gauge, CollectorRegistry, generate_latest
from aiohttp import web

# Global registry
REGISTRY = CollectorRegistry()

# Strategy metrics
strategy_signals = Counter(
    "enthropic_strategy_signals_total",
    "Total strategy signals generated",
    ["strategy", "side", "symbol"],
    registry=REGISTRY,
)

strategy_execution_duration = Histogram(
    "enthropic_strategy_execution_duration_seconds",
    "Strategy execution duration",
    ["strategy"],
    buckets=[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0],
    registry=REGISTRY,
)

active_strategies = Gauge(
    "enthropic_active_strategies",
    "Number of active strategies",
    registry=REGISTRY,
)

# Resilience metrics
circuit_breaker_state = Gauge(
    "enthropic_circuit_breaker_state",
    "Circuit breaker state",
    ["name"],
    registry=REGISTRY,
)

retry_attempts = Counter(
    "enthropic_retry_attempts_total",
    "Retry attempts",
    ["operation", "outcome"],
    registry=REGISTRY,
)


class TradingMetrics:
    """Metrics accessor class."""
    
    strategy_signals = strategy_signals
    strategy_execution_duration = strategy_execution_duration
    active_strategies = active_strategies
    circuit_breaker_state = circuit_breaker_state
    retry_attempts = retry_attempts


_metrics = TradingMetrics()


def init_metrics() -> TradingMetrics:
    """Initialize metrics."""
    return _metrics


def get_metrics() -> TradingMetrics:
    """Get metrics instance."""
    return _metrics


class MetricsServer:
    """HTTP server for metrics and health endpoints."""
    
    def __init__(self, port: int = 9102):
        self.port = port
        self.app = web.Application()
        self.app.router.add_get("/metrics", self.metrics_handler)
        self.app.router.add_get("/health", self.health_handler)
        self._healthy = True
    
    async def metrics_handler(self, request: web.Request) -> web.Response:
        return web.Response(
            body=generate_latest(REGISTRY),
            content_type="text/plain",
        )
    
    async def health_handler(self, request: web.Request) -> web.Response:
        if self._healthy:
            return web.json_response({"status": "healthy"})
        return web.json_response({"status": "unhealthy"}, status=503)
    
    def set_healthy(self, healthy: bool) -> None:
        self._healthy = healthy
    
    async def start(self) -> None:
        runner = web.AppRunner(self.app)
        await runner.setup()
        site = web.TCPSite(runner, "0.0.0.0", self.port)
        await site.start()
