use std::mem;
use std::time::{Duration, Instant};

use anyhow::Context;
use chrono::{DateTime, Utc};
use event_bus::SignalEventPayload;
use ninja_gekko_core::types::AccountId;
use serde::Serialize;
use tracing::warn;
use uuid::Uuid;
use wasmtime::{
    AsContextMut, Caller, Config, Engine, Linker, Memory, Module, Store, StoreLimits,
    StoreLimitsBuilder, TypedFunc,
};

use crate::traits::{
    MarketSnapshot, StrategyContext, StrategyDecision, StrategyError, StrategyMetrics,
    WasmSignalInstruction,
};

const DEFAULT_MEMORY_LIMIT: u64 = 16 * 1024 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(5);

#[derive(Clone)]
pub struct WasmStrategyConfig {
    pub memory_limit: u64,
    pub evaluation_timeout: Duration,
}

impl Default for WasmStrategyConfig {
    fn default() -> Self {
        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT,
            evaluation_timeout: DEFAULT_TIMEOUT,
        }
    }
}

pub struct WasmStrategyModule {
    engine: Engine,
    module: Module,
}

impl WasmStrategyModule {
    pub fn from_bytes(bytes: &[u8], config: &WasmStrategyConfig) -> Result<Self, StrategyError> {
        let mut wasm_config = Config::new();
        wasm_config.wasm_multi_memory(true);
        wasm_config.static_memory_maximum_size(config.memory_limit);
        wasm_config.dynamic_memory_guard_size(0);
        wasm_config.static_memory_guard_size(0);

        let engine = Engine::new(&wasm_config).map_err(StrategyError::Wasm)?;
        let module = Module::new(&engine, bytes).map_err(StrategyError::Wasm)?;
        Ok(Self { engine, module })
    }

    pub fn instantiate(
        &self,
        config: WasmStrategyConfig,
    ) -> Result<WasmStrategyInstance, StrategyError> {
        WasmStrategyInstance::new(self.engine.clone(), self.module.clone(), config)
    }
}

struct StrategyEnvState {
    limits: StoreLimits,
    logs: Vec<String>,
    signals: Vec<SignalEventPayload>,
}

pub struct WasmStrategyInstance {
    config: WasmStrategyConfig,
    store: Store<StrategyEnvState>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    evaluate: TypedFunc<(i32, i32), i32>,
}

impl WasmStrategyInstance {
    fn new(
        engine: Engine,
        module: Module,
        config: WasmStrategyConfig,
    ) -> Result<Self, StrategyError> {
        let limits = StoreLimitsBuilder::new()
            .memory_size(config.memory_limit as usize)
            .instances(1)
            .build();

        let state = StrategyEnvState {
            limits,
            logs: Vec::new(),
            signals: Vec::new(),
        };

        let mut store = Store::new(&engine, state);
        store.limiter(|state| &mut state.limits);

        let mut linker = Linker::new(&engine);
        link_host_functions(&mut linker)?;
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(StrategyError::Wasm)?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| StrategyError::sandbox("wasm module missing exported memory"))?;

        let alloc: TypedFunc<u32, u32> = instance
            .get_typed_func(&mut store, "alloc")
            .map_err(|_| StrategyError::sandbox("wasm module must export alloc(u32) -> u32"))?;

        let evaluate: TypedFunc<(i32, i32), i32> = instance
            .get_typed_func(&mut store, "evaluate")
            .map_err(|_| {
                StrategyError::sandbox("wasm module must export evaluate(ptr, len) -> status")
            })?;

        Ok(Self {
            config,
            store,
            memory,
            alloc,
            evaluate,
        })
    }

    pub fn evaluate<const N: usize>(
        &mut self,
        context: &StrategyContext<'_, N>,
    ) -> Result<StrategyDecision, StrategyError> {
        let payload = serde_json::to_vec(&SerializableContext::from(context))?;
        let len = payload.len() as u32;

        let ptr = self
            .alloc
            .call(&mut self.store, len)
            .map_err(StrategyError::Wasm)?;
        self.memory
            .write(self.store.as_context_mut(), ptr as usize, &payload)
            .map_err(|err| StrategyError::Wasm(err.into()))?;

        let start = Instant::now();
        self.evaluate
            .call(&mut self.store, (ptr as i32, len as i32))
            .map_err(StrategyError::Wasm)?;
        let elapsed = start.elapsed();

        if elapsed > self.config.evaluation_timeout {
            warn!("strategy evaluation exceeded timeout: {:?}", elapsed);
            return Err(StrategyError::Timeout(elapsed));
        }

        let state = self.store.data_mut();
        let signals = mem::take(&mut state.signals);
        let logs = mem::take(&mut state.logs);

        Ok(StrategyDecision {
            signals,
            logs,
            metrics: StrategyMetrics {
                evaluation_latency: elapsed,
            },
        })
    }
}

fn link_host_functions(linker: &mut Linker<StrategyEnvState>) -> Result<(), StrategyError> {
    linker
        .func_wrap(
            "host",
            "log",
            |mut caller: Caller<'_, StrategyEnvState>, ptr: i32, len: i32| -> anyhow::Result<()> {
                if let Ok(bytes) = read_guest(&mut caller, ptr, len) {
                    if let Ok(message) = String::from_utf8(bytes) {
                        caller.data_mut().logs.push(message);
                    }
                }
                Ok(())
            },
        )
        .map_err(StrategyError::Wasm)?;

    linker
        .func_wrap(
            "host",
            "emit_signal",
            |mut caller: Caller<'_, StrategyEnvState>, ptr: i32, len: i32| -> anyhow::Result<()> {
                let bytes = read_guest(&mut caller, ptr, len)?;
                let instruction: WasmSignalInstruction = serde_json::from_slice(&bytes)?;
                caller.data_mut().signals.push(SignalEventPayload {
                    strategy_id: instruction.strategy_id,
                    account_id: instruction.account_id,
                    priority: instruction.priority,
                    signal: instruction.signal,
                });
                Ok(())
            },
        )
        .map_err(StrategyError::Wasm)?;

    Ok(())
}

fn read_guest(
    caller: &mut Caller<'_, StrategyEnvState>,
    ptr: i32,
    len: i32,
) -> anyhow::Result<Vec<u8>> {
    let memory = caller
        .get_export("memory")
        .and_then(|export| export.into_memory())
        .context("webassembly memory export missing")?;
    let mut buf = vec![0u8; len as usize];
    memory.read(caller.as_context_mut(), ptr as usize, &mut buf)?;
    Ok(buf)
}

#[derive(Serialize)]
struct SerializableContext<'a> {
    account_id: &'a AccountId,
    evaluation_id: Uuid,
    timestamp: DateTime<Utc>,
    snapshots: Vec<&'a MarketSnapshot>,
}

impl<'a, const N: usize> From<&'a StrategyContext<'a, N>> for SerializableContext<'a> {
    fn from(ctx: &'a StrategyContext<'a, N>) -> Self {
        Self {
            account_id: ctx.account_id(),
            evaluation_id: ctx.evaluation_id(),
            timestamp: ctx.timestamp(),
            snapshots: ctx.snapshots().iter().collect(),
        }
    }
}
