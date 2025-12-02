//! Proxy module for forwarding requests to target services
//!
//! This module handles request forwarding, including:
//! - Path manipulation (strip prefix)
//! - Header injection (API keys, custom headers)
//! - Request/Response transformation

use crate::api_key::SharedApiKeySelector;
use crate::config::RouteConfig;
use crate::metrics::GatewayMetrics;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Proxy service for forwarding requests
#[derive(Clone)]
pub struct ProxyService {
    client: Client<
        hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
        http_body_util::combinators::BoxBody<bytes::Bytes, hyper::Error>,
    >,
    routes: Vec<ProxyRoute>,
    metrics: Arc<GatewayMetrics>,
}

/// A compiled proxy route with its selector
#[derive(Clone)]
pub struct ProxyRoute {
    /// Route name (optional)
    pub name: Option<String>,
    /// Path pattern
    pub path_pattern: String,
    /// Target URL
    pub target: String,
    /// Whether to strip the prefix
    pub strip_prefix: bool,
    /// HTTP methods to match (empty = all)
    pub methods: Vec<String>,
    /// API key selector
    pub api_key_selector: Option<SharedApiKeySelector>,
    /// Additional headers
    pub headers: HashMap<String, String>,
    /// Route description
    pub description: Option<String>,
}

impl ProxyRoute {
    /// Check if this route matches the given path and method
    pub fn matches(&self, path: &str, method: &str) -> bool {
        // Check method
        if !self.methods.is_empty() && !self.methods.iter().any(|m| m.eq_ignore_ascii_case(method))
        {
            return false;
        }

        // Check path pattern
        self.path_matches(path)
    }

    /// Check if path matches the pattern
    fn path_matches(&self, path: &str) -> bool {
        let pattern = &self.path_pattern;

        // Handle wildcard patterns
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            return path == prefix || path.starts_with(&format!("{}/", prefix));
        }

        // Handle exact match with optional trailing slash
        if pattern.ends_with('/') {
            let base = &pattern[..pattern.len() - 1];
            return path == base || path == pattern || path.starts_with(pattern);
        }

        // Exact match
        path == pattern || path.starts_with(&format!("{}/", pattern))
    }

    /// Get the target URL for a request path
    pub fn get_target_url(&self, path: &str, query: Option<&str>) -> String {
        let target_path = if self.strip_prefix {
            self.strip_path_prefix(path)
        } else {
            path.to_string()
        };

        let base = self.target.trim_end_matches('/');
        let path_part = if target_path.starts_with('/') {
            target_path
        } else {
            format!("/{}", target_path)
        };

        match query {
            Some(q) if !q.is_empty() => format!("{}{}?{}", base, path_part, q),
            _ => format!("{}{}", base, path_part),
        }
    }

    /// Strip the matched prefix from the path
    fn strip_path_prefix(&self, path: &str) -> String {
        let pattern = &self.path_pattern;

        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            if let Some(remainder) = path.strip_prefix(prefix) {
                if remainder.is_empty() || remainder == "/" {
                    return "/".to_string();
                }
                return remainder.to_string();
            }
        } else if pattern.ends_with('/') {
            let prefix = &pattern[..pattern.len() - 1];
            if let Some(remainder) = path.strip_prefix(prefix) {
                if remainder.is_empty() {
                    return "/".to_string();
                }
                return remainder.to_string();
            }
        }

        path.to_string()
    }
}

impl ProxyService {
    /// Create a new proxy service
    pub fn new(routes: Vec<ProxyRoute>, metrics: Arc<GatewayMetrics>) -> Self {
        // Create HTTPS connector with native roots
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("Failed to load native root certificates")
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        let client = Client::builder(TokioExecutor::new()).build(https);

        Self {
            client,
            routes,
            metrics,
        }
    }

    /// Create proxy routes from configuration
    pub fn routes_from_config(
        routes: &[RouteConfig],
        api_key_selectors: &HashMap<String, SharedApiKeySelector>,
    ) -> Vec<ProxyRoute> {
        routes
            .iter()
            .filter(|r| r.enabled)
            .map(|route| {
                let api_key_selector = route
                    .api_key_pool
                    .as_ref()
                    .and_then(|name| api_key_selectors.get(name).cloned());

                ProxyRoute {
                    name: route.name.clone(),
                    path_pattern: route.path.clone(),
                    target: route.target.clone(),
                    strip_prefix: route.strip_prefix,
                    methods: route.methods.clone(),
                    api_key_selector,
                    headers: route.headers.clone(),
                    description: route.description.clone(),
                }
            })
            .collect()
    }

    /// Forward a request to the appropriate target
    pub async fn forward(
        &self,
        req: Request<Body>,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        // Find matching route
        let route = self
            .routes
            .iter()
            .find(|r| r.matches(&path, &method))
            .ok_or_else(|| {
                self.metrics
                    .record_request(&method, &path, 404, start.elapsed());
                (StatusCode::NOT_FOUND, "No matching route found".to_string())
            })?;

        // Build target URL
        let query = req.uri().query();
        let target_url = route.get_target_url(&path, query);

        // Build new request
        let (parts, body) = req.into_parts();

        let mut builder = Request::builder().method(parts.method).uri(&target_url);

        // Copy headers
        if let Some(headers) = builder.headers_mut() {
            for (key, value) in parts.headers.iter() {
                // Skip hop-by-hop headers
                if !is_hop_by_hop_header(key.as_str()) {
                    headers.insert(key.clone(), value.clone());
                }
            }

            // Add custom headers
            for (key, value) in &route.headers {
                if let Ok(header_name) = key.parse::<axum::http::header::HeaderName>() {
                    if let Ok(header_value) = value.parse::<axum::http::header::HeaderValue>() {
                        headers.insert(header_name, header_value);
                    }
                }
            }

            // Inject API key if configured
            if let Some(selector) = &route.api_key_selector {
                if let Some(api_key) = selector.get_key() {
                    if let Ok(header_name) = selector
                        .header_name
                        .parse::<axum::http::header::HeaderName>()
                    {
                        if let Ok(header_value) = api_key.parse::<axum::http::header::HeaderValue>()
                        {
                            headers.insert(header_name, header_value);
                        }
                    }
                }
            }
        }

        // Convert body to the expected type
        let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
            Ok(bytes) => bytes,
            Err(e) => {
                self.metrics
                    .record_request(&method, &path, 500, start.elapsed());
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read request body: {}", e),
                ));
            }
        };

        let boxed_body = http_body_util::Full::new(body_bytes)
            .map_err(|e| match e {})
            .boxed();

        let new_req = builder.body(boxed_body).map_err(|e| {
            self.metrics
                .record_request(&method, &path, 500, start.elapsed());
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build request: {}", e),
            )
        })?;

        // Send request
        let response = self.client.request(new_req).await.map_err(|e| {
            self.metrics
                .record_request(&method, &path, 502, start.elapsed());
            (
                StatusCode::BAD_GATEWAY,
                format!("Failed to forward request: {}", e),
            )
        })?;

        let status = response.status().as_u16();
        self.metrics
            .record_request(&method, &path, status, start.elapsed());

        // Convert response body
        let (parts, body) = response.into_parts();
        let body_bytes = match http_body_util::BodyExt::collect(body).await {
            Ok(collected) => collected.to_bytes(),
            Err(e) => {
                return Err((
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to read response body: {}", e),
                ));
            }
        };

        let response = Response::from_parts(parts, Body::from(body_bytes));

        Ok(response)
    }

    /// Get all configured routes
    pub fn get_routes(&self) -> &[ProxyRoute] {
        &self.routes
    }
}

/// Check if a header is a hop-by-hop header that should not be forwarded
fn is_hop_by_hop_header(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_route() -> ProxyRoute {
        ProxyRoute {
            name: None,
            path_pattern: "/api/*".to_string(),
            target: "http://localhost:8081".to_string(),
            strip_prefix: true,
            methods: vec![],
            api_key_selector: None,
            headers: HashMap::new(),
            description: Some("Test route".to_string()),
        }
    }

    #[test]
    fn test_route_matching() {
        let route = create_test_route();

        assert!(route.matches("/api/users", "GET"));
        assert!(route.matches("/api/users/1", "POST"));
        assert!(route.matches("/api", "GET"));
        assert!(!route.matches("/other/path", "GET"));
    }

    #[test]
    fn test_method_filtering() {
        let route = ProxyRoute {
            methods: vec!["GET".to_string(), "POST".to_string()],
            ..create_test_route()
        };

        assert!(route.matches("/api/users", "GET"));
        assert!(route.matches("/api/users", "POST"));
        assert!(!route.matches("/api/users", "DELETE"));
    }

    #[test]
    fn test_target_url_with_strip_prefix() {
        let route = create_test_route();

        assert_eq!(
            route.get_target_url("/api/users", None),
            "http://localhost:8081/users"
        );
        assert_eq!(
            route.get_target_url("/api/users/1", None),
            "http://localhost:8081/users/1"
        );
        assert_eq!(
            route.get_target_url("/api/users", Some("page=1")),
            "http://localhost:8081/users?page=1"
        );
    }

    #[test]
    fn test_target_url_without_strip_prefix() {
        let route = ProxyRoute {
            strip_prefix: false,
            ..create_test_route()
        };

        assert_eq!(
            route.get_target_url("/api/users", None),
            "http://localhost:8081/api/users"
        );
    }
}
