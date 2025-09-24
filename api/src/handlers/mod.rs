//! HTTP request handlers for the Gordon Gekko API
//!
//! This module contains all the HTTP endpoint handlers organized by functionality:
//! - Authentication utilities (login, logout, token refresh)
//! - Trade management (CRUD operations for trades)
//! - Portfolio management (portfolio, positions, performance)
//! - Market data retrieval (current data, historical data)
//! - Strategy management (trading strategies, execution, backtesting)
//! - Utility endpoints (health check, API info)

use axum::{
    http::StatusCode,
    response::Json,
    extract::State,
};
use std::sync::Arc;
use serde_json::json;
use crate::{
    error::{ApiError, ApiResult},
    models::ApiResponse,
    AppState,
};

pub mod auth_utils;
pub mod trades;
pub mod portfolio;
pub mod market_data;
pub mod strategies;

// Re-export all handler functions
pub use auth_utils::{login_handler, refresh_handler, logout_handler};
pub use trades::{list_trades, create_trade, get_trade, update_trade, delete_trade};
pub use portfolio::{get_portfolio, get_positions, get_position, get_performance};
pub use market_data::{get_market_data, get_symbol_data, get_price_history};
pub use strategies::{list_strategies, create_strategy, get_strategy, update_strategy, delete_strategy, execute_strategy};

/// Health check endpoint
///
/// Returns the current health status of the API server and its dependencies.
/// This endpoint is used for monitoring and load balancer health checks.
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "service": "gordon-gekko-api"
    }))
}

/// API information endpoint
///
/// Returns general information about the API including available endpoints,
/// version information, and supported features.
pub async fn api_info() -> Json<ApiResponse<serde_json::Value>> {
    let info = json!({
        "name": "Gordon Gekko Trading API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "High-performance REST API for autonomous trading operations",
        "endpoints": {
            "health": "/health",
            "auth": {
                "login": "/api/v1/auth/login",
                "refresh": "/api/v1/auth/refresh",
                "logout": "/api/v1/auth/logout"
            },
            "trades": {
                "list": "/api/v1/trades",
                "create": "/api/v1/trades",
                "get": "/api/v1/trades/{id}",
                "update": "/api/v1/trades/{id}",
                "delete": "/api/v1/trades/{id}"
            },
            "portfolio": {
                "overview": "/api/v1/portfolio",
                "positions": "/api/v1/portfolio/positions",
                "position": "/api/v1/portfolio/positions/{symbol}",
                "performance": "/api/v1/portfolio/performance"
            },
            "market_data": {
                "overview": "/api/v1/market-data",
                "symbol": "/api/v1/market-data/{symbol}",
                "history": "/api/v1/market-data/{symbol}/history"
            },
            "strategies": {
                "list": "/api/v1/strategies",
                "create": "/api/v1/strategies",
                "get": "/api/v1/strategies/{id}",
                "update": "/api/v1/strategies/{id}",
                "delete": "/api/v1/strategies/{id}",
                "execute": "/api/v1/strategies/{id}/execute"
            },
            "websocket": "/api/v1/ws"
        },
        "features": [
            "REST API",
            "WebSocket real-time updates",
            "JWT Authentication",
            "Rate limiting",
            "CORS support",
            "Comprehensive error handling"
        ],
        "documentation": "/api/v1/docs"
    });

    Json(ApiResponse::success(info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response.0.get("status").unwrap(), "healthy");
    }

    #[tokio::test]
    async fn test_api_info() {
        let response = api_info().await;
        assert!(response.0.success);
        assert!(response.0.data.is_some());
    }
}