use chrono::Utc;
use event_bus::{EventBusBuilder, SignalEventPayload};
use rust_decimal::Decimal;
use uuid::Uuid;
use wat::parse_str as parse_wat;

use crate::{
    sandbox::{WasmStrategyConfig, WasmStrategyModule},
    traits::{MarketSnapshot, StrategyContext, StrategyMetrics},
    StrategyEventBridge,
};

const TEST_WASM: &str = r#"(module
  (import "host" "log" (func $log (param i32 i32)))
  (import "host" "emit_signal" (func $emit (param i32 i32)))
  (memory (export "memory") 1)
  (global $next (mut i32) (i32.const 1024))
  (data (i32.const 0) "{\"strategy_id\":\"00000000-0000-0000-0000-000000000000\",\"account_id\":\"sandbox-account\",\"priority\":\"High\",\"signal\":{\"exchange\":null,\"symbol\":\"BTC-USD\",\"side\":\"Buy\",\"order_type\":\"Market\",\"quantity\":\"1\",\"limit_price\":null,\"confidence\":1.0,\"metadata\":{}}}")
  (data (i32.const 512) "wasm")
  (func (export "alloc") (param $size i32) (result i32)
        (local $ptr i32)
        (local.set $ptr (global.get $next))
        (global.set $next (i32.add (local.get $ptr) (local.get $size)))
        (local.get $ptr))
  (func (export "evaluate") (param $ctx_ptr i32) (param $ctx_len i32) (result i32)
        (call $log (i32.const 512) (i32.const 4))
        (call $emit (i32.const 0) (i32.const 249))
        (i32.const 0)))"#;

#[test]
fn wasm_strategy_emits_signal() {
    let wasm_bytes = parse_wat(TEST_WASM).expect("valid test wasm");
    let module =
        WasmStrategyModule::from_bytes(&wasm_bytes, &WasmStrategyConfig::default()).unwrap();
    let mut instance = module.instantiate(WasmStrategyConfig::default()).unwrap();

    let account_id = String::from("sandbox-account");
    let snapshots = [MarketSnapshot {
        symbol: "BTC-USD".into(),
        bid: Decimal::from(30_000u32),
        ask: Decimal::from(30_010u32),
        last: Decimal::from(30_005u32),
        timestamp: Utc::now(),
    }];

    let context = StrategyContext::new(&account_id, &snapshots, Uuid::nil(), Utc::now());
    let decision = instance
        .evaluate(&context)
        .expect("strategy evaluation succeeds");

    assert_eq!(decision.logs, vec!["wasm".to_string()]);
    assert_eq!(decision.signals.len(), 1);
    let SignalEventPayload {
        strategy_id,
        account_id: signal_account,
        ..
    } = &decision.signals[0];
    assert_eq!(*strategy_id, Uuid::nil());
    assert_eq!(signal_account, "sandbox-account");
}

#[test]
fn bridge_publishes_signals() {
    let wasm_bytes = parse_wat(TEST_WASM).expect("valid test wasm");
    let module =
        WasmStrategyModule::from_bytes(&wasm_bytes, &WasmStrategyConfig::default()).unwrap();
    let mut instance = module.instantiate(WasmStrategyConfig::default()).unwrap();

    let account_id = String::from("sandbox-account");
    let snapshots = [MarketSnapshot {
        symbol: "BTC-USD".into(),
        bid: Decimal::from(30_000u32),
        ask: Decimal::from(30_010u32),
        last: Decimal::from(30_005u32),
        timestamp: Utc::now(),
    }];
    let context = StrategyContext::new(&account_id, &snapshots, Uuid::nil(), Utc::now());
    let decision = instance
        .evaluate(&context)
        .expect("strategy evaluation succeeds");

    let bus = EventBusBuilder::default().build();
    let receiver = bus.signal_receiver();
    let bridge = StrategyEventBridge::new(Uuid::nil(), "test-strategy", bus.signal_sender());

    bridge
        .publish(&decision, &StrategyMetrics::default())
        .unwrap();
    let event = receiver.recv().expect("signal dispatched");
    assert_eq!(event.payload().strategy_id, Uuid::nil());
    assert_eq!(event.payload().account_id, "sandbox-account");
}
