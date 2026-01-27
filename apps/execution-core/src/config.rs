//! Configuration Module for Execution Core

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub nats_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub pool_min_connections: u32,
    pub pool_max_connections: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/enthropic".into()),
            nats_url: env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".into()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            pool_min_connections: env::var("POOL_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".into())
                .parse()
                .unwrap_or(5),
            pool_max_connections: env::var("POOL_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".into())
                .parse()
                .unwrap_or(20),
        })
    }
}
