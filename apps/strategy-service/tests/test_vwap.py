"""Unit tests for VWAP Calculator."""

import pytest
from decimal import Decimal

import sys
sys.path.insert(0, '../libs/shared/vwap/src')

from vwap import VWAPCalculator, TWAPCalculator


class TestVWAPCalculator:
    @pytest.fixture
    def calculator(self):
        return VWAPCalculator(window_size=5)

    def test_initial_state(self, calculator):
        assert calculator.get_vwap("BTC-USD") is None

    def test_single_tick(self, calculator):
        calculator.add_tick("BTC-USD", Decimal("50000"), Decimal("10"))
        vwap = calculator.get_vwap("BTC-USD")
        
        # Single tick VWAP equals the price
        assert vwap == Decimal("50000")

    def test_multiple_ticks_equal_volume(self, calculator):
        calculator.add_tick("BTC-USD", Decimal("50000"), Decimal("10"))
        calculator.add_tick("BTC-USD", Decimal("51000"), Decimal("10"))
        vwap = calculator.get_vwap("BTC-USD")
        
        # Equal volume: (50000*10 + 51000*10) / 20 = 50500
        assert vwap == Decimal("50500")

    def test_multiple_ticks_different_volume(self, calculator):
        calculator.add_tick("BTC-USD", Decimal("50000"), Decimal("30"))
        calculator.add_tick("BTC-USD", Decimal("52000"), Decimal("10"))
        vwap = calculator.get_vwap("BTC-USD")
        
        # (50000*30 + 52000*10) / 40 = 50500
        assert vwap == Decimal("50500")

    def test_window_sliding(self, calculator):
        # Fill window
        for i in range(5):
            calculator.add_tick("BTC-USD", Decimal(str(50000 + i * 1000)), Decimal("10"))
        
        # Add one more - oldest should be removed
        calculator.add_tick("BTC-USD", Decimal("55000"), Decimal("10"))
        
        # Window should have prices 51000-55000, not 50000
        vwap = calculator.get_vwap("BTC-USD")
        expected = (51000 + 52000 + 53000 + 54000 + 55000) * 10 / 50
        assert vwap == Decimal(str(expected))

    def test_multiple_symbols(self, calculator):
        calculator.add_tick("BTC-USD", Decimal("50000"), Decimal("10"))
        calculator.add_tick("ETH-USD", Decimal("3000"), Decimal("100"))
        
        btc_vwap = calculator.get_vwap("BTC-USD")
        eth_vwap = calculator.get_vwap("ETH-USD")
        
        assert btc_vwap == Decimal("50000")
        assert eth_vwap == Decimal("3000")

    def test_reset(self, calculator):
        calculator.add_tick("BTC-USD", Decimal("50000"), Decimal("10"))
        calculator.reset("BTC-USD")
        
        assert calculator.get_vwap("BTC-USD") is None


class TestTWAPCalculator:
    @pytest.fixture
    def calculator(self):
        return TWAPCalculator(window_size=5)

    def test_initial_state(self, calculator):
        assert calculator.get_twap("BTC-USD") is None

    def test_single_price(self, calculator):
        calculator.add_price("BTC-USD", Decimal("50000"))
        twap = calculator.get_twap("BTC-USD")
        
        assert twap == Decimal("50000")

    def test_multiple_prices(self, calculator):
        prices = [50000, 51000, 52000]
        for price in prices:
            calculator.add_price("BTC-USD", Decimal(str(price)))
        
        twap = calculator.get_twap("BTC-USD")
        expected = sum(prices) / len(prices)
        
        assert twap == Decimal(str(expected))

    def test_window_sliding(self, calculator):
        # Fill window
        for i in range(5):
            calculator.add_price("BTC-USD", Decimal(str(50000 + i * 1000)))
        
        # Add one more
        calculator.add_price("BTC-USD", Decimal("55000"))
        
        twap = calculator.get_twap("BTC-USD")
        # Window: 51000, 52000, 53000, 54000, 55000
        expected = (51000 + 52000 + 53000 + 54000 + 55000) / 5
        
        assert twap == Decimal(str(expected))
