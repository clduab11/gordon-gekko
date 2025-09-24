//! # Database Layer
//!
//! High-performance database layer with PostgreSQL, Redis, and Supabase integration.
//! Provides enterprise-grade database operations with connection pooling, caching,
//! migrations, and transaction support.

pub mod config;
pub mod database;
pub mod cache;
pub mod supabase;
pub mod migrations;
pub mod connection;
pub mod error;
pub mod types;

// Re-export commonly used types
pub use config::*;
pub use database::*;
pub use cache::*;
pub use supabase::*;
pub use migrations::*;
pub use connection::*;
pub use error::*;
pub use types::*;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");