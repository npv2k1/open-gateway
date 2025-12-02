//! Open Gateway - CLI Application
//!
//! A simple and fast API gateway service with:
//! - Route configuration via TOML
//! - API key pool management
//! - Prometheus metrics
//! - TUI monitoring
//! - Master access token guard for gateway protection
//! - Hot reload support when config file changes

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use clap::{Parser, Subcommand};
use notify::{Event, RecursiveMode, Watcher};
use open_gateway::{
    api_key::{create_selector, SharedApiKeySelector},
    config::GatewayConfig,
    health::HealthChecker,
    metrics::GatewayMetrics,
    proxy::ProxyService,
    tui::MonitorApp,
    MasterAccessTokenConfig,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::watch;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

/// Open Gateway - A simple and fast API gateway service
#[derive(Parser)]
#[command(name = "open-gateway")]
#[command(version, about = "A simple and fast API gateway service", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the gateway server
    Start {
        /// Configuration file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
        /// Watch config file for changes and hot reload
        #[arg(short, long, default_value = "false")]
        watch: bool,
    },
    /// Start the TUI monitor
    Monitor {
        /// Configuration file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
    },
    /// Validate the configuration file
    Validate {
        /// Configuration file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
    },
    /// Generate a sample configuration file
    Init {
        /// Output file path
        #[arg(short, long, default_value = "config.toml")]
        output: String,
    },
}

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    proxy: Arc<ProxyService>,
    metrics: Arc<GatewayMetrics>,
    health: Arc<HealthChecker>,
    master_access_token: MasterAccessTokenConfig,
    #[allow(dead_code)]
    config: GatewayConfig,
}

/// Master access token guard middleware
///
/// When enabled, this middleware validates that incoming requests include a valid
/// access token in the configured header. This applies to ALL endpoints including
/// health checks and metrics endpoints for maximum security.
///
/// If you need to exclude health/metrics from authentication, consider running
/// a separate server instance without the guard for internal monitoring.
async fn master_access_token_guard(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // If guard is not enabled, pass through
    if !state.master_access_token.enabled {
        return next.run(req).await;
    }

    // Get the token from the configured header
    let token = req
        .headers()
        .get(&state.master_access_token.header_name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Validate the token
    if state.master_access_token.validate_token(token) {
        next.run(req).await
    } else {
        (StatusCode::UNAUTHORIZED, "Invalid or missing access token").into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config, watch } => start_server(&config, watch).await?,
        Commands::Monitor { config } => start_monitor(&config).await?,
        Commands::Validate { config } => validate_config(&config)?,
        Commands::Init { output } => generate_sample_config(&output)?,
    }

    Ok(())
}

/// Start the gateway server with optional hot reload
async fn start_server(config_path: &str, watch_config: bool) -> anyhow::Result<()> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Create a channel for shutdown signaling
    let (shutdown_tx, _) = watch::channel(false);

    // Start config file watcher if enabled
    let config_path_owned = config_path.to_string();
    let shutdown_tx_clone = shutdown_tx.clone();

    if watch_config {
        info!("Hot reload enabled - watching {} for changes", config_path);
        let config_path_for_watcher = config_path_owned.clone();
        tokio::spawn(async move {
            watch_config_file(&config_path_for_watcher, shutdown_tx_clone).await;
        });
    }

    // Run server loop (restarts on config change when watch is enabled)
    loop {
        let mut shutdown_rx = shutdown_tx.subscribe();

        match run_servers(&config_path_owned, shutdown_rx.clone()).await {
            Ok(()) => {
                if watch_config {
                    // Check if we got a shutdown signal (config changed)
                    if *shutdown_rx.borrow() {
                        info!("Config changed, reloading servers...");
                        // Reset the shutdown signal for the next iteration
                        let _ = shutdown_tx.send(false);
                        continue;
                    }
                }
                break;
            }
            Err(e) => {
                error!("Server error: {}", e);
                if watch_config {
                    warn!("Waiting for config change to retry...");
                    // Wait for config change before retrying
                    loop {
                        if shutdown_rx.changed().await.is_err() {
                            return Err(e);
                        }
                        if *shutdown_rx.borrow() {
                            let _ = shutdown_tx.send(false);
                            break;
                        }
                    }
                    continue;
                }
                return Err(e);
            }
        }
    }

    Ok(())
}

/// Watch config file for changes and trigger reload
async fn watch_config_file(config_path: &str, shutdown_tx: watch::Sender<bool>) {
    let path = Path::new(config_path);
    let parent_dir = path.parent().unwrap_or(Path::new("."));
    let config_file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_string();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Event, notify::Error>>(10);

    let mut watcher = match notify::recommended_watcher(move |res| {
        // Use try_send to avoid blocking the file system event thread
        // If the channel is full, we drop the event (the next event will still trigger reload)
        let _ = tx.try_send(res);
    }) {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to create file watcher: {}", e);
            return;
        }
    };

    if let Err(e) = watcher.watch(parent_dir, RecursiveMode::NonRecursive) {
        error!("Failed to watch config directory: {}", e);
        return;
    }

    info!("Watching {} for changes", config_path);

    while let Some(result) = rx.recv().await {
        match result {
            Ok(event) => {
                // Check if the event is for our config file
                let is_config_file = event.paths.iter().any(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n == config_file_name)
                        .unwrap_or(false)
                });

                if is_config_file {
                    match event.kind {
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                            // Validate new config before triggering reload
                            match GatewayConfig::from_file(config_path) {
                                Ok(_) => {
                                    info!("Config file changed, triggering reload...");
                                    let _ = shutdown_tx.send(true);
                                }
                                Err(e) => {
                                    warn!("Config file changed but invalid: {}", e);
                                    warn!("Keeping current configuration");
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("File watch error: {}", e);
            }
        }
    }
}

/// Run all servers from configuration
async fn run_servers(
    config_path: &str,
    mut shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    // Load configuration
    let config = GatewayConfig::from_file(config_path)?;
    info!("Loaded configuration from {}", config_path);

    // Create API key selectors
    let api_key_selectors: HashMap<String, SharedApiKeySelector> = config
        .api_key_pools
        .iter()
        .map(|(name, pool)| (name.clone(), create_selector(pool)))
        .collect();

    // Create shared metrics
    let metrics = Arc::new(GatewayMetrics::new());

    // Create shared health checker
    let health = Arc::new(HealthChecker::new());

    // Get all servers to start
    let servers = config.get_servers();
    info!("Starting {} server(s)", servers.len());
    info!("Routes configured: {}", config.routes.len());
    info!("API key pools configured: {}", config.api_key_pools.len());
    if config.master_access_token.enabled {
        info!(
            "Master access token guard enabled (header: {})",
            config.master_access_token.header_name
        );
    }

    // Spawn a task for each server
    let mut handles = Vec::new();

    for server in servers {
        // Get routes for this server
        let server_routes: Vec<_> = config
            .routes_for_server(server)
            .into_iter()
            .cloned()
            .collect();

        let proxy_routes = ProxyService::routes_from_config(&server_routes, &api_key_selectors);
        let proxy = Arc::new(ProxyService::new(proxy_routes, metrics.clone()));

        // Create app state for this server
        let state = AppState {
            proxy,
            metrics: metrics.clone(),
            health: health.clone(),
            master_access_token: config.master_access_token.clone(),
            config: config.clone(),
        };

        // Build router with master access token guard middleware
        let app = Router::new()
            .route(&config.health.path, get(health_handler))
            .route(&config.metrics.path, get(metrics_handler))
            .fallback(proxy_handler)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                master_access_token_guard,
            ))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        // Get server address
        let addr: SocketAddr = GatewayConfig::server_addr_for(server).parse()?;
        let server_name = server
            .name
            .clone()
            .unwrap_or_else(|| format!("{}:{}", server.host, server.port));

        info!(
            "Starting server '{}' on {} with {} route(s)",
            server_name,
            addr,
            server_routes.len()
        );

        if config.health.enabled {
            info!("  Health endpoint at {}", config.health.path);
        }
        if config.metrics.enabled {
            info!("  Metrics endpoint at {}", config.metrics.path);
        }

        // Spawn the server task with graceful shutdown support
        let server_shutdown_rx = shutdown_rx.clone();
        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app.into_make_service())
                .with_graceful_shutdown(async move {
                    let mut rx = server_shutdown_rx;
                    loop {
                        if rx.changed().await.is_err() {
                            break;
                        }
                        if *rx.borrow() {
                            break;
                        }
                    }
                })
                .await?;
            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }

    // Wait for shutdown signal or server error
    tokio::select! {
        _ = async {
            loop {
                if shutdown_rx.changed().await.is_err() {
                    break;
                }
                if *shutdown_rx.borrow() {
                    break;
                }
            }
        } => {
            info!("Shutdown signal received, stopping servers...");
        }
        result = async {
            for handle in handles {
                handle.await??;
            }
            Ok::<(), anyhow::Error>(())
        } => {
            return result;
        }
    }

    Ok(())
}

/// Start the TUI monitor
async fn start_monitor(config_path: &str) -> anyhow::Result<()> {
    // Load configuration
    let config = GatewayConfig::from_file(config_path)?;

    // Create API key selectors
    let api_key_selectors: HashMap<String, SharedApiKeySelector> = config
        .api_key_pools
        .iter()
        .map(|(name, pool)| (name.clone(), create_selector(pool)))
        .collect();

    // Create metrics (for display, not connected to real server)
    let metrics = Arc::new(GatewayMetrics::new());

    // Create health checker
    let health = Arc::new(HealthChecker::new());

    // Create proxy routes for display
    let proxy_routes = ProxyService::routes_from_config(&config.routes, &api_key_selectors);

    // Run TUI
    let mut app = MonitorApp::new(config, metrics, health, proxy_routes);
    app.run().await?;

    Ok(())
}

/// Validate configuration file
fn validate_config(config_path: &str) -> anyhow::Result<()> {
    match GatewayConfig::from_file(config_path) {
        Ok(config) => {
            println!("✓ Configuration is valid!");
            println!();

            // Display servers
            let servers = config.get_servers();
            println!("Servers: {}", servers.len());
            for server in &servers {
                let name = server
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("{}:{}", server.host, server.port));
                let route_count = config.routes_for_server(server).len();
                println!(
                    "  {} ({}:{}) - {} route(s)",
                    name, server.host, server.port, route_count
                );
            }
            println!();

            println!("Routes: {}", config.routes.len());
            for route in &config.routes {
                let status = if route.enabled { "✓" } else { "✗" };
                let name = route
                    .name
                    .clone()
                    .map(|n| format!("[{}] ", n))
                    .unwrap_or_default();
                println!("  {} {}{} → {}", status, name, route.path, route.target);
            }
            println!();

            println!("API Key Pools: {}", config.api_key_pools.len());
            for (name, pool) in &config.api_key_pools {
                println!("  {} ({:?}, {} keys)", name, pool.strategy, pool.keys.len());
            }
            println!();

            println!(
                "Master Access Token Guard: {}",
                if config.master_access_token.enabled {
                    format!(
                        "enabled (header: {}, {} token(s))",
                        config.master_access_token.header_name,
                        config.master_access_token.tokens.len()
                    )
                } else {
                    "disabled".to_string()
                }
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Configuration is invalid:");
            eprintln!("  {}", e);
            std::process::exit(1);
        }
    }
}

/// Generate sample configuration file
fn generate_sample_config(output_path: &str) -> anyhow::Result<()> {
    let sample_config = r#"# Open Gateway Configuration
# This configuration shows both single-server (backward compatible) and
# multi-server configurations. Use either `[server]` OR `[[servers]]`.
#
# Features:
# - HTTP and HTTPS target support
# - Hot reload: use `--watch` flag to auto-reload on config changes

# Option 1: Single server configuration (backward compatible)
# [server]
# host = "0.0.0.0"
# port = 8080
# timeout = 30

# Option 2: Multiple servers configuration
# Each server can have its own routes. If no routes are specified,
# all enabled routes are used for that server.

[[servers]]
name = "api-server"
host = "0.0.0.0"
port = 8080
timeout = 30
routes = ["api-v1", "api-v2"]  # Reference routes by name or path

[[servers]]
name = "admin-server"
host = "0.0.0.0"
port = 9090
timeout = 30
# No routes specified - uses all enabled routes

[metrics]
enabled = true
path = "/metrics"

[health]
enabled = true
path = "/health"

# Master Access Token Guard Configuration
# When enabled, all requests must include a valid token in the specified header
# to access the gateway. This protects the gateway from unauthorized access.
# NOTE: This applies to ALL endpoints including /health and /metrics.
# For internal monitoring without authentication, use a separate server instance.
[master_access_token]
enabled = false  # Set to true to enable the guard
header_name = "Authorization"  # Header name to check for the token
tokens = [
    # "Bearer your-secret-token-1",
    # "Bearer your-secret-token-2",
]

# Route configurations
# Routes can have a `name` field to be referenced by servers
# Target can be HTTP or HTTPS URLs
[[routes]]
name = "api-v1"
path = "/api/v1/*"
target = "http://localhost:3001"  # HTTP target
strip_prefix = true
methods = ["GET", "POST", "PUT", "DELETE"]
api_key_pool = "default"
description = "API v1 routes"
enabled = true

[[routes]]
name = "api-v2"
path = "/api/v2/*"
target = "https://api.example.com"  # HTTPS target
strip_prefix = true
description = "API v2 routes (HTTPS)"
enabled = true

[[routes]]
name = "admin"
path = "/admin/*"
target = "http://localhost:4000"
strip_prefix = true
description = "Admin routes"
enabled = true

# API Key Pools
# API keys can be injected as headers (header_name) or query parameters (query_param_name)
[api_key_pools.default]
strategy = "round_robin"  # Options: round_robin, random, weight
header_name = "X-API-Key"  # Inject API key as header
keys = [
    { key = "api-key-1", weight = 1, enabled = true },
    { key = "api-key-2", weight = 2, enabled = true },
    { key = "api-key-3", weight = 1, enabled = true },
]

[api_key_pools.openai]
strategy = "weight"
header_name = "Authorization"
keys = [
    { key = "Bearer sk-key-1", weight = 3, enabled = true },
    { key = "Bearer sk-key-2", weight = 1, enabled = true },
]

# Example: Inject API key as query parameter instead of header
[api_key_pools.query_key]
strategy = "round_robin"
query_param_name = "api_key"  # Inject API key as query parameter: ?api_key=...
keys = [
    { key = "key-1", weight = 1, enabled = true },
    { key = "key-2", weight = 1, enabled = true },
]
"#;

    std::fs::write(output_path, sample_config)?;
    println!("Sample configuration written to {}", output_path);
    Ok(())
}

/// Health check handler
async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let health = state.health.liveness();
    (
        if matches!(health.status, open_gateway::health::HealthStatus::Healthy) {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(health),
    )
}

/// Metrics handler
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let output = state.metrics.prometheus_output();
    (StatusCode::OK, output)
}

/// Proxy handler - forwards requests to target services
async fn proxy_handler(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    match state.proxy.forward(req).await {
        Ok(response) => response.into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}
