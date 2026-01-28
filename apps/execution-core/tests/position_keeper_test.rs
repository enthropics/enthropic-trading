//! Unit Tests for Position Keeper
//! Phase 4: Position calculation tests - without signum()

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

#[cfg(test)]
mod position_tests {
    use super::*;

    /// Helper function to get sign multiplier (replaces signum)
    fn sign_of(d: Decimal) -> Decimal {
        if d > dec!(0) {
            dec!(1)
        } else if d < dec!(0) {
            dec!(-1)
        } else {
            dec!(0)
        }
    }

    /// Check if two decimals have the same sign
    fn same_sign(a: Decimal, b: Decimal) -> bool {
        (a > dec!(0) && b > dec!(0)) || (a < dec!(0) && b < dec!(0))
    }

    #[derive(Debug, Clone)]
    struct Position {
        account_id: Uuid,
        symbol: String,
        quantity: Decimal,
        avg_price: Decimal,
        realized_pnl: Decimal,
        unrealized_pnl: Decimal,
    }

    impl Position {
        fn new(account_id: Uuid, symbol: &str) -> Self {
            Self {
                account_id,
                symbol: symbol.to_string(),
                quantity: dec!(0),
                avg_price: dec!(0),
                realized_pnl: dec!(0),
                unrealized_pnl: dec!(0),
            }
        }

        fn apply_fill(&mut self, qty: Decimal, price: Decimal, is_buy: bool) {
            let signed_qty = if is_buy { qty } else { -qty };
            let old_qty = self.quantity;
            let new_qty = old_qty + signed_qty;

            if old_qty == dec!(0) {
                // New position
                self.avg_price = price;
            } else if same_sign(old_qty, signed_qty) {
                // Increasing position - calculate weighted average
                let old_value = old_qty.abs() * self.avg_price;
                let new_value = qty * price;
                self.avg_price = (old_value + new_value) / new_qty.abs();
            } else if new_qty.abs() < old_qty.abs() && same_sign(new_qty, old_qty) {
                // Reducing position - realize P&L
                let realized = qty * (price - self.avg_price) * sign_of(old_qty);
                self.realized_pnl += realized;
            } else if new_qty == dec!(0) {
                // Closing completely
                let realized = old_qty.abs() * (price - self.avg_price) * sign_of(old_qty);
                self.realized_pnl += realized;
            } else if !same_sign(new_qty, old_qty) && old_qty != dec!(0) {
                // Crossing zero - close old position, open new
                let close_qty = old_qty.abs();
                let realized = close_qty * (price - self.avg_price) * sign_of(old_qty);
                self.realized_pnl += realized;
                self.avg_price = price;
            }

            self.quantity = new_qty;
        }

        fn update_unrealized_pnl(&mut self, market_price: Decimal) {
            if self.quantity != dec!(0) {
                self.unrealized_pnl = self.quantity * (market_price - self.avg_price);
            } else {
                self.unrealized_pnl = dec!(0);
            }
        }
    }

    #[test]
    fn test_new_long_position() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(1.0), dec!(50000), true);

        assert_eq!(pos.quantity, dec!(1.0));
        assert_eq!(pos.avg_price, dec!(50000));
        assert_eq!(pos.realized_pnl, dec!(0));
    }

    #[test]
    fn test_new_short_position() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(1.0), dec!(50000), false);

        assert_eq!(pos.quantity, dec!(-1.0));
        assert_eq!(pos.avg_price, dec!(50000));
    }

    #[test]
    fn test_increase_long_position() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(1.0), dec!(50000), true);
        pos.apply_fill(dec!(1.0), dec!(51000), true);

        assert_eq!(pos.quantity, dec!(2.0));
        // Weighted avg: (1*50000 + 1*51000) / 2 = 50500
        assert_eq!(pos.avg_price, dec!(50500));
    }

    #[test]
    fn test_partial_close_long() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(2.0), dec!(50000), true); // Buy 2 @ 50000
        pos.apply_fill(dec!(1.0), dec!(51000), false); // Sell 1 @ 51000

        assert_eq!(pos.quantity, dec!(1.0));
        assert_eq!(pos.avg_price, dec!(50000)); // Avg price unchanged
        assert_eq!(pos.realized_pnl, dec!(1000)); // 1 * (51000 - 50000)
    }

    #[test]
    fn test_full_close_position() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(1.0), dec!(50000), true);
        pos.apply_fill(dec!(1.0), dec!(52000), false);

        assert_eq!(pos.quantity, dec!(0));
        assert_eq!(pos.realized_pnl, dec!(2000));
    }

    #[test]
    fn test_flip_long_to_short() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(1.0), dec!(50000), true); // Long 1
        pos.apply_fill(dec!(2.0), dec!(51000), false); // Sell 2 -> Short 1

        assert_eq!(pos.quantity, dec!(-1.0));
        assert_eq!(pos.avg_price, dec!(51000));
        assert_eq!(pos.realized_pnl, dec!(1000)); // Realized on closing long
    }

    #[test]
    fn test_unrealized_pnl_long() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(1.0), dec!(50000), true);
        pos.update_unrealized_pnl(dec!(55000));

        assert_eq!(pos.unrealized_pnl, dec!(5000));
    }

    #[test]
    fn test_unrealized_pnl_short() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(1.0), dec!(50000), false); // Short
        pos.update_unrealized_pnl(dec!(48000)); // Price went down = profit

        // unrealized = quantity * (market - avg) = -1 * (48000 - 50000) = 2000
        assert_eq!(pos.unrealized_pnl, dec!(2000));
    }

    #[test]
    fn test_increase_short_position() {
        let mut pos = Position::new(Uuid::new_v4(), "BTC-USD");
        pos.apply_fill(dec!(1.0), dec!(50000), false); // Short 1 @ 50000
        pos.apply_fill(dec!(1.0), dec!(51000), false); // Short 1 more @ 51000

        assert_eq!(pos.quantity, dec!(-2.0));
        // Weighted avg: (1*50000 + 1*51000) / 2 = 50500
        assert_eq!(pos.avg_price, dec!(50500));
    }
}