# Ninja Gekko Rust Migration Specification

This document captures the authoritative baseline for the fully Rust-native Ninja Gekko platform following the removal of all legacy Python assets.

## Goals

1. **Zero Python Dependencies**: No Python runtime, packaging files, or interpreted orchestration scripts remain in the repository.
2. **Unified Rust Workspace**: Cargo workspace manages all binaries, services, integration tests, and supporting tooling.
3. **Native Integrations**: MCP, neural inference, Supabase, Redis, OpenRouter, LiteLLM, and Playwright connectors operate through Rust crates.
4. **Regulatory Alignment**: SOX, GDPR, and FINRA controls embedded through compile-time checks, audit logging, and deterministic workflows.

## Workstream Summary

| Area | Previous State | Rust-First Replacement |
|------|----------------|------------------------|
| Deployment automation | Python CLI + pip requirements | `ninja-gekko` binary with `clap` interface + Kubernetes manifests |
| Neural pipelines | Python scripts invoking ML frameworks | `NeuralEngine` with async loading + inference traits |
| Config management | `settings.py` and `.env` wrappers | `serde`-powered config modules + `clap` overrides |
| Testing harness | PyTest suites | `cargo test`, property tests, and Criterion benchmarks |

## Validation Checklist

- [x] Remove `pyproject.toml`, `requirements.txt`, `.coverage`, and cached bytecode.
- [x] Delete Python package directories and tests under `src/gordon_gekko` and `tests/`.
- [x] Rename crates and binaries to `ninja-gekko-*`.
- [x] Add missing workspace dependencies for `serde_json` and `chrono`.
- [x] Update documentation, README, and environment references to the Ninja Gekko identity.
- [x] Provide Rust-native developer workflows (`cargo fmt`, `cargo clippy`, `cargo test`, `cargo bench`).

## Next Steps

1. Expand integration tests for MCP adapters to cover Playwright, Supabase, Redis, OpenRouter, and LiteLLM contracts.
2. Build Criterion benchmark suite to continuously validate <100ms latency and 50k+ orders/sec throughput.
3. Harden Kubernetes manifests under `deployments/` to reflect the new binary names and telemetry stack.
4. Complete formal compliance review with audit-ready logging and retention policies.

This specification evolves with each release to guarantee the Ninja Gekko platform remains Rust-first, high-performance, and enterprise compliant.
