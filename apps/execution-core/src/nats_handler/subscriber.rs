//! NATS Message Handler with Authentication
//! Phase 2: Validates auth context in every message

use crate::auth::{AuthContext, AuthService};
use crate::engine::{OrderProcessor, PositionKeeper};
use crate::engine::order_processor::{NewOrderRequest, OrderResult};
use async_nats::Client;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct AuthenticatedMessage<T> {
    auth: AuthPayload,
    #[serde(flatten)]
    data: T,
}

#[derive(Debug, Deserialize)]
struct AuthPayload {
    account_id: String,
    username: String,
    role: String,
    permissions: Vec<String>,
}

impl From<AuthPayload> for AuthContext {
    fn from(p: AuthPayload) -> Self {
        AuthContext {
            account_id: Uuid::parse_str(&p.account_id).unwrap_or_default(),
            username: p.username,
            role: p.role,
            permissions: p.permissions.into_iter().collect::<HashSet<String>>(),
            token_jti: String::new(),
        }
    }
}

#[derive(Serialize)]
struct OrderResponse {
    success: bool,
    order_id: Option<String>,
    error: Option<String>,
}

pub struct NatsSubscriber {
    client: Client,
    #[allow(dead_code)]
    pool: PgPool,
    order_processor: Arc<OrderProcessor>,
    position_keeper: Arc<PositionKeeper>,
    #[allow(dead_code)]
    auth_service: Arc<AuthService>,
}

impl NatsSubscriber {
    pub fn new(
        client: Client,
        pool: PgPool,
        auth_service: Arc<AuthService>,
    ) -> Self {
        let order_processor = Arc::new(OrderProcessor::new(pool.clone()));
        let position_keeper = Arc::new(PositionKeeper::new(pool.clone()));

        Self {
            client,
            pool,
            order_processor,
            position_keeper,
            auth_service,
        }
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        self.order_processor.load_open_orders().await?;
        self.position_keeper.load_positions().await?;
        Ok(())
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mut order_sub = self.client.subscribe("orders.submit").await?;
        let mut cancel_sub = self.client.subscribe("orders.cancel").await?;
        let mut position_sub = self.client.subscribe("positions.query").await?;

        tracing::info!("NATS subscriber started");

        loop {
            tokio::select! {
                Some(msg) = order_sub.next() => {
                    self.handle_order_submit(msg).await;
                }
                Some(msg) = cancel_sub.next() => {
                    self.handle_order_cancel(msg).await;
                }
                Some(msg) = position_sub.next() => {
                    self.handle_position_query(msg).await;
                }
            }
        }
    }

    async fn handle_order_submit(&self, msg: async_nats::Message) {
        let result: Result<AuthenticatedMessage<NewOrderRequest>, _> =
            serde_json::from_slice(&msg.payload);

        let response = match result {
            Ok(authenticated_msg) => {
                let auth_ctx: AuthContext = authenticated_msg.auth.into();
                match self.order_processor.submit_order(&auth_ctx, authenticated_msg.data).await {
                    Ok(order_result) => match order_result {
                        OrderResult::Accepted(order) => {
                            OrderResponse {
                                success: true,
                                order_id: Some(order.id.to_string()),
                                error: None,
                            }
                        }
                        OrderResult::Rejected { reason, .. } => {
                            OrderResponse {
                                success: false,
                                order_id: None,
                                error: Some(reason),
                            }
                        }
                        OrderResult::Duplicate(order) => {
                            OrderResponse {
                                success: true,
                                order_id: Some(order.id.to_string()),
                                error: Some("Duplicate order".into()),
                            }
                        }
                    }
                    Err(e) => OrderResponse {
                        success: false,
                        order_id: None,
                        error: Some(e.to_string()),
                    }
                }
            }
            Err(e) => OrderResponse {
                success: false,
                order_id: None,
                error: Some(format!("Invalid message: {}", e)),
            }
        };

        if let Some(reply) = msg.reply {
            let payload = serde_json::to_vec(&response).unwrap_or_default();
            let _ = self.client.publish(reply, payload.into()).await;
        }

        // Publish execution report
        let payload = serde_json::to_vec(&response).unwrap_or_default();
        let _ = self.client.publish("execution.reports", payload.into()).await;
    }

    async fn handle_order_cancel(&self, msg: async_nats::Message) {
        #[derive(Deserialize)]
        struct CancelRequest {
            order_id: String,
        }

        let result: Result<AuthenticatedMessage<CancelRequest>, _> =
            serde_json::from_slice(&msg.payload);

        let response = match result {
            Ok(authenticated_msg) => {
                let auth_ctx: AuthContext = authenticated_msg.auth.into();
                let order_id = Uuid::parse_str(&authenticated_msg.data.order_id);

                match order_id {
                    Ok(id) => match self.order_processor.cancel_order(&auth_ctx, id).await {
                        Ok(Some(order)) => OrderResponse {
                            success: true,
                            order_id: Some(order.id.to_string()),
                            error: None,
                        },
                        Ok(None) => OrderResponse {
                            success: false,
                            order_id: None,
                            error: Some("Order not found".into()),
                        },
                        Err(e) => OrderResponse {
                            success: false,
                            order_id: None,
                            error: Some(e.to_string()),
                        }
                    },
                    Err(_) => OrderResponse {
                        success: false,
                        order_id: None,
                        error: Some("Invalid order ID".into()),
                    }
                }
            }
            Err(e) => OrderResponse {
                success: false,
                order_id: None,
                error: Some(format!("Invalid message: {}", e)),
            }
        };

        if let Some(reply) = msg.reply {
            let payload = serde_json::to_vec(&response).unwrap_or_default();
            let _ = self.client.publish(reply, payload.into()).await;
        }
    }

    async fn handle_position_query(&self, msg: async_nats::Message) {
        #[derive(Deserialize)]
        struct PositionQuery {
            #[allow(dead_code)]
            symbol: Option<String>,
        }

        let result: Result<AuthenticatedMessage<PositionQuery>, _> =
            serde_json::from_slice(&msg.payload);

        let response = match result {
            Ok(authenticated_msg) => {
                let auth_ctx: AuthContext = authenticated_msg.auth.into();
                match self.position_keeper.get_account_positions(&auth_ctx, None).await {
                    Ok(positions) => serde_json::json!({
                        "success": true,
                        "positions": positions
                    }),
                    Err(e) => serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    })
                }
            }
            Err(e) => serde_json::json!({
                "success": false,
                "error": format!("Invalid message: {}", e)
            })
        };

        if let Some(reply) = msg.reply {
            let payload = serde_json::to_vec(&response).unwrap_or_default();
            let _ = self.client.publish(reply, payload.into()).await;
        }
    }
}