//! Position Management with Weighted Average Price Calculation
//! Phase 1: Persistence + Phase 2: Auth checks

use crate::auth::{AuthContext, AuthError, permissions};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Position {
    pub account_id: Uuid,
    pub symbol: String,
    pub net_quantity: Decimal,
    pub avg_price: Decimal,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Decimal,
    pub cost_basis: Decimal,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Fill {
    pub account_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub quantity: Decimal,
    pub price: Decimal,
}

pub struct PositionKeeper {
    pool: PgPool,
    positions: Arc<RwLock<HashMap<(Uuid, String), Position>>>,
}

impl PositionKeeper {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            positions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load positions from database on startup
    pub async fn load_positions(&self) -> anyhow::Result<usize> {
        let rows: Vec<Position> = sqlx::query_as(
            r#"SELECT account_id, symbol, net_quantity, avg_price,
                      realized_pnl, unrealized_pnl, cost_basis, updated_at
               FROM positions WHERE net_quantity != 0"#
        )
            .fetch_all(&self.pool)
            .await?;

        let count = rows.len();
        let mut positions = self.positions.write().await;
        for pos in rows {
            positions.insert((pos.account_id, pos.symbol.clone()), pos);
        }
        tracing::info!("Loaded {} positions from database", count);
        Ok(count)
    }

    /// Apply a fill to update position (weighted average calculation)
    pub async fn apply_fill(&self, fill: &Fill) -> anyhow::Result<Position> {
        let key = (fill.account_id, fill.symbol.clone());

        // Get current position
        let current = {
            let positions = self.positions.read().await;
            positions.get(&key).cloned()
        };

        let (new_quantity, new_avg_price, realized_pnl) = match current {
            Some(ref pos) => self.calculate_new_position(pos, fill),
            None => self.calculate_new_position_from_zero(fill),
        };

        let cost_basis = new_quantity.abs() * new_avg_price;

        // Upsert to database atomically
        let position: Position = sqlx::query_as(
            r#"INSERT INTO positions (account_id, symbol, net_quantity, avg_price,
                                      realized_pnl, cost_basis, unrealized_pnl, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, 0, NOW())
               ON CONFLICT (account_id, symbol) DO UPDATE SET
                   net_quantity = $3,
                   avg_price = $4,
                   realized_pnl = positions.realized_pnl + $5,
                   cost_basis = $6,
                   updated_at = NOW()
               RETURNING account_id, symbol, net_quantity, avg_price,
                         realized_pnl, unrealized_pnl, cost_basis, updated_at"#
        )
            .bind(fill.account_id)
            .bind(&fill.symbol)
            .bind(new_quantity)
            .bind(new_avg_price)
            .bind(realized_pnl)
            .bind(cost_basis)
            .fetch_one(&self.pool)
            .await?;

        // Update cache
        {
            let mut positions = self.positions.write().await;
            if new_quantity == dec!(0) {
                positions.remove(&key);
            } else {
                positions.insert(key, position.clone());
            }
        }

        Ok(position)
    }

    /// Calculate new position after fill using weighted average rules
    fn calculate_new_position(&self, pos: &Position, fill: &Fill) -> (Decimal, Decimal, Decimal) {
        let fill_qty_signed = if fill.side == "buy" {
            fill.quantity
        } else {
            -fill.quantity
        };

        let new_quantity = pos.net_quantity + fill_qty_signed;

        // Helper: get sign multiplier for a decimal
        let sign_multiplier = |d: Decimal| -> Decimal {
            if d > dec!(0) { dec!(1) } else { dec!(-1) }
        };

        // Helper: check if same direction
        let same_direction = (pos.net_quantity > dec!(0) && fill_qty_signed > dec!(0)) ||
            (pos.net_quantity < dec!(0) && fill_qty_signed < dec!(0));

        // Rule 1: Increasing position (same direction)
        if same_direction {
            let total_cost = pos.net_quantity.abs() * pos.avg_price + fill.quantity * fill.price;
            let new_avg = total_cost / new_quantity.abs();
            return (new_quantity, new_avg, dec!(0));
        }

        // Rule 2: Reducing position (opposite direction, same sign result)
        let still_same_side = (pos.net_quantity > dec!(0) && new_quantity > dec!(0)) ||
            (pos.net_quantity < dec!(0) && new_quantity < dec!(0));

        if new_quantity != dec!(0) && still_same_side {
            let realized = fill.quantity * (fill.price - pos.avg_price) * sign_multiplier(pos.net_quantity);
            return (new_quantity, pos.avg_price, realized);
        }

        // Rule 3: Closing position exactly
        if new_quantity == dec!(0) {
            let realized = pos.net_quantity.abs() * (fill.price - pos.avg_price) * sign_multiplier(pos.net_quantity);
            return (dec!(0), dec!(0), realized);
        }

        // Rule 4: Crossing zero (close old + open new)
        let close_qty = pos.net_quantity.abs();
        let realized = close_qty * (fill.price - pos.avg_price) * sign_multiplier(pos.net_quantity);
        let new_avg = fill.price; // New position at fill price
        (new_quantity, new_avg, realized)
    }

    /// Calculate position from zero
    fn calculate_new_position_from_zero(&self, fill: &Fill) -> (Decimal, Decimal, Decimal) {
        let qty = if fill.side == "buy" { fill.quantity } else { -fill.quantity };
        (qty, fill.price, dec!(0))
    }

    /// Get position with auth check
    pub async fn get_position(
        &self,
        auth: &AuthContext,
        symbol: &str,
    ) -> Result<Option<Position>, AuthError> {
        if !auth.has_permission(permissions::POSITIONS_READ) {
            return Err(AuthError::InsufficientPermissions(
                "positions:read required".into()
            ));
        }

        let position: Option<Position> = sqlx::query_as(
            "SELECT account_id, symbol, net_quantity, avg_price, realized_pnl, \
             unrealized_pnl, cost_basis, updated_at FROM positions WHERE account_id = $1 AND symbol = $2"
        )
            .bind(auth.account_id)
            .bind(symbol)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(position)
    }

    /// Get all positions for account with auth check
    pub async fn get_account_positions(
        &self,
        auth: &AuthContext,
        account_id: Option<Uuid>,
    ) -> Result<Vec<Position>, AuthError> {
        if !auth.has_permission(permissions::POSITIONS_READ) {
            return Err(AuthError::InsufficientPermissions(
                "positions:read required".into()
            ));
        }

        let target = account_id.unwrap_or(auth.account_id);

        if target != auth.account_id && !auth.has_permission("positions:read_all") {
            return Err(AuthError::InsufficientPermissions(
                "Cannot view others' positions".into()
            ));
        }

        let positions: Vec<Position> = sqlx::query_as(
            "SELECT account_id, symbol, net_quantity, avg_price, realized_pnl, \
             unrealized_pnl, cost_basis, updated_at FROM positions WHERE account_id = $1"
        )
            .bind(target)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(positions)
    }
}