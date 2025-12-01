//! Health check module
//!
//! This module provides health check functionality for the gateway service:
//! - Basic liveness check
//! - Readiness check with upstream service health

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
        }
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Health checker service
#[derive(Clone)]
pub struct HealthChecker {
    start_time: Instant,
    ready: Arc<AtomicBool>,
    version: String,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            ready: Arc::new(AtomicBool::new(true)),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Get liveness status (always healthy if the service is running)
    pub fn liveness(&self) -> HealthResponse {
        HealthResponse {
            status: HealthStatus::Healthy,
            version: self.version.clone(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            message: None,
        }
    }

    /// Get readiness status
    pub fn readiness(&self) -> HealthResponse {
        let is_ready = self.ready.load(Ordering::Relaxed);

        HealthResponse {
            status: if is_ready {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            },
            version: self.version.clone(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            message: if is_ready {
                None
            } else {
                Some("Service is not ready".to_string())
            },
        }
    }

    /// Set the readiness status
    pub fn set_ready(&self, ready: bool) {
        self.ready.store(ready, Ordering::Relaxed);
    }

    /// Check if the service is ready
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get formatted uptime string
    pub fn uptime_formatted(&self) -> String {
        let total_seconds = self.uptime_seconds();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liveness() {
        let checker = HealthChecker::new();
        let health = checker.liveness();

        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(!health.version.is_empty());
    }

    #[test]
    fn test_readiness() {
        let checker = HealthChecker::new();

        // Default is ready
        let health = checker.readiness();
        assert_eq!(health.status, HealthStatus::Healthy);

        // Set not ready
        checker.set_ready(false);
        let health = checker.readiness();
        assert_eq!(health.status, HealthStatus::Unhealthy);

        // Set ready again
        checker.set_ready(true);
        let health = checker.readiness();
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_uptime_formatted() {
        let checker = HealthChecker::new();
        let uptime = checker.uptime_formatted();

        // Should start with a number
        assert!(uptime.chars().next().unwrap().is_ascii_digit());
    }
}
