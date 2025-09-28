#![allow(missing_docs)]

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Notify;

use crate::channel::{EventBus, EventReceiver};
#[cfg(feature = "exchange-integration")]
use crate::envelope::MarketEvent;
use crate::envelope::RiskEvent;
#[cfg(feature = "core-integration")]
use crate::envelope::{ExecutionEvent, OrderEvent, SignalEvent};
use crate::error::EventBusError;

/// Handler trait invoked by the dispatcher when a new event arrives.
#[async_trait]
pub trait EventHandler<T>: Send + Sync + 'static
where
    T: Send + Sync + 'static,
{
    /// Processes an event and optionally emits follow-up events.
    async fn handle(&self, event: T) -> Result<(), EventBusError>;
}

#[derive(Default)]
struct Handlers {
    #[cfg(feature = "exchange-integration")]
    market: Option<Arc<dyn EventHandler<MarketEvent>>>,
    #[cfg(feature = "core-integration")]
    signal: Option<Arc<dyn EventHandler<SignalEvent>>>,
    #[cfg(feature = "core-integration")]
    order: Option<Arc<dyn EventHandler<OrderEvent>>>,
    #[cfg(feature = "core-integration")]
    execution: Option<Arc<dyn EventHandler<ExecutionEvent>>>,
    risk: Option<Arc<dyn EventHandler<RiskEvent>>>,
}

impl fmt::Debug for Handlers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handlers").finish_non_exhaustive()
    }
}

/// Builder for wiring handlers into the dispatcher.
#[derive(Debug)]
pub struct EventDispatcherBuilder {
    #[cfg(feature = "exchange-integration")]
    market_rx: EventReceiver<MarketEvent>,
    #[cfg(feature = "core-integration")]
    signal_rx: EventReceiver<SignalEvent>,
    #[cfg(feature = "core-integration")]
    order_rx: EventReceiver<OrderEvent>,
    #[cfg(feature = "core-integration")]
    execution_rx: EventReceiver<ExecutionEvent>,
    risk_rx: EventReceiver<RiskEvent>,
    handlers: Handlers,
}

impl EventDispatcherBuilder {
    pub fn new(bus: &EventBus) -> Self {
        Self {
            #[cfg(feature = "exchange-integration")]
            market_rx: bus.market_receiver(),
            #[cfg(feature = "core-integration")]
            signal_rx: bus.signal_receiver(),
            #[cfg(feature = "core-integration")]
            order_rx: bus.order_receiver(),
            #[cfg(feature = "core-integration")]
            execution_rx: bus.execution_receiver(),
            risk_rx: bus.risk_receiver(),
            handlers: Handlers::default(),
        }
    }

    #[cfg(feature = "exchange-integration")]
    /// Registers a handler for market events.
    pub fn on_market(mut self, handler: Arc<dyn EventHandler<MarketEvent>>) -> Self {
        self.handlers.market = Some(handler);
        self
    }

    #[cfg(feature = "core-integration")]
    /// Registers a handler for signal events.
    pub fn on_signal(mut self, handler: Arc<dyn EventHandler<SignalEvent>>) -> Self {
        self.handlers.signal = Some(handler);
        self
    }

    #[cfg(feature = "core-integration")]
    /// Registers a handler for order events.
    pub fn on_order(mut self, handler: Arc<dyn EventHandler<OrderEvent>>) -> Self {
        self.handlers.order = Some(handler);
        self
    }

    #[cfg(feature = "core-integration")]
    /// Registers a handler for execution events.
    pub fn on_execution(mut self, handler: Arc<dyn EventHandler<ExecutionEvent>>) -> Self {
        self.handlers.execution = Some(handler);
        self
    }

    /// Registers a handler for risk events.
    pub fn on_risk(mut self, handler: Arc<dyn EventHandler<RiskEvent>>) -> Self {
        self.handlers.risk = Some(handler);
        self
    }

    /// Builds the dispatcher using the configured handlers.
    pub fn build(self) -> EventDispatcher {
        EventDispatcher::from_builder(self)
    }
}

/// Multiplexes events from the bus using `tokio::select!`, delegating to registered handlers.
#[derive(Debug)]
pub struct EventDispatcher {
    #[cfg(feature = "exchange-integration")]
    market_rx: EventReceiver<MarketEvent>,
    #[cfg(feature = "core-integration")]
    signal_rx: EventReceiver<SignalEvent>,
    #[cfg(feature = "core-integration")]
    order_rx: EventReceiver<OrderEvent>,
    #[cfg(feature = "core-integration")]
    execution_rx: EventReceiver<ExecutionEvent>,
    risk_rx: EventReceiver<RiskEvent>,

    handlers: Handlers,
    shutdown_flag: Arc<AtomicBool>,
    shutdown_notify: Arc<Notify>,
}

impl EventDispatcher {
    fn from_builder(builder: EventDispatcherBuilder) -> Self {
        #[cfg(feature = "exchange-integration")]
        let market_rx = builder.market_rx.clone();
        #[cfg(feature = "core-integration")]
        let signal_rx = builder.signal_rx.clone();
        #[cfg(feature = "core-integration")]
        let order_rx = builder.order_rx.clone();
        #[cfg(feature = "core-integration")]
        let execution_rx = builder.execution_rx.clone();
        let risk_rx = builder.risk_rx.clone();
        let handlers = builder.handlers;

        Self {
            #[cfg(feature = "exchange-integration")]
            market_rx,
            #[cfg(feature = "core-integration")]
            signal_rx,
            #[cfg(feature = "core-integration")]
            order_rx,
            #[cfg(feature = "core-integration")]
            execution_rx,
            risk_rx,
            handlers,
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            shutdown_notify: Arc::new(Notify::new()),
        }
    }

    /// Requests dispatcher shutdown and wakes the event loop.
    pub fn shutdown(&self) {
        if !self.shutdown_flag.swap(true, Ordering::SeqCst) {
            self.shutdown_notify.notify_waiters();
        }
    }

    /// Returns whether the dispatcher has been asked to stop.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }

    /// Provides a controller for external shutdown orchestration.
    pub fn controller(&self) -> EventDispatcherController {
        EventDispatcherController {
            flag: Arc::clone(&self.shutdown_flag),
            notify: Arc::clone(&self.shutdown_notify),
        }
    }

    /// Runs the event loop until shutdown is requested.
    pub async fn run(self) -> Result<(), EventBusError> {
        self.run_impl().await
    }

    #[cfg(all(feature = "exchange-integration", feature = "core-integration"))]
    #[allow(unused_mut)]
    async fn run_impl(mut self) -> Result<(), EventBusError> {
        loop {
            tokio::select! {
                _ = self.shutdown_notify.notified(), if self.is_shutdown() => {
                    break;
                }
                event = self.market_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.market {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                event = self.signal_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.signal {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                event = self.order_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.order {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                event = self.execution_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.execution {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                event = self.risk_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.risk {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(all(feature = "exchange-integration", not(feature = "core-integration")))]
    #[allow(unused_mut)]
    async fn run_impl(mut self) -> Result<(), EventBusError> {
        loop {
            tokio::select! {
                _ = self.shutdown_notify.notified(), if self.is_shutdown() => {
                    break;
                }
                event = self.market_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.market {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                event = self.risk_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.risk {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(all(not(feature = "exchange-integration"), feature = "core-integration"))]
    #[allow(unused_mut)]
    async fn run_impl(mut self) -> Result<(), EventBusError> {
        loop {
            tokio::select! {
                _ = self.shutdown_notify.notified(), if self.is_shutdown() => {
                    break;
                }
                event = self.signal_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.signal {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                event = self.order_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.order {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                event = self.execution_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.execution {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
                event = self.risk_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.risk {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(all(
        not(feature = "exchange-integration"),
        not(feature = "core-integration")
    ))]
    #[allow(unused_mut)]
    async fn run_impl(mut self) -> Result<(), EventBusError> {
        loop {
            tokio::select! {
                _ = self.shutdown_notify.notified(), if self.is_shutdown() => {
                    break;
                }
                event = self.risk_rx.recv_async() => {
                    match event {
                        Ok(event) => {
                            if let Some(handler) = &self.handlers.risk {
                                handler.handle(event).await?;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }

        Ok(())
    }
}

/// Handle used to coordinate dispatcher shutdown from outside the run loop.
#[derive(Clone)]
pub struct EventDispatcherController {
    flag: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl fmt::Debug for EventDispatcherController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventDispatcherController")
            .finish_non_exhaustive()
    }
}

impl EventDispatcherController {
    /// Requests shutdown of the associated dispatcher.
    pub fn shutdown(&self) {
        if !self.flag.swap(true, Ordering::SeqCst) {
            self.notify.notify_waiters();
        }
    }

    /// Whether shutdown has been requested.
    pub fn is_shutdown(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

/// Helper for building ad-hoc async handlers from closures.
pub struct ClosureHandler<T, F> {
    inner: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T, F> fmt::Debug for ClosureHandler<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClosureHandler").finish_non_exhaustive()
    }
}

impl<T, F> ClosureHandler<T, F> {
    pub fn new(inner: F) -> Self {
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T, F, Fut> EventHandler<T> for ClosureHandler<T, F>
where
    T: Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(T) -> Fut,
    Fut: Send + 'static + std::future::Future<Output = Result<(), EventBusError>>,
{
    async fn handle(&self, event: T) -> Result<(), EventBusError> {
        (self.inner)(event).await
    }
}
