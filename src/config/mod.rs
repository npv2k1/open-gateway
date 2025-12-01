//! Configuration module for the gateway service
//!
//! This module handles loading and parsing configuration from TOML files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// API key selection strategy
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyStrategy {
    /// Round-robin selection
    #[default]
    RoundRobin,
    /// Random selection
    Random,
    /// Weighted selection based on configured weights
    Weight,
}

/// API key configuration with optional weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// The API key value
    pub key: String,
    /// Weight for weighted selection (default: 1)
    #[serde(default = "default_weight")]
    pub weight: u32,
    /// Whether the key is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_weight() -> u32 {
    1
}

fn default_enabled() -> bool {
    true
}

/// API key pool configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeyPool {
    /// List of API keys
    #[serde(default)]
    pub keys: Vec<ApiKeyConfig>,
    /// Selection strategy
    #[serde(default)]
    pub strategy: ApiKeyStrategy,
    /// Header name to inject the API key
    #[serde(default = "default_header_name")]
    pub header_name: String,
}

fn default_header_name() -> String {
    "Authorization".to_string()
}

/// Route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Path pattern to match (e.g., "/api/v1/*")
    pub path: String,
    /// Target URL to forward requests to
    pub target: String,
    /// Optional methods to match (if empty, all methods are matched)
    #[serde(default)]
    pub methods: Vec<String>,
    /// Whether to strip the matched prefix from the path
    #[serde(default)]
    pub strip_prefix: bool,
    /// API key pool name to use for this route
    pub api_key_pool: Option<String>,
    /// Additional headers to add to the request
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Route description
    pub description: Option<String>,
    /// Whether the route is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,
    /// Port to bind to
    #[serde(default = "default_port")]
    pub port: u16,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_timeout() -> u64 {
    30
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            timeout: default_timeout(),
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether metrics are enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Path to expose metrics
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: default_metrics_path(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Whether health check is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Path for health check endpoint
    #[serde(default = "default_health_path")]
    pub path: String,
}

fn default_health_path() -> String {
    "/health".to_string()
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: default_health_path(),
        }
    }
}

/// Main gateway configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
    /// Health check configuration
    #[serde(default)]
    pub health: HealthConfig,
    /// Route configurations
    #[serde(default)]
    pub routes: Vec<RouteConfig>,
    /// API key pools
    #[serde(default)]
    pub api_key_pools: HashMap<String, ApiKeyPool>,
}

impl GatewayConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: GatewayConfig = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a TOML string
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let config: GatewayConfig = toml::from_str(s)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        // Check that all routes reference valid API key pools
        for route in &self.routes {
            if let Some(pool_name) = &route.api_key_pool {
                if !self.api_key_pools.contains_key(pool_name) {
                    anyhow::bail!(
                        "Route '{}' references unknown API key pool '{}'",
                        route.path,
                        pool_name
                    );
                }
            }
        }

        // Check that all API key pools have at least one enabled key
        for (name, pool) in &self.api_key_pools {
            let enabled_keys: Vec<_> = pool.keys.iter().filter(|k| k.enabled).collect();
            if enabled_keys.is_empty() {
                anyhow::bail!("API key pool '{}' has no enabled keys", name);
            }
        }

        Ok(())
    }

    /// Get server address
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get enabled routes
    pub fn enabled_routes(&self) -> Vec<&RouteConfig> {
        self.routes.iter().filter(|r| r.enabled).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GatewayConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert!(config.metrics.enabled);
        assert!(config.health.enabled);
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
[server]
host = "127.0.0.1"
port = 3000
timeout = 60

[metrics]
enabled = true
path = "/metrics"

[health]
enabled = true
path = "/health"

[[routes]]
path = "/api/*"
target = "http://localhost:8081"
strip_prefix = true
api_key_pool = "default"
description = "API route"

[api_key_pools.default]
strategy = "round_robin"
header_name = "X-API-Key"
keys = [
    { key = "key1", weight = 2, enabled = true },
    { key = "key2", weight = 1, enabled = true },
]
"#;

        let config = GatewayConfig::parse(toml).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.routes.len(), 1);
        assert_eq!(config.routes[0].path, "/api/*");
        assert!(config.api_key_pools.contains_key("default"));
        assert_eq!(config.api_key_pools["default"].keys.len(), 2);
    }

    #[test]
    fn test_invalid_pool_reference() {
        let toml = r#"
[[routes]]
path = "/api/*"
target = "http://localhost:8081"
api_key_pool = "nonexistent"
"#;

        let result = GatewayConfig::parse(toml);
        assert!(result.is_err());
    }
}
