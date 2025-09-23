//! # Gordon Gekko Trading System
//!
//! A high-performance autonomous trading system with neural network integration
//! built in Rust. This library provides comprehensive trading capabilities
//! including market data processing, neural network predictions, risk management,
//! and multi-platform trading execution.
//!
//! ## Architecture
//!
//! The system is organized into modular crates:
//! - `core`: Core types, error handling, and trading system orchestration
//! - `trading`: Trading engine, order management, and execution algorithms
//! - `market_data`: Market data processing and real-time streams
//! - `neural`: Neural network integration and ML pipeline
//! - `database`: Database operations and caching
//! - `api`: REST and WebSocket APIs
//! - `security`: Security and authentication
//! - `utils`: Utility functions and helpers
//!
//! ## Features
//!
//! - **High Performance**: Async/await with Tokio runtime, zero-copy where possible
//! - **Neural Networks**: FANN integration with GPU acceleration support
//! - **Multi-Platform**: Support for Coinbase, Binance.US, OANDA, and others
//! - **Risk Management**: Real-time VaR calculations and position limits
//! - **Database**: PostgreSQL with Redis caching and pub/sub
//! - **Security**: End-to-end encryption, JWT auth, audit logging
//! - **Monitoring**: Comprehensive logging, metrics, and health checks
//!
//! ## Security
//!
//! - Zero Trust Architecture: All requests validated regardless of origin
//! - End-to-End Encryption: TLS 1.3 for all data in transit
//! - Credential Management: No hard-coded secrets, MCP-secured credentials
//! - Input Validation: Comprehensive sanitization of all inputs
//! - Audit Logging: Complete audit trail for all trading activities

pub mod core;
pub mod trading;
pub mod market_data;
pub mod neural;
pub mod database;
pub mod api;
pub mod security;
pub mod utils;

// Re-export commonly used types
pub use core::*;
pub use trading::*;
pub use market_data::*;
pub use neural::*;
pub use database::*;
pub use api::*;
pub use security::*;
pub use utils::*;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");