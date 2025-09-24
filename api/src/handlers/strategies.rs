//! Trading strategy endpoint handlers
//!
//! This module provides HTTP handlers for trading strategy operations including
//! strategy management, execution, backtesting, and performance analysis.

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
        CreateStrategyRequest, StrategyResponse, StrategyExecutionRequest,
        StrategyExecutionResponse, BacktestRequest, BacktestResponse,
        StrategyPerformance, StrategyOptimizationRequest, StrategyOptimizationResponse,
    },
    AppState,
};

/// Get all available trading strategies
pub async fn list_strategies(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<StrategyResponse>>> {
    info!("Retrieving strategies with pagination: {:?}", params);

    // Validate pagination parameters
    let mut params = params;
    if let Err(e) = params.validate() {
        return Err(ApiError::Validation { message: e, field: Some("pagination".to_string()) });
    }

    match state.strategy_manager.list_strategies(params).await {
        Ok(strategies) => {
            let response = PaginatedResponse {
                data: strategies.data.into_iter()
                    .map(|strategy| StrategyResponse {
                        id: strategy.id,
                        name: strategy.name,
                        description: strategy.description,
                        parameters: strategy.parameters,
                        is_active: strategy.is_active,
                        account_ids: strategy.account_ids,
                        created_at: strategy.created_at,
                        updated_at: strategy.updated_at,
                        performance: StrategyPerformance {
                            total_trades: strategy.performance.total_trades,
                            win_rate: strategy.performance.win_rate,
                            total_pnl: strategy.performance.total_pnl,
                            avg_trade_duration: strategy.performance.avg_trade_duration,
                            max_drawdown: strategy.performance.max_drawdown,
                        },
                    })
                    .collect(),
                total: strategies.total,
                page: strategies.page,
                limit: strategies.limit,
                total_pages: strategies.total_pages,
            };

            Ok(Json(response))
        }
        Err(e) => {
            warn!("Failed to list strategies: {}", e);
            Err(ApiError::Strategy { message: format!("Failed to list strategies: {}", e) })
        }
    }
}

/// Get a specific trading strategy by ID
pub async fn get_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
) -> ApiResult<Json<ApiResponse<StrategyResponse>>> {
    info!("Retrieving strategy: {}", strategy_id);

    match state.strategy_manager.get_strategy(&strategy_id).await {
        Ok(Some(strategy)) => {
            let response = StrategyResponse {
                id: strategy.id,
                name: strategy.name,
                description: strategy.description,
                parameters: strategy.parameters,
                is_active: strategy.is_active,
                account_ids: strategy.account_ids,
                created_at: strategy.created_at,
                updated_at: strategy.updated_at,
                performance: StrategyPerformance {
                    total_trades: strategy.performance.total_trades,
                    win_rate: strategy.performance.win_rate,
                    total_pnl: strategy.performance.total_pnl,
                    avg_trade_duration: strategy.performance.avg_trade_duration,
                    max_drawdown: strategy.performance.max_drawdown,
                },
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Ok(None) => Err(ApiError::NotFound { resource: format!("Strategy with ID {}", strategy_id) }),
        Err(e) => {
            warn!("Failed to retrieve strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy { message: format!("Failed to retrieve strategy: {}", e) })
        }
    }
}

/// Create a new trading strategy
pub async fn create_strategy(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateStrategyRequest>,
) -> ApiResult<Json<ApiResponse<StrategyResponse>>> {
    info!("Creating new strategy: {}", request.name);

    // Validate the request
    if let Err(e) = request.validate() {
        return Err(ApiError::Validation { message: e, field: Some("strategy".to_string()) });
    }

    match state.strategy_manager.create_strategy(request).await {
        Ok(strategy) => {
            let response = StrategyResponse {
                id: strategy.id,
                name: strategy.name,
                description: strategy.description,
                parameters: strategy.parameters,
                is_active: strategy.is_active,
                account_ids: strategy.account_ids,
                created_at: strategy.created_at,
                updated_at: strategy.updated_at,
                performance: StrategyPerformance {
                    total_trades: strategy.performance.total_trades,
                    win_rate: strategy.performance.win_rate,
                    total_pnl: strategy.performance.total_pnl,
                    avg_trade_duration: strategy.performance.avg_trade_duration,
                    max_drawdown: strategy.performance.max_drawdown,
                },
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to create strategy: {}", e);
            Err(ApiError::Strategy { message: format!("Failed to create strategy: {}", e) })
        }
    }
}

/// Update an existing trading strategy
pub async fn update_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Json(request): Json<UpdateStrategyRequest>,
) -> ApiResult<Json<ApiResponse<StrategyResponse>>> {
    info!("Updating strategy: {}", strategy_id);

    // Validate the request
    if let Err(e) = request.validate() {
        return Err(ApiError::Validation { message: e, field: Some("strategy".to_string()) });
    }

    match state.strategy_manager.update_strategy(&strategy_id, request).await {
        Ok(strategy) => {
            let response = StrategyResponse {
                id: strategy.id,
                name: strategy.name,
                description: strategy.description,
                parameters: strategy.parameters,
                is_active: strategy.is_active,
                account_ids: strategy.account_ids,
                created_at: strategy.created_at,
                updated_at: strategy.updated_at,
                performance: StrategyPerformance {
                    total_trades: strategy.performance.total_trades,
                    win_rate: strategy.performance.win_rate,
                    total_pnl: strategy.performance.total_pnl,
                    avg_trade_duration: strategy.performance.avg_trade_duration,
                    max_drawdown: strategy.performance.max_drawdown,
                },
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to update strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy { message: format!("Failed to update strategy: {}", e) })
        }
    }
}

/// Delete a trading strategy
pub async fn delete_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
) -> ApiResult<Json<ApiResponse<serde_json::Value>>> {
    info!("Deleting strategy: {}", strategy_id);

    match state.strategy_manager.delete_strategy(&strategy_id).await {
        Ok(_) => {
            let response = json!({
                "message": "Strategy deleted successfully",
                "strategy_id": strategy_id
            });

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to delete strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy { message: format!("Failed to delete strategy: {}", e) })
        }
    }
}

/// Execute a trading strategy
pub async fn execute_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Json(request): Json<StrategyExecutionRequest>,
) -> ApiResult<Json<ApiResponse<StrategyExecutionResponse>>> {
    info!("Executing strategy: {}", strategy_id);

    // Validate the request
    if let Err(e) = request.validate() {
        return Err(ApiError::Validation { message: e, field: Some("execution".to_string()) });
    }

    match state.strategy_manager.execute_strategy(&strategy_id, request).await {
        Ok(execution_result) => {
            let response = StrategyExecutionResponse {
                execution_id: execution_result.execution_id,
                strategy_id: execution_result.strategy_id,
                status: execution_result.status,
                orders_created: execution_result.orders_created,
                total_value: execution_result.total_value,
                estimated_pnl: execution_result.estimated_pnl,
                executed_at: execution_result.executed_at,
                message: execution_result.message,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to execute strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy { message: format!("Failed to execute strategy: {}", e) })
        }
    }
}

/// Get strategy execution history
pub async fn get_strategy_executions(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<StrategyExecutionResponse>>> {
    info!("Retrieving execution history for strategy: {} with params: {:?}", strategy_id, params);

    match state.strategy_manager.get_execution_history(&strategy_id, params).await {
        Ok(executions) => {
            let response = PaginatedResponse {
                data: executions.data,
                total: executions.total,
                page: executions.page,
                limit: executions.limit,
                total_pages: executions.total_pages,
            };

            Ok(Json(response))
        }
        Err(e) => {
            warn!("Failed to retrieve execution history for {}: {}", strategy_id, e);
            Err(ApiError::Strategy { message: format!("Failed to retrieve execution history: {}", e) })
        }
    }
}

/// Backtest a trading strategy
pub async fn backtest_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Json(request): Json<BacktestRequest>,
) -> ApiResult<Json<ApiResponse<BacktestResponse>>> {
    info!("Backtesting strategy: {}", strategy_id);

    // Validate the request
    if let Err(e) = request.validate() {
        return Err(ApiError::Validation { message: e, field: Some("backtest".to_string()) });
    }

    match state.strategy_manager.backtest_strategy(&strategy_id, request).await {
        Ok(backtest_result) => {
            let response = BacktestResponse {
                backtest_id: backtest_result.backtest_id,
                strategy_id: backtest_result.strategy_id,
                start_date: backtest_result.start_date,
                end_date: backtest_result.end_date,
                initial_balance: backtest_result.initial_balance,
                final_balance: backtest_result.final_balance,
                total_return: backtest_result.total_return,
                total_trades: backtest_result.total_trades,
                winning_trades: backtest_result.winning_trades,
                losing_trades: backtest_result.losing_trades,
                win_rate: backtest_result.win_rate,
                max_drawdown: backtest_result.max_drawdown,
                sharpe_ratio: backtest_result.sharpe_ratio,
                trades: backtest_result.trades,
                completed_at: backtest_result.completed_at,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to backtest strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy { message: format!("Failed to backtest strategy: {}", e) })
        }
    }
}

/// Optimize strategy parameters
pub async fn optimize_strategy(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
    Json(request): Json<StrategyOptimizationRequest>,
) -> ApiResult<Json<ApiResponse<StrategyOptimizationResponse>>> {
    info!("Optimizing strategy: {}", strategy_id);

    // Validate the request
    if let Err(e) = request.validate() {
        return Err(ApiError::Validation { message: e, field: Some("optimization".to_string()) });
    }

    match state.strategy_manager.optimize_strategy(&strategy_id, request).await {
        Ok(optimization_result) => {
            let response = StrategyOptimizationResponse {
                optimization_id: optimization_result.optimization_id,
                strategy_id: optimization_result.strategy_id,
                original_parameters: optimization_result.original_parameters,
                optimized_parameters: optimization_result.optimized_parameters,
                optimization_metric: optimization_result.optimization_metric,
                improvement_percentage: optimization_result.improvement_percentage,
                backtest_results: optimization_result.backtest_results,
                completed_at: optimization_result.completed_at,
                message: optimization_result.message,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to optimize strategy {}: {}", strategy_id, e);
            Err(ApiError::Strategy { message: format!("Failed to optimize strategy: {}", e) })
        }
    }
}

/// Get strategy performance metrics
pub async fn get_strategy_performance(
    State(state): State<Arc<AppState>>,
    Path(strategy_id): Path<String>,
) -> ApiResult<Json<ApiResponse<DetailedStrategyPerformance>>> {
    info!("Retrieving detailed performance for strategy: {}", strategy_id);

    match state.strategy_manager.get_detailed_performance(&strategy_id).await {
        Ok(performance) => {
            let response = DetailedStrategyPerformance {
                strategy_id: performance.strategy_id,
                total_trades: performance.total_trades,
                winning_trades: performance.winning_trades,
                losing_trades: performance.losing_trades,
                win_rate: performance.win_rate,
                total_pnl: performance.total_pnl,
                average_win: performance.average_win,
                average_loss: performance.average_loss,
                profit_factor: performance.profit_factor,
                max_drawdown: performance.max_drawdown,
                sharpe_ratio: performance.sharpe_ratio,
                sortino_ratio: performance.sortino_ratio,
                calmar_ratio: performance.calmar_ratio,
                max_consecutive_wins: performance.max_consecutive_wins,
                max_consecutive_losses: performance.max_consecutive_losses,
                average_trade_duration: performance.average_trade_duration,
                monthly_returns: performance.monthly_returns,
                risk_metrics: performance.risk_metrics,
                last_updated: performance.last_updated,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve performance for {}: {}", strategy_id, e);
            Err(ApiError::Strategy { message: format!("Failed to retrieve strategy performance: {}", e) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_list_strategies_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let params = PaginationParams::default();
        let result = list_strategies(State(state), Query(params)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_strategy_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let result = get_strategy(State(state), Path("test-strategy".to_string())).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_strategy_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let request = CreateStrategyRequest {
            name: "Test Strategy".to_string(),
            description: Some("Test strategy description".to_string()),
            parameters: std::collections::HashMap::new(),
            is_active: Some(true),
            account_ids: Some(vec!["test-account".to_string()]),
        };
        let result = create_strategy(State(state), Json(request)).await;

        assert!(result.is_ok());
    }
}