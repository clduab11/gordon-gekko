# Ninja Gekko Copilot Instructions

## Project Overview

**Ninja Gekko** is a next-generation autonomous trading bot built in Rust with native MCP (Model Context Protocol) integration. This project represents the evolution from the original Gordon Gekko Python implementation to a high-performance, memory-safe, completely autonomous trading system.

## Core Architecture

### Technology Stack
- **Primary Language**: Rust (transitioning from Python)
- **Neural Networks**: ruv-FANN (Rust-based FANN implementation)
- **Swarm Intelligence**: flow-nexus integration
- **Protocols**: Native MCP support with 70+ server integrations
- **Performance**: WebAssembly compatible, GPU-accelerated (CUDA + Metal)
- **Deployment**: Docker, Kubernetes, edge computing capable

### Key Performance Targets
- **Speed**: <100ms decision times (5-20x faster than traditional bots)
- **Accuracy**: 84.8% neural prediction accuracy
- **Efficiency**: 32.3% lower resource usage
- **Reliability**: 99.95% uptime target

## Development Guidelines

### Code Style and Standards
1. **Rust Best Practices**
   - Use `#![forbid(unsafe_code)]` for memory safety
   - Implement proper error handling with `Result<T, E>`
   - Follow Rust API Guidelines and naming conventions
   - Use `clippy` and `rustfmt` for code quality

2. **Performance Focus**
   - Optimize for zero-copy operations where possible
   - Use `async`/`await` for concurrent operations
   - Implement efficient data structures (Vec, HashMap, etc.)
   - Profile critical paths with `criterion` benchmarks

3. **Neural Network Integration**
   - Integrate ruv-FANN for all ML operations
   - Implement NHITS/NBEATSx models for forecasting
   - Support both CPU and GPU execution paths
   - Maintain compatibility with existing Python models during transition

### MCP Integration Strategy

#### Core MCP Servers (Required)
- **Playwright MCP**: Browser automation, web scraping, market data collection
- **Filesystem MCP**: Data persistence, configuration management, model storage
- **GitHub MCP**: Repository management, CI/CD integration, version control
- **Supabase MCP**: Real-time database operations, analytics, user management

#### Extended MCP Ecosystem (70+ servers)
- **AI/ML Services**: OpenAI, Cohere, HuggingFace, Replicate, StabilityAI
- **Data Analytics**: Code Interpreter, Mem0, Ahrefs, Perplexity AI
- **Communication**: Gmail, Slack, Telegram, Discord, LinkedIn
- **Cloud Storage**: Google Drive, Dropbox, OneDrive, Box
- **Development**: GitLab, Docker, NPM, PyPI, Bitbucket
- **Social Intelligence**: Twitter, Reddit, YouTube, Medium

### Architecture Patterns

#### 1. **Autonomous Operation Modes**
```rust
pub enum OperationMode {
    Stealth,    // Minimal market impact, fragmented orders
    Precision,  // Microsecond timing, neural predictions
    Swarm,      // Distributed intelligence, collaborative decisions
}
```

#### 2. **Neural Intelligence Stack**
```rust
pub struct NeuralStack {
    ruv_fann_core: RuvFannEngine,
    forecasting_models: Vec<ForecastingModel>,
    swarm_intelligence: FlowNexusClient,
    gpu_acceleration: Option<GpuRuntime>,
}
```

#### 3. **MCP Integration Layer**
```rust
pub struct McpManager {
    core_servers: HashMap<String, McpServer>,
    extended_servers: Vec<McpServer>,
    connection_pool: ConnectionPool,
    failover_strategy: FailoverConfig,
}
```

### Trading System Requirements

#### Performance Specifications
- **Latency**: Sub-100ms from signal to execution
- **Throughput**: 50,000+ orders per second
- **Accuracy**: 84.8%+ neural prediction accuracy
- **Risk Management**: <8.2% maximum drawdown
- **Uptime**: 99.95% operational availability

#### Security Requirements
- **Memory Safety**: Zero unsafe code in critical paths
- **Zero Trust**: Verify all external connections
- **Encryption**: End-to-end encryption for all data
- **Audit Trail**: Complete transaction logging
- **Compliance**: SOC 2, PCI DSS compliance ready

### Testing Strategy

#### Unit Testing
- Test all core functions with `cargo test`
- Maintain >90% code coverage
- Use property-based testing with `proptest`
- Mock external dependencies

#### Integration Testing
- Test MCP server connections
- Validate neural network outputs
- Verify trading platform integrations
- Test failover and recovery scenarios

#### Performance Testing
- Benchmark critical paths with `criterion`
- Load test with simulated market conditions
- Memory profiling with `valgrind`/`heaptrack`
- GPU performance validation

### Deployment Architecture

#### Container Strategy
```dockerfile
# Multi-stage Rust build
FROM rust:1.80 as builder
# Optimized runtime
FROM debian:bookworm-slim
# WebAssembly support
FROM wasmedge/wasmedge:latest
```

#### Kubernetes Deployment
- **HPA**: Horizontal Pod Autoscaling based on CPU/memory
- **GPU Nodes**: NVIDIA GPU operator for CUDA workloads
- **Storage**: Persistent volumes for model storage
- **Networking**: Istio service mesh for MCP communications

### Documentation Standards

#### Code Documentation
- Use `///` for public API documentation
- Include examples in doc comments
- Generate docs with `cargo doc --no-deps --open`
- Maintain architectural decision records (ADRs)

#### API Documentation
- OpenAPI 3.0 specifications for REST endpoints
- MCP protocol documentation for all integrations
- WebSocket API documentation for real-time feeds
- GraphQL schema for complex queries

### Migration Path from Python

#### Phase 1: Core Infrastructure (Weeks 1-4)
1. Set up Rust project structure and dependencies
2. Implement basic MCP integration layer
3. Create database and caching abstractions
4. Build trading platform connectors

#### Phase 2: Neural Networks (Weeks 5-8)
1. Integrate ruv-FANN neural network engine
2. Port existing Python models to Rust
3. Implement GPU acceleration layer
4. Add WASM compilation targets

#### Phase 3: Trading Logic (Weeks 9-12)
1. Port trading strategies from Python
2. Implement risk management systems
3. Add portfolio optimization algorithms
4. Integrate swarm intelligence features

#### Phase 4: Production Deployment (Weeks 13-16)
1. Performance optimization and tuning
2. Security audit and compliance validation
3. Monitoring and observability setup
4. Gradual rollout with A/B testing

### Continuous Integration

#### GitHub Actions Workflow
```yaml
# .github/workflows/ci.yml
name: Ninja Gekko CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo bench
```

### Monitoring and Observability

#### Metrics Collection
- **Prometheus**: System metrics, trading performance
- **Grafana**: Real-time dashboards and alerting
- **Jaeger**: Distributed tracing for MCP calls
- **ELK Stack**: Centralized logging and analysis

#### Key Performance Indicators
- Trading performance metrics (Sharpe ratio, drawdown, alpha)
- System performance (latency, throughput, error rates)
- Neural network accuracy and training metrics
- MCP server health and response times

### Troubleshooting Guide

#### Common Issues
1. **MCP Connection Failures**: Check server health, retry with exponential backoff
2. **Neural Network Training**: Validate data preprocessing, check GPU memory
3. **Trading Platform APIs**: Handle rate limits, implement circuit breakers
4. **Performance Degradation**: Profile with `perf`, optimize hot paths

#### Debugging Tools
- **GDB**: Native debugging for Rust applications
- **Valgrind**: Memory leak detection and profiling
- **strace**: System call tracing for I/O issues
- **tcpdump**: Network traffic analysis for API calls

### Contributing Guidelines

#### Code Contributions
1. Fork the repository and create a feature branch
2. Write tests for new functionality
3. Ensure all tests pass and clippy warnings are resolved
4. Submit a pull request with clear description
5. Address code review feedback promptly

#### Documentation Contributions
1. Update relevant documentation for code changes
2. Add examples for new features
3. Maintain consistency with existing documentation style
4. Validate documentation builds correctly

This copilot-instructions.md serves as the source of truth for developing Ninja Gekko as a completely autonomous, high-performance trading bot with advanced MCP functionality and neural intelligence integration.