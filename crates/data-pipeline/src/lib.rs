//! Ninja Gekko data pipeline crate.
//!
//! This crate implements the three-stage market data pipeline described in the
//! AGENTS architecture:
//!
//! 1. **Ingestion** — streams raw WebSocket payloads from exchanges.
//! 2. **Normalization** — converts exchange-specific payloads into
//!    `event_bus::MarketEvent` envelopes enriched with sequence numbers and
//!    timestamps while maintaining Level 2 order books.
//! 3. **Distribution** — fans out normalized events onto the validated
//!    `event-bus` channels with bounded backpressure control.
//!
//! The primitives exported here integrate tightly with the existing
//! `ExchangeConnector` trait and the high-performance event bus without
//! modifying those foundational crates.

pub mod distributor;
pub mod ingestion;
pub mod normalizer;
pub mod order_book;
pub mod pipeline;
pub mod websocket;

pub use distributor::Distributor;
pub use ingestion::{IngestionConfig, StreamIngestor};
pub use normalizer::{MarketNormalizer, NormalizedEvent};
pub use order_book::{LevelTwoBook, OrderBookSide};
pub use pipeline::{DataPipeline, DataPipelineBuilder, DataPipelineHandle};
pub use websocket::{
    spawn_stream as spawn_websocket_stream, BackoffConfig, HeartbeatConfig, WebSocketConfig,
    WebSocketEvent,
};
