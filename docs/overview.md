# Ninja Gekko Platform Overview

Ninja Gekko is a fully Rust-native autonomous trading platform engineered for ultra-low latency execution, advanced neural intelligence, and native Model Context Protocol (MCP) interoperability.

## Architecture Highlights

- **Language**: 100% Rust across binaries, services, orchestration tools, and tests.
- **Workspace Layout**: Cargo workspace with crates for `ninja-gekko` (core binary), `ninja-gekko-core`, `ninja-gekko-database`, and `ninja-gekko-api`.
- **Concurrency**: Tokio runtime with structured logging, tracing, and async orchestration.
- **Configuration**: `clap` command-line interface combined with `serde`-driven configuration loading.
- **Neural Intelligence**: `NeuralEngine` abstraction with pluggable backends (ruv-FANN, Candle, PyTorch bindings) and unified telemetry metrics.
- **Swarm + MCP Integration**: `McpManager` coordinates 70+ MCP servers, enabling deterministic orchestration for Playwright, Supabase, Redis, OpenRouter, LiteLLM, and additional ecosystems.
- **Security + Compliance**: Strict `#![forbid(unsafe_code)]`, audit-grade logging, deterministic execution plans, and controls aligned with SOX/GDPR/FINRA requirements.

## Deployment Blueprint

```bash
# Build optimized release artifacts
cargo build --release --workspace

# Produce container images
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -f deployments/Dockerfile.runtime \
  -t registry.example.com/ninja-gekko/runtime:$(git rev-parse --short HEAD) .

# Apply Kubernetes manifests
kubectl apply -k deployments/overlays/production
```

## Runtime Health + Observability

- **Telemetry Stack**: `tracing` + OpenTelemetry exporters, Prometheus, and Grafana dashboards.
- **Latency Target**: <100ms signal-to-execution measured via `criterion` benchmarks and live market replay harnesses.
- **Uptime Target**: 99.95%+ with active-active deployment topology and MCP-aware failover playbooks.
- **Order Throughput**: 50k+ orders/sec sustained via sharded execution loops and lock-free data structures.

## Testing Matrix

```bash
# Static analysis + formatting
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Unit and integration suites
cargo test --workspace --all-features

# Property-based checks
cargo test -p ninja-gekko-core --features property-tests

# Performance regression benchmarks
cargo bench --workspace
```

The documentation within `docs/` is intentionally lean and Rust-focused, replacing legacy Python artifacts while preserving deployment-grade readiness for the Ninja Gekko stack.
