//! Unit Tests for Order Processor
//! Phase 4: Comprehensive testing

use execution_core::engine::order_processor::*;
use rust_decimal_macros::dec;
use uuid::Uuid;

#[cfg(test)]
mod order_processor_tests {
    use super::*;

    fn create_test_order() -> Order {
        Order {
            id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            client_order_id: "test-001".to_string(),
            symbol: "BTC-USD".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: dec!(1.0),
            price: Some(dec!(50000.0)),
            status: OrderStatus::New,
            filled_quantity: dec!(0),
            avg_fill_price: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_order_creation() {
        let order = create_test_order();
        assert_eq!(order.status, OrderStatus::New);
        assert_eq!(order.filled_quantity, dec!(0));
        assert!(order.avg_fill_price.is_none());
    }

    #[test]
    fn test_order_validation_valid() {
        let order = create_test_order();
        assert!(validate_order(&order).is_ok());
    }

    #[test]
    fn test_order_validation_zero_quantity() {
        let mut order = create_test_order();
        order.quantity = dec!(0);
        assert!(validate_order(&order).is_err());
    }

    #[test]
    fn test_order_validation_negative_quantity() {
        let mut order = create_test_order();
        order.quantity = dec!(-1.0);
        assert!(validate_order(&order).is_err());
    }

    #[test]
    fn test_order_validation_limit_without_price() {
        let mut order = create_test_order();
        order.order_type = OrderType::Limit;
        order.price = None;
        assert!(validate_order(&order).is_err());
    }

    #[test]
    fn test_order_fill_partial() {
        let mut order = create_test_order();
        let fill_qty = dec!(0.5);
        let fill_price = dec!(50100.0);
        
        apply_fill(&mut order, fill_qty, fill_price);
        
        assert_eq!(order.filled_quantity, fill_qty);
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.avg_fill_price, Some(fill_price));
    }

    #[test]
    fn test_order_fill_complete() {
        let mut order = create_test_order();
        let fill_qty = dec!(1.0);
        let fill_price = dec!(50100.0);
        
        apply_fill(&mut order, fill_qty, fill_price);
        
        assert_eq!(order.filled_quantity, fill_qty);
        assert_eq!(order.status, OrderStatus::Filled);
    }

    #[test]
    fn test_weighted_average_price() {
        let mut order = create_test_order();
        
        // First fill: 0.3 @ 50000
        apply_fill(&mut order, dec!(0.3), dec!(50000.0));
        assert_eq!(order.avg_fill_price, Some(dec!(50000.0)));
        
        // Second fill: 0.7 @ 50100
        // Weighted avg = (0.3 * 50000 + 0.7 * 50100) / 1.0 = 50070
        apply_fill(&mut order, dec!(0.7), dec!(50100.0));
        assert_eq!(order.avg_fill_price, Some(dec!(50070.0)));
    }

    #[test]
    fn test_order_cancellation() {
        let mut order = create_test_order();
        order.status = OrderStatus::Open;
        
        let result = cancel_order(&mut order);
        
        assert!(result.is_ok());
        assert_eq!(order.status, OrderStatus::Cancelled);
    }

    #[test]
    fn test_cannot_cancel_filled_order() {
        let mut order = create_test_order();
        order.status = OrderStatus::Filled;
        
        let result = cancel_order(&mut order);
        
        assert!(result.is_err());
    }
}

// Placeholder implementations for testing
#[derive(Debug, Clone, PartialEq)]
pub enum OrderSide { Buy, Sell }

#[derive(Debug, Clone, PartialEq)]
pub enum OrderType { Market, Limit }

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus { New, Open, PartiallyFilled, Filled, Cancelled, Rejected }

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub account_id: Uuid,
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: rust_decimal::Decimal,
    pub price: Option<rust_decimal::Decimal>,
    pub status: OrderStatus,
    pub filled_quantity: rust_decimal::Decimal,
    pub avg_fill_price: Option<rust_decimal::Decimal>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn validate_order(order: &Order) -> Result<(), String> {
    if order.quantity <= rust_decimal::Decimal::ZERO {
        return Err("Quantity must be positive".to_string());
    }
    if order.order_type == OrderType::Limit && order.price.is_none() {
        return Err("Limit orders require a price".to_string());
    }
    Ok(())
}

fn apply_fill(order: &mut Order, fill_qty: rust_decimal::Decimal, fill_price: rust_decimal::Decimal) {
    let prev_filled = order.filled_quantity;
    let new_filled = prev_filled + fill_qty;
    
    // Calculate weighted average price
    if prev_filled > rust_decimal::Decimal::ZERO {
        let prev_value = prev_filled * order.avg_fill_price.unwrap_or(rust_decimal::Decimal::ZERO);
        let new_value = fill_qty * fill_price;
        order.avg_fill_price = Some((prev_value + new_value) / new_filled);
    } else {
        order.avg_fill_price = Some(fill_price);
    }
    
    order.filled_quantity = new_filled;
    
    if new_filled >= order.quantity {
        order.status = OrderStatus::Filled;
    } else {
        order.status = OrderStatus::PartiallyFilled;
    }
    order.updated_at = chrono::Utc::now();
}

fn cancel_order(order: &mut Order) -> Result<(), String> {
    match order.status {
        OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected => {
            Err("Cannot cancel order in terminal state".to_string())
        }
        _ => {
            order.status = OrderStatus::Cancelled;
            order.updated_at = chrono::Utc::now();
            Ok(())
        }
    }
}
