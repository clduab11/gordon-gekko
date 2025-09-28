#![allow(missing_docs)]

//! Utilities for adapting exchange connector streams into bus events.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use exchange_connectors::{ExchangeConnector, ExchangeId, MarketTick, StreamMessage, TradingPair};
use tracing::trace;

use crate::channel::{EventSender, PublishMode};
use crate::dispatcher::EventHandler;
use crate::envelope::{MarketEvent, MarketPayload};
use crate::error::EventBusError;
use crate::metadata::{EventMetadata, EventSource, Priority};

/// Emits market events for a specific exchange using a shared sender.
pub struct MarketEventEmitter {
    exchange: ExchangeId,
    sender: EventSender<MarketEvent>,
    mode: PublishMode,
}

impl MarketEventEmitter {
    /// Creates a new emitter tied to a specific exchange identifier.
    pub fn new(exchange: ExchangeId, sender: EventSender<MarketEvent>, mode: PublishMode) -> Self {
        Self {
            exchange,
            sender,
            mode,
        }
    }

    /// Emits a single tick message into the event bus.
    pub fn emit_tick(&self, tick: MarketTick, pair: TradingPair) -> Result<(), EventBusError> {
        let source = EventSource::new(format!("exchange.{:?}", self.exchange));
        let metadata = EventMetadata::new(source, Priority::High);
        let payload = MarketPayload::Tick { tick, pair };
        let event = MarketEvent::new(metadata, payload);
        self.sender.publish(event, self.mode)
    }
}

/// Processes streaming messages from connectors and forwards ticks into the bus.
pub struct StreamMessageHandler {
    exchange: ExchangeId,
    emitter: MarketEventEmitter,
}

impl StreamMessageHandler {
    /// Constructs a new handler that forwards events through the supplied emitter.
    pub fn new(exchange: ExchangeId, emitter: MarketEventEmitter) -> Self {
        Self { exchange, emitter }
    }
}

#[async_trait]
impl EventHandler<StreamMessage> for StreamMessageHandler {
    async fn handle(&self, message: StreamMessage) -> Result<(), EventBusError> {
        match message {
            StreamMessage::Tick(tick) => {
                // Derive trading pair symbol from the tick data.
                let pair = TradingPair {
                    base: tick.symbol.split('-').next().unwrap_or("?").to_string(),
                    quote: tick.symbol.split('-').nth(1).unwrap_or("?").to_string(),
                    symbol: tick.symbol.clone(),
                };
                self.emitter.emit_tick(tick, pair)?;
            }
            StreamMessage::Error(err) => {
                return Err(EventBusError::upstream(err));
            }
            _ => {
                trace!(target: "event_bus.exchange", ?message, exchange = ?self.exchange, "non-tick stream message ignored")
            }
        }
        Ok(())
    }
}

/// Helper that pulls trading pairs from a connector to seed downstream modules.
pub async fn fetch_pairs(
    connector: Arc<dyn ExchangeConnector>,
) -> Result<Vec<TradingPair>, EventBusError> {
    connector
        .get_trading_pairs()
        .await
        .map_err(EventBusError::upstream)
}

impl fmt::Debug for MarketEventEmitter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MarketEventEmitter")
            .field("exchange", &self.exchange)
            .finish()
    }
}

impl fmt::Debug for StreamMessageHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamMessageHandler")
            .field("exchange", &self.exchange)
            .finish_non_exhaustive()
    }
}
