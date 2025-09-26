//! # Ninja Gekko Core
//!
//! Core types, error handling, and trading system orchestration for the
//! autonomous trading platform. This crate provides the fundamental building
//! blocks for the entire trading system.

pub mod error;
pub mod types;
pub mod order_manager;
pub mod smart_router;

// Re-export commonly used types
pub use error::*;
pub use types::*;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");