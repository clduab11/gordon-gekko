#![allow(missing_docs)]

use std::time::Duration;

use crossbeam_channel::{
    RecvError, RecvTimeoutError, SendError, SendTimeoutError, TryRecvError, TrySendError,
};
use thiserror::Error;

use crate::metadata::EventKind;

/// Errors emitted by the event bus layers.
#[derive(Error, Debug)]
pub enum EventBusError {
    /// Channel send failed because receivers dropped or channel closed.
    #[error("channel send failure: {0}")]
    ChannelSend(String),
    /// Channel receive failed because the sender dropped.
    #[error("channel receive failure: {0}")]
    ChannelReceive(String),
    /// Operation exceeded the configured timeout.
    #[error("operation timed out after {0:?}")]
    Timeout(Duration),
    /// Serialization failure while encoding payload.
    #[error("serialization failure: {0}")]
    Serialization(String),
    /// Deserialization failure while decoding payload.
    #[error("deserialization failure: {0}")]
    Deserialization(String),
    /// Unexpected event kind encountered during decoding.
    #[error("event kind mismatch: expected {expected:?}, got {actual:?}")]
    KindMismatch {
        expected: EventKind,
        actual: EventKind,
    },
    /// Join error while awaiting blocking task completion.
    #[error("join error: {0}")]
    Join(String),
    /// Upstream module failure bubbled through the bus.
    #[error("upstream module failure: {0}")]
    Upstream(String),
}

impl EventBusError {
    pub(crate) fn serialization(err: impl std::fmt::Display) -> Self {
        EventBusError::Serialization(err.to_string())
    }

    pub(crate) fn deserialization(err: impl std::fmt::Display) -> Self {
        EventBusError::Deserialization(err.to_string())
    }

    pub(crate) fn kind_mismatch(expected: EventKind, actual: EventKind) -> Self {
        EventBusError::KindMismatch { expected, actual }
    }

    pub(crate) fn from_send_error<T>(error: SendError<T>) -> Self {
        EventBusError::ChannelSend(error.to_string())
    }

    pub(crate) fn from_try_send_error<T>(error: TrySendError<T>) -> Self {
        match error {
            TrySendError::Full(_) => EventBusError::ChannelSend("channel full".into()),
            TrySendError::Disconnected(_) => {
                EventBusError::ChannelSend("channel disconnected".into())
            }
        }
    }

    pub(crate) fn from_recv_error(error: RecvError) -> Self {
        EventBusError::ChannelReceive(error.to_string())
    }

    pub(crate) fn from_recv_timeout(error: RecvTimeoutError) -> Self {
        match error {
            RecvTimeoutError::Timeout => EventBusError::Timeout(Duration::ZERO),
            RecvTimeoutError::Disconnected => {
                EventBusError::ChannelReceive("channel disconnected".into())
            }
        }
    }

    pub(crate) fn from_send_timeout_error<T>(
        error: SendTimeoutError<T>,
        timeout: Duration,
    ) -> Self {
        match error {
            SendTimeoutError::Timeout(_) => EventBusError::Timeout(timeout),
            SendTimeoutError::Disconnected(_) => {
                EventBusError::ChannelSend("channel disconnected".into())
            }
        }
    }

    pub(crate) fn from_try_recv_error(error: TryRecvError) -> Self {
        match error {
            TryRecvError::Empty => EventBusError::ChannelReceive("channel empty".into()),
            TryRecvError::Disconnected => {
                EventBusError::ChannelReceive("channel disconnected".into())
            }
        }
    }

    pub(crate) fn upstream(err: impl std::fmt::Display) -> Self {
        EventBusError::Upstream(err.to_string())
    }
}
