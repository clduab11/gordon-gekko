//! API endpoint handlers
//!
//! This module provides all the HTTP handlers for the trading API endpoints.
//! Each handler is responsible for processing requests, validating input,
//! calling the appropriate business logic, and returning structured responses.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    body::Body,
};
use std::sync::Arc;
use serde_json::json;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        ApiResponse, PaginationParams, PaginatedResponse, CreateTradeRequest,
        UpdateTradeRequest, TradeResponse, CreateStrategyRequest, StrategyResponse,
        StrategyExecutionRequest, StrategyExecutionResponse, BacktestRequest,
        BacktestResponse, StrategyOptimizationRequest, StrategyOptimizationResponse,
    },
};

/// Handler for the health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "ninja-gekko-api"
    }))
}

/// Handler for the root API information endpoint
pub async fn api_info() -> Json<serde_json::Value> {
    Json(json!({
        "name": "Ninja Gekko Trading API",
        "version": "1.0.0",
        "description": "High-performance REST API for autonomous trading",
        "endpoints": {
            "health": "GET /health",
            "trades": {
                "list": "GET /trades",
                "create": "POST /trades",
                "get": "GET /trades/{id}",
                "update": "PUT /trades/{id}",
                "delete": "DELETE /trades/{id}"
            },
            "portfolio": "GET /portfolio",
            "market_data": "GET /market-data/{symbol}",
            "strategies": {
                "list": "GET /strategies",
                "create": "POST /strategies",
                "get": "GET /strategies/{id}",
                "update": "PUT /strategies/{id}"
            },
            "websocket": "WS /ws"
        },
        "features": [
            "Real-time WebSocket updates",
            "JWT authentication",
            "Rate limiting",
            "CORS support",
            "Comprehensive error handling"
        ]
    }))
}

/// Trade-related endpoint handlers
pub mod trades;

/// Strategy-related endpoint handlers
pub mod strategies;

/// Authentication-related endpoint handlers
pub mod auth_utils;