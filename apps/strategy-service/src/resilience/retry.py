"""Retry with Exponential Backoff for Python."""

import asyncio
from dataclasses import dataclass
from typing import Callable, TypeVar, Optional
import structlog

from ..observability.metrics import get_metrics

logger = structlog.get_logger()

T = TypeVar("T")


@dataclass
class RetryConfig:
    max_retries: int = 3
    initial_delay: float = 0.1
    max_delay: float = 10.0
    multiplier: float = 2.0


async def with_retry(
    operation: str,
    func: Callable[..., T],
    *args,
    config: RetryConfig = None,
    **kwargs,
) -> T:
    """Execute function with retry and exponential backoff."""
    config = config or RetryConfig()
    delay = config.initial_delay
    last_exception = None
    
    for attempt in range(1, config.max_retries + 1):
        try:
            result = await func(*args, **kwargs)
            if attempt > 1:
                get_metrics().retry_attempts.labels(
                    operation=operation, outcome="success"
                ).inc()
                logger.info(
                    "retry_succeeded",
                    operation=operation,
                    attempt=attempt,
                )
            return result
        except Exception as e:
            last_exception = e
            get_metrics().retry_attempts.labels(
                operation=operation, outcome="failure"
            ).inc()
            
            if attempt >= config.max_retries:
                logger.error(
                    "retry_exhausted",
                    operation=operation,
                    attempt=attempt,
                    error=str(e),
                )
                raise
            
            logger.warning(
                "retry_attempt",
                operation=operation,
                attempt=attempt,
                delay_seconds=delay,
                error=str(e),
            )
            
            await asyncio.sleep(delay)
            delay = min(delay * config.multiplier, config.max_delay)
    
    raise last_exception
