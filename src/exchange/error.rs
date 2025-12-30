//! Exchange error types.

use crate::types::OrderId;
use thiserror::Error;

/// Errors that can occur when interacting with an exchange.
#[derive(Debug, Error)]
pub enum ExchangeError {
    /// Connection to the exchange failed.
    #[error("Failed to connect to exchange: {0}")]
    ConnectionFailed(String),

    /// Authentication with the exchange failed.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// The requested instrument was not found.
    #[error("Instrument not found: {0}")]
    InstrumentNotFound(String),

    /// The order was rejected by the exchange.
    #[error("Order rejected: {0}")]
    OrderRejected(String),

    /// The order was not found.
    #[error("Order not found: {0}")]
    OrderNotFound(OrderId),

    /// Insufficient funds to execute the order.
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: String, available: String },

    /// Insufficient position to sell.
    #[error("Insufficient position: required {required}, available {available}")]
    InsufficientPosition { required: String, available: String },

    /// The market is closed.
    #[error("Market is closed")]
    MarketClosed,

    /// The trading session is outside allowed hours.
    #[error("Outside trading hours")]
    OutsideTradingHours,

    /// Rate limit exceeded.
    #[error("Rate limit exceeded, retry after {retry_after_ms}ms")]
    RateLimitExceeded { retry_after_ms: u64 },

    /// Network error occurred.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Timeout waiting for response.
    #[error("Request timed out")]
    Timeout,

    /// Invalid request parameters.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Internal exchange error.
    #[error("Internal exchange error: {0}")]
    InternalError(String),

    /// Generic API error with code.
    #[error("API error {code}: {message}")]
    ApiError { code: i32, message: String },

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl ExchangeError {
    /// Returns true if the error is transient and the operation can be retried.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionFailed(_)
                | Self::NetworkError(_)
                | Self::Timeout
                | Self::RateLimitExceeded { .. }
        )
    }

    /// Returns true if the error is related to authentication.
    #[must_use]
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::AuthenticationFailed(_))
    }

    /// Returns true if the error is related to insufficient resources.
    #[must_use]
    pub fn is_insufficient_resources(&self) -> bool {
        matches!(
            self,
            Self::InsufficientFunds { .. } | Self::InsufficientPosition { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable() {
        assert!(ExchangeError::ConnectionFailed("test".to_string()).is_retryable());
        assert!(ExchangeError::Timeout.is_retryable());
        assert!(ExchangeError::RateLimitExceeded {
            retry_after_ms: 1000
        }
        .is_retryable());
        assert!(!ExchangeError::OrderRejected("test".to_string()).is_retryable());
    }

    #[test]
    fn test_is_auth_error() {
        assert!(ExchangeError::AuthenticationFailed("test".to_string()).is_auth_error());
        assert!(!ExchangeError::ConnectionFailed("test".to_string()).is_auth_error());
    }

    #[test]
    fn test_is_insufficient_resources() {
        assert!(ExchangeError::InsufficientFunds {
            required: "100".to_string(),
            available: "50".to_string()
        }
        .is_insufficient_resources());
        assert!(ExchangeError::InsufficientPosition {
            required: "10".to_string(),
            available: "5".to_string()
        }
        .is_insufficient_resources());
        assert!(!ExchangeError::Timeout.is_insufficient_resources());
    }
}
