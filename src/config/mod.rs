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
    /// Header name to inject the API key (optional, used when injecting as header)
    #[serde(default = "default_header_name")]
    pub header_name: String,
    /// Query parameter name to inject the API key (optional, used when injecting as query param)
    #[serde(default)]
    pub query_param_name: Option<String>,
}

fn default_header_name() -> String {
    "Authorization".to_string()
}

/// Route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Route name (optional, for referencing from servers)
    #[serde(default)]
    pub name: Option<String>,
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
    /// Server name (optional, for display purposes)
    #[serde(default)]
    pub name: Option<String>,
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,
    /// Port to bind to
    #[serde(default = "default_port")]
    pub port: u16,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Routes associated with this server (optional, if not set uses global routes)
    #[serde(default)]
    pub routes: Vec<String>,
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
            name: None,
            host: default_host(),
            port: default_port(),
            timeout: default_timeout(),
            routes: vec![],
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

/// Master access token guard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterAccessTokenConfig {
    /// Whether the master access token guard is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Header name to check for the access token
    #[serde(default = "default_master_token_header_name")]
    pub header_name: String,
    /// List of valid tokens (any one of these tokens will be accepted)
    #[serde(default)]
    pub tokens: Vec<String>,
}

fn default_master_token_header_name() -> String {
    "Authorization".to_string()
}

impl Default for MasterAccessTokenConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            header_name: default_master_token_header_name(),
            tokens: vec![],
        }
    }
}

impl MasterAccessTokenConfig {
    /// Validate an incoming token against the configured tokens
    /// Returns true if access should be allowed, false otherwise
    pub fn validate_token(&self, token: &str) -> bool {
        // If guard is not enabled, allow all access
        if !self.enabled {
            return true;
        }
        // Defense in depth: if enabled but no tokens configured, deny access
        // (This case should be caught by config validation, but handle it safely)
        if self.tokens.is_empty() {
            return false;
        }
        // Check if the provided token matches any configured token
        self.tokens.iter().any(|t| t == token)
    }
}

/// Main gateway configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Single server configuration (for backward compatibility)
    #[serde(default)]
    pub server: ServerConfig,
    /// Multiple servers configuration
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,
    /// Health check configuration
    #[serde(default)]
    pub health: HealthConfig,
    /// Master access token guard configuration
    #[serde(default)]
    pub master_access_token: MasterAccessTokenConfig,
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

        // Check that servers reference valid routes
        for server in &self.servers {
            for route_ref in &server.routes {
                let route_exists = self.routes.iter().any(|r| {
                    r.name.as_ref().map(|n| n == route_ref).unwrap_or(false) || r.path == *route_ref
                });
                if !route_exists {
                    anyhow::bail!(
                        "Server '{}' references unknown route '{}'",
                        server
                            .name
                            .as_deref()
                            .unwrap_or(&format!("{}:{}", server.host, server.port)),
                        route_ref
                    );
                }
            }
        }

        // Validate master access token configuration
        if self.master_access_token.enabled && self.master_access_token.tokens.is_empty() {
            anyhow::bail!("Master access token guard is enabled but no tokens are configured");
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

    /// Get all configured servers (returns either `servers` list or a single-item list with `server`)
    pub fn get_servers(&self) -> Vec<&ServerConfig> {
        if !self.servers.is_empty() {
            self.servers.iter().collect()
        } else {
            vec![&self.server]
        }
    }

    /// Get routes for a specific server
    /// If the server has no routes specified, returns all enabled routes
    pub fn routes_for_server(&self, server: &ServerConfig) -> Vec<&RouteConfig> {
        if server.routes.is_empty() {
            // No specific routes - use all enabled routes
            self.enabled_routes()
        } else {
            // Filter routes by the server's route references
            self.routes
                .iter()
                .filter(|r| {
                    r.enabled
                        && server.routes.iter().any(|route_ref| {
                            r.name.as_ref().map(|n| n == route_ref).unwrap_or(false)
                                || r.path == *route_ref
                        })
                })
                .collect()
        }
    }

    /// Get server address for a specific server
    pub fn server_addr_for(server: &ServerConfig) -> String {
        format!("{}:{}", server.host, server.port)
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

    #[test]
    fn test_multiple_servers_config() {
        let toml = r#"
[metrics]
enabled = true
path = "/metrics"

[health]
enabled = true
path = "/health"

[[servers]]
name = "api-server"
host = "0.0.0.0"
port = 8080
routes = ["api-v1"]

[[servers]]
name = "admin-server"
host = "0.0.0.0"
port = 9090
routes = ["admin"]

[[routes]]
name = "api-v1"
path = "/api/v1/*"
target = "http://localhost:3001"
strip_prefix = true
description = "API v1 routes"
enabled = true

[[routes]]
name = "admin"
path = "/admin/*"
target = "http://localhost:3002"
strip_prefix = true
description = "Admin routes"
enabled = true
"#;

        let config = GatewayConfig::parse(toml).unwrap();
        assert_eq!(config.servers.len(), 2);
        assert_eq!(config.servers[0].name, Some("api-server".to_string()));
        assert_eq!(config.servers[0].port, 8080);
        assert_eq!(config.servers[1].name, Some("admin-server".to_string()));
        assert_eq!(config.servers[1].port, 9090);
        assert_eq!(config.routes.len(), 2);

        // Test get_servers
        let servers = config.get_servers();
        assert_eq!(servers.len(), 2);

        // Test routes_for_server
        let api_routes = config.routes_for_server(&config.servers[0]);
        assert_eq!(api_routes.len(), 1);
        assert_eq!(api_routes[0].path, "/api/v1/*");

        let admin_routes = config.routes_for_server(&config.servers[1]);
        assert_eq!(admin_routes.len(), 1);
        assert_eq!(admin_routes[0].path, "/admin/*");
    }

    #[test]
    fn test_server_with_all_routes() {
        let toml = r#"
[[servers]]
name = "main-server"
host = "0.0.0.0"
port = 8080
# No routes specified - should use all enabled routes

[[routes]]
path = "/api/v1/*"
target = "http://localhost:3001"

[[routes]]
path = "/api/v2/*"
target = "http://localhost:3002"
"#;

        let config = GatewayConfig::parse(toml).unwrap();
        let routes = config.routes_for_server(&config.servers[0]);
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_invalid_server_route_reference() {
        let toml = r#"
[[servers]]
name = "main-server"
host = "0.0.0.0"
port = 8080
routes = ["nonexistent-route"]

[[routes]]
name = "api-v1"
path = "/api/v1/*"
target = "http://localhost:3001"
"#;

        let result = GatewayConfig::parse(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_backward_compatibility_single_server() {
        let toml = r#"
[server]
host = "127.0.0.1"
port = 3000

[[routes]]
path = "/api/*"
target = "http://localhost:8081"
"#;

        let config = GatewayConfig::parse(toml).unwrap();
        // Should fall back to single server config
        let servers = config.get_servers();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].host, "127.0.0.1");
        assert_eq!(servers[0].port, 3000);
    }

    #[test]
    fn test_master_access_token_default() {
        let config = GatewayConfig::default();
        assert!(!config.master_access_token.enabled);
        assert_eq!(config.master_access_token.header_name, "Authorization");
        assert!(config.master_access_token.tokens.is_empty());
    }

    #[test]
    fn test_master_access_token_parse() {
        let toml = r#"
[master_access_token]
enabled = true
header_name = "X-Gateway-Token"
tokens = ["token1", "token2"]

[[routes]]
path = "/api/*"
target = "http://localhost:8081"
"#;

        let config = GatewayConfig::parse(toml).unwrap();
        assert!(config.master_access_token.enabled);
        assert_eq!(config.master_access_token.header_name, "X-Gateway-Token");
        assert_eq!(config.master_access_token.tokens.len(), 2);
        assert_eq!(config.master_access_token.tokens[0], "token1");
        assert_eq!(config.master_access_token.tokens[1], "token2");
    }

    #[test]
    fn test_master_access_token_validation() {
        let config = MasterAccessTokenConfig {
            enabled: true,
            header_name: "Authorization".to_string(),
            tokens: vec!["valid-token".to_string(), "another-valid-token".to_string()],
        };

        assert!(config.validate_token("valid-token"));
        assert!(config.validate_token("another-valid-token"));
        assert!(!config.validate_token("invalid-token"));
    }

    #[test]
    fn test_master_access_token_disabled_allows_all() {
        let config = MasterAccessTokenConfig {
            enabled: false,
            header_name: "Authorization".to_string(),
            tokens: vec!["valid-token".to_string()],
        };

        // When disabled, any token should be valid
        assert!(config.validate_token("any-token"));
        assert!(config.validate_token(""));
    }

    #[test]
    fn test_master_access_token_enabled_no_tokens_error() {
        let toml = r#"
[master_access_token]
enabled = true
tokens = []

[[routes]]
path = "/api/*"
target = "http://localhost:8081"
"#;

        let result = GatewayConfig::parse(toml);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Master access token guard is enabled but no tokens are configured"));
    }

    #[test]
    fn test_master_access_token_defense_in_depth() {
        // Test that validate_token returns false when enabled but tokens are empty
        // This is a defense-in-depth check that should never happen in practice
        // because config validation catches this case
        let config = MasterAccessTokenConfig {
            enabled: true,
            header_name: "Authorization".to_string(),
            tokens: vec![], // Empty tokens - should deny access
        };

        // Should deny access even with any token
        assert!(!config.validate_token("any-token"));
        assert!(!config.validate_token(""));
    }
}
