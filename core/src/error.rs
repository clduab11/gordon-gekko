//! # Error Types and Handling for Core Components
//!
//! This module defines the core error types and error handling utilities
//! used throughout the trading system core components.

use std::fmt;

/// Core trading error types
#[derive(Debug, Clone)]
pub enum TradingError {
    /// Order management errors
    OrderError(String),
    /// Order not found errors
    OrderNotFound(String),
    /// Order validation errors
    OrderValidation(String),
    /// Platform not found errors
    PlatformNotFound(String),
    /// No available platforms errors
    NoAvailablePlatforms(String),
    /// Risk management errors
    RiskError(String),
    /// Fee calculation errors
    FeeError(String),
    /// Database connection errors
    DatabaseError(String),
    /// Configuration errors
    ConfigError(String),
    /// Validation errors
    ValidationError(String),
    /// External service errors
    ExternalError(String),
}

impl fmt::Display for TradingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradingError::OrderError(msg) => write!(f, "Order error: {}", msg),
            TradingError::OrderNotFound(msg) => write!(f, "Order not found: {}", msg),
            TradingError::OrderValidation(msg) => write!(f, "Order validation error: {}", msg),
            TradingError::PlatformNotFound(msg) => write!(f, "Platform not found: {}", msg),
            TradingError::NoAvailablePlatforms(msg) => write!(f, "No available platforms: {}", msg),
            TradingError::RiskError(msg) => write!(f, "Risk error: {}", msg),
            TradingError::FeeError(msg) => write!(f, "Fee error: {}", msg),
            TradingError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            TradingError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            TradingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TradingError::ExternalError(msg) => write!(f, "External error: {}", msg),
        }
    }
}

impl std::error::Error for TradingError {}

/// Core result type alias
pub type TradingResult<T> = Result<T, TradingError>;

/// Security-specific error types for validation and middleware
#[derive(Debug, Clone)]
pub enum SecurityError {
    /// Authentication errors
    AuthError(String),
    /// Authorization errors
    AuthorizationError(String),
    /// Input validation errors
    ValidationError(String),
    /// Rate limiting errors
    RateLimitError(String),
    /// Environment configuration errors
    EnvironmentError(String),
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            SecurityError::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            SecurityError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            SecurityError::RateLimitError(msg) => write!(f, "Rate limit error: {}", msg),
            SecurityError::EnvironmentError(msg) => write!(f, "Environment error: {}", msg),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Security result type alias
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Helper functions for creating common errors
impl TradingError {
    pub fn order(msg: impl Into<String>) -> Self {
        Self::OrderError(msg.into())
    }

    pub fn risk(msg: impl Into<String>) -> Self {
        Self::RiskError(msg.into())
    }

    pub fn fee(msg: impl Into<String>) -> Self {
        Self::FeeError(msg.into())
    }

    pub fn database(msg: impl Into<String>) -> Self {
        Self::DatabaseError(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    pub fn external(msg: impl Into<String>) -> Self {
        Self::ExternalError(msg.into())
    }
}

impl SecurityError {
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::AuthError(msg.into())
    }

    pub fn authorization(msg: impl Into<String>) -> Self {
        Self::AuthorizationError(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    pub fn rate_limit(msg: impl Into<String>) -> Self {
        Self::RateLimitError(msg.into())
    }

    pub fn environment(msg: impl Into<String>) -> Self {
        Self::EnvironmentError(msg.into())
    }
}