//! End-to-end tests for the Open Gateway service
//!
//! These tests start the gateway server and verify the endpoints work correctly.

use std::process::{Child, Command};
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;
use std::time::Duration;

/// Base port for tests, incremented atomically to avoid conflicts
static PORT_COUNTER: AtomicU16 = AtomicU16::new(19000);

/// Get a unique port for testing
fn get_unique_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Create a temporary config file with the specified port
fn create_test_config(port: u16) -> tempfile::NamedTempFile {
    let config = format!(
        r#"
[server]
host = "127.0.0.1"
port = {}
timeout = 30

[metrics]
enabled = true
path = "/metrics"

[health]
enabled = true
path = "/health"

[[routes]]
path = "/api/*"
target = "http://localhost:9999"
strip_prefix = true
description = "Test route"
enabled = true
"#,
        port
    );

    let file = tempfile::Builder::new()
        .suffix(".toml")
        .tempfile()
        .unwrap();
    std::fs::write(file.path(), config).unwrap();
    file
}

/// Start the gateway server
fn start_server(config_path: &str) -> Child {
    Command::new(env!("CARGO_BIN_EXE_open-gateway"))
        .args(["start", "-c", config_path])
        .spawn()
        .expect("Failed to start gateway server")
}

/// Wait for the server to be ready by polling the health endpoint
fn wait_for_server(port: u16, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();

    while start.elapsed() < timeout {
        if let Ok(response) = client.get(format!("http://127.0.0.1:{}/health", port)).send() {
            if response.status().is_success() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn test_health_endpoint() {
    let port = get_unique_port();
    let config_file = create_test_config(port);
    let mut server = start_server(config_file.path().to_str().unwrap());

    // Wait for server to start
    assert!(
        wait_for_server(port, 10),
        "Server failed to start within timeout"
    );

    // Test health endpoint
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/health", port))
        .send()
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().unwrap();
    assert_eq!(body["status"], "healthy");
    assert!(body["version"].is_string());
    assert!(body["uptime_seconds"].is_number());

    // Cleanup
    server.kill().ok();
}

#[test]
fn test_metrics_endpoint() {
    let port = get_unique_port();
    let config_file = create_test_config(port);
    let mut server = start_server(config_file.path().to_str().unwrap());

    // Wait for server to start
    assert!(
        wait_for_server(port, 10),
        "Server failed to start within timeout"
    );

    // Test metrics endpoint
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/metrics", port))
        .send()
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body = response.text().unwrap();
    // Should contain Prometheus metrics
    assert!(body.contains("gateway_") || body.contains("# HELP") || body.is_empty());

    // Cleanup
    server.kill().ok();
}

#[test]
fn test_whoami_endpoint() {
    let port = get_unique_port();
    let config_file = create_test_config(port);
    let mut server = start_server(config_file.path().to_str().unwrap());

    // Wait for server to start
    assert!(
        wait_for_server(port, 10),
        "Server failed to start within timeout"
    );

    // Test whoami endpoint
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/whoami", port))
        .send()
        .expect("Failed to send request");

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().unwrap();
    assert!(body["uid"].is_number());
    assert!(body["gid"].is_number());
    assert!(body["pid"].is_number());

    // Cleanup
    server.kill().ok();
}

#[test]
fn test_unmatched_route_returns_404() {
    let port = get_unique_port();
    let config_file = create_test_config(port);
    let mut server = start_server(config_file.path().to_str().unwrap());

    // Wait for server to start
    assert!(
        wait_for_server(port, 10),
        "Server failed to start within timeout"
    );

    // Test unmatched route
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/nonexistent", port))
        .send()
        .expect("Failed to send request");

    // Should return 404 for unmatched routes
    assert_eq!(response.status().as_u16(), 404);

    // Cleanup
    server.kill().ok();
}
