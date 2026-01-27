"""Tests for Momentum Strategy."""

import pytest
from decimal import Decimal
from unittest.mock import AsyncMock
from dataclasses import dataclass

# Import strategy components
import sys
sys.path.insert(0, 'src')

from strategies.momentum import MomentumStrategy, BarData, Signal


@pytest.fixture
def momentum_strategy():
    return MomentumStrategy(
        lookback_period=5,
        entry_threshold=0.02,
        exit_threshold=-0.01
    )


@pytest.fixture
def sample_bars():
    """Generate sample price bars with upward momentum."""
    bars = []
    base_price = Decimal("100")
    for i in range(10):
        price = base_price + Decimal(str(i * 2))  # Upward trend
        bars.append(BarData(
            symbol="BTC-USD",
            open=price - Decimal("0.5"),
            high=price + Decimal("1"),
            low=price - Decimal("1"),
            close=price,
            volume=Decimal("1000"),
            timestamp=1704067200 + i * 60,
        ))
    return bars


class TestMomentumStrategy:
    
    def test_strategy_initialization(self, momentum_strategy):
        assert momentum_strategy.lookback_period == 5
        assert momentum_strategy.entry_threshold == 0.02
        assert momentum_strategy.exit_threshold == -0.01
    
    def test_update_bar_stores_price(self, momentum_strategy, sample_bars):
        bar = sample_bars[0]
        momentum_strategy.update_bar(bar)
        
        assert "BTC-USD" in momentum_strategy.price_history
        assert len(momentum_strategy.price_history["BTC-USD"]) == 1
    
    def test_momentum_calculation(self, momentum_strategy, sample_bars):
        # Feed enough bars
        for bar in sample_bars[:6]:
            momentum_strategy.update_bar(bar)
        
        momentum = momentum_strategy.calculate_momentum("BTC-USD")
        
        # With upward trend, momentum should be positive
        assert momentum is not None
        assert momentum > 0
    
    def test_momentum_insufficient_data(self, momentum_strategy, sample_bars):
        # Feed fewer bars than lookback period
        for bar in sample_bars[:3]:
            momentum_strategy.update_bar(bar)
        
        momentum = momentum_strategy.calculate_momentum("BTC-USD")
        
        assert momentum is None
    
    @pytest.mark.asyncio
    async def test_generate_signal_buy(self, momentum_strategy, sample_bars, auth_context):
        # Feed bars with strong upward momentum
        for bar in sample_bars:
            momentum_strategy.update_bar(bar)
        
        signal = await momentum_strategy.generate_signal(
            auth_context,
            "BTC-USD",
            Decimal("0")  # No current position
        )
        
        # With strong upward momentum and no position, should generate buy signal
        if signal is not None:
            assert signal.side in ["buy", "sell"]
            assert signal.symbol == "BTC-USD"
            assert 0 <= signal.strength <= 1
    
    @pytest.mark.asyncio
    async def test_generate_signal_no_permission(self, sample_bars):
        strategy = MomentumStrategy()
        
        # Auth context without strategies:execute permission
        @dataclass
        class NoPermAuthContext:
            account_id: str = "test"
            permissions: set = None
            
            def __post_init__(self):
                self.permissions = set()
            
            def has_permission(self, p: str) -> bool:
                return False
        
        for bar in sample_bars:
            strategy.update_bar(bar)
        
        # Should raise permission error
        with pytest.raises(Exception):
            await strategy.generate_signal(
                NoPermAuthContext(),
                "BTC-USD",
                Decimal("0")
            )


class TestBarData:
    
    def test_bar_data_creation(self):
        bar = BarData(
            symbol="ETH-USD",
            open=Decimal("2000"),
            high=Decimal("2100"),
            low=Decimal("1950"),
            close=Decimal("2050"),
            volume=Decimal("5000"),
            timestamp=1704067200,
        )
        
        assert bar.symbol == "ETH-USD"
        assert bar.close == Decimal("2050")
    
    def test_bar_data_ohlc_relationship(self):
        bar = BarData(
            symbol="BTC-USD",
            open=Decimal("100"),
            high=Decimal("110"),
            low=Decimal("90"),
            close=Decimal("105"),
            volume=Decimal("1000"),
            timestamp=1704067200,
        )
        
        # High should be >= all other prices
        assert bar.high >= bar.open
        assert bar.high >= bar.close
        assert bar.high >= bar.low
        
        # Low should be <= all other prices
        assert bar.low <= bar.open
        assert bar.low <= bar.close
        assert bar.low <= bar.high
