//! Open Gateway - A simple and fast API gateway service
//!
//! This is a gateway service that provides:
//! - Request routing to microservices
//! - API key pool management with multiple selection strategies
//! - Prometheus metrics
//! - Health checks
//! - TUI monitoring

pub mod api_key;
pub mod config;
pub mod health;
pub mod metrics;
pub mod proxy;
pub mod tui;

pub use config::GatewayConfig;

/// Application result type
pub type Result<T> = anyhow::Result<T>;
