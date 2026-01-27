"""Unit tests for Momentum Strategy."""

import pytest
from decimal import Decimal
from unittest.mock import AsyncMock, MagicMock

from src.strategies.momentum import MomentumStrategy, BarData, Signal
from src.auth import AuthContext, Permissions


@pytest.fixture
def momentum_strategy():
    return MomentumStrategy(
        lookback_period=5,
        entry_threshold=0.02,
        exit_threshold=-0.01
    )


@pytest.fixture
def auth_context():
    return AuthContext(
        account_id="acc-123",
        username="trader1",
        role="trader",
        permissions={Permissions.STRATEGIES_EXECUTE, Permissions.ORDERS_CREATE},
        token_jti="jti-123"
    )


class TestMomentumStrategy:
    def test_initialization(self, momentum_strategy):
        assert momentum_strategy.lookback_period == 5
        assert momentum_strategy.entry_threshold == 0.02
        assert momentum_strategy.exit_threshold == -0.01

    def test_update_bar(self, momentum_strategy):
        bar = BarData(
            symbol="BTC-USD",
            open=Decimal("50000"),
            high=Decimal("51000"),
            low=Decimal("49500"),
            close=Decimal("50500"),
            volume=Decimal("100"),
            timestamp=1234567890
        )
        
        momentum_strategy.update_bar(bar)
        
        assert len(momentum_strategy.price_history["BTC-USD"]) == 1
        assert momentum_strategy.price_history["BTC-USD"][0] == Decimal("50500")

    def test_calculate_momentum_insufficient_data(self, momentum_strategy):
        # Add only 3 bars (need 5 for lookback)
        for i in range(3):
            bar = BarData(
                symbol="BTC-USD",
                open=Decimal("50000"),
                high=Decimal("51000"),
                low=Decimal("49500"),
                close=Decimal(str(50000 + i * 100)),
                volume=Decimal("100"),
                timestamp=1234567890 + i
            )
            momentum_strategy.update_bar(bar)
        
        momentum = momentum_strategy.calculate_momentum("BTC-USD")
        assert momentum is None

    def test_calculate_momentum_positive(self, momentum_strategy):
        # Add bars with increasing prices
        prices = [50000, 50500, 51000, 51500, 52000]
        for i, price in enumerate(prices):
            bar = BarData(
                symbol="BTC-USD",
                open=Decimal(str(price - 100)),
                high=Decimal(str(price + 100)),
                low=Decimal(str(price - 200)),
                close=Decimal(str(price)),
                volume=Decimal("100"),
                timestamp=1234567890 + i
            )
            momentum_strategy.update_bar(bar)
        
        momentum = momentum_strategy.calculate_momentum("BTC-USD")
        
        assert momentum is not None
        assert momentum > 0  # Positive momentum

    def test_calculate_momentum_negative(self, momentum_strategy):
        # Add bars with decreasing prices
        prices = [52000, 51500, 51000, 50500, 50000]
        for i, price in enumerate(prices):
            bar = BarData(
                symbol="BTC-USD",
                open=Decimal(str(price + 100)),
                high=Decimal(str(price + 200)),
                low=Decimal(str(price - 100)),
                close=Decimal(str(price)),
                volume=Decimal("100"),
                timestamp=1234567890 + i
            )
            momentum_strategy.update_bar(bar)
        
        momentum = momentum_strategy.calculate_momentum("BTC-USD")
        
        assert momentum is not None
        assert momentum < 0  # Negative momentum

    @pytest.mark.asyncio
    async def test_generate_signal_buy(self, momentum_strategy, auth_context):
        # Setup strong positive momentum
        prices = [50000, 51000, 52000, 53000, 54000]
        for i, price in enumerate(prices):
            bar = BarData(
                symbol="BTC-USD",
                open=Decimal(str(price - 100)),
                high=Decimal(str(price + 100)),
                low=Decimal(str(price - 200)),
                close=Decimal(str(price)),
                volume=Decimal("100"),
                timestamp=1234567890 + i
            )
            momentum_strategy.update_bar(bar)
        
        signal = await momentum_strategy.generate_signal(
            auth_context, "BTC-USD", Decimal("0")
        )
        
        assert signal is not None
        assert signal.side == "buy"
        assert signal.strength > 0

    @pytest.mark.asyncio
    async def test_generate_signal_sell(self, momentum_strategy, auth_context):
        # Setup strong negative momentum
        prices = [54000, 53000, 52000, 51000, 50000]
        for i, price in enumerate(prices):
            bar = BarData(
                symbol="BTC-USD",
                open=Decimal(str(price + 100)),
                high=Decimal(str(price + 200)),
                low=Decimal(str(price - 100)),
                close=Decimal(str(price)),
                volume=Decimal("100"),
                timestamp=1234567890 + i
            )
            momentum_strategy.update_bar(bar)
        
        signal = await momentum_strategy.generate_signal(
            auth_context, "BTC-USD", Decimal("1")  # Has position
        )
        
        assert signal is not None
        assert signal.side == "sell"

    @pytest.mark.asyncio
    async def test_generate_signal_no_permission(self, momentum_strategy):
        # Auth context without execute permission
        auth = AuthContext(
            account_id="acc-123",
            username="viewer1",
            role="viewer",
            permissions={Permissions.MARKET_READ},
            token_jti="jti-123"
        )
        
        with pytest.raises(Exception) as exc_info:
            await momentum_strategy.generate_signal(auth, "BTC-USD", Decimal("0"))
        
        assert "permission" in str(exc_info.value).lower()


class TestSignal:
    def test_signal_creation(self):
        signal = Signal(
            symbol="BTC-USD",
            side="buy",
            strength=0.8,
            reason="Strong momentum"
        )
        
        assert signal.symbol == "BTC-USD"
        assert signal.side == "buy"
        assert signal.strength == 0.8
        assert signal.reason == "Strong momentum"
