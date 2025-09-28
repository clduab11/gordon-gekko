use std::sync::atomic::{AtomicU64, Ordering};

use event_bus::{EventBusError, EventMetadata, EventSender, EventSource, PublishMode, SignalEvent};
use tracing::trace;
use uuid::Uuid;

use crate::traits::{StrategyDecision, StrategyMetrics};

static SIGNAL_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// Publishes strategy decisions onto the canonical event bus.
pub struct StrategyEventBridge {
    strategy_id: Uuid,
    strategy_name: String,
    signal_sender: EventSender<SignalEvent>,
}

impl StrategyEventBridge {
    pub fn new(
        strategy_id: Uuid,
        strategy_name: impl Into<String>,
        signal_sender: EventSender<SignalEvent>,
    ) -> Self {
        Self {
            strategy_id,
            strategy_name: strategy_name.into(),
            signal_sender,
        }
    }

    pub fn publish(
        &self,
        decision: &StrategyDecision,
        metrics: &StrategyMetrics,
    ) -> Result<(), EventBusError> {
        for payload in &decision.signals {
            let mut metadata = EventMetadata::new(
                EventSource::new(format!("strategy.{}", self.strategy_name)),
                payload.priority,
            );
            metadata.sequence = SIGNAL_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let event = SignalEvent::new(metadata, payload.clone());
            self.signal_sender.publish(event, PublishMode::Blocking)?;
        }

        trace!(
            strategy = %self.strategy_name,
            strategy_id = %self.strategy_id,
            signals = decision.signals.len(),
            latency_ms = metrics.evaluation_latency.as_secs_f64() * 1_000.0,
            "published strategy decision"
        );

        Ok(())
    }
}
