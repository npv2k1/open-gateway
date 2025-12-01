//! Open Gateway - CLI Application
//!
//! A simple and fast API gateway service with:
//! - Route configuration via TOML
//! - API key pool management
//! - Prometheus metrics
//! - TUI monitoring

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use clap::{Parser, Subcommand};
use open_gateway::{
    api_key::{create_selector, SharedApiKeySelector},
    config::GatewayConfig,
    health::HealthChecker,
    metrics::GatewayMetrics,
    proxy::ProxyService,
    tui::MonitorApp,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
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
    #[allow(dead_code)]
    config: GatewayConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config } => start_server(&config).await?,
        Commands::Monitor { config } => start_monitor(&config).await?,
        Commands::Validate { config } => validate_config(&config)?,
        Commands::Init { output } => generate_sample_config(&output)?,
    }

    Ok(())
}

/// Start the gateway server
async fn start_server(config_path: &str) -> anyhow::Result<()> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load configuration
    let config = GatewayConfig::from_file(config_path)?;
    info!("Loaded configuration from {}", config_path);

    // Create API key selectors
    let api_key_selectors: HashMap<String, SharedApiKeySelector> = config
        .api_key_pools
        .iter()
        .map(|(name, pool)| (name.clone(), create_selector(pool)))
        .collect();

    // Create metrics
    let metrics = Arc::new(GatewayMetrics::new());

    // Create health checker
    let health = Arc::new(HealthChecker::new());

    // Create proxy routes
    let proxy_routes = ProxyService::routes_from_config(&config.routes, &api_key_selectors);
    let proxy = Arc::new(ProxyService::new(proxy_routes, metrics.clone()));

    // Create app state
    let state = AppState {
        proxy,
        metrics: metrics.clone(),
        health: health.clone(),
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route(&config.health.path, get(health_handler))
        .route(&config.metrics.path, get(metrics_handler))
        .fallback(proxy_handler)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    if config.health.enabled {
        info!("Health endpoint enabled at {}", config.health.path);
    }
    if config.metrics.enabled {
        info!("Metrics endpoint enabled at {}", config.metrics.path);
    }

    // Start server
    let addr: SocketAddr = config.server_addr().parse()?;
    info!("Starting gateway server on {}", addr);
    info!("Routes configured: {}", config.routes.len());
    info!("API key pools configured: {}", config.api_key_pools.len());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

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
            println!("Server: {}:{}", config.server.host, config.server.port);
            println!("Routes: {}", config.routes.len());
            println!("API Key Pools: {}", config.api_key_pools.len());
            println!();
            println!("Routes:");
            for route in &config.routes {
                let status = if route.enabled { "✓" } else { "✗" };
                println!("  {} {} → {}", status, route.path, route.target);
            }
            println!();
            println!("API Key Pools:");
            for (name, pool) in &config.api_key_pools {
                println!(
                    "  {} ({:?}, {} keys)",
                    name,
                    pool.strategy,
                    pool.keys.len()
                );
            }
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

[server]
host = "0.0.0.0"
port = 8080
timeout = 30

[metrics]
enabled = true
path = "/metrics"

[health]
enabled = true
path = "/health"

# Route configurations
[[routes]]
path = "/api/v1/*"
target = "http://localhost:3001"
strip_prefix = true
methods = ["GET", "POST", "PUT", "DELETE"]
api_key_pool = "default"
description = "API v1 routes"
enabled = true

[[routes]]
path = "/api/v2/*"
target = "http://localhost:3002"
strip_prefix = true
description = "API v2 routes"
enabled = true

# API Key Pools
[api_key_pools.default]
strategy = "round_robin"  # Options: round_robin, random, weight
header_name = "X-API-Key"
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
async fn proxy_handler(
    State(state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    match state.proxy.forward(req).await {
        Ok(response) => response.into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}
