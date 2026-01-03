//! Metrics module for Prometheus
//!
//! This module provides metrics collection for the gateway service:
//! - Request count by method, path, and status
//! - Request latency histogram
//! - Active connections gauge
//! - API key usage counter

use prometheus::{
    CounterVec, Encoder, GaugeVec, HistogramOpts, HistogramVec, Opts, Registry, TextEncoder,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Gateway metrics collector
#[derive(Clone)]
pub struct GatewayMetrics {
    registry: Registry,
    request_counter: CounterVec,
    request_latency: HistogramVec,
    active_connections: GaugeVec,
    api_key_usage_counter: CounterVec,
    // Simple counters for TUI display
    total_requests: Arc<AtomicU64>,
    total_errors: Arc<AtomicU64>,
}

impl GatewayMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        let registry = Registry::new();

        let request_counter = CounterVec::new(
            Opts::new("gateway_requests_total", "Total number of requests"),
            &["method", "path", "status"],
        )
        .expect("Failed to create request counter");

        let request_latency = HistogramVec::new(
            HistogramOpts::new(
                "gateway_request_latency_seconds",
                "Request latency in seconds",
            )
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
            &["method", "path"],
        )
        .expect("Failed to create latency histogram");

        let active_connections = GaugeVec::new(
            Opts::new("gateway_active_connections", "Number of active connections"),
            &["route"],
        )
        .expect("Failed to create active connections gauge");

        let api_key_usage_counter = CounterVec::new(
            Opts::new("gateway_api_key_usage_total", "Total number of requests per API key"),
            &["api_key", "route"],
        )
        .expect("Failed to create API key usage counter");

        registry
            .register(Box::new(request_counter.clone()))
            .expect("Failed to register request counter");
        registry
            .register(Box::new(request_latency.clone()))
            .expect("Failed to register latency histogram");
        registry
            .register(Box::new(active_connections.clone()))
            .expect("Failed to register active connections");
        registry
            .register(Box::new(api_key_usage_counter.clone()))
            .expect("Failed to register API key usage counter");

        Self {
            registry,
            request_counter,
            request_latency,
            active_connections,
            api_key_usage_counter,
            total_requests: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a request with its status and latency
    pub fn record_request(&self, method: &str, path: &str, status: u16, latency: Duration) {
        let status_str = status.to_string();

        // Normalize path for metrics (to avoid high cardinality)
        let normalized_path = Self::normalize_path(path);

        self.request_counter
            .with_label_values(&[method, &normalized_path, &status_str])
            .inc();

        self.request_latency
            .with_label_values(&[method, &normalized_path])
            .observe(latency.as_secs_f64());

        // Update simple counters
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if status >= 400 {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Increment active connections for a route
    pub fn inc_active_connections(&self, route: &str) {
        self.active_connections.with_label_values(&[route]).inc();
    }

    /// Decrement active connections for a route
    pub fn dec_active_connections(&self, route: &str) {
        self.active_connections.with_label_values(&[route]).dec();
    }

    /// Record API key usage for a route
    pub fn record_api_key_usage(&self, api_key: &str, route: &str) {
        self.api_key_usage_counter
            .with_label_values(&[api_key, route])
            .inc();
    }

    /// Get the Prometheus metrics output
    pub fn prometheus_output(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    /// Get total request count
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Get total error count
    pub fn total_errors(&self) -> u64 {
        self.total_errors.load(Ordering::Relaxed)
    }

    /// Get error rate (percentage)
    pub fn error_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            (self.total_errors() as f64 / total as f64) * 100.0
        }
    }

    /// Normalize path to reduce cardinality
    /// Replace IDs and numbers with placeholders
    fn normalize_path(path: &str) -> String {
        let parts: Vec<&str> = path.split('/').collect();
        let normalized: Vec<String> = parts
            .iter()
            .map(|part| {
                if part.chars().all(|c| c.is_ascii_digit()) && !part.is_empty() {
                    ":id".to_string()
                } else if part.chars().all(|c| c.is_ascii_hexdigit()) && part.len() >= 8 {
                    ":uuid".to_string()
                } else {
                    (*part).to_string()
                }
            })
            .collect();
        normalized.join("/")
    }

    /// Get metrics snapshot for TUI display
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests(),
            total_errors: self.total_errors(),
            error_rate: self.error_rate(),
        }
    }
}

impl Default for GatewayMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// A snapshot of metrics for display
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_errors: u64,
    pub error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = GatewayMetrics::new();
        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.total_errors(), 0);
    }

    #[test]
    fn test_record_request() {
        let metrics = GatewayMetrics::new();

        metrics.record_request("GET", "/api/users", 200, Duration::from_millis(10));
        assert_eq!(metrics.total_requests(), 1);
        assert_eq!(metrics.total_errors(), 0);

        metrics.record_request("POST", "/api/users", 500, Duration::from_millis(50));
        assert_eq!(metrics.total_requests(), 2);
        assert_eq!(metrics.total_errors(), 1);
    }

    #[test]
    fn test_error_rate() {
        let metrics = GatewayMetrics::new();

        // No requests
        assert_eq!(metrics.error_rate(), 0.0);

        // Add requests
        metrics.record_request("GET", "/", 200, Duration::from_millis(1));
        metrics.record_request("GET", "/", 200, Duration::from_millis(1));
        metrics.record_request("GET", "/", 500, Duration::from_millis(1));
        metrics.record_request("GET", "/", 404, Duration::from_millis(1));

        // 2 errors out of 4 requests = 50%
        assert!((metrics.error_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            GatewayMetrics::normalize_path("/api/users/123"),
            "/api/users/:id"
        );
        assert_eq!(
            GatewayMetrics::normalize_path("/api/users/abc123def456"),
            "/api/users/:uuid"
        );
        assert_eq!(GatewayMetrics::normalize_path("/api/users"), "/api/users");
    }

    #[test]
    fn test_prometheus_output() {
        let metrics = GatewayMetrics::new();
        metrics.record_request("GET", "/api/test", 200, Duration::from_millis(10));

        let output = metrics.prometheus_output();
        assert!(output.contains("gateway_requests_total"));
        assert!(output.contains("gateway_request_latency_seconds"));
    }

    #[test]
    fn test_api_key_usage_counter() {
        let metrics = GatewayMetrics::new();

        // Record some API key usages
        metrics.record_api_key_usage("key1", "/api/v1");
        metrics.record_api_key_usage("key1", "/api/v1");
        metrics.record_api_key_usage("key2", "/api/v1");
        metrics.record_api_key_usage("key1", "/api/v2");

        let output = metrics.prometheus_output();
        assert!(output.contains("gateway_api_key_usage_total"));
        // Check that key1 was recorded for /api/v1
        assert!(output.contains("api_key=\"key1\""));
        assert!(output.contains("api_key=\"key2\""));
    }
}
