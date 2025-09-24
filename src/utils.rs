//! Utility functions for Ninja Gekko

/// Utility functions
pub mod utils {
    //! Placeholder for utility module
    //! This will be implemented as part of the Rust migration
    
    /// Format currency values
    pub fn format_currency(value: f64) -> String {
        format!("${:.2}", value)
    }
    
    /// Calculate percentage change
    pub fn percentage_change(old_value: f64, new_value: f64) -> f64 {
        ((new_value - old_value) / old_value) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;
    
    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(1234.56), "$1234.56");
        assert_eq!(format_currency(0.99), "$0.99");
    }
    
    #[test]
    fn test_percentage_change() {
        assert_eq!(percentage_change(100.0, 110.0), 10.0);
        assert_eq!(percentage_change(100.0, 90.0), -10.0);
    }
}