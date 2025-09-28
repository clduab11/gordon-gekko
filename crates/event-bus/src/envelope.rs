#![allow(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;

use bincode::Options;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EventBusError;
use crate::metadata::{EventKind, EventMetadata, Priority};

#[cfg(feature = "exchange-integration")]
use exchange_connectors::{ExchangeId, MarketTick, TradingPair};

#[cfg(feature = "core-integration")]
use ninja_gekko_core::types::{AccountId, Execution, Order, OrderSide, OrderType};

/// Serialized event frame containing metadata and a zero-copy payload buffer.
#[derive(Debug, Clone)]
pub struct EventFrame {
    kind: EventKind,
    metadata: EventMetadata,
    payload: Arc<[u8]>,
}

impl EventFrame {
    /// Constructs an event frame from raw payload bytes.
    pub fn from_parts(kind: EventKind, metadata: EventMetadata, payload: Arc<[u8]>) -> Self {
        Self {
            kind,
            metadata,
            payload,
        }
    }

    /// Serializes a payload into a frame using bincode's compact encoding.
    pub fn from_payload<T>(
        kind: EventKind,
        metadata: EventMetadata,
        payload: &T,
    ) -> Result<Self, EventBusError>
    where
        T: Serialize,
    {
        let bytes = serialize(payload)?;
        Ok(Self::from_parts(kind, metadata, bytes))
    }

    /// Deserializes the payload into the requested type.
    pub fn decode<T>(&self) -> Result<T, EventBusError>
    where
        T: for<'de> Deserialize<'de>,
    {
        deserialize(&self.payload)
    }

    /// Returns the event kind.
    pub fn kind(&self) -> EventKind {
        self.kind
    }

    /// Returns immutable metadata.
    pub fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    /// Returns the shared payload buffer without copying.
    pub fn payload(&self) -> Arc<[u8]> {
        Arc::clone(&self.payload)
    }
}

fn serialize<T: Serialize>(value: &T) -> Result<Arc<[u8]>, EventBusError> {
    let config = bincode::options()
        .with_fixint_encoding()
        .allow_trailing_bytes();
    let bytes = config
        .serialize(value)
        .map_err(EventBusError::serialization)?;
    Ok(Arc::from(bytes.into_boxed_slice()))
}

fn deserialize<T>(bytes: &[u8]) -> Result<T, EventBusError>
where
    T: for<'de> Deserialize<'de>,
{
    let config = bincode::options()
        .with_fixint_encoding()
        .allow_trailing_bytes();
    config
        .deserialize(bytes)
        .map_err(EventBusError::deserialization)
}

/// Market data payload level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    /// Price for this level.
    pub price: Decimal,
    /// Aggregate size at this price.
    pub size: Decimal,
}

/// Market event payload content.
#[cfg(feature = "exchange-integration")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketPayload {
    /// Standard best-bid-offer tick.
    Tick { tick: MarketTick, pair: TradingPair },
    /// Level 2 snapshot with capped depth.
    OrderBookSnapshot {
        pair: TradingPair,
        bids: Vec<OrderBookLevel>,
        asks: Vec<OrderBookLevel>,
        depth: usize,
    },
    /// Delta update derived from exchange diff streams.
    OrderBookDelta {
        pair: TradingPair,
        bid_updates: Vec<OrderBookLevel>,
        ask_updates: Vec<OrderBookLevel>,
        sequence: u64,
    },
}

/// Market event delivered over the bus.
#[cfg(feature = "exchange-integration")]
#[derive(Debug, Clone)]
pub struct MarketEvent {
    metadata: EventMetadata,
    payload: Arc<MarketPayload>,
}

#[cfg(feature = "exchange-integration")]
impl MarketEvent {
    /// Creates a new market event.
    pub fn new(metadata: EventMetadata, payload: MarketPayload) -> Self {
        Self {
            metadata,
            payload: Arc::new(payload),
        }
    }

    /// Event metadata accessor.
    pub fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    /// Market payload accessor.
    pub fn payload(&self) -> &MarketPayload {
        &self.payload
    }

    /// Returns a clone of the shared payload pointer for zero-copy fan-out.
    pub fn payload_arc(&self) -> Arc<MarketPayload> {
        Arc::clone(&self.payload)
    }

    /// Converts the event into an encoded frame.
    pub fn to_frame(&self) -> Result<EventFrame, EventBusError> {
        EventFrame::from_payload(EventKind::Market, self.metadata.clone(), &*self.payload)
    }

    /// Reconstructs a typed market event from a frame.
    pub fn from_frame(frame: &EventFrame) -> Result<Self, EventBusError> {
        if frame.kind() != EventKind::Market {
            return Err(EventBusError::kind_mismatch(
                EventKind::Market,
                frame.kind(),
            ));
        }
        let payload: MarketPayload = frame.decode()?;
        Ok(Self::new(frame.metadata().clone(), payload))
    }
}

/// Strategy signal payload describing an intent to trade.
#[cfg(feature = "core-integration")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySignal {
    /// Exchange target.
    pub exchange: Option<ExchangeId>,
    /// Trading symbol for the action.
    pub symbol: String,
    /// Requested order side.
    pub side: OrderSide,
    /// Order type to employ.
    pub order_type: OrderType,
    /// Desired quantity.
    pub quantity: Decimal,
    /// Optional limit/stop price.
    pub limit_price: Option<Decimal>,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
    /// Additional metadata emitted by the strategy.
    pub metadata: HashMap<String, String>,
}

/// Signal event produced by strategy runners.
#[cfg(feature = "core-integration")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEventPayload {
    /// Strategy identifier.
    pub strategy_id: Uuid,
    /// Account against which orders will be placed.
    pub account_id: AccountId,
    /// Priority hint for the dispatcher.
    pub priority: Priority,
    /// Signal description.
    pub signal: StrategySignal,
}

#[cfg(feature = "core-integration")]
#[derive(Debug, Clone)]
pub struct SignalEvent {
    metadata: EventMetadata,
    payload: Arc<SignalEventPayload>,
}

#[cfg(feature = "core-integration")]
impl SignalEvent {
    pub fn new(metadata: EventMetadata, payload: SignalEventPayload) -> Self {
        Self {
            metadata,
            payload: Arc::new(payload),
        }
    }

    pub fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    pub fn payload(&self) -> &SignalEventPayload {
        &self.payload
    }

    pub fn payload_arc(&self) -> Arc<SignalEventPayload> {
        Arc::clone(&self.payload)
    }

    pub fn to_frame(&self) -> Result<EventFrame, EventBusError> {
        EventFrame::from_payload(EventKind::Signal, self.metadata.clone(), &*self.payload)
    }

    pub fn from_frame(frame: &EventFrame) -> Result<Self, EventBusError> {
        if frame.kind() != EventKind::Signal {
            return Err(EventBusError::kind_mismatch(
                EventKind::Signal,
                frame.kind(),
            ));
        }
        let payload: SignalEventPayload = frame.decode()?;
        Ok(Self::new(frame.metadata().clone(), payload))
    }
}

/// Order event payload containing a concrete order request.
#[cfg(feature = "core-integration")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEventPayload {
    /// Order generated by upstream modules.
    pub order: Order,
}

#[cfg(feature = "core-integration")]
#[derive(Debug, Clone)]
pub struct OrderEvent {
    metadata: EventMetadata,
    payload: Arc<OrderEventPayload>,
}

#[cfg(feature = "core-integration")]
impl OrderEvent {
    pub fn new(metadata: EventMetadata, order: Order) -> Self {
        Self {
            metadata,
            payload: Arc::new(OrderEventPayload { order }),
        }
    }

    pub fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    pub fn order(&self) -> &Order {
        &self.payload.order
    }

    pub fn payload_arc(&self) -> Arc<OrderEventPayload> {
        Arc::clone(&self.payload)
    }

    pub fn to_frame(&self) -> Result<EventFrame, EventBusError> {
        EventFrame::from_payload(EventKind::Order, self.metadata.clone(), &*self.payload)
    }

    pub fn from_frame(frame: &EventFrame) -> Result<Self, EventBusError> {
        if frame.kind() != EventKind::Order {
            return Err(EventBusError::kind_mismatch(EventKind::Order, frame.kind()));
        }
        let payload: OrderEventPayload = frame.decode()?;
        Ok(Self::new(frame.metadata().clone(), payload.order))
    }
}

/// Execution event payload wrapping fills and order state transitions.
#[cfg(feature = "core-integration")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEventPayload {
    pub execution: Execution,
}

#[cfg(feature = "core-integration")]
#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    metadata: EventMetadata,
    payload: Arc<ExecutionEventPayload>,
}

#[cfg(feature = "core-integration")]
impl ExecutionEvent {
    pub fn new(metadata: EventMetadata, execution: Execution) -> Self {
        Self {
            metadata,
            payload: Arc::new(ExecutionEventPayload { execution }),
        }
    }

    pub fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    pub fn execution(&self) -> &Execution {
        &self.payload.execution
    }

    pub fn payload_arc(&self) -> Arc<ExecutionEventPayload> {
        Arc::clone(&self.payload)
    }

    pub fn to_frame(&self) -> Result<EventFrame, EventBusError> {
        EventFrame::from_payload(EventKind::Execution, self.metadata.clone(), &*self.payload)
    }

    pub fn from_frame(frame: &EventFrame) -> Result<Self, EventBusError> {
        if frame.kind() != EventKind::Execution {
            return Err(EventBusError::kind_mismatch(
                EventKind::Execution,
                frame.kind(),
            ));
        }
        let payload: ExecutionEventPayload = frame.decode()?;
        Ok(Self::new(frame.metadata().clone(), payload.execution))
    }
}

/// Actions emitted by the risk/routing subsystems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskAction {
    /// Immediate halt of all executions.
    HaltAll { reason: String },
    /// Resume normal trading.
    Resume { reason: String },
    /// Adjust exposure by a factor (0.0 - 1.0 scale).
    AdjustExposure { factor: f64, reason: String },
    /// Notify but continue trading.
    Advisory { message: String },
}

/// Risk event payload accompanying a control action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEventPayload {
    pub action: RiskAction,
    pub priority: Priority,
    pub tags: HashMap<String, String>,
}

/// Risk event struct.
#[derive(Debug, Clone)]
pub struct RiskEvent {
    metadata: EventMetadata,
    payload: Arc<RiskEventPayload>,
}

impl RiskEvent {
    pub fn new(metadata: EventMetadata, payload: RiskEventPayload) -> Self {
        Self {
            metadata,
            payload: Arc::new(payload),
        }
    }

    pub fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    pub fn payload(&self) -> &RiskEventPayload {
        &self.payload
    }

    pub fn payload_arc(&self) -> Arc<RiskEventPayload> {
        Arc::clone(&self.payload)
    }

    pub fn to_frame(&self) -> Result<EventFrame, EventBusError> {
        EventFrame::from_payload(EventKind::Risk, self.metadata.clone(), &*self.payload)
    }

    pub fn from_frame(frame: &EventFrame) -> Result<Self, EventBusError> {
        if frame.kind() != EventKind::Risk {
            return Err(EventBusError::kind_mismatch(EventKind::Risk, frame.kind()));
        }
        let payload: RiskEventPayload = frame.decode()?;
        Ok(Self::new(frame.metadata().clone(), payload))
    }
}
