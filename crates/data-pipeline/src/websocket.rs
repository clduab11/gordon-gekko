//! Shared WebSocket client utilities for the Ninja Gekko data pipeline.
//!
//! This module provides resilient connection management for public market data
//! feeds. It is intentionally generic so exchange-specific adapters can build
//! on top of it without re-implementing reconnection, heartbeat, or
//! instrumentation logic. The implementation favours clear control-flow over
//! cleverness to make latency characteristics predictable and easy to audit.

use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rand::{rngs::OsRng, RngCore};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Instant};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use url::Url;

/// Reconnection backoff policy with optional jitter.
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// Starting delay before a reconnect attempt.
    pub initial: Duration,
    /// Maximum backoff delay.
    pub max: Duration,
    /// Exponential multiplier applied per attempt (must be >= 1.0).
    pub multiplier: f64,
    /// Optional additive jitter applied to each computed backoff.
    pub jitter: Option<Duration>,
}

impl BackoffConfig {
    /// Creates a sane default tuned for public market data feeds.
    pub fn default_streaming() -> Self {
        Self {
            initial: Duration::from_millis(250),
            max: Duration::from_secs(30),
            multiplier: 1.8,
            jitter: Some(Duration::from_millis(120)),
        }
    }

    fn compute_delay(&self, attempt: u32) -> Duration {
        let exp = (self.multiplier).powi(attempt.saturating_sub(1) as i32);
        let mut delay = self.initial.mul_f64(exp);
        if delay > self.max {
            delay = self.max;
        }
        if let Some(jitter) = self.jitter {
            // bounded jitter based on secure RNG to avoid synchronised reconnect storms.
            let mut buf = [0u8; 8];
            if OsRng.try_fill_bytes(&mut buf).is_ok() {
                let noise = u64::from_le_bytes(buf) % (jitter.as_millis().max(1) as u64);
                delay = delay.saturating_add(Duration::from_millis(noise));
            }
        }
        delay
    }
}

/// Heartbeat behaviour emitted by some exchanges.
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// Interval at which to emit ping frames if the upstream remains idle.
    pub interval: Duration,
    /// Optional payload to accompany ping frames (for exchanges that require it).
    pub ping_payload: Option<Vec<u8>>,
}

/// Event surfaced to pipeline components.
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    /// Text payload (typically JSON).
    Text(String),
    /// Binary payload.
    Binary(Vec<u8>),
    /// The upstream requested we keep-alive the connection.
    Ping(Vec<u8>),
    /// Pong payload received from upstream.
    Pong(Vec<u8>),
    /// Upstream closed the connection. The bool captures whether the closure was graceful.
    Closed(bool),
}

/// Configuration required to spin up a resilient WebSocket client task.
#[derive(Clone)]
pub struct WebSocketConfig {
    /// Human readable identifier used for tracing.
    pub name: Cow<'static, str>,
    /// Endpoint URL to connect to.
    pub endpoint: Url,
    /// Closure invoked on every successful connection to produce subscription frames.
    pub on_connect: Arc<dyn Fn() -> Vec<Message> + Send + Sync>,
    /// Optional heartbeat strategy.
    pub heartbeat: Option<HeartbeatConfig>,
    /// Backoff policy to apply between reconnect attempts.
    pub backoff: BackoffConfig,
    /// Idle read timeout; if exceeded the connection is considered stalled.
    pub read_timeout: Duration,
}

impl WebSocketConfig {
    pub fn builder(url: Url) -> WebSocketConfigBuilder {
        WebSocketConfigBuilder {
            name: Cow::Borrowed("ws"),
            endpoint: url,
            on_connect: Arc::new(|| Vec::new()),
            heartbeat: None,
            backoff: BackoffConfig::default_streaming(),
            read_timeout: Duration::from_secs(15),
        }
    }
}

/// Builder that helps construct a [`WebSocketConfig`].
pub struct WebSocketConfigBuilder {
    name: Cow<'static, str>,
    endpoint: Url,
    on_connect: Arc<dyn Fn() -> Vec<Message> + Send + Sync>,
    heartbeat: Option<HeartbeatConfig>,
    backoff: BackoffConfig,
    read_timeout: Duration,
}

impl WebSocketConfigBuilder {
    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = name.into();
        self
    }

    pub fn on_connect<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> Vec<Message> + Send + Sync + 'static,
    {
        self.on_connect = Arc::new(factory);
        self
    }

    pub fn heartbeat(mut self, heartbeat: HeartbeatConfig) -> Self {
        self.heartbeat = Some(heartbeat);
        self
    }

    pub fn backoff(mut self, backoff: BackoffConfig) -> Self {
        self.backoff = backoff;
        self
    }

    pub fn read_timeout(mut self, read_timeout: Duration) -> Self {
        self.read_timeout = read_timeout;
        self
    }

    pub fn build(self) -> WebSocketConfig {
        WebSocketConfig {
            name: self.name,
            endpoint: self.endpoint,
            on_connect: self.on_connect,
            heartbeat: self.heartbeat,
            backoff: self.backoff,
            read_timeout: self.read_timeout,
        }
    }
}

/// Spawns a resilient WebSocket worker returning a receiver for upstream events.
pub fn spawn_stream(
    config: WebSocketConfig,
) -> (JoinHandle<()>, UnboundedReceiver<WebSocketEvent>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let handle = tokio::spawn(run_stream(config, tx));
    (handle, rx)
}

async fn run_stream(config: WebSocketConfig, sender: UnboundedSender<WebSocketEvent>) {
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        debug!(name = %config.name, url = %config.endpoint, attempt, "attempting websocket connection");
        match connect_async(config.endpoint.clone()).await {
            Ok((mut ws_stream, _)) => {
                info!(name = %config.name, "websocket connection established");
                attempt = 0; // reset backoff after a successful connection

                // Send subscription frames
                for message in (config.on_connect)() {
                    if let Err(err) = ws_stream.send(message).await {
                        warn!(name = %config.name, %err, "failed to transmit subscription frame");
                    }
                }

                let mut last_frame = Instant::now();
                let heartbeat = config.heartbeat.clone();

                loop {
                    tokio::select! {
                        biased;
                        _ = async {
                            if let Some(hb) = &heartbeat {
                                sleep(hb.interval).await;
                            }
                        }, if heartbeat.is_some() => {
                            if last_frame.elapsed() >= heartbeat.as_ref().unwrap().interval {
                                let payload = heartbeat.as_ref().and_then(|hb| hb.ping_payload.clone()).unwrap_or_default();
                                if let Err(err) = ws_stream.send(Message::Ping(payload.clone())).await {
                                    warn!(name = %config.name, %err, "failed to send ping frame");
                                    break;
                                }
                                let _ = sender.send(WebSocketEvent::Ping(payload));
                                last_frame = Instant::now();
                            }
                        }
                        msg = ws_stream.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    last_frame = Instant::now();
                                    let _ = sender.send(WebSocketEvent::Text(text));
                                }
                                Some(Ok(Message::Binary(bin))) => {
                                    last_frame = Instant::now();
                                    let _ = sender.send(WebSocketEvent::Binary(bin));
                                }
                                Some(Ok(Message::Ping(payload))) => {
                                    last_frame = Instant::now();
                                    let _ = ws_stream.send(Message::Pong(payload.clone())).await;
                                    let _ = sender.send(WebSocketEvent::Ping(payload));
                                }
                                Some(Ok(Message::Pong(payload))) => {
                                    last_frame = Instant::now();
                                    let _ = sender.send(WebSocketEvent::Pong(payload));
                                }
                                Some(Ok(Message::Close(_))) => {
                                    let _ = sender.send(WebSocketEvent::Closed(true));
                                    info!(name = %config.name, "websocket closed by upstream");
                                    break;
                                }
                                Some(Ok(_)) => {
                                    last_frame = Instant::now();
                                }
                                Some(Err(err)) => {
                                    warn!(name = %config.name, %err, "websocket error");
                                    break;
                                }
                                None => {
                                    debug!(name = %config.name, "websocket stream ended");
                                    break;
                                }
                            }

                            if last_frame.elapsed() > config.read_timeout {
                                warn!(name = %config.name, "websocket stalled; reconnecting");
                                break;
                            }
                        }
                    }
                }
            }
            Err(err) => {
                error!(name = %config.name, %err, "websocket connection attempt failed");
            }
        }

        if sender.is_closed() {
            debug!(name = %config.name, "caller dropped receiver; stopping websocket task");
            return;
        }

        // compute backoff and wait
        let delay = config.backoff.compute_delay(attempt);
        debug!(name = %config.name, ?delay, "sleeping before reconnect attempt");
        sleep(delay).await;
    }
}
