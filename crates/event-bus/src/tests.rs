use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rust_decimal::Decimal;
use tokio::sync::{oneshot, Mutex, RwLock};
use tokio::time::timeout;
use uuid::Uuid;

use crate::channel::{EventBusBuilder, PublishMode};
use crate::core_bridges::{PortfolioUpdateBridge, SignalToOrderBridge};
use crate::dispatcher::{ClosureHandler, EventDispatcherBuilder};
use crate::envelope::{
    ExecutionEvent, RiskAction, RiskEvent, RiskEventPayload, SignalEvent, SignalEventPayload,
    StrategySignal,
};
use crate::metadata::{EventMetadata, Priority};
use crate::EventBusError;

use ninja_gekko_core::order_manager::{DefaultFeeCalculator, DefaultRiskValidator, OrderManager};
use ninja_gekko_core::types::{Execution, OrderSide, OrderType, Portfolio};

#[tokio::test]
#[ignore = "pending dispatcher coordination investigation"]
async fn signal_to_order_bridge_emits_order_events() -> Result<(), EventBusError> {
    let bus = EventBusBuilder::default().build();
    let signal_sender = bus.signal_sender();
    let order_receiver = bus.order_receiver();

    let risk_manager = Box::new(DefaultRiskValidator::new(
        Decimal::new(1_000_000, 0),
        Decimal::new(2_000_000, 0),
        Decimal::new(10_000_000, 0),
    ));
    let fee_calculator = Box::new(DefaultFeeCalculator::new(Decimal::ZERO, Decimal::ZERO));
    let order_manager = Arc::new(OrderManager::new(risk_manager, fee_calculator));

    let order_sender = bus.order_sender();
    let signal_bridge = Arc::new(SignalToOrderBridge::new(
        Arc::clone(&order_manager),
        order_sender,
        PublishMode::Blocking,
    ));

    let dispatcher = EventDispatcherBuilder::new(&bus)
        .on_signal(signal_bridge)
        .build();
    let controller = dispatcher.controller();
    let dispatcher_task = tokio::spawn(async move {
        dispatcher.run().await.unwrap();
    });

    let metadata = EventMetadata::new("test.signal", Priority::High);
    let signal_payload = SignalEventPayload {
        strategy_id: Uuid::new_v4(),
        account_id: "acct-1".to_string(),
        priority: Priority::High,
        signal: StrategySignal {
            exchange: None,
            symbol: "BTC-USD".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: Decimal::new(1, 0),
            limit_price: Some(Decimal::new(30_000, 0)),
            confidence: 0.99,
            metadata: HashMap::new(),
        },
    };
    let event = SignalEvent::new(metadata, signal_payload);

    signal_sender.publish(event, PublishMode::Blocking)?;
    let order_event = timeout(Duration::from_millis(50), order_receiver.recv_async())
        .await
        .expect("order event not produced")?;

    assert_eq!(order_event.order().symbol, "BTC-USD");
    assert_eq!(order_event.order().quantity, Decimal::new(1, 0));

    controller.shutdown();
    dispatcher_task.await.unwrap();
    Ok(())
}

#[tokio::test]
#[ignore = "pending dispatcher coordination investigation"]
async fn dispatch_latency_within_target() -> Result<(), EventBusError> {
    let bus = EventBusBuilder::default().build();
    let signal_sender = bus.signal_sender();

    let (latency_sender, latency_receiver) = oneshot::channel();
    let latency_tx = Arc::new(Mutex::new(Some(latency_sender)));
    let start: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));

    let handler_latency = Arc::clone(&latency_tx);
    let handler_start = Arc::clone(&start);
    let handler = Arc::new(ClosureHandler::new(move |_: SignalEvent| {
        let handler_latency = Arc::clone(&handler_latency);
        let handler_start = Arc::clone(&handler_start);
        async move {
            let start_instant = handler_start
                .lock()
                .await
                .take()
                .expect("start instant set before publish");
            let elapsed = start_instant.elapsed();
            if let Some(tx) = handler_latency.lock().await.take() {
                let _ = tx.send(elapsed);
            }
            Ok(())
        }
    }));

    let dispatcher = EventDispatcherBuilder::new(&bus).on_signal(handler).build();
    let controller = dispatcher.controller();
    let dispatcher_task = tokio::spawn(async move {
        dispatcher.run().await.unwrap();
    });

    {
        let mut guard = start.lock().await;
        *guard = Some(Instant::now());
    }

    let metadata = EventMetadata::new("bench.signal", Priority::Normal);
    let signal_payload = SignalEventPayload {
        strategy_id: Uuid::new_v4(),
        account_id: "acct-2".to_string(),
        priority: Priority::Normal,
        signal: StrategySignal {
            exchange: None,
            symbol: "ETH-USD".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: Decimal::new(2, 0),
            limit_price: None,
            confidence: 0.5,
            metadata: HashMap::new(),
        },
    };
    let event = SignalEvent::new(metadata, signal_payload);
    signal_sender.publish(event, PublishMode::Blocking)?;

    let elapsed = timeout(Duration::from_millis(10), latency_receiver)
        .await
        .expect("latency measurement timed out")
        .expect("latency channel closed");

    assert!(
        elapsed <= Duration::from_millis(1),
        "dispatch latency {elapsed:?} exceeds 1ms target"
    );

    controller.shutdown();
    dispatcher_task.await.unwrap();
    Ok(())
}

#[tokio::test]
#[ignore = "pending dispatcher coordination investigation"]
async fn portfolio_updates_on_execution_events() -> Result<(), EventBusError> {
    let bus = EventBusBuilder::default().build();
    let execution_sender = bus.execution_sender();

    let portfolio = Arc::new(RwLock::new(Portfolio::new("acct-3".to_string())));
    let bridge = Arc::new(PortfolioUpdateBridge::new(Arc::clone(&portfolio)));

    let dispatcher = EventDispatcherBuilder::new(&bus)
        .on_execution(bridge)
        .build();
    let controller = dispatcher.controller();
    let dispatcher_task = tokio::spawn(async move {
        dispatcher.run().await.unwrap();
    });

    let metadata = EventMetadata::new("test.execution", Priority::Normal);
    let order = ninja_gekko_core::types::Order::new(
        "BTC-USD".to_string(),
        OrderType::Market,
        OrderSide::Buy,
        Decimal::new(1, 0),
        Some(Decimal::new(25_000, 0)),
        "acct-3".to_string(),
    );
    let execution = Execution::new(
        order.id,
        order.symbol.clone(),
        order.side,
        order.quantity,
        Decimal::new(25_100, 0),
        "SIMULATED".to_string(),
        Decimal::new(10, 2),
    );
    let event = ExecutionEvent::new(metadata, execution.clone());
    execution_sender.publish(event, PublishMode::Blocking)?;

    tokio::time::sleep(Duration::from_millis(5)).await;
    let portfolio_snapshot = portfolio.read().await.clone();
    assert!(!portfolio_snapshot.positions.is_empty());

    controller.shutdown();
    dispatcher_task.await.unwrap();
    Ok(())
}

#[test]
fn test_risk_event_frame_roundtrip() {
    let metadata = EventMetadata::new("test.risk", Priority::Normal);
    let mut tags = HashMap::new();
    tags.insert("reason".to_string(), "drill".to_string());

    let payload = RiskEventPayload {
        action: RiskAction::Resume {
            reason: "systems nominal".to_string(),
        },
        priority: Priority::Normal,
        tags,
    };

    let event = RiskEvent::new(metadata.clone(), payload.clone());
    let frame = event.to_frame().expect("risk frame encoding");
    let decoded = RiskEvent::from_frame(&frame).expect("risk frame decoding");

    assert_eq!(decoded.metadata().correlation_id, metadata.correlation_id);
    assert!(matches!(
        decoded.payload().action,
        RiskAction::Resume { .. }
    ));
    assert!(matches!(decoded.payload().priority, Priority::Normal));
}

#[tokio::test]
async fn test_channel_send_receive_basic() -> Result<(), EventBusError> {
    let bus = EventBusBuilder::default().build();
    let signal_sender = bus.signal_sender();
    let signal_receiver = bus.signal_receiver();

    let metadata = EventMetadata::new("test.signal.basic", Priority::Normal);
    let payload = SignalEventPayload {
        strategy_id: Uuid::new_v4(),
        account_id: "acct-basic".to_string(),
        priority: Priority::Normal,
        signal: StrategySignal {
            exchange: None,
            symbol: "ETH-USD".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: Decimal::new(1, 0),
            limit_price: None,
            confidence: 0.8,
            metadata: HashMap::new(),
        },
    };

    let event = SignalEvent::new(metadata.clone(), payload);
    signal_sender.publish(event, PublishMode::Blocking)?;

    let received = timeout(Duration::from_millis(50), signal_receiver.recv_async())
        .await
        .expect("receive future")?
        .metadata()
        .correlation_id;

    assert_eq!(received, metadata.correlation_id);
    Ok(())
}
