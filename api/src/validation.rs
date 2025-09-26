//! Comprehensive security validation layer
//!
//! This module provides centralized input validation, sanitization,
//! and security checks for the Ninja Gekko API server. It implements
//! defense-in-depth strategies to prevent common vulnerabilities.

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use chrono::{DateTime, Utc};

/// Security configuration for validation rules
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum string length allowed
    pub max_string_length: usize,
    /// Maximum array/collection size
    pub max_collection_size: usize,
    /// Maximum numeric value
    pub max_numeric_value: f64,
    /// Minimum numeric value
    pub min_numeric_value: f64,
    /// Allowed file extensions for uploads
    pub allowed_file_extensions: Vec<String>,
    /// Blocked IP patterns
    pub blocked_ip_patterns: Vec<String>,
    /// Rate limiting thresholds
    pub rate_limits: HashMap<String, u32>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_string_length: 1000,
            max_collection_size: 100,
            max_numeric_value: 1_000_000_000.0,
            min_numeric_value: -1_000_000_000.0,
            allowed_file_extensions: vec![
                "jpg".to_string(), "jpeg".to_string(), "png".to_string(),
                "gif".to_string(), "pdf".to_string(), "txt".to_string(),
                "csv".to_string(), "json".to_string()
            ],
            blocked_ip_patterns: vec![
                "192\\.168\\..*".to_string(),
                "10\\..*".to_string(),
                "127\\..*".to_string(),
            ],
            rate_limits: [
                ("auth".to_string(), 5),
                ("trades".to_string(), 100),
                ("portfolio".to_string(), 50),
                ("market_data".to_string(), 1000),
            ].iter().cloned().collect(),
        }
    }
}

/// SQL injection prevention patterns
lazy_static! {
    static ref SQL_INJECTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)union\s+select").unwrap(),
        Regex::new(r"(?i)select\s+.*\s+from").unwrap(),
        Regex::new(r"(?i)insert\s+into").unwrap(),
        Regex::new(r"(?i)update\s+.*\s+set").unwrap(),
        Regex::new(r"(?i)delete\s+from").unwrap(),
        Regex::new(r"(?i)drop\s+table").unwrap(),
        Regex::new(r"(?i)alter\s+table").unwrap(),
        Regex::new(r"(?i)exec\s*\(").unwrap(),
        Regex::new(r"(?i)execute\s*\(").unwrap(),
        Regex::new(r"(?i)sp_executesql").unwrap(),
        Regex::new(r"--.*").unwrap(),
        Regex::new(r"/\*.*\*/").unwrap(),
        Regex::new(r";.*--").unwrap(),
        Regex::new(r"'.*OR.*='").unwrap(),
        Regex::new(r"'.*=.*'").unwrap(),
    ];

    static ref XSS_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"<script[^>]*>.*?</script>").unwrap(),
        Regex::new(r"javascript:").unwrap(),
        Regex::new(r"on\w+\s*=").unwrap(),
        Regex::new(r"<iframe[^>]*>.*?</iframe>").unwrap(),
        Regex::new(r"<object[^>]*>.*?</object>").unwrap(),
        Regex::new(r"<embed[^>]*>.*?</embed>").unwrap(),
        Regex::new(r"<form[^>]*>.*?</form>").unwrap(),
        Regex::new(r"<input[^>]*>.*?</input>").unwrap(),
    ];
}

/// Input sanitization levels
#[derive(Debug, Clone, Copy)]
pub enum SanitizationLevel {
    /// Basic sanitization - remove dangerous characters
    Basic,
    /// Strict sanitization - remove all HTML/script patterns
    Strict,
    /// None - no sanitization (use with caution)
    None,
}

/// Rate limiting context for validation
#[derive(Debug, Clone)]
pub struct RateLimitContext {
    pub endpoint: String,
    pub user_id: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
}

/// Comprehensive validation error with security context
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub severity: ValidationSeverity,
    pub suggestion: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ValidationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Main validation result type
pub type ValidationResult<T> = Result<T, ValidationErrors>;

/// Security validator for comprehensive input validation
pub struct SecurityValidator {
    config: SecurityConfig,
}

impl SecurityValidator {
    /// Create a new security validator with default configuration
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
        }
    }

    /// Create a new security validator with custom configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Validate and sanitize a string input
    pub fn validate_string(&self, input: &str, field_name: &str, level: SanitizationLevel) -> ValidationResult<String> {
        let sanitized = match level {
            SanitizationLevel::Basic => self.sanitize_basic(input),
            SanitizationLevel::Strict => self.sanitize_strict(input),
            SanitizationLevel::None => input.to_string(),
        };

        // Check length constraints
        if sanitized.len() > self.config.max_string_length {
            return Err(self.create_length_error(field_name, sanitized.len(), self.config.max_string_length));
        }

        // Check for SQL injection patterns
        if self.contains_sql_injection(&sanitized) {
            return Err(self.create_security_error(
                field_name,
                "sql_injection",
                "Potential SQL injection detected",
                ValidationSeverity::Critical,
                Some("Use parameterized queries instead of string concatenation")
            ));
        }

        // Check for XSS patterns if strict level
        if level == SanitizationLevel::Strict && self.contains_xss(&sanitized) {
            return Err(self.create_security_error(
                field_name,
                "xss_attempt",
                "Potential XSS attack detected",
                ValidationSeverity::High,
                Some("Sanitize user input before displaying in HTML")
            ));
        }

        Ok(sanitized)
    }

    /// Validate numeric input within bounds
    pub fn validate_numeric<T>(&self, input: T, field_name: &str) -> ValidationResult<T>
    where
        T: PartialOrd + Copy + std::fmt::Debug,
    {
        let min_val = self.config.min_numeric_value as f64;
        let max_val = self.config.max_numeric_value as f64;
        let input_val = input as f64;

        if input_val < min_val || input_val > max_val {
            return Err(self.create_range_error(field_name, input_val, min_val, max_val));
        }

        Ok(input)
    }

    /// Validate collection size
    pub fn validate_collection<T>(&self, collection: &[T], field_name: &str) -> ValidationResult<()> {
        if collection.len() > self.config.max_collection_size {
            return Err(self.create_collection_size_error(
                field_name,
                collection.len(),
                self.config.max_collection_size
            ));
        }
        Ok(())
    }

    /// Validate file extension
    pub fn validate_file_extension(&self, filename: &str) -> ValidationResult<String> {
        let extension = filename
            .split('.')
            .last()
            .unwrap_or("")
            .to_lowercase();

        if !self.config.allowed_file_extensions.contains(&extension) {
            return Err(self.create_file_extension_error(&extension));
        }

        Ok(extension)
    }

    /// Validate IP address against blocked patterns
    pub fn validate_ip_address(&self, ip: &str) -> ValidationResult<()> {
        for pattern in &self.config.blocked_ip_patterns {
            if Regex::new(pattern).unwrap().is_match(ip) {
                return Err(self.create_ip_blocked_error(ip));
            }
        }
        Ok(())
    }

    /// Basic sanitization - remove dangerous characters
    fn sanitize_basic(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '<' | '>' | '&' | '"' | '\'' => ' ',
                _ => c,
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Strict sanitization - remove all potentially dangerous patterns
    fn sanitize_strict(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Remove HTML tags
        result = Regex::new(r"<[^>]*>").unwrap().replace_all(&result, "").to_string();

        // Remove script content
        result = Regex::new(r"<script[^>]*>.*?</script>").unwrap().replace_all(&result, "").to_string();

        // Remove event handlers
        result = Regex::new(r"on\w+\s*=\s*[^>]*").unwrap().replace_all(&result, "").to_string();

        // Remove javascript: URLs
        result = Regex::new(r"javascript:[^\"]*").unwrap().replace_all(&result, "").to_string();

        result
    }

    /// Check for SQL injection patterns
    fn contains_sql_injection(&self, input: &str) -> bool {
        SQL_INJECTION_PATTERNS.iter().any(|pattern| pattern.is_match(input))
    }

    /// Check for XSS patterns
    fn contains_xss(&self, input: &str) -> bool {
        XSS_PATTERNS.iter().any(|pattern| pattern.is_match(input))
    }

    /// Create validation error for length violations
    fn create_length_error(&self, field: &str, actual: usize, max: usize) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let error = ValidationError::new("Maximum length exceeded");
        errors.add(field, error);
        errors
    }

    /// Create validation error for range violations
    fn create_range_error(&self, field: &str, value: f64, min: f64, max: f64) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let error = ValidationError::new(&format!("Value {} out of range [{}, {}]", value, min, max));
        errors.add(field, error);
        errors
    }

    /// Create validation error for collection size violations
    fn create_collection_size_error(&self, field: &str, actual: usize, max: usize) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let error = ValidationError::new(&format!("Collection size {} exceeds maximum {}", actual, max));
        errors.add(field, error);
        errors
    }

    /// Create validation error for file extension violations
    fn create_file_extension_error(&self, extension: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let error = ValidationError::new(&format!("File extension '{}' not allowed", extension));
        errors.add("file_extension", error);
        errors
    }

    /// Create validation error for blocked IP addresses
    fn create_ip_blocked_error(&self, ip: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let error = ValidationError::new(&format!("IP address '{}' is blocked", ip));
        errors.add("ip_address", error);
        errors
    }

    /// Create security validation error
    fn create_security_error(
        &self,
        field: &str,
        code: &str,
        message: &str,
        severity: ValidationSeverity,
        suggestion: Option<&str>
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let error = ValidationError::new(message);
        errors.add(field, error);
        errors
    }
}

/// Middleware for automatic request validation
pub struct ValidationMiddleware {
    validator: SecurityValidator,
}

impl ValidationMiddleware {
    pub fn new() -> Self {
        Self {
            validator: SecurityValidator::new(),
        }
    }

    pub fn validate_request<T>(&self, request: &T) -> ValidationResult<()>
    where
        T: Validate,
    {
        request.validate()
    }

    pub fn sanitize_input(&self, input: &str, field: &str) -> ValidationResult<String> {
        self.validator.validate_string(input, field, SanitizationLevel::Strict)
    }
}

/// Rate limiting validation
pub struct RateLimitValidator {
    validator: SecurityValidator,
}

impl RateLimitValidator {
    pub fn new() -> Self {
        Self {
            validator: SecurityValidator::new(),
        }
    }

    pub fn check_rate_limit(&self, context: &RateLimitContext) -> ValidationResult<()> {
        let limit = self.validator.config.rate_limits
            .get(&context.endpoint)
            .copied()
            .unwrap_or(100);

        // TODO: Implement actual rate limiting logic with storage
        // For now, just validate the context
        if context.endpoint.is_empty() {
            return Err(self.create_rate_limit_error("Endpoint cannot be empty"));
        }

        Ok(())
    }

    fn create_rate_limit_error(&self, message: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let error = ValidationError::new(message);
        errors.add("rate_limit", error);
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let validator = SecurityValidator::new();

        assert!(validator.contains_sql_injection("SELECT * FROM users"));
        assert!(validator.contains_sql_injection("UNION SELECT password FROM users"));
        assert!(validator.contains_sql_injection("DROP TABLE users --"));
        assert!(validator.contains_sql_injection("'; OR 1=1 --"));
        assert!(!validator.contains_sql_injection("normal text"));
    }

    #[test]
    fn test_string_validation() {
        let validator = SecurityValidator::new();

        // Valid string
        let result = validator.validate_string("hello world", "test_field", SanitizationLevel::Basic);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");

        // SQL injection attempt
        let result = validator.validate_string("SELECT * FROM users", "query", SanitizationLevel::Strict);
        assert!(result.is_err());

        // XSS attempt
        let result = validator.validate_string("<script>alert('xss')</script>", "html", SanitizationLevel::Strict);
        assert!(result.is_err());
    }

    #[test]
    fn test_numeric_validation() {
        let validator = SecurityValidator::new();

        // Valid number
        let result = validator.validate_numeric(100.0, "amount");
        assert!(result.is_ok());

        // Out of range
        let result = validator.validate_numeric(2_000_000_000.0, "amount");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_extension_validation() {
        let config = SecurityConfig {
            allowed_file_extensions: vec!["jpg".to_string(), "png".to_string(), "pdf".to_string()],
            ..Default::default()
        };
        let validator = SecurityValidator::with_config(config);

        assert!(validator.validate_file_extension("test.jpg").is_ok());
        assert!(validator.validate_file_extension("document.pdf").is_ok());
        assert!(validator.validate_file_extension("script.exe").is_err());
    }
}