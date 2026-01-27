"""Tests for Circuit Breaker implementation."""

import pytest
import asyncio
from datetime import timedelta

import sys
sys.path.insert(0, 'src')

from resilience.circuit_breaker import (
    CircuitBreaker,
    CircuitBreakerConfig,
    CircuitBreakerState,
    CircuitBreakerOpenError,
)


@pytest.fixture
def circuit_breaker():
    config = CircuitBreakerConfig(
        failure_threshold=3,
        success_threshold=2,
        timeout=timedelta(seconds=1),
        half_open_max_calls=2,
    )
    return CircuitBreaker("test-breaker", config)


class TestCircuitBreaker:
    
    def test_initial_state_closed(self, circuit_breaker):
        assert circuit_breaker.state == CircuitBreakerState.CLOSED
    
    @pytest.mark.asyncio
    async def test_successful_call(self, circuit_breaker):
        async def success():
            return "ok"
        
        result = await circuit_breaker.call(success)
        
        assert result == "ok"
        assert circuit_breaker.state == CircuitBreakerState.CLOSED
    
    @pytest.mark.asyncio
    async def test_opens_after_failures(self, circuit_breaker):
        async def failure():
            raise Exception("Service unavailable")
        
        # Trigger failures up to threshold
        for _ in range(3):
            with pytest.raises(Exception):
                await circuit_breaker.call(failure)
        
        assert circuit_breaker.state == CircuitBreakerState.OPEN
    
    @pytest.mark.asyncio
    async def test_rejects_when_open(self, circuit_breaker):
        async def failure():
            raise Exception("Service unavailable")
        
        # Open the circuit
        for _ in range(3):
            with pytest.raises(Exception):
                await circuit_breaker.call(failure)
        
        # Subsequent calls should be rejected
        with pytest.raises(CircuitBreakerOpenError):
            await circuit_breaker.call(failure)
    
    @pytest.mark.asyncio
    async def test_half_open_after_timeout(self, circuit_breaker):
        async def failure():
            raise Exception("Service unavailable")
        
        # Open the circuit
        for _ in range(3):
            with pytest.raises(Exception):
                await circuit_breaker.call(failure)
        
        # Wait for timeout
        await asyncio.sleep(1.1)
        
        # Check state (should transition to half-open)
        _ = await circuit_breaker._check_state()
        assert circuit_breaker.state == CircuitBreakerState.HALF_OPEN
    
    @pytest.mark.asyncio
    async def test_closes_after_successes_in_half_open(self, circuit_breaker):
        async def failure():
            raise Exception("Service unavailable")
        
        async def success():
            return "ok"
        
        # Open the circuit
        for _ in range(3):
            with pytest.raises(Exception):
                await circuit_breaker.call(failure)
        
        # Wait for timeout
        await asyncio.sleep(1.1)
        
        # Successful calls in half-open should close circuit
        await circuit_breaker.call(success)
        await circuit_breaker.call(success)
        
        assert circuit_breaker.state == CircuitBreakerState.CLOSED
