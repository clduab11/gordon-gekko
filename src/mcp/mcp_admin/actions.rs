//! MCP Admin Actions for Arbitrage Operations
//! 
//! This module implements administrative actions for the Tenno-MCP system
//! specifically focused on fund transfers, balance queries, and risk management
//! for the Gordon Gekko arbitrage trading system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Re-export types from arbitrage engine for MCP integration  
pub use exchange_connectors::{ExchangeId, TransferUrgency, TransferStatus};
pub use arbitrage_engine::{AllocationPriority, ArbitrageConfig};

/// Transfer request for cross-exchange fund movements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub id: Uuid,
    pub from_exchange: ExchangeId,
    pub to_exchange: ExchangeId,
    pub currency: String,
    pub amount: rust_decimal::Decimal,
    pub urgency: TransferUrgency,
    pub reason: String,
    pub requested_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Balance query request for exchange account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceQuery {
    pub id: Uuid,
    pub exchange_id: Option<ExchangeId>, // None = all exchanges
    pub currency: Option<String>,        // None = all currencies
    pub include_unavailable: bool,
    pub requested_at: chrono::DateTime<chrono::Utc>,
}

/// Balance response containing account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub query_id: Uuid,
    pub exchange_balances: HashMap<ExchangeId, Vec<CurrencyBalance>>,
    pub total_portfolio_value_usd: rust_decimal::Decimal,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}

/// Individual currency balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyBalance {
    pub currency: String,
    pub available: rust_decimal::Decimal,
    pub total: rust_decimal::Decimal,
    pub reserved: rust_decimal::Decimal,
    pub usd_value: rust_decimal::Decimal,
}

/// Emergency shutdown request for risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyShutdown {
    pub id: Uuid,
    pub reason: ShutdownReason,
    pub scope: ShutdownScope,
    pub initiated_by: String,
    pub initiated_at: chrono::DateTime<chrono::Utc>,
    pub auto_resume_after: Option<chrono::Duration>,
}

/// Reasons for emergency shutdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShutdownReason {
    RiskLimitExceeded,
    ManualIntervention,
    SystemError,
    ExchangeConnectivityIssue,
    RegulatoryCompliance,
    MarketVolatilityExtreme,
}

/// Scope of emergency shutdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShutdownScope {
    AllTrading,
    SpecificExchange(ExchangeId),
    ArbitrageOnly,
    SpecificSymbol(String),
}

/// Position rebalancing request for portfolio optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRebalance {
    pub id: Uuid,
    pub target_allocation: HashMap<ExchangeId, rust_decimal::Decimal>, // Percentage allocations
    pub currency: String,
    pub priority: AllocationPriority,
    pub max_slippage_percent: f64,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub reason: String,
}

/// Batch operation for multiple fund movements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub id: Uuid,
    pub operations: Vec<BatchOperationType>,
    pub execute_atomically: bool, // All succeed or all fail
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub deadline: chrono::DateTime<chrono::Utc>,
}

/// Types of operations that can be batched
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperationType {
    Transfer(TransferRequest),
    BalanceQuery(BalanceQuery),
    PositionRebalance(PositionRebalance),
}

/// System health status for arbitrage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageSystemHealth {
    pub overall_status: SystemStatus,
    pub exchange_status: HashMap<ExchangeId, ExchangeStatus>,
    pub capital_allocation_health: AllocationHealth,
    pub risk_monitor_status: RiskMonitorStatus,
    pub neural_engine_status: NeuralEngineStatus,
    pub last_arbitrage_execution: Option<chrono::DateTime<chrono::Utc>>,
    pub active_opportunities: u32,
    pub success_rate_24h: f64,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExchangeStatus {
    Connected,
    Connecting,
    Disconnected,
    RateLimited,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationHealth {
    Optimal,
    Suboptimal,
    Critical,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskMonitorStatus {
    Active,
    Warning,
    CircuitBreakerTriggered,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NeuralEngineStatus {
    Operational,
    Training,
    Loading,
    Error(String),
}
