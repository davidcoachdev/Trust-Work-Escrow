//! Error types and utilities for Trust Work Escrow applications

use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    /// SDK-related errors
    #[error("SDK error: {0}")]
    Sdk(#[from] trust_escrow_sdk::error::EscrowError),

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Network connection errors
    #[error("Network error: {0}")]
    Network(#[from] solana_client::client_error::ClientError),

    /// IO errors (file operations, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// TOML parsing errors for config
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Invalid user input
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// Missing required data
    #[error("Missing required data: {message}")]
    MissingData { message: String },

    /// Generic operation failed
    #[error("Operation failed: {message}")]
    OperationFailed { message: String },

    /// Unknown error with context
    #[error("Unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

impl AppError {
    /// Create a config error
    pub fn config<M: Into<String>>(message: M) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create an invalid input error
    pub fn invalid_input<M: Into<String>>(message: M) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    /// Create a missing data error
    pub fn missing_data<M: Into<String>>(message: M) -> Self {
        Self::MissingData {
            message: message.into(),
        }
    }

    /// Create an operation failed error
    pub fn operation_failed<M: Into<String>>(message: M) -> Self {
        Self::OperationFailed {
            message: message.into(),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            AppError::Network(_) => true,
            AppError::Sdk(_) => false, // Depends on specific SDK error
            _ => false,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AppError::Network(_) => {
                "Network connection failed. Please check your internet connection and try again."
                    .to_string()
            }
            AppError::Config { message } => format!("Configuration problem: {}", message),
            AppError::InvalidInput { message } => format!("Invalid input: {}", message),
            AppError::MissingData { message } => {
                format!("Missing required information: {}", message)
            }
            _ => self.to_string(),
        }
    }
}

/// Application result type
pub type AppResult<T> = Result<T, AppError>;

/// Utility trait for converting Results into AppError
pub trait IntoAppError<T> {
    fn into_app_error(self) -> AppResult<T>;
}

impl<T, E> IntoAppError<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn into_app_error(self) -> AppResult<T> {
        self.map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AppError::config("Test message");
        assert!(matches!(error, AppError::Config { .. }));
        assert_eq!(error.to_string(), "Configuration error: Test message");
    }

    #[test]
    fn test_user_message() {
        let error = AppError::invalid_input("Bad value");
        assert_eq!(error.user_message(), "Invalid input: Bad value");
    }

    #[test]
    fn test_retryable() {
        let io_error = AppError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Connection refused",
        ));
        assert!(!io_error.is_retryable());
    }
}
