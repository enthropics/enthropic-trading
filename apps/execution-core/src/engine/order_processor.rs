//! Order Processing Engine with Authentication
//! Phase 1: Persistence + Phase 2: Auth checks

use crate::auth::{AuthContext, AuthError, permissions};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub account_id: Uuid,
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub filled_quantity: Decimal,
    pub avg_fill_price: Option<Decimal>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderRequest {
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
}

#[derive(Debug)]
pub enum OrderResult {
    Accepted(Order),
    Rejected { reason: String, code: String },
    Duplicate(Order),
}

pub struct OrderProcessor {
    pool: PgPool,
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
}

impl OrderProcessor {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            orders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load open orders from database on startup
    pub async fn load_open_orders(&self) -> anyhow::Result<usize> {
        let rows = sqlx::query_as!(
            Order,
            r#"SELECT id, account_id, client_order_id, symbol, side, order_type,
                      quantity, price, filled_quantity, avg_fill_price, status,
                      created_at, updated_at
               FROM orders WHERE status IN ('pending', 'partially_filled')"#
        )
        .fetch_all(&self.pool)
        .await?;

        let count = rows.len();
        let mut orders = self.orders.write().await;
        for order in rows {
            orders.insert(order.id, order);
        }
        tracing::info!("Loaded {} open orders from database", count);
        Ok(count)
    }

    /// Submit a new order with authentication check
    pub async fn submit_order(
        &self,
        auth: &AuthContext,
        req: NewOrderRequest,
    ) -> Result<OrderResult, AuthError> {
        // Check permission
        if !auth.has_permission(permissions::ORDERS_CREATE) {
            return Err(AuthError::InsufficientPermissions(
                "orders:create required".into()
            ));
        }

        // Check for duplicate client_order_id (idempotency)
        let existing = sqlx::query_as!(
            Order,
            "SELECT * FROM orders WHERE account_id = $1 AND client_order_id = $2",
            auth.account_id,
            req.client_order_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        if let Some(order) = existing {
            return Ok(OrderResult::Duplicate(order));
        }

        // Create order
        let order_id = Uuid::new_v4();
        let now = Utc::now();

        let order = sqlx::query_as!(
            Order,
            r#"INSERT INTO orders (id, account_id, client_order_id, symbol, side, order_type,
                                   quantity, price, filled_quantity, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, 'pending', $9, $9)
               RETURNING id, account_id, client_order_id, symbol, side, order_type,
                         quantity, price, filled_quantity, avg_fill_price, status,
                         created_at, updated_at"#,
            order_id,
            auth.account_id,
            req.client_order_id,
            req.symbol,
            req.side,
            req.order_type,
            req.quantity,
            req.price,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        // Cache in memory
        {
            let mut orders = self.orders.write().await;
            orders.insert(order.id, order.clone());
        }

        // Log order event
        sqlx::query!(
            "INSERT INTO order_events (order_id, event_type, event_data) VALUES ($1, $2, $3)",
            order_id,
            "submitted",
            serde_json::json!({ "auth_user": auth.username }).to_string()
        )
        .execute(&self.pool)
        .await
        .ok();

        Ok(OrderResult::Accepted(order))
    }

    /// Cancel an order with auth check
    pub async fn cancel_order(
        &self,
        auth: &AuthContext,
        order_id: Uuid,
    ) -> Result<Option<Order>, AuthError> {
        if !auth.has_permission(permissions::ORDERS_CANCEL) {
            return Err(AuthError::InsufficientPermissions(
                "orders:cancel required".into()
            ));
        }

        // Fetch order and verify ownership
        let order = sqlx::query_as!(Order, "SELECT * FROM orders WHERE id = $1", order_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(AuthError::DatabaseError)?;

        let order = match order {
            Some(o) => o,
            None => return Ok(None),
        };

        // Check ownership (unless admin)
        if !auth.can_access_account(&order.account_id) {
            return Err(AuthError::InsufficientPermissions(
                "Cannot cancel others' orders".into()
            ));
        }

        // Can only cancel pending/partially_filled
        if order.status != "pending" && order.status != "partially_filled" {
            return Ok(Some(order));
        }

        // Update status
        let cancelled = sqlx::query_as!(
            Order,
            r#"UPDATE orders SET status = 'cancelled', updated_at = NOW()
               WHERE id = $1 RETURNING *"#,
            order_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        // Update cache
        {
            let mut orders = self.orders.write().await;
            orders.remove(&order_id);
        }

        Ok(Some(cancelled))
    }

    /// Get orders for account with auth check
    pub async fn get_account_orders(
        &self,
        auth: &AuthContext,
        account_id: Option<Uuid>,
    ) -> Result<Vec<Order>, AuthError> {
        if !auth.has_permission(permissions::ORDERS_READ) {
            return Err(AuthError::InsufficientPermissions(
                "orders:read required".into()
            ));
        }

        let target_account = account_id.unwrap_or(auth.account_id);
        
        // Check if can view other accounts
        if target_account != auth.account_id && !auth.has_permission("orders:read_all") {
            return Err(AuthError::InsufficientPermissions(
                "Cannot view others' orders".into()
            ));
        }

        let orders = sqlx::query_as!(
            Order,
            "SELECT * FROM orders WHERE account_id = $1 ORDER BY created_at DESC LIMIT 100",
            target_account
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AuthError::DatabaseError)?;

        Ok(orders)
    }
}
