use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use event_bus::{MarketEvent, Priority, SignalEventPayload, StrategySignal};
use ninja_gekko_core::types::AccountId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Compile-time sized market snapshot buffer supplied to strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub bid: Decimal,
    pub ask: Decimal,
    pub last: Decimal,
    pub timestamp: DateTime<Utc>,
}

impl MarketSnapshot {
    pub fn from_market_event(
        symbol: impl Into<String>,
        bid: Decimal,
        ask: Decimal,
        last: Decimal,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            bid,
            ask,
            last,
            timestamp: Utc::now(),
        }
    }
}

/// Context made available to a strategy evaluation cycle.
pub struct StrategyContext<'a, const N: usize> {
    account_id: &'a AccountId,
    snapshots: &'a [MarketSnapshot; N],
    evaluation_id: Uuid,
    as_of: DateTime<Utc>,
    market_events: Option<&'a [MarketEvent]>,
}

impl<'a, const N: usize> StrategyContext<'a, N> {
    pub fn new(
        account_id: &'a AccountId,
        snapshots: &'a [MarketSnapshot; N],
        evaluation_id: Uuid,
        as_of: DateTime<Utc>,
    ) -> Self {
        Self {
            account_id,
            snapshots,
            evaluation_id,
            as_of,
            market_events: None,
        }
    }

    pub fn with_events(mut self, events: &'a [MarketEvent]) -> Self {
        self.market_events = Some(events);
        self
    }

    pub fn account_id(&self) -> &AccountId {
        self.account_id
    }

    pub fn evaluation_id(&self) -> Uuid {
        self.evaluation_id
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.as_of
    }

    pub fn snapshot(&self, index: usize) -> Option<&MarketSnapshot> {
        self.snapshots.get(index)
    }

    pub fn snapshots(&self) -> &[MarketSnapshot] {
        &self.snapshots[..]
    }

    pub fn market_events(&self) -> Option<&[MarketEvent]> {
        self.market_events
    }
}

/// Initialization context executed once prior to evaluation.
pub struct StrategyInitContext<'a> {
    pub strategy_id: Uuid,
    pub account_id: &'a AccountId,
}

/// Result produced by a strategy, including generated signals and logging output.
pub struct StrategyDecision {
    pub signals: Vec<SignalEventPayload>,
    pub logs: Vec<String>,
    pub metrics: StrategyMetrics,
}

impl StrategyDecision {
    pub fn empty() -> Self {
        Self {
            signals: Vec::new(),
            logs: Vec::new(),
            metrics: StrategyMetrics::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StrategyMetrics {
    pub evaluation_latency: Duration,
}

/// Contracts every strategy implementation must satisfy.
pub trait StrategyExecutor<const N: usize>: Send {
    fn name(&self) -> &str;

    fn initialize(&mut self, _ctx: StrategyInitContext<'_>) -> Result<(), StrategyError> {
        Ok(())
    }

    fn evaluate(&mut self, ctx: StrategyContext<'_, N>) -> Result<StrategyDecision, StrategyError>;
}

/// Errors surfaced during strategy execution or sandbox orchestration.
#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("serialization failure: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("sandbox error: {0}")]
    Sandbox(String),
    #[error("wasm trap: {0}")]
    Wasm(#[from] anyhow::Error),
    #[error("strategy evaluation exceeded {0:?}")]
    Timeout(Duration),
}

impl StrategyError {
    pub fn sandbox(message: impl Into<String>) -> Self {
        StrategyError::Sandbox(message.into())
    }
}

/// Helper structure emitted by WASM host callbacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSignalInstruction {
    pub strategy_id: Uuid,
    pub account_id: AccountId,
    pub priority: Priority,
    pub signal: StrategySignal,
}

impl fmt::Display for WasmSignalInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {} {:?} {:?} qty {}",
            self.strategy_id,
            self.account_id,
            self.priority,
            self.signal.symbol,
            self.signal.quantity
        )
    }
}
