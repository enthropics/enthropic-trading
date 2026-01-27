//! Authentication & Authorization Module
//! Phase 2: JWT validation, RBAC, token blacklist

use chrono::Utc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashSet;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub account_id: Uuid,
    pub username: String,
    pub role: String,
    pub permissions: HashSet<String>,
    pub token_jti: String,
}

impl AuthContext {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission) || self.permissions.contains("admin:full")
    }

    pub fn can_access_account(&self, target: &Uuid) -> bool {
        &self.account_id == target || self.has_permission("admin:full") || self.has_permission("accounts:read_all")
    }
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Token revoked")]
    TokenRevoked,
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    #[error("Account not found")]
    AccountNotFound,
    #[error("Account disabled")]
    AccountDisabled,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

pub struct AuthService {
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new(jwt_secret: &str) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
        }
    }

    pub async fn validate_token(
        &self,
        token: &str,
        pool: &PgPool,
        redis: &mut redis::aio::ConnectionManager,
    ) -> Result<AuthContext, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(e.to_string()),
            })?;

        let claims = token_data.claims;

        // Check blacklist
        let blacklist_key = format!("token_blacklist:{}", claims.jti);
        let is_blacklisted: bool = redis.exists(&blacklist_key).await?;
        if is_blacklisted {
            return Err(AuthError::TokenRevoked);
        }

        // Verify account
        let account = sqlx::query!(
            "SELECT id, is_active, locked_until FROM accounts WHERE id = $1",
            Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken("Invalid UUID".into()))?
        )
        .fetch_optional(pool)
        .await?
        .ok_or(AuthError::AccountNotFound)?;

        if !account.is_active {
            return Err(AuthError::AccountDisabled);
        }

        if let Some(locked) = account.locked_until {
            if locked > Utc::now() {
                return Err(AuthError::AccountDisabled);
            }
        }

        Ok(AuthContext {
            account_id: account.id,
            username: claims.username,
            role: claims.role,
            permissions: claims.permissions.into_iter().collect(),
            token_jti: claims.jti,
        })
    }
}

pub mod permissions {
    pub const ORDERS_CREATE: &str = "orders:create";
    pub const ORDERS_READ: &str = "orders:read";
    pub const ORDERS_CANCEL: &str = "orders:cancel";
    pub const POSITIONS_READ: &str = "positions:read";
    pub const MARKET_READ: &str = "market:read";
    pub const ADMIN_FULL: &str = "admin:full";
}
