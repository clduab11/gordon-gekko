//! API response models and data structures
//!
//! This module defines all the request/response structures used by the API endpoints,
//! along with common pagination and error response types.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use ninja_gekko_core::types::{Order, OrderSide, OrderStatus, OrderType, Position, Portfolio};
use std::collections::HashMap;

/// Standardized API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,

    /// Response data (None if error occurred)
    pub data: Option<T>,

    /// Error message (None if successful)
    pub error: Option<String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Request ID for tracing
    pub request_id: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    /// Create a successful response with request ID
    pub fn success_with_request_id(data: T, request_id: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: Some(request_id),
        }
    }

    /// Create an error response
    pub fn error(error_message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error_message),
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    /// Create an error response with request ID
    pub fn error_with_request_id(error_message: String, request_id: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error_message),
            timestamp: Utc::now(),
            request_id: Some(request_id),
        }
    }
}

/// Pagination parameters for list endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-based, default: 1)
    pub page: Option<usize>,

    /// Items per page (default: 50, max: 1000)
    pub limit: Option<usize>,

    /// Sort field name
    pub sort_by: Option<String>,

    /// Sort direction
    pub sort_order: Option<String>,

    /// Filter parameters (key-value pairs)
    pub filters: Option<HashMap<String, String>>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(50),
            sort_by: None,
            sort_order: Some("desc".to_string()),
            filters: None,
        }
    }
}

impl PaginationParams {
    /// Get the offset for database queries
    pub fn offset(&self) -> usize {
        let page = self.page.unwrap_or(1);
        let limit = self.limit.unwrap_or(50);
        (page - 1) * limit
    }

    /// Validate and sanitize pagination parameters
    pub fn validate(&mut self) -> Result<(), String> {
        if let Some(limit) = self.limit {
            if limit == 0 {
                return Err("Limit must be greater than 0".to_string());
            }
            if limit > 1000 {
                self.limit = Some(1000);
            }
        }

        if let Some(page) = self.page {
            if page == 0 {
                return Err("Page must be greater than 0".to_string());
            }
        }

        // Validate sort order
        if let Some(ref mut sort_order) = self.sort_order {
            sort_order.make_ascii_lowercase();
            if !matches!(sort_order.as_str(), "asc" | "desc") {
                *sort_order = "desc".to_string();
            }
        }

        Ok(())
    }
}

/// Pagination metadata returned with list responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number
    pub page: usize,

    /// Items per page
    pub limit: usize,

    /// Total number of items
    pub total: usize,

    /// Total number of pages
    pub total_pages: usize,

    /// Whether there are more pages
    pub has_next: bool,

    /// Whether there are previous pages
    pub has_prev: bool,
}

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Response data
    #[serde(flatten)]
    pub response: ApiResponse<Vec<T>>,

    /// Pagination metadata
    pub pagination: PaginationMeta,
}

/// Trade creation request
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTradeRequest {
    /// Trading symbol (e.g., "AAPL", "BTCUSD")
    pub symbol: String,

    /// Order side (buy/sell)
    pub side: String,

    /// Order quantity
    pub quantity: f64,

    /// Order type (market/limit/stop)
    pub order_type: String,

    /// Price for limit/stop orders
    pub price: Option<f64>,

    /// Account ID for the trade
    pub account_id: Option<String>,

    /// Additional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl CreateTradeRequest {
    /// Validate the trade request
    pub fn validate(&self) -> Result<(), String> {
        if self.symbol.trim().is_empty() {
            return Err("Symbol cannot be empty".to_string());
        }

        if self.quantity <= 0.0 {
            return Err("Quantity must be positive".to_string());
        }

        // Validate order side
        match self.side.to_lowercase().as_str() {
            "buy" | "sell" => {},
            _ => return Err("Side must be 'buy' or 'sell'".to_string()),
        }

        // Validate order type
        match self.order_type.to_lowercase().as_str() {
            "market" | "limit" | "stop" | "stop_limit" => {},
            _ => return Err("Order type must be 'market', 'limit', 'stop', or 'stop_limit'".to_string()),
        }

        // Validate price for non-market orders
        if self.order_type.to_lowercase() != "market" && self.price.is_none() {
            return Err("Price is required for non-market orders".to_string());
        }

        Ok(())
    }

    /// Convert to core Order type
    pub fn to_order(&self, order_id: String) -> Result<Order, String> {
        let side = match self.side.to_lowercase().as_str() {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => return Err("Invalid order side".to_string()),
        };

        let order_type = match self.order_type.to_lowercase().as_str() {
            "market" => OrderType::Market,
            "limit" => OrderType::Limit,
            "stop" => OrderType::Stop,
            "stop_limit" => OrderType::StopLimit,
            _ => return Err("Invalid order type".to_string()),
        };

        Ok(Order::new(
            self.symbol.clone(),
            order_type,
            side,
            self.quantity,
            self.price.unwrap_or(0.0),
            self.account_id.clone().unwrap_or_default(),
        ))
    }
}

/// Trade update request
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTradeRequest {
    /// New quantity (optional)
    pub quantity: Option<f64>,

    /// New price (optional)
    pub price: Option<f64>,

    /// New order type (optional)
    pub order_type: Option<String>,

    /// Additional metadata updates
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl UpdateTradeRequest {
    /// Validate the update request
    pub fn validate(&self) -> Result<(), String> {
        if let Some(quantity) = self.quantity {
            if quantity <= 0.0 {
                return Err("Quantity must be positive".to_string());
            }
        }

        if let Some(price) = self.price {
            if price <= 0.0 {
                return Err("Price must be positive".to_string());
            }
        }

        if let Some(ref order_type) = self.order_type {
            match order_type.to_lowercase().as_str() {
                "market" | "limit" | "stop" | "stop_limit" => {},
                _ => return Err("Order type must be 'market', 'limit', 'stop', or 'stop_limit'".to_string()),
            }
        }

        Ok(())
    }
}

/// Trade response (API representation)
#[derive(Debug, Serialize, Deserialize)]
pub struct TradeResponse {
    /// Order ID
    pub id: String,

    /// Trading symbol
    pub symbol: String,

    /// Order side
    pub side: String,

    /// Order quantity
    pub quantity: f64,

    /// Order price
    pub price: f64,

    /// Order type
    pub order_type: String,

    /// Order status
    pub status: String,

    /// Filled quantity
    pub filled_quantity: f64,

    /// Average fill price
    pub average_fill_price: f64,

    /// Order timestamp
    pub timestamp: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Account ID
    pub account_id: String,

    /// Additional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl From<Order> for TradeResponse {
    fn from(order: Order) -> Self {
        Self {
            id: order.id,
            symbol: order.symbol,
            side: format!("{:?}", order.side),
            quantity: order.quantity,
            price: order.price,
            order_type: format!("{:?}", order.order_type),
            status: format!("{:?}", order.status),
            filled_quantity: order.filled_quantity,
            average_fill_price: order.average_fill_price,
            timestamp: order.timestamp,
            updated_at: order.updated_at,
            account_id: order.account_id,
            metadata: None, // Core Order doesn't have metadata
        }
    }
}

/// Market data request
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataRequest {
    /// Trading symbols
    pub symbols: Vec<String>,

    /// Include historical data
    pub include_history: Option<bool>,

    /// Number of historical periods
    pub history_limit: Option<usize>,
}

/// Market data response
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataResponse {
    /// Trading symbol
    pub symbol: String,

    /// Current price
    pub price: f64,

    /// Price change (24h)
    pub change_24h: f64,

    /// Volume (24h)
    pub volume_24h: f64,

    /// Market cap
    pub market_cap: Option<f64>,

    /// Last update timestamp
    pub timestamp: DateTime<Utc>,

    /// Historical data (if requested)
    pub history: Option<Vec<MarketDataPoint>>,
}

/// Individual market data point
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataPoint {
    /// Price at this point
    pub price: f64,

    /// Volume at this point
    pub volume: f64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Portfolio summary response
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioResponse {
    /// Account ID
    pub account_id: String,

    /// Total portfolio value
    pub total_value: f64,

    /// Available cash
    pub available_cash: f64,

    /// Total positions value
    pub positions_value: f64,

    /// Total unrealized P&L
    pub unrealized_pnl: f64,

    /// Positions count
    pub positions_count: usize,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Performance metrics
    pub performance: PortfolioPerformance,
}

/// Portfolio performance metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioPerformance {
    /// Daily P&L
    pub daily_pnl: f64,

    /// Daily return percentage
    pub daily_return: f64,

    /// Total return percentage
    pub total_return: f64,

    /// Sharpe ratio
    pub sharpe_ratio: f64,

    /// Maximum drawdown
    pub max_drawdown: f64,
}

impl From<Portfolio> for PortfolioResponse {
    fn from(portfolio: Portfolio) -> Self {
        Self {
            account_id: portfolio.account_id,
            total_value: portfolio.total_value,
            available_cash: portfolio.available_cash,
            positions_value: portfolio.positions_value,
            unrealized_pnl: portfolio.unrealized_pnl,
            positions_count: portfolio.positions.len(),
            updated_at: portfolio.updated_at,
            performance: PortfolioPerformance {
                daily_pnl: 0.0, // Would be calculated from historical data
                daily_return: 0.0,
                total_return: 0.0,
                sharpe_ratio: 0.0,
                max_drawdown: 0.0,
            },
        }
    }
}

/// Strategy creation request
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    /// Strategy name
    pub name: String,

    /// Strategy description
    pub description: Option<String>,

    /// Strategy parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Is this strategy active?
    pub is_active: Option<bool>,

    /// Account IDs to apply this strategy to
    pub account_ids: Option<Vec<String>>,
}

impl CreateStrategyRequest {
    /// Validate the strategy request
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Strategy name cannot be empty".to_string());
        }

        if self.name.len() > 100 {
            return Err("Strategy name cannot exceed 100 characters".to_string());
        }

        Ok(())
    }
}

/// Strategy response
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyResponse {
    /// Strategy ID
    pub id: String,

    /// Strategy name
    pub name: String,

    /// Strategy description
    pub description: Option<String>,

    /// Strategy parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Is this strategy active?
    pub is_active: bool,

    /// Account IDs
    pub account_ids: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Performance metrics
    pub performance: StrategyPerformance,
}

/// Strategy performance metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyPerformance {
    /// Total trades executed
    pub total_trades: usize,

    /// Win rate (percentage)
    pub win_rate: f64,

    /// Total P&L
    pub total_pnl: f64,

    /// Average trade duration
    pub avg_trade_duration: f64,

    /// Maximum drawdown
    pub max_drawdown: f64,
}

/// WebSocket subscription request
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    /// Type of subscription
    pub subscription_type: String,

    /// Symbols to subscribe to
    pub symbols: Option<Vec<String>>,

    /// Additional parameters
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

impl SubscriptionRequest {
    /// Validate the subscription request
    pub fn validate(&self) -> Result<(), String> {
        match self.subscription_type.as_str() {
            "market_data" | "trades" | "portfolio" => {},
            _ => return Err("Invalid subscription type".to_string()),
        }

        if let Some(ref symbols) = self.symbols {
            if symbols.is_empty() {
                return Err("Symbols list cannot be empty".to_string());
            }
        }

        Ok(())
    }
}

/// WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Market data update
    #[serde(rename = "market_data")]
    MarketData {
        symbol: String,
        price: f64,
        timestamp: DateTime<Utc>,
    },

    /// Trade update
    #[serde(rename = "trade_update")]
    TradeUpdate {
        trade_id: String,
        status: String,
        filled_quantity: f64,
        timestamp: DateTime<Utc>,
    },

    /// Portfolio update
    #[serde(rename = "portfolio_update")]
    PortfolioUpdate {
        account_id: String,
        total_value: f64,
        unrealized_pnl: f64,
        timestamp: DateTime<Utc>,
    },

    /// Error message
    #[serde(rename = "error")]
    Error {
        message: String,
        timestamp: DateTime<Utc>,
    },
}

/// Portfolio response model
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioResponse {
    /// Portfolio ID
    pub portfolio_id: String,

    /// Total portfolio value
    pub total_value: f64,

    /// Total unrealized P&L
    pub total_unrealized_pnl: f64,

    /// Total realized P&L
    pub total_realized_pnl: f64,

    /// Portfolio positions
    pub positions: Vec<PositionResponse>,

    /// Performance metrics
    pub performance: PerformanceMetricsResponse,

    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Individual position response
#[derive(Debug, Serialize, Deserialize)]
pub struct PositionResponse {
    /// Trading symbol
    pub symbol: String,

    /// Position quantity
    pub quantity: f64,

    /// Average cost basis
    pub average_cost: f64,

    /// Current market price
    pub current_price: f64,

    /// Current market value
    pub market_value: f64,

    /// Unrealized P&L
    pub unrealized_pnl: f64,

    /// Realized P&L
    pub realized_pnl: f64,

    /// Position allocation percentage
    pub allocation_percentage: f64,
}

/// Performance metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetricsResponse {
    /// Daily return percentage
    pub daily_return: f64,

    /// Weekly return percentage
    pub weekly_return: f64,

    /// Monthly return percentage
    pub monthly_return: f64,

    /// Yearly return percentage
    pub yearly_return: f64,

    /// Sharpe ratio
    pub sharpe_ratio: f64,

    /// Maximum drawdown
    pub max_drawdown: f64,

    /// Volatility measure
    pub volatility: f64,
}

/// Allocation breakdown response
#[derive(Debug, Serialize, Deserialize)]
pub struct AllocationResponse {
    /// Trading symbol
    pub symbol: String,

    /// Allocation percentage
    pub allocation_percentage: f64,

    /// Market value
    pub market_value: f64,

    /// Weight in portfolio
    pub weight: f64,
}

/// Portfolio summary request parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioSummaryRequest {
    /// Include historical data
    pub include_history: Option<bool>,

    /// Historical data period (days)
    pub history_period: Option<usize>,

    /// Include performance metrics
    pub include_performance: Option<bool>,

    /// Include risk metrics
    pub include_risk_metrics: Option<bool>,

    /// Account IDs to filter by
    pub account_ids: Option<Vec<String>>,
}

impl Default for PortfolioSummaryRequest {
    fn default() -> Self {
        Self {
            include_history: Some(false),
            history_period: Some(30),
            include_performance: Some(true),
            include_risk_metrics: Some(false),
            account_ids: None,
        }
    }
}

/// Portfolio rebalance request
#[derive(Debug, Serialize, Deserialize)]
pub struct RebalanceRequest {
    /// Target allocation by symbol
    pub target_allocations: HashMap<String, f64>,

    /// Maximum rebalance threshold
    pub max_rebalance_threshold: Option<f64>,

    /// Allow selling to rebalance
    pub allow_selling: Option<bool>,

    /// Rebalance strategy
    pub strategy: Option<String>,
}

impl Default for RebalanceRequest {
    fn default() -> Self {
        Self {
            target_allocations: HashMap::new(),
            max_rebalance_threshold: Some(0.02), // 2%
            allow_selling: Some(true),
            strategy: Some("equal_weight".to_string()),
        }
    }
}

/// Rebalance response
#[derive(Debug, Serialize, Deserialize)]
pub struct RebalanceResponse {
    /// Whether rebalance was successful
    pub success: bool,

    /// Number of orders created
    pub orders_created: usize,

    /// Total orders planned
    pub total_orders: usize,

    /// Estimated cost of rebalance
    pub estimated_cost: f64,

    /// Response message
    pub message: String,
}

/// Portfolio historical data point
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioHistoryResponse {
    /// Portfolio value at this point
    pub portfolio_value: f64,

    /// Total P&L at this point
    pub total_pnl: f64,

    /// Daily return at this point
    pub daily_return: f64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Risk metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskMetricsResponse {
    /// Value at Risk (95% confidence)
    pub var_95: f64,

    /// Value at Risk (99% confidence)
    pub var_99: f64,

    /// Conditional Value at Risk (95%)
    pub cvar_95: f64,

    /// Portfolio beta
    pub beta: f64,

    /// Portfolio alpha
    pub alpha: f64,

    /// Treynor ratio
    pub treynor_ratio: f64,

    /// Sortino ratio
    pub sortino_ratio: f64,

    /// Information ratio
    pub information_ratio: f64,
}

/// Market data response with technical indicators
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataWithIndicators {
    /// Trading symbol
    pub symbol: String,

    /// Current market price
    pub current_price: f64,

    /// Simple moving average (20 periods)
    pub sma_20: f64,

    /// Simple moving average (50 periods)
    pub sma_50: f64,

    /// Exponential moving average (12 periods)
    pub ema_12: f64,

    /// Exponential moving average (26 periods)
    pub ema_26: f64,

    /// Relative strength index (14 periods)
    pub rsi_14: f64,

    /// MACD line
    pub macd_line: f64,

    /// MACD signal line
    pub macd_signal: f64,

    /// Bollinger bands upper
    pub bollinger_upper: f64,

    /// Bollinger bands middle
    pub bollinger_middle: f64,

    /// Bollinger bands lower
    pub bollinger_lower: f64,

    /// Volume SMA
    pub volume_sma: f64,

    /// Last update timestamp
    pub timestamp: DateTime<Utc>,
}

/// Symbol information for search results
#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Trading symbol
    pub symbol: String,

    /// Full company/instrument name
    pub name: String,

    /// Exchange where the symbol is traded
    pub exchange: String,

    /// Asset type (stock, crypto, forex, etc.)
    pub asset_type: String,

    /// Whether the symbol is actively traded
    pub is_active: bool,
}

/// Market overview response
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketOverview {
    /// Top gaining symbols
    pub top_gainers: Vec<MarketDataResponse>,

    /// Top losing symbols
    pub top_losers: Vec<MarketDataResponse>,

    /// Highest volume symbols
    pub volume_leaders: Vec<MarketDataResponse>,

    /// Market indices
    pub market_indices: Vec<MarketIndex>,

    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Market index information
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketIndex {
    /// Index symbol (e.g., SPX, NDX, VIX)
    pub symbol: String,

    /// Index name
    pub name: String,

    /// Current value
    pub value: f64,

    /// Daily change
    pub change: f64,

    /// Change percentage
    pub change_percent: f64,

    /// Last update timestamp
    pub timestamp: DateTime<Utc>,
}

/// Stream subscription response
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamSubscriptionResponse {
    /// Subscription ID
    pub subscription_id: String,

    /// Symbol being subscribed to
    pub symbol: String,

    /// Type of stream
    pub stream_type: String,

    /// Whether subscription is active
    pub is_active: bool,

    /// Response message
    pub message: String,
}

/// Symbol search request parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchSymbolsRequest {
    /// Search query string
    pub query: String,

    /// Maximum number of results to return
    pub limit: Option<usize>,
}

impl Default for SearchSymbolsRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: Some(20),
        }
    }
}

/// Price statistics for market data
#[derive(Debug, Serialize, Deserialize)]
pub struct PriceStatistics {
    /// Opening price
    pub open: f64,

    /// Highest price
    pub high: f64,

    /// Lowest price
    pub low: f64,

    /// Closing price
    pub close: f64,

    /// Volume
    pub volume: f64,

    /// Volume-weighted average price
    pub vwap: f64,
}

/// Volatility metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct VolatilityMetrics {
    /// Daily volatility
    pub daily_volatility: f64,

    /// Weekly volatility
    pub weekly_volatility: f64,

    /// Monthly volatility
    pub monthly_volatility: f64,

    /// Average true range
    pub average_true_range: f64,
}

/// Liquidity metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct LiquidityMetrics {
    /// Bid-ask spread
    pub bid_ask_spread: f64,

    /// Market depth
    pub market_depth: f64,

    /// Turnover ratio
    pub turnover_ratio: f64,
}

/// Trading activity metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct TradingActivity {
    /// Total number of trades
    pub total_trades: usize,

    /// Average trade size
    pub average_trade_size: f64,

    /// Trade frequency (trades per minute)
    pub trade_frequency: f64,
}

/// Complete market statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct MarketStatistics {
    /// Trading symbol
    pub symbol: String,

    /// Price statistics
    pub price_statistics: PriceStatistics,

    /// Volatility metrics
    pub volatility_metrics: VolatilityMetrics,

    /// Liquidity metrics
    pub liquidity_metrics: LiquidityMetrics,

    /// Trading activity
    pub trading_activity: TradingActivity,

    /// Last update timestamp
    pub timestamp: DateTime<Utc>,
}