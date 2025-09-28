#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    unused_qualifications,
    missing_debug_implementations
)]

//! Event bus crate providing high-performance event-driven plumbing for the Ninja Gekko
//! trading engine. It delivers sub-millisecond dispatch guarantees through bounded
//! crossbeam channels, zero-copy event frames, and async dispatchers that integrate
//! existing core modules without mutating their APIs.

mod channel;
mod dispatcher;
mod envelope;
mod error;
mod metadata;
mod util;

pub use channel::{
    EventBus, EventBusBuilder, EventPublishResult, EventReceiver, EventSender, PublishMode,
};
pub use dispatcher::{
    ClosureHandler, EventDispatcher, EventDispatcherBuilder, EventDispatcherController,
    EventHandler,
};
pub use envelope::{
    EventFrame, ExecutionEvent, ExecutionEventPayload, MarketEvent, MarketPayload, OrderBookLevel,
    OrderEvent, OrderEventPayload, RiskAction, RiskEvent, RiskEventPayload, SignalEvent,
    SignalEventPayload, StrategySignal,
};
pub use error::EventBusError;
pub use metadata::{EventKind, EventMetadata, EventSource, Priority};

/// Convenience prelude for consumers of the event bus.
pub mod prelude {
    pub use super::channel::{EventBus, EventBusBuilder, EventReceiver, EventSender, PublishMode};
    pub use super::dispatcher::{
        ClosureHandler, EventDispatcher, EventDispatcherBuilder, EventDispatcherController,
        EventHandler,
    };
    pub use super::envelope::{
        EventFrame, ExecutionEvent, ExecutionEventPayload, MarketEvent, MarketPayload, OrderEvent,
        OrderEventPayload, RiskAction, RiskEvent, RiskEventPayload, SignalEvent,
        SignalEventPayload, StrategySignal,
    };
    pub use super::error::EventBusError;
    pub use super::metadata::{EventKind, EventMetadata, EventSource, Priority};
}

#[cfg(feature = "core-integration")]
pub mod core_bridges;

#[cfg(feature = "exchange-integration")]
pub mod exchange_bridges;

#[cfg(test)]
mod tests;
