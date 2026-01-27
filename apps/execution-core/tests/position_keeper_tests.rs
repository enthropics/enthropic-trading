//! Unit tests for Position Keeper
//! Tests weighted average calculation for position management

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

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
        fn new(symbol: &str) -> Self {
            Self {
                account_id: Uuid::new_v4(),
                symbol: symbol.to_string(),
                quantity: Decimal::ZERO,
                avg_price: Decimal::ZERO,
                realized_pnl: Decimal::ZERO,
                unrealized_pnl: Decimal::ZERO,
            }
        }

        /// Apply a fill to the position using weighted average
        fn apply_fill(&mut self, fill_qty: Decimal, fill_price: Decimal, is_buy: bool) {
            let signed_qty = if is_buy { fill_qty } else { -fill_qty };
            let old_qty = self.quantity;
            let new_qty = old_qty + signed_qty;

            if old_qty == Decimal::ZERO {
                // Opening new position
                self.avg_price = fill_price;
            } else if (old_qty > Decimal::ZERO && is_buy) || (old_qty < Decimal::ZERO && !is_buy) {
                // Increasing position - weighted average
                let total_cost = (old_qty.abs() * self.avg_price) + (fill_qty * fill_price);
                self.avg_price = total_cost / new_qty.abs();
            } else if new_qty == Decimal::ZERO {
                // Closing position completely
                let pnl = if old_qty > Decimal::ZERO {
                    fill_qty * (fill_price - self.avg_price)
                } else {
                    fill_qty * (self.avg_price - fill_price)
                };
                self.realized_pnl += pnl;
                self.avg_price = Decimal::ZERO;
            } else if (old_qty > Decimal::ZERO && new_qty < Decimal::ZERO) 
                   || (old_qty < Decimal::ZERO && new_qty > Decimal::ZERO) {
                // Crossing zero - close old, open new
                let closing_qty = old_qty.abs();
                let opening_qty = new_qty.abs();
                
                let pnl = if old_qty > Decimal::ZERO {
                    closing_qty * (fill_price - self.avg_price)
                } else {
                    closing_qty * (self.avg_price - fill_price)
                };
                self.realized_pnl += pnl;
                self.avg_price = fill_price;
            } else {
                // Reducing position
                let pnl = if old_qty > Decimal::ZERO {
                    fill_qty * (fill_price - self.avg_price)
                } else {
                    fill_qty * (self.avg_price - fill_price)
                };
                self.realized_pnl += pnl;
                // avg_price stays the same when reducing
            }

            self.quantity = new_qty;
        }

        fn update_unrealized_pnl(&mut self, current_price: Decimal) {
            if self.quantity > Decimal::ZERO {
                self.unrealized_pnl = self.quantity * (current_price - self.avg_price);
            } else if self.quantity < Decimal::ZERO {
                self.unrealized_pnl = self.quantity.abs() * (self.avg_price - current_price);
            } else {
                self.unrealized_pnl = Decimal::ZERO;
            }
        }
    }

    #[test]
    fn test_open_long_position() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), true);
        
        assert_eq!(pos.quantity, dec!(10));
        assert_eq!(pos.avg_price, dec!(100));
        assert_eq!(pos.realized_pnl, Decimal::ZERO);
    }

    #[test]
    fn test_increase_long_position_weighted_average() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), true);  // Buy 10 @ 100
        pos.apply_fill(dec!(10), dec!(120), true);  // Buy 10 @ 120
        
        assert_eq!(pos.quantity, dec!(20));
        assert_eq!(pos.avg_price, dec!(110));  // (10*100 + 10*120) / 20 = 110
    }

    #[test]
    fn test_reduce_long_position() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), true);   // Buy 10 @ 100
        pos.apply_fill(dec!(5), dec!(120), false);   // Sell 5 @ 120
        
        assert_eq!(pos.quantity, dec!(5));
        assert_eq!(pos.avg_price, dec!(100));  // Avg price stays same
        assert_eq!(pos.realized_pnl, dec!(100));  // 5 * (120 - 100) = 100
    }

    #[test]
    fn test_close_long_position() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), true);   // Buy 10 @ 100
        pos.apply_fill(dec!(10), dec!(150), false);  // Sell 10 @ 150
        
        assert_eq!(pos.quantity, Decimal::ZERO);
        assert_eq!(pos.realized_pnl, dec!(500));  // 10 * (150 - 100) = 500
    }

    #[test]
    fn test_open_short_position() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), false);  // Sell 10 @ 100
        
        assert_eq!(pos.quantity, dec!(-10));
        assert_eq!(pos.avg_price, dec!(100));
    }

    #[test]
    fn test_close_short_position_profit() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), false);  // Sell 10 @ 100
        pos.apply_fill(dec!(10), dec!(80), true);    // Buy 10 @ 80
        
        assert_eq!(pos.quantity, Decimal::ZERO);
        assert_eq!(pos.realized_pnl, dec!(200));  // 10 * (100 - 80) = 200
    }

    #[test]
    fn test_unrealized_pnl_long() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), true);
        pos.update_unrealized_pnl(dec!(120));
        
        assert_eq!(pos.unrealized_pnl, dec!(200));  // 10 * (120 - 100)
    }

    #[test]
    fn test_unrealized_pnl_short() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), false);
        pos.update_unrealized_pnl(dec!(80));
        
        assert_eq!(pos.unrealized_pnl, dec!(200));  // 10 * (100 - 80)
    }

    #[test]
    fn test_cross_zero_long_to_short() {
        let mut pos = Position::new("BTC-USD");
        pos.apply_fill(dec!(10), dec!(100), true);   // Buy 10 @ 100
        pos.apply_fill(dec!(15), dec!(120), false);  // Sell 15 @ 120
        
        assert_eq!(pos.quantity, dec!(-5));
        assert_eq!(pos.avg_price, dec!(120));  // New short position at fill price
        assert_eq!(pos.realized_pnl, dec!(200));  // Closed 10 long: 10 * (120 - 100)
    }
}
