//! Market data endpoint handlers
//!
//! This module provides HTTP handlers for market data operations including
//! real-time price feeds, historical data, and market indicators.

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
        MarketDataResponse, MarketDataRequest, MarketDataPoint,
    },
    AppState,
};

/// Get current market data for a specific symbol
pub async fn get_market_data(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<ApiResponse<MarketDataResponse>>> {
    info!("Retrieving market data for symbol: {}", symbol);

    match state.market_data_service.get_latest_data(&symbol).await {
        Ok(data) => {
            let response = MarketDataResponse {
                symbol: data.symbol,
                price: data.price,
                change_24h: data.change_24h,
                volume_24h: data.volume_24h,
                market_cap: data.market_cap,
                timestamp: data.timestamp,
                history: None, // Current data only
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve market data for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve market data: {}", e) })
        }
    }
}

/// Get market data for multiple symbols
pub async fn get_batch_market_data(
    State(state): State<Arc<AppState>>,
    Query(request): Query<MarketDataRequest>,
) -> ApiResult<Json<ApiResponse<Vec<MarketDataResponse>>>> {
    info!("Retrieving batch market data for symbols: {:?}", request.symbols);

    if request.symbols.is_empty() {
        return Err(ApiError::Validation { message: "Symbols list cannot be empty".to_string(), field: Some("symbols".to_string()) });
    }

    match state.market_data_service.get_batch_data(&request.symbols).await {
        Ok(data_list) => {
            let response = data_list.into_iter()
                .map(|data| MarketDataResponse {
                    symbol: data.symbol,
                    price: data.price,
                    change_24h: data.change_24h,
                    volume_24h: data.volume_24h,
                    market_cap: data.market_cap,
                    timestamp: data.timestamp,
                    history: None,
                })
                .collect();

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve batch market data: {}", e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve batch market data: {}", e) })
        }
    }
}

/// Get historical market data for a symbol
pub async fn get_historical_data(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<PaginatedResponse<MarketDataPoint>>> {
    info!("Retrieving historical data for symbol: {} with params: {:?}", symbol, params);

    // Validate pagination parameters
    let mut params = params;
    if let Err(e) = params.validate() {
        return Err(ApiError::Validation { message: e, field: Some("pagination".to_string()) });
    }

    match state.market_data_service.get_historical_data(&symbol, params).await {
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
            warn!("Failed to retrieve historical data for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve historical data: {}", e) })
        }
    }
}

/// Get price history with technical indicators
pub async fn get_price_with_indicators(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<MarketDataWithIndicators>>> {
    info!("Retrieving price data with indicators for symbol: {}", symbol);

    match state.market_data_service.get_data_with_indicators(&symbol, params).await {
        Ok(data) => {
            let response = MarketDataWithIndicators {
                symbol: data.symbol,
                current_price: data.current_price,
                sma_20: data.sma_20,
                sma_50: data.sma_50,
                ema_12: data.ema_12,
                ema_26: data.ema_26,
                rsi_14: data.rsi_14,
                macd_line: data.macd_line,
                macd_signal: data.macd_signal,
                bollinger_upper: data.bollinger_upper,
                bollinger_middle: data.bollinger_middle,
                bollinger_lower: data.bollinger_lower,
                volume_sma: data.volume_sma,
                timestamp: data.timestamp,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve price with indicators for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve price with indicators: {}", e) })
        }
    }
}

/// Search for symbols based on query
pub async fn search_symbols(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchSymbolsRequest>,
) -> ApiResult<Json<ApiResponse<Vec<SymbolInfo>>>> {
    info!("Searching symbols with query: {}", params.query);

    if params.query.trim().is_empty() {
        return Err(ApiError::Validation { message: "Search query cannot be empty".to_string(), field: Some("query".to_string()) });
    }

    match state.market_data_service.search_symbols(&params.query, params.limit).await {
        Ok(symbols) => {
            let response = symbols.into_iter()
                .map(|symbol| SymbolInfo {
                    symbol: symbol.symbol,
                    name: symbol.name,
                    exchange: symbol.exchange,
                    asset_type: symbol.asset_type,
                    is_active: symbol.is_active,
                })
                .collect();

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to search symbols: {}", e);
            Err(ApiError::MarketData { message: format!("Failed to search symbols: {}", e) })
        }
    }
}

/// Get market overview with top gainers, losers, and volume leaders
pub async fn get_market_overview(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiResponse<MarketOverview>>> {
    info!("Retrieving market overview");

    match state.market_data_service.get_market_overview().await {
        Ok(overview) => {
            let response = MarketOverview {
                top_gainers: overview.top_gainers.into_iter()
                    .map(|gainer| MarketDataResponse {
                        symbol: gainer.symbol,
                        price: gainer.price,
                        change_24h: gainer.change_24h,
                        volume_24h: gainer.volume_24h,
                        market_cap: gainer.market_cap,
                        timestamp: gainer.timestamp,
                        history: None,
                    })
                    .collect(),
                top_losers: overview.top_losers.into_iter()
                    .map(|loser| MarketDataResponse {
                        symbol: loser.symbol,
                        price: loser.price,
                        change_24h: loser.change_24h,
                        volume_24h: loser.volume_24h,
                        market_cap: loser.market_cap,
                        timestamp: loser.timestamp,
                        history: None,
                    })
                    .collect(),
                volume_leaders: overview.volume_leaders.into_iter()
                    .map(|leader| MarketDataResponse {
                        symbol: leader.symbol,
                        price: leader.price,
                        change_24h: leader.change_24h,
                        volume_24h: leader.volume_24h,
                        market_cap: leader.market_cap,
                        timestamp: leader.timestamp,
                        history: None,
                    })
                    .collect(),
                market_indices: overview.market_indices,
                last_updated: overview.last_updated,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve market overview: {}", e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve market overview: {}", e) })
        }
    }
}

/// Get real-time price stream for a symbol (WebSocket upgrade)
pub async fn get_price_stream(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<ApiResponse<StreamSubscriptionResponse>>> {
    info!("Starting price stream for symbol: {}", symbol);

    match state.market_data_service.subscribe_to_price_stream(&symbol).await {
        Ok(subscription) => {
            let response = StreamSubscriptionResponse {
                subscription_id: subscription.subscription_id,
                symbol: subscription.symbol,
                stream_type: subscription.stream_type,
                is_active: subscription.is_active,
                message: subscription.message,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to start price stream for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to start price stream: {}", e) })
        }
    }
}

/// Get market statistics for a symbol
pub async fn get_market_statistics(
    State(state): State<Arc<AppState>>,
    Path(symbol): Path<String>,
) -> ApiResult<Json<ApiResponse<MarketStatistics>>> {
    info!("Retrieving market statistics for symbol: {}", symbol);

    match state.market_data_service.get_market_statistics(&symbol).await {
        Ok(stats) => {
            let response = MarketStatistics {
                symbol: stats.symbol,
                price_statistics: PriceStatistics {
                    open: stats.price_statistics.open,
                    high: stats.price_statistics.high,
                    low: stats.price_statistics.low,
                    close: stats.price_statistics.close,
                    volume: stats.price_statistics.volume,
                    vwap: stats.price_statistics.vwap,
                },
                volatility_metrics: VolatilityMetrics {
                    daily_volatility: stats.volatility_metrics.daily_volatility,
                    weekly_volatility: stats.volatility_metrics.weekly_volatility,
                    monthly_volatility: stats.volatility_metrics.monthly_volatility,
                    average_true_range: stats.volatility_metrics.average_true_range,
                },
                liquidity_metrics: LiquidityMetrics {
                    bid_ask_spread: stats.liquidity_metrics.bid_ask_spread,
                    market_depth: stats.liquidity_metrics.market_depth,
                    turnover_ratio: stats.liquidity_metrics.turnover_ratio,
                },
                trading_activity: TradingActivity {
                    total_trades: stats.trading_activity.total_trades,
                    average_trade_size: stats.trading_activity.average_trade_size,
                    trade_frequency: stats.trading_activity.trade_frequency,
                },
                timestamp: stats.timestamp,
            };

            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("Failed to retrieve market statistics for {}: {}", symbol, e);
            Err(ApiError::MarketData { message: format!("Failed to retrieve market statistics: {}", e) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_market_data_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let result = get_market_data(State(state), Path("AAPL".to_string())).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_batch_market_data_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let request = MarketDataRequest {
            symbols: vec!["AAPL".to_string(), "GOOGL".to_string()],
            include_history: Some(false),
            history_limit: None,
        };
        let result = get_batch_market_data(State(state), Query(request)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_symbols_success() {
        let state = Arc::new(AppState::new().await.unwrap());
        let params = SearchSymbolsRequest {
            query: "Apple".to_string(),
            limit: Some(10),
        };
        let result = search_symbols(State(state), Query(params)).await;

        assert!(result.is_ok());
    }
}