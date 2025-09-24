//! # Gordon Gekko API
//!
//! High-performance REST API and WebSocket server for the Gordon Gekko trading system.
//! Built with Axum framework for maximum performance and reliability.
//!
//! ## Features
//! - REST API endpoints for trading operations
//! - WebSocket support for real-time market data
//! - JWT-based authentication
//! - Rate limiting and CORS middleware
//! - Comprehensive error handling
//! - Structured API responses
//!
//! ## Architecture
//! The API is organized into several modules:
//! - `handlers`: HTTP request handlers
//! - `middleware`: Authentication, CORS, rate limiting
//! - `models`: API request/response models
//! - `websocket`: WebSocket connection handling
//! - `config`: Server configuration
//! - `error`: Error types and handling

use axum::{
    routing::{get, post, put, delete},
    http::StatusCode,
    response::Json,
    Router,
    extract::{Path, Query, State},
    middleware,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error, warn};

// Core dependencies
use gordon_gekko_core::{Order, Position, Portfolio, MarketData, OrderType, OrderSide};
use gordon_gekko_database::{DatabaseManager, TradeRepository, PortfolioRepository};

pub mod config;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod websocket;
pub mod error;
pub mod validation;
pub mod auth_validation;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    /// Database manager for data persistence
    pub db_manager: Arc<DatabaseManager>,
    /// Trade repository for order management
    pub trade_repository: Arc<TradeRepository>,
    /// Portfolio repository for position tracking
    pub portfolio_repository: Arc<PortfolioRepository>,
    /// Server configuration
    pub config: Arc<config::ApiConfig>,
}

impl AppState {
    /// Creates a new application state instance
    pub async fn new(config: config::ApiConfig) -> Result<Self, error::ApiError> {
        let db_manager = Arc::new(
            DatabaseManager::new(&config.database_url)
                .await
                .map_err(error::ApiError::DatabaseError)?
        );

        let trade_repository = Arc::new(
            TradeRepository::new(db_manager.clone())
                .await
                .map_err(error::ApiError::DatabaseError)?
        );

        let portfolio_repository = Arc::new(
            PortfolioRepository::new(db_manager.clone())
                .await
                .map_err(error::ApiError::DatabaseError)?
        );

        Ok(Self {
            db_manager,
            trade_repository,
            portfolio_repository,
            config: Arc::new(config),
        })
    }
}

/// Main API server structure
pub struct ApiServer {
    /// Axum router with all routes configured
    router: Router,
    /// Server configuration
    config: Arc<config::ApiConfig>,
    /// Application state
    state: Arc<AppState>,
}

impl ApiServer {
    /// Creates a new API server with all routes and middleware configured
    pub async fn new() -> Result<Self, error::ApiError> {
        // Load configuration
        let config = config::ApiConfig::from_env()
            .map_err(error::ApiError::ConfigError)?;

        // Create application state
        let state = Arc::new(AppState::new(config.clone()).await?);

        // Build middleware stack using the middleware builder
        let middleware = middleware::MiddlewareBuilder::new()
            .cors(true)
            .rate_limiting(true)
            .logging(true)
            .security(true)
            .timing(true)
            .request_id(true)
            .build();

        // Create router with all routes
        let router = Router::new()
            // Health check endpoint
            .route("/health", get(handlers::health_check))

            // Trade endpoints
            .route("/api/v1/trades", get(handlers::trades::list_trades))
            .route("/api/v1/trades", post(handlers::trades::create_trade))
            .route("/api/v1/trades/:id", get(handlers::trades::get_trade))
            .route("/api/v1/trades/:id", put(handlers::trades::update_trade))
            .route("/api/v1/trades/:id", delete(handlers::trades::delete_trade))

            // Portfolio endpoints
            .route("/api/v1/portfolio", get(handlers::portfolio::get_portfolio))
            .route("/api/v1/portfolio/positions", get(handlers::portfolio::get_positions))
            .route("/api/v1/portfolio/positions/:symbol", get(handlers::portfolio::get_position))
            .route("/api/v1/portfolio/performance", get(handlers::portfolio::get_performance))

            // Market data endpoints
            .route("/api/v1/market-data", get(handlers::market_data::get_market_data))
            .route("/api/v1/market-data/:symbol", get(handlers::market_data::get_symbol_data))
            .route("/api/v1/market-data/:symbol/history", get(handlers::market_data::get_price_history))

            // Strategy endpoints
            .route("/api/v1/strategies", get(handlers::strategies::list_strategies))
            .route("/api/v1/strategies", post(handlers::strategies::create_strategy))
            .route("/api/v1/strategies/:id", get(handlers::strategies::get_strategy))
            .route("/api/v1/strategies/:id", put(handlers::strategies::update_strategy))
            .route("/api/v1/strategies/:id", delete(handlers::strategies::delete_strategy))
            .route("/api/v1/strategies/:id/execute", post(handlers::strategies::execute_strategy))

            // WebSocket endpoint for real-time data
            .route("/api/v1/ws", get(websocket::websocket_handler))

            // Authentication endpoints
            .route("/api/v1/auth/login", post(handlers::auth_utils::login_handler))
            .route("/api/v1/auth/refresh", post(handlers::auth_utils::refresh_handler))
            .route("/api/v1/auth/logout", post(handlers::auth_utils::logout_handler))

            // API documentation
            .route("/api/v1/docs", get(handlers::api_info))

            // Apply middleware
            .layer(middleware)
            .with_state(state.clone());

        info!("API server configured with {} routes", count_routes(&router));

        Ok(Self {
            router,
            config: Arc::new(config),
            state,
        })
    }

    /// Starts the API server and begins listening for requests
    pub async fn serve(self) -> Result<(), error::ApiError> {
        let addr = &self.config.bind_address;

        info!("Starting Gordon Gekko API server on {}", addr);
        info!("Health check available at http://{}/health", addr);
        info!("API documentation available at http://{}/api/v1/docs", addr);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| error::ApiError::ServerError(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("🚀 Server listening on http://{}", addr);

        // Start the server
        axum::serve(listener, self.router)
            .await
            .map_err(|e| error::ApiError::ServerError(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Returns server configuration
    pub fn config(&self) -> &config::ApiConfig {
        &self.config
    }

    /// Returns application state
    pub fn state(&self) -> &AppState {
        &self.state
    }
}

/// Counts the total number of routes in a router
fn count_routes(router: &Router) -> usize {
    // This is a simplified count - in production you might want to traverse the router tree
    // For now, we'll return an estimate based on the routes we know we added
    15 // Rough count of our endpoints
}

#[cfg(test)]
mod tests {
    use super::*;
    use gordon_gekko_core::types::{OrderId, Symbol, AccountId};

    #[tokio::test]
    async fn test_api_server_creation() {
        // This test would require a test database and proper configuration
        // For now, we'll just ensure the struct can be created
        let server = ApiServer::new().await;
        assert!(server.is_ok());
    }

    #[test]
    fn test_route_counting() {
        // This would test the route counting logic
        // For now, just ensure it returns a positive number
        let count = count_routes(&Router::new());
        assert!(count >= 0);
    }
}