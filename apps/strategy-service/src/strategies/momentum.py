"""Momentum trading strategy with authentication."""

from dataclasses import dataclass
from decimal import Decimal
from typing import Optional, List
from collections import deque

from ..auth import AuthContext, require_permission, Permissions


@dataclass
class Signal:
    """Trading signal."""
    symbol: str
    side: str  # 'buy' or 'sell'
    strength: float  # 0.0 to 1.0
    reason: str


@dataclass
class BarData:
    """OHLCV bar data."""
    symbol: str
    open: Decimal
    high: Decimal
    low: Decimal
    close: Decimal
    volume: Decimal
    timestamp: int


class MomentumStrategy:
    """Simple momentum strategy based on price movement.
    
    Generates buy signals when price momentum is positive,
    sell signals when momentum is negative.
    """

    def __init__(
        self,
        lookback_period: int = 20,
        entry_threshold: float = 0.02,
        exit_threshold: float = -0.01,
    ):
        self.lookback_period = lookback_period
        self.entry_threshold = entry_threshold
        self.exit_threshold = exit_threshold
        self._price_history: dict[str, deque[Decimal]] = {}
        self._position: dict[str, Decimal] = {}

    def _get_history(self, symbol: str) -> deque[Decimal]:
        """Get price history for symbol."""
        if symbol not in self._price_history:
            self._price_history[symbol] = deque(maxlen=self.lookback_period)
        return self._price_history[symbol]

    def update_bar(self, bar: BarData) -> None:
        """Update strategy with new bar data."""
        history = self._get_history(bar.symbol)
        history.append(bar.close)

    def calculate_momentum(self, symbol: str) -> Optional[float]:
        """Calculate momentum as percentage change over lookback period."""
        history = self._get_history(symbol)
        if len(history) < self.lookback_period:
            return None
        
        old_price = history[0]
        current_price = history[-1]
        
        if old_price == 0:
            return None
        
        return float((current_price - old_price) / old_price)

    @require_permission(Permissions.STRATEGIES_EXECUTE)
    async def generate_signal(
        self,
        auth: AuthContext,
        symbol: str,
        current_position: Decimal = Decimal("0")
    ) -> Optional[Signal]:
        """Generate trading signal based on momentum.
        
        Requires strategies:execute permission.
        """
        momentum = self.calculate_momentum(symbol)
        if momentum is None:
            return None

        self._position[symbol] = current_position

        # Entry logic
        if current_position == 0:
            if momentum >= self.entry_threshold:
                return Signal(
                    symbol=symbol,
                    side="buy",
                    strength=min(1.0, momentum / self.entry_threshold),
                    reason=f"Positive momentum: {momentum:.2%}"
                )
            elif momentum <= -self.entry_threshold:
                return Signal(
                    symbol=symbol,
                    side="sell",
                    strength=min(1.0, abs(momentum) / self.entry_threshold),
                    reason=f"Negative momentum: {momentum:.2%}"
                )

        # Exit logic
        elif current_position > 0:
            if momentum <= self.exit_threshold:
                return Signal(
                    symbol=symbol,
                    side="sell",
                    strength=1.0,
                    reason=f"Exit long - momentum reversal: {momentum:.2%}"
                )

        elif current_position < 0:
            if momentum >= -self.exit_threshold:
                return Signal(
                    symbol=symbol,
                    side="buy",
                    strength=1.0,
                    reason=f"Exit short - momentum reversal: {momentum:.2%}"
                )

        return None

    def get_state(self) -> dict:
        """Get strategy state for serialization."""
        return {
            "lookback_period": self.lookback_period,
            "entry_threshold": self.entry_threshold,
            "exit_threshold": self.exit_threshold,
            "positions": {k: str(v) for k, v in self._position.items()},
            "history_lengths": {k: len(v) for k, v in self._price_history.items()},
        }
