use crossbeam_channel::Receiver;
use event_bus::{EventBusError, MarketEvent, PublishMode};
use tracing::trace;

/// Fan-out component that publishes normalized events to the main event bus.
pub struct Distributor {
    market_sender: event_bus::EventSender<MarketEvent>,
    publish_mode: PublishMode,
}

impl Distributor {
    pub fn new(market_sender: event_bus::EventSender<MarketEvent>) -> Self {
        Self {
            market_sender,
            publish_mode: PublishMode::Blocking,
        }
    }

    pub fn with_mode(mut self, mode: PublishMode) -> Self {
        self.publish_mode = mode;
        self
    }

    pub fn dispatch(&self, event: MarketEvent) -> Result<(), EventBusError> {
        self.market_sender.publish(event, self.publish_mode)
    }

    /// Continuously drains a normalized event channel until it closes.
    pub fn drain(&self, receiver: Receiver<MarketEvent>) {
        for event in receiver.iter() {
            if let Err(err) = self.dispatch(event) {
                trace!(%err, "market dispatch failed; stopping distributor loop");
                break;
            }
        }
    }
}
