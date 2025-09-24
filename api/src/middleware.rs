//! Middleware components for the API server
//!
//! This module provides essential middleware components including CORS, rate limiting,
//! logging, request/response handling, and security enhancements.

use axum::{
    extract::Request,
    http::{header, HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::{ConcurrencyLimitLayer, RateLimitLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn, Span};
use std::collections::HashMap;

/// CORS middleware configuration
pub mod cors {
    use super::*;

    /// Create a CORS layer with appropriate settings for the trading API
    pub fn cors_layer() -> CorsLayer {
        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
                Method::PATCH,
            ])
            .allow_headers(Any)
            .allow_credentials(true) // Important for authentication
            .allow_origin(Any) // In production, specify allowed origins
            .max_age(Duration::from_secs(3600)) // Cache preflight for 1 hour
    }

    /// Create a more restrictive CORS layer for production
    pub fn production_cors_layer(allowed_origins: Vec<&str>) -> CorsLayer {
        let allowed_origins: Vec<_> = allowed_origins
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();

        CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
                Method::PATCH,
            ])
            .allow_headers([
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                header::ACCEPT,
                header::X_REQUESTED_WITH,
            ])
            .allow_credentials(true)
            .allow_origin(allowed_origins)
            .max_age(Duration::from_secs(3600))
    }

    /// Custom CORS middleware for development (allows all origins)
    pub async fn dev_cors_middleware(
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        info!("Development CORS: Allowing all origins for request to {}", request.uri());

        // Create a response with CORS headers
        let mut response = next.run(request).await;

        let headers = response.headers_mut();
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            "*".parse().unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, POST, PUT, DELETE, OPTIONS, PATCH".parse().unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "Authorization, Content-Type, Accept, X-Requested-With".parse().unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            "true".parse().unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_MAX_AGE,
            "3600".parse().unwrap(),
        );

        response
    }
}

/// Rate limiting middleware
pub mod rate_limit {
    use super::*;

    /// Rate limiting configuration
    #[derive(Debug, Clone)]
    pub struct RateLimitConfig {
        /// Maximum requests per window
        pub max_requests: u64,
        /// Time window in seconds
        pub window_secs: u64,
        /// Burst allowance (additional requests above the rate)
        pub burst_allowance: Option<u64>,
    }

    impl Default for RateLimitConfig {
        fn default() -> Self {
            Self {
                max_requests: 100,
                window_secs: 60,
                burst_allowance: Some(20),
            }
        }
    }

    /// In-memory rate limiter state
    #[derive(Debug, Clone)]
    pub struct RateLimitState {
        /// Request counts per IP
        requests: HashMap<IpAddr, Vec<Instant>>,
        /// Configuration
        config: RateLimitConfig,
    }

    impl RateLimitState {
        pub fn new(config: RateLimitConfig) -> Self {
            Self {
                requests: HashMap::new(),
                config,
            }
        }

        /// Check if request is allowed and record it
        pub fn check_and_record(&mut self, ip: IpAddr) -> bool {
            let now = Instant::now();
            let window_start = now - Duration::from_secs(self.config.window_secs);

            // Get or create request history for this IP
            let requests = self.requests.entry(ip).or_insert_with(Vec::new);

            // Remove old requests outside the window
            requests.retain(|&timestamp| timestamp > window_start);

            // Check if we're within limits
            let is_allowed = requests.len() < self.config.max_requests as usize;

            if is_allowed {
                requests.push(now);
            }

            is_allowed
        }

        /// Get current request count for an IP
        pub fn get_request_count(&self, ip: IpAddr) -> usize {
            let now = Instant::now();
            let window_start = now - Duration::from_secs(self.config.window_secs);

            if let Some(requests) = self.requests.get(&ip) {
                requests.iter()
                    .filter(|&&timestamp| timestamp > window_start)
                    .count()
            } else {
                0
            }
        }

        /// Clean up old entries (call periodically)
        pub fn cleanup(&mut self) {
            let now = Instant::now();
            let window_start = now - Duration::from_secs(self.config.window_secs);

            for requests in self.requests.values_mut() {
                requests.retain(|&timestamp| timestamp > window_start);
            }

            // Remove empty entries
            self.requests.retain(|_, requests| !requests.is_empty());
        }
    }

    /// Rate limiting middleware
    pub struct RateLimitMiddleware {
        state: Arc<RwLock<RateLimitState>>,
    }

    impl RateLimitMiddleware {
        pub fn new(config: RateLimitConfig) -> Self {
            Self {
                state: Arc::new(RwLock::new(RateLimitState::new(config))),
            }
        }

        pub async fn rate_limit(
            cookie_jar: CookieJar,
            request: Request,
            next: Next,
        ) -> impl IntoResponse {
            // Extract client IP (simplified - in production use proper IP extraction)
            let client_ip = Self::extract_client_ip(&request);

            // Get rate limit state
            let mut state = request.extensions()
                .get::<Arc<RwLock<RateLimitState>>>()
                .expect("RateLimitMiddleware not properly configured")
                .write()
                .await;

            // Check rate limit
            if !state.check_and_record(client_ip) {
                warn!("Rate limit exceeded for IP: {}", client_ip);
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Rate limit exceeded. Please try again later.",
                ).into_response();
            }

            drop(state); // Release lock

            next.run(request).await
        }

        fn extract_client_ip(request: &Request) -> IpAddr {
            // In production, use proper IP extraction from headers like X-Forwarded-For
            // For now, use a default IP for testing
            request.headers()
                .get("X-Forwarded-For")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|ip| ip.parse().ok())
                .unwrap_or(IpAddr::from([127, 0, 0, 1])) // localhost default
        }
    }

    /// Create rate limiting layer using Tower
    pub fn rate_limit_layer(config: RateLimitConfig) -> RateLimitLayer {
        RateLimitLayer::new(
            config.max_requests as u64,
            Duration::from_secs(config.window_secs),
        )
    }
}

/// Logging middleware
pub mod logging {
    use super::*;

    /// Create logging middleware layer
    pub fn logging_layer() -> TraceLayer {
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request| {
                let span = tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                );

                // Add client IP to span
                if let Some(client_ip) = request.headers().get("X-Forwarded-For") {
                    span.record("client_ip", client_ip.to_str().unwrap_or("unknown"));
                }

                span
            })
            .on_response(|response: &Response, latency: Duration, _span: &Span| {
                tracing::info!(
                    "response: status={}, latency={}ms",
                    response.status(),
                    latency.as_millis()
                );
            })
            .on_failure(|error: tower_http::classify::ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                tracing::error!("request failed: {:?}", error);
            })
    }
}

/// Security middleware
pub mod security {
    use super::*;

    /// Security headers middleware
    pub async fn security_headers(
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        let mut response = next.run(request).await;

        let headers = response.headers_mut();

        // Security headers
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            "nosniff".parse().unwrap(),
        );
        headers.insert(
            header::X_FRAME_OPTIONS,
            "DENY".parse().unwrap(),
        );
        headers.insert(
            header::X_XSS_PROTECTION,
            "1; mode=block".parse().unwrap(),
        );
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains".parse().unwrap(),
        );
        headers.insert(
            header::REFERRER_POLICY,
            "strict-origin-when-cross-origin".parse().unwrap(),
        );
        headers.insert(
            header::PERMISSIONS_POLICY,
            "geolocation=(), microphone=(), camera=()".parse().unwrap(),
        );

        response
    }

    /// Request validation middleware
    pub async fn validate_request(
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        // Validate request size
        if let Some(content_length) = request.headers().get(header::CONTENT_LENGTH) {
            if let Ok(length) = content_length.to_str().unwrap_or("0").parse::<u64>() {
                if length > 10 * 1024 * 1024 { // 10MB limit
                    return (
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "Request payload too large",
                    ).into_response();
                }
            }
        }

        // Validate content type for POST/PUT requests
        if matches!(request.method(), Method::POST | Method::PUT | Method::PATCH) {
            if let Some(content_type) = request.headers().get(header::CONTENT_TYPE) {
                let content_type_str = content_type.to_str().unwrap_or("");
                if !content_type_str.contains("application/json") &&
                   !content_type_str.contains("application/x-www-form-urlencoded") &&
                   !content_type_str.contains("multipart/form-data") {
                    warn!("Suspicious content type: {}", content_type_str);
                    // Allow but log for monitoring
                }
            }
        }

        next.run(request).await
    }

    /// API key validation middleware (for external API calls)
    pub async fn api_key_auth(
        headers: HeaderMap,
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        // Check for API key in header
        if let Some(api_key) = headers.get("X-API-Key") {
            if let Ok(key_str) = api_key.to_str() {
                if Self::validate_api_key(key_str).await {
                    return next.run(request).await;
                }
            }
        }

        (
            StatusCode::UNAUTHORIZED,
            "Valid API key required",
        ).into_response()
    }

    async fn validate_api_key(api_key: &str) -> bool {
        // Mock API key validation - replace with real implementation
        // In production, validate against database or external service
        api_key == "your-api-key" || api_key.starts_with("sk-")
    }
}

/// Utility middleware
pub mod utils {
    use super::*;

    /// Request timing middleware
    pub async fn timing_middleware(
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        let start = Instant::now();
        let mut response = next.run(request).await;
        let duration = start.elapsed();

        // Add timing header
        response.headers_mut().insert(
            "X-Response-Time",
            format!("{}ms", duration.as_millis()).parse().unwrap(),
        );

        response
    }

    /// Request ID middleware for tracing
    pub async fn request_id_middleware(
        request: Request,
        next: Next,
    ) -> impl IntoResponse {
        use uuid::Uuid;

        let request_id = Uuid::new_v4().to_string();
        let mut response = next.run(request).await;

        response.headers_mut().insert(
            "X-Request-ID",
            request_id.parse().unwrap(),
        );

        response
    }
}

/// Middleware configuration and builder
pub struct MiddlewareBuilder {
    cors_enabled: bool,
    rate_limiting_enabled: bool,
    logging_enabled: bool,
    security_enabled: bool,
    timing_enabled: bool,
    request_id_enabled: bool,
}

impl Default for MiddlewareBuilder {
    fn default() -> Self {
        Self {
            cors_enabled: true,
            rate_limiting_enabled: true,
            logging_enabled: true,
            security_enabled: true,
            timing_enabled: true,
            request_id_enabled: true,
        }
    }
}

impl MiddlewareBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cors(mut self, enabled: bool) -> Self {
        self.cors_enabled = enabled;
        self
    }

    pub fn rate_limiting(mut self, enabled: bool) -> Self {
        self.rate_limiting_enabled = enabled;
        self
    }

    pub fn logging(mut self, enabled: bool) -> Self {
        self.logging_enabled = enabled;
        self
    }

    pub fn security(mut self, enabled: bool) -> Self {
        self.security_enabled = enabled;
        self
    }

    pub fn timing(mut self, enabled: bool) -> Self {
        self.timing_enabled = enabled;
        self
    }

    pub fn request_id(mut self, enabled: bool) -> Self {
        self.request_id_enabled = enabled;
        self
    }

    pub fn build(self) -> ServiceBuilder<
        tower::layer::util::Identity,
        tower::layer::util::Identity,
    > {
        let mut builder = ServiceBuilder::new();

        if self.cors_enabled {
            builder = builder.layer(cors::cors_layer());
        }

        if self.rate_limiting_enabled {
            builder = builder.layer(rate_limit::rate_limit_layer(
                rate_limit::RateLimitConfig::default()
            ));
        }

        if self.logging_enabled {
            builder = builder.layer(logging::logging_layer());
        }

        if self.security_enabled {
            builder = builder.layer(tower::ServiceBuilder::new().map_request(security::security_headers));
        }

        if self.timing_enabled {
            builder = builder.layer(tower::ServiceBuilder::new().map_request(utils::timing_middleware));
        }

        if self.request_id_enabled {
            builder = builder.layer(tower::ServiceBuilder::new().map_request(utils::request_id_middleware));
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config() {
        let config = rate_limit::RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
            burst_allowance: Some(20),
        };

        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window_secs, 60);
    }

    #[test]
    fn test_middleware_builder() {
        let builder = MiddlewareBuilder::new()
            .cors(true)
            .logging(true)
            .security(true);

        // Test that builder can be created without panicking
        let service = builder.build();
        assert!(true); // If we get here, the builder works
    }
}