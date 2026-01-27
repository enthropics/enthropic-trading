"""VWAP and TWAP calculators."""

from collections import deque
from decimal import Decimal, ROUND_HALF_UP
from dataclasses import dataclass
from typing import Optional


@dataclass
class Trade:
    price: Decimal
    quantity: Decimal
    timestamp: int


class VWAPCalculator:
    """Volume-Weighted Average Price calculator."""

    def __init__(self, window_size: int = 100):
        self.window_size = window_size
        self._trades: deque[Trade] = deque(maxlen=window_size)
        self._cumulative_volume = Decimal("0")
        self._cumulative_value = Decimal("0")

    def add_trade(self, price: Decimal, quantity: Decimal, timestamp: int) -> None:
        """Add a trade to the calculator."""
        trade = Trade(price=price, quantity=quantity, timestamp=timestamp)
        
        # If at capacity, remove oldest trade's contribution
        if len(self._trades) == self.window_size:
            old_trade = self._trades[0]
            self._cumulative_volume -= old_trade.quantity
            self._cumulative_value -= old_trade.price * old_trade.quantity

        self._trades.append(trade)
        self._cumulative_volume += quantity
        self._cumulative_value += price * quantity

    def get_vwap(self) -> Optional[Decimal]:
        """Calculate current VWAP."""
        if self._cumulative_volume == 0:
            return None
        return (self._cumulative_value / self._cumulative_volume).quantize(
            Decimal("0.00000001"), rounding=ROUND_HALF_UP
        )

    def reset(self) -> None:
        """Reset the calculator."""
        self._trades.clear()
        self._cumulative_volume = Decimal("0")
        self._cumulative_value = Decimal("0")


class TWAPCalculator:
    """Time-Weighted Average Price calculator."""

    def __init__(self, window_size: int = 100):
        self.window_size = window_size
        self._prices: deque[tuple[Decimal, int]] = deque(maxlen=window_size)

    def add_price(self, price: Decimal, timestamp: int) -> None:
        """Add a price observation."""
        self._prices.append((price, timestamp))

    def get_twap(self) -> Optional[Decimal]:
        """Calculate current TWAP (simple average over window)."""
        if not self._prices:
            return None

        total = sum(p[0] for p in self._prices)
        return (total / len(self._prices)).quantize(
            Decimal("0.00000001"), rounding=ROUND_HALF_UP
        )

    def reset(self) -> None:
        """Reset the calculator."""
        self._prices.clear()
