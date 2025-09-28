use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::sequence::next_sequence;

/// Enumerates the canonical kinds of events carried across the bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// Normalized market data (ticks, book deltas, candles).
    Market,
    /// Strategy intent emitted after evaluation.
    Signal,
    /// Orders flowing toward execution systems.
    Order,
    /// Executions and order state updates.
    Execution,
    /// Risk controls, halts, or portfolio advisories.
    Risk,
}

/// Event priority used to bias scheduling or backpressure decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// Monitoring or low-urgency telemetry.
    Low,
    /// Standard priority for most events.
    Normal,
    /// Elevated priority demanding faster handling.
    High,
    /// Critical actions such as kill-switch triggers.
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Identifies the producer module of an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    /// Logical module name (e.g. "binance_ws", "strategy.alpha", "risk.guard").
    pub module: String,
    /// Optional instance identifier (e.g. worker shard ID).
    pub instance: Option<String>,
}

impl EventSource {
    /// Creates a new event source with the supplied module name.
    pub fn new(module: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            instance: None,
        }
    }

    /// Attaches an instance identifier.
    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }
}

impl From<&str> for EventSource {
    fn from(value: &str) -> Self {
        EventSource::new(value)
    }
}

impl From<String> for EventSource {
    fn from(value: String) -> Self {
        EventSource::new(value)
    }
}

/// Metadata attached to every event envelope to enable tracing and diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Correlates events across the lifecycle (ingest → execution → settlement).
    pub correlation_id: Uuid,
    /// Unique identifier for this hop/span in the event lifecycle.
    pub span_id: Uuid,
    /// Optional parent span identifier.
    pub parent_span_id: Option<Uuid>,
    /// Monotonic sequence number generated at emission time.
    pub sequence: u64,
    /// Timestamp assigned at emission time (UTC).
    pub timestamp: DateTime<Utc>,
    /// Event priority for backpressure-aware scheduling.
    pub priority: Priority,
    /// Source module metadata.
    pub source: EventSource,
}

impl EventMetadata {
    /// Creates metadata for a root event produced by the given source.
    pub fn new(source: impl Into<EventSource>, priority: Priority) -> Self {
        Self::with_parent(source, priority, None)
    }

    /// Creates metadata that optionally links to a parent span.
    pub fn with_parent(
        source: impl Into<EventSource>,
        priority: Priority,
        parent_span_id: Option<Uuid>,
    ) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            span_id: Uuid::new_v4(),
            parent_span_id,
            sequence: next_sequence(),
            timestamp: Utc::now(),
            priority,
            source: source.into(),
        }
    }

    /// Creates metadata that shares an existing correlation identifier.
    pub fn with_correlation(
        correlation_id: Uuid,
        source: impl Into<EventSource>,
        priority: Priority,
    ) -> Self {
        Self {
            correlation_id,
            span_id: Uuid::new_v4(),
            parent_span_id: None,
            sequence: next_sequence(),
            timestamp: Utc::now(),
            priority,
            source: source.into(),
        }
    }

    /// Generates a child span inheriting the correlation and current span.
    pub fn child(&self, source: impl Into<EventSource>, priority: Priority) -> Self {
        Self {
            correlation_id: self.correlation_id,
            span_id: Uuid::new_v4(),
            parent_span_id: Some(self.span_id),
            sequence: next_sequence(),
            timestamp: Utc::now(),
            priority,
            source: source.into(),
        }
    }
}
