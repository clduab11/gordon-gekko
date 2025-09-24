//! Portfolio management endpoint handlers
//!
//! This module provides HTTP handlers for portfolio-related operations including
//! retrieving portfolio information, positions, and performance metrics.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use serde_json::json;
use tracing::{info, warn};

use crate::{
    error::{ApiError, ApiResult},
    models::{
        ApiResponse, PaginationParams, PaginatedResponse,
        PortfolioResponse, PositionResponse, PerformanceMetricsResponse,
        PortfolioSummaryRequest, RebalanceRequest, AllocationRequest,
    },
    AppState,
};

/// Get complete portfolio information
pub async fn get_portfolio(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<PortfolioResponse>>> {
    info!("Retrieving complete portfolio information");

    match state.portfolio_manager.get_portfolio().await {
        Ok(portfolio) => {
            let response = PortfolioResponse {
                portfolio_id: portfolio.id.clone(),
                total_value: portfolio.total_value,
                total_unrealized_pnl: portfolio.total_unrealized_pnl,
                total_realized_pnl: portfolio.total_realized_pnl,
                positions: portfolio.positions.into_iter()
                    .map(|pos| PositionResponse {
                        symbol: pos.symbol,
                        quantity: pos.quantity,
                        average_cost: pos.average_cost,
                        current_price: pos.current_price,
                        market_value: pos.market_value,
                        unrealized_pnl: pos.unrealized_pnl,
                        realized_pnl: pos.realized_pnl,
                        allocation_percentage: pos.allocation_percentage,
                    })
                    .collect(),
                performance: PerformanceMetricsResponse {
                    daily_return: portfolio.performance.daily_return,
                    weekly_return: portfolio.performance.weekly_return,
                    monthly_return: portfolio.performance.monthly_return,
                    yearly_return: portfolio.performance.yearly_return,
                    sharpe_ratio: portfolio.performance.sharpe_ratio,
                    max_drawdown: portfolio.performance.max_drawdown,
                    volatility: portfolio.performance.volatility,
                },
                last_updated: portfolio.last_updated,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve portfolio: {}", e);
            Err(ApiError::Portfolio { message: format!("Failed to retrieve portfolio: {}", e) })
        }
    }
}

/// Get portfolio summary with optional filtering
pub async fn get_portfolio_summary(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PortfolioSummaryRequest>,
) -> ApiResult<Json<ApiResponse<PortfolioResponse>>> {
    info!("Retrieving portfolio summary with params: {:?}", params);

    match state.portfolio_manager.get_portfolio_summary(params).await {
        Ok(summary) => {
            let response = PortfolioResponse {
                portfolio_id: summary.id,
                total_value: summary.total_value,
                total_unrealized_pnl: summary.total_unrealized_pnl,
                total_realized_pnl: summary.total_realized_pnl,
                positions: summary.positions,
                performance: summary.performance,
                last_updated: summary.last_updated,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve portfolio summary: {}", e);
            Err(ApiError::Portfolio { message: format!("Failed to retrieve portfolio summary: {}", e) })
        }
    }
}

/// Get positions with pagination and filtering
pub async fn get_positions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<PositionResponse>>> {
    info!("Retrieving positions with pagination: {:?}", params);

    match state.portfolio_manager.get_positions(params).await {
        Ok(positions) => {
            let response = PaginatedResponse {
                data: positions.data.into_iter()
                    .map(|pos| PositionResponse {
                        symbol: pos.symbol,
                        quantity: pos.quantity,
                        average_cost: pos.average_cost,
                        current_price: pos.current_price,
                        market_value: pos.market_value,
                        unrealized_pnl: pos.unrealized_pnl,
                        realized_pnl: pos.realized_pnl,
                        allocation_percentage: pos.allocation_percentage,
                    })
                    .collect(),
                total: positions.total,
                page: positions.page,
                limit: positions.limit,
                total_pages: positions.total_pages,
            };

            Ok(Json(response))
        }
        Err(e) => {
            warn!("Failed to retrieve positions: {}", e);
            Err(ApiError::Portfolio { message: format!("Failed to retrieve positions: {}", e) })
        }
    }
}

/// Get specific position by symbol
pub async fn get_position(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<ApiResponse<PositionResponse>>> {
    info!("Retrieving position for symbol: {}", symbol);

    match state.portfolio_manager.get_position(&symbol).await {
        Ok(Some(position)) => {
            let response = PositionResponse {
                symbol: position.symbol,
                quantity: position.quantity,
                average_cost: position.average_cost,
                current_price: position.current_price,
                market_value: position.market_value,
                unrealized_pnl: position.unrealized_pnl,
                realized_pnl: position.realized_pnl,
                allocation_percentage: position.allocation_percentage,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Ok(None) => Err(ApiError::NotFound { resource: format!("Position for symbol {}", symbol) }),
        Err(e) => {
            warn!("Failed to retrieve position for {}: {}", symbol, e);
            Err(ApiError::Portfolio { message: format!("Failed to retrieve position: {}", e) })
        }
    }
}

/// Get portfolio performance metrics
pub async fn get_performance_metrics(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<PerformanceMetricsResponse>>> {
    info!("Retrieving portfolio performance metrics");

    match state.portfolio_manager.get_performance_metrics().await {
        Ok(metrics) => {
            let response = PerformanceMetricsResponse {
                daily_return: metrics.daily_return,
                weekly_return: metrics.weekly_return,
                monthly_return: metrics.monthly_return,
                yearly_return: metrics.yearly_return,
                sharpe_ratio: metrics.sharpe_ratio,
                max_drawdown: metrics.max_drawdown,
                volatility: metrics.volatility,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve performance metrics: {}", e);
            Err(ApiError::Portfolio { message: format!("Failed to retrieve performance metrics: {}", e) })
        }
    }
}

/// Get portfolio allocation breakdown
pub async fn get_allocation_breakdown(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<Vec<AllocationResponse>>>> {
    info!("Retrieving portfolio allocation breakdown");

    match state.portfolio_manager.get_allocation_breakdown().await {
        Ok(allocations) => {
            let response = allocations.into_iter()
                .map(|alloc| AllocationResponse {
                    symbol: alloc.symbol,
                    allocation_percentage: alloc.allocation_percentage,
                    market_value: alloc.market_value,
                    weight: alloc.weight,
                })
                .collect();

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve allocation breakdown: {}", e);
            Err(ApiError::Portfolio { message: format!("Failed to retrieve allocation breakdown: {}", e) })
        }
    }
}

/// Rebalance portfolio based on target allocations
pub async fn rebalance_portfolio(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RebalanceRequest>,
) -> ApiResult<Json<ApiResponse<RebalanceResponse>>> {
    info!("Rebalancing portfolio with request: {:?}", request);

    match state.portfolio_manager.rebalance_portfolio(request).await {
        Ok(rebalance_result) => {
            let response = RebalanceResponse {
                success: rebalance_result.success,
                orders_created: rebalance_result.orders_created,
                total_orders: rebalance_result.total_orders,
                estimated_cost: rebalance_result.estimated_cost,
                message: rebalance_result.message,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to rebalance portfolio: {}", e);
            Err(ApiError::Portfolio { message: format!("Failed to rebalance portfolio: {}", e) })
        }
    }
}

/// Get portfolio historical data
pub async fn get_portfolio_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<PortfolioHistoryResponse>>> {
    info!("Retrieving portfolio history with pagination: {:?}", params);

    match state.portfolio_manager.get_portfolio_history(params).await {
        Ok(history) => {
            let response = PaginatedResponse {
                data: history.data,
                total: history.total,
                page: history.page,
                limit: history.limit,
                total_pages: history.total_pages,
            };

            Ok(Json(response))
        }
        Err(e) => {
            warn!("Failed to retrieve portfolio history: {}", e);
            Err(ApiError::Portfolio { message: format!("Failed to retrieve portfolio history: {}", e) })
        }
    }
}

/// Get portfolio risk metrics
pub async fn get_risk_metrics(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<RiskMetricsResponse>>> {
    info!("Retrieving portfolio risk metrics");

    match state.portfolio_manager.get_risk_metrics().await {
        Ok(metrics) => {
            let response = RiskMetricsResponse {
                var_95: metrics.var_95,
                var_99: metrics.var_99,
                cvar_95: metrics.cvar_95,
                beta: metrics.beta,
                alpha: metrics.alpha,
                treynor_ratio: metrics.treynor_ratio,
                sortino_ratio: metrics.sortino_ratio,
                information_ratio: metrics.information_ratio,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve risk metrics: {}", e);
            Err(ApiError::Portfolio { message: format!("Failed to retrieve risk metrics: {}", e) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_portfolio_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let result = get_portfolio(State(state)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_portfolio_summary_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let params = PortfolioSummaryRequest::default();
        let result = get_portfolio_summary(State(state), Query(params)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_positions_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let params = PaginationParams::default();
        let result = get_positions(State(state), Query(params)).await;

        assert!(result.is_ok());
    }
}