"""Circuit Breaker Implementation for Python."""

import asyncio
from dataclasses import dataclass
from datetime import datetime, timedelta
from enum import Enum
from typing import Callable, TypeVar, Optional
import structlog

from ..observability.metrics import get_metrics

logger = structlog.get_logger()

T = TypeVar("T")


class CircuitBreakerState(Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


@dataclass
class CircuitBreakerConfig:
    failure_threshold: int = 5
    success_threshold: int = 3
    timeout: timedelta = timedelta(seconds=30)
    half_open_max_calls: int = 3


class CircuitBreaker:
    """Circuit breaker for async functions."""
    
    def __init__(self, name: str, config: CircuitBreakerConfig = None):
        self.name = name
        self.config = config or CircuitBreakerConfig()
        self._state = CircuitBreakerState.CLOSED
        self._failure_count = 0
        self._success_count = 0
        self._half_open_calls = 0
        self._last_failure_time: Optional[datetime] = None
        self._lock = asyncio.Lock()
    
    @property
    def state(self) -> CircuitBreakerState:
        return self._state
    
    async def _check_state(self) -> CircuitBreakerState:
        async with self._lock:
            if self._state == CircuitBreakerState.OPEN:
                if self._last_failure_time and \
                   datetime.now() - self._last_failure_time >= self.config.timeout:
                    self._state = CircuitBreakerState.HALF_OPEN
                    self._half_open_calls = 0
                    self._update_metric()
                    logger.info("circuit_breaker_half_open", name=self.name)
            return self._state
    
    async def call(self, func: Callable[..., T], *args, **kwargs) -> T:
        """Execute function with circuit breaker protection."""
        current_state = await self._check_state()
        
        if current_state == CircuitBreakerState.OPEN:
            logger.warning("circuit_breaker_rejected", name=self.name)
            raise CircuitBreakerOpenError(f"Circuit breaker {self.name} is open")
        
        if current_state == CircuitBreakerState.HALF_OPEN:
            async with self._lock:
                if self._half_open_calls >= self.config.half_open_max_calls:
                    raise CircuitBreakerOpenError(f"Circuit breaker {self.name} half-open limit reached")
                self._half_open_calls += 1
        
        try:
            result = await func(*args, **kwargs)
            await self._on_success()
            return result
        except Exception as e:
            await self._on_failure()
            raise
    
    async def _on_success(self) -> None:
        async with self._lock:
            if self._state == CircuitBreakerState.HALF_OPEN:
                self._success_count += 1
                if self._success_count >= self.config.success_threshold:
                    self._state = CircuitBreakerState.CLOSED
                    self._reset_counts()
                    self._update_metric()
                    logger.info("circuit_breaker_closed", name=self.name)
            elif self._state == CircuitBreakerState.CLOSED:
                self._failure_count = 0
    
    async def _on_failure(self) -> None:
        async with self._lock:
            self._failure_count += 1
            self._last_failure_time = datetime.now()
            
            if self._state == CircuitBreakerState.CLOSED:
                if self._failure_count >= self.config.failure_threshold:
                    self._state = CircuitBreakerState.OPEN
                    self._update_metric()
                    logger.warning("circuit_breaker_opened", name=self.name, failures=self._failure_count)
            elif self._state == CircuitBreakerState.HALF_OPEN:
                self._state = CircuitBreakerState.OPEN
                self._reset_counts()
                self._update_metric()
                logger.warning("circuit_breaker_reopened", name=self.name)
    
    def _reset_counts(self) -> None:
        self._failure_count = 0
        self._success_count = 0
        self._half_open_calls = 0
    
    def _update_metric(self) -> None:
        value = {
            CircuitBreakerState.CLOSED: 0,
            CircuitBreakerState.HALF_OPEN: 0.5,
            CircuitBreakerState.OPEN: 1,
        }[self._state]
        get_metrics().circuit_breaker_state.labels(name=self.name).set(value)


class CircuitBreakerOpenError(Exception):
    """Raised when circuit breaker is open."""
    pass


class CircuitBreakerManager:
    """Manager for multiple circuit breakers."""
    
    def __init__(self):
        self._breakers: dict[str, CircuitBreaker] = {}
    
    def get_or_create(self, name: str, config: CircuitBreakerConfig = None) -> CircuitBreaker:
        if name not in self._breakers:
            self._breakers[name] = CircuitBreaker(name, config)
        return self._breakers[name]
    
    def get_states(self) -> dict[str, CircuitBreakerState]:
        return {name: breaker.state for name, breaker in self._breakers.items()}
