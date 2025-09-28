#![allow(missing_docs)]

//! Bridges wiring core Ninja Gekko modules onto the event bus without altering
//! their existing public APIs.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::sync::RwLock;
use tracing::debug;

use ninja_gekko_core::order_manager::OrderManager;
use ninja_gekko_core::types::{Execution, Order, OrderSide, OrderType, Portfolio};

use crate::channel::{EventSender, PublishMode};
use crate::dispatcher::EventHandler;
use crate::envelope::{ExecutionEvent, OrderEvent, RiskEvent, SignalEvent};
use crate::error::EventBusError;
use crate::metadata::Priority;

#[cfg(feature = "exchange-integration")]
use exchange_connectors::{
    ExchangeConnector, ExchangeId, ExchangeOrder, OrderSide as ExOrderSide,
    OrderType as ExOrderType,
};

/// Transforms strategy signals into validated orders via the existing OrderManager.
pub struct SignalToOrderBridge {
    manager: Arc<OrderManager>,
    order_sender: EventSender<OrderEvent>,
    mode: PublishMode,
}

impl SignalToOrderBridge {
    /// Creates a new bridge that forwards validated orders onto the event bus.
    pub fn new(
        manager: Arc<OrderManager>,
        order_sender: EventSender<OrderEvent>,
        mode: PublishMode,
    ) -> Self {
        Self {
            manager,
            order_sender,
            mode,
        }
    }
}

#[async_trait]
impl EventHandler<SignalEvent> for SignalToOrderBridge {
    async fn handle(&self, event: SignalEvent) -> Result<(), EventBusError> {
        let payload = event.payload_arc();
        let signal = &payload.signal;

        let order_id = self
            .manager
            .submit_order(
                signal.symbol.clone(),
                signal.order_type,
                signal.side,
                signal.quantity,
                signal.limit_price,
                payload.account_id.clone(),
            )
            .await
            .map_err(EventBusError::upstream)?;

        let order = self
            .manager
            .get_order(order_id)
            .await
            .map_err(EventBusError::upstream)?;

        let metadata = event
            .metadata()
            .child("event_bus.signal_to_order", payload.priority);
        let order_event = OrderEvent::new(metadata, order);
        self.order_sender.publish(order_event, self.mode)?;
        Ok(())
    }
}

/// Maintains portfolio state by applying execution events.
pub struct PortfolioUpdateBridge {
    portfolio: Arc<RwLock<Portfolio>>,
}

impl PortfolioUpdateBridge {
    /// Creates a new portfolio updater backed by the provided portfolio reference.
    pub fn new(portfolio: Arc<RwLock<Portfolio>>) -> Self {
        Self { portfolio }
    }
}

#[async_trait]
impl EventHandler<ExecutionEvent> for PortfolioUpdateBridge {
    async fn handle(&self, event: ExecutionEvent) -> Result<(), EventBusError> {
        let execution = event.execution().clone();
        let mut portfolio = self.portfolio.write().await;
        portfolio.update_from_execution(&execution);
        Ok(())
    }
}

/// Routes order events to exchange connectors and publishes execution updates.
#[cfg(feature = "exchange-integration")]
pub struct OrderExecutionBridge {
    connector: Arc<dyn ExchangeConnector>,
    execution_sender: EventSender<ExecutionEvent>,
    mode: PublishMode,
}

#[cfg(feature = "exchange-integration")]
impl OrderExecutionBridge {
    /// Creates a new execution bridge that forwards exchange fills onto the bus.
    pub fn new(
        connector: Arc<dyn ExchangeConnector>,
        execution_sender: EventSender<ExecutionEvent>,
        mode: PublishMode,
    ) -> Self {
        Self {
            connector,
            execution_sender,
            mode,
        }
    }
}

#[cfg(feature = "exchange-integration")]
#[async_trait]
impl EventHandler<OrderEvent> for OrderExecutionBridge {
    async fn handle(&self, event: OrderEvent) -> Result<(), EventBusError> {
        let order = event.order().clone();
        let exchange_order = self
            .connector
            .place_order(
                &order.symbol,
                map_side(order.side),
                map_order_type(order.order_type),
                order.quantity,
                order.price,
            )
            .await
            .map_err(EventBusError::upstream)?;

        let execution = to_execution(&order, exchange_order, self.connector.exchange_id());
        let metadata = event
            .metadata()
            .child("event_bus.order_execution_bridge", Priority::High);
        let exec_event = ExecutionEvent::new(metadata, execution);
        self.execution_sender.publish(exec_event, self.mode)?;
        Ok(())
    }
}

#[cfg(feature = "exchange-integration")]
fn map_side(side: OrderSide) -> ExOrderSide {
    match side {
        OrderSide::Buy => ExOrderSide::Buy,
        OrderSide::Sell => ExOrderSide::Sell,
    }
}

#[cfg(feature = "exchange-integration")]
fn map_order_type(order_type: OrderType) -> ExOrderType {
    match order_type {
        OrderType::Market => ExOrderType::Market,
        OrderType::Limit => ExOrderType::Limit,
        OrderType::Stop => ExOrderType::Stop,
        OrderType::StopLimit => ExOrderType::StopLimit,
        OrderType::Iceberg => ExOrderType::Limit,
        OrderType::TWAP => ExOrderType::Limit,
        OrderType::VWAP => ExOrderType::Limit,
    }
}

#[cfg(feature = "exchange-integration")]
fn to_execution(
    order: &Order,
    exchange_order: ExchangeOrder,
    exchange_id: ExchangeId,
) -> Execution {
    let price = exchange_order
        .price
        .or_else(|| order.price)
        .or_else(|| exchange_order.fills.first().map(|fill| fill.price))
        .unwrap_or(Decimal::ZERO);
    let fees = exchange_order
        .fills
        .iter()
        .fold(Decimal::ZERO, |acc, fill| acc + fill.fee);

    Execution::new(
        order.id,
        order.symbol.clone(),
        order.side,
        order.quantity,
        price,
        format!("{:?}", exchange_id),
        fees,
    )
}

/// Simple handler that logs and forwards risk events. Provided as a convenience
/// for modules that want to react to halts without bespoke wiring.
pub struct RiskLoggingHandler {
    log_target: &'static str,
}

impl fmt::Debug for SignalToOrderBridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignalToOrderBridge")
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for PortfolioUpdateBridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PortfolioUpdateBridge")
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "exchange-integration")]
impl fmt::Debug for OrderExecutionBridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrderExecutionBridge")
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for RiskLoggingHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RiskLoggingHandler")
            .field("log_target", &self.log_target)
            .finish()
    }
}

impl RiskLoggingHandler {
    /// Creates a logging handler for risk events with the supplied tracing target.
    pub fn new(log_target: &'static str) -> Self {
        Self { log_target }
    }
}

#[async_trait]
impl EventHandler<RiskEvent> for RiskLoggingHandler {
    async fn handle(&self, event: RiskEvent) -> Result<(), EventBusError> {
        debug!(
            ?event,
            log_target = self.log_target,
            "risk event propagated"
        );
        Ok(())
    }
}
