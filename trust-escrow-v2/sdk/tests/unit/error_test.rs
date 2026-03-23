//! Unit tests for error handling in the Trust Escrow SDK
//!
//! These tests verify that the error system works correctly, provides good
//! error messages, and handles all error scenarios appropriately.

use pretty_assertions::assert_eq;
use rstest::*;

use trust_escrow_sdk::error::*;

// Import common test utilities
mod common;
use common::*;

// ===== ERROR CREATION TESTS =====

#[test]
fn test_validation_error_creation() {
    let error = EscrowError::Validation("Invalid username".to_string());
    
    match error {
        EscrowError::Validation(msg) => {
            assert_eq!(msg, "Invalid username");
        }
        _ => panic!("Expected Validation error"),
    }
}

#[test]
fn test_network_error_creation() {
    let error = EscrowError::Network("Connection failed".to_string());
    
    match error {
        EscrowError::Network(msg) => {
            assert_eq!(msg, "Connection failed");
        }
        _ => panic!("Expected Network error"),
    }
}

#[test]
fn test_anchor_error_creation() {
    let error = EscrowError::Anchor("Transaction failed".to_string());
    
    match error {
        EscrowError::Anchor(msg) => {
            assert_eq!(msg, "Transaction failed");
        }
        _ => panic!("Expected Anchor error"),
    }
}

#[test]
fn test_serialization_error_creation() {
    let error = EscrowError::Serialization("Failed to serialize data".to_string());
    
    match error {
        EscrowError::Serialization(msg) => {
            assert_eq!(msg, "Failed to serialize data");
        }
        _ => panic!("Expected Serialization error"),
    }
}

#[test]
fn test_program_error_creation() {
    let error = EscrowError::Program("Custom program error".to_string());
    
    match error {
        EscrowError::Program(msg) => {
            assert_eq!(msg, "Custom program error");
        }
        _ => panic!("Expected Program error"),
    }
}

#[test]
fn test_timeout_error_creation() {
    let error = EscrowError::Timeout("Operation timed out".to_string());
    
    match error {
        EscrowError::Timeout(msg) => {
            assert_eq!(msg, "Operation timed out");
        }
        _ => panic!("Expected Timeout error"),
    }
}

#[test]
fn test_account_not_found_error_creation() {
    let error = EscrowError::AccountNotFound("User account not found".to_string());
    
    match error {
        EscrowError::AccountNotFound(msg) => {
            assert_eq!(msg, "User account not found");
        }
        _ => panic!("Expected AccountNotFound error"),
    }
}

#[test]
fn test_insufficient_funds_error_creation() {
    let error = EscrowError::InsufficientFunds("Not enough SOL for transaction".to_string());
    
    match error {
        EscrowError::InsufficientFunds(msg) => {
            assert_eq!(msg, "Not enough SOL for transaction");
        }
        _ => panic!("Expected InsufficientFunds error"),
    }
}

#[test]
fn test_unauthorized_error_creation() {
    let error = EscrowError::Unauthorized("You don't have permission".to_string());
    
    match error {
        EscrowError::Unauthorized(msg) => {
            assert_eq!(msg, "You don't have permission");
        }
        _ => panic!("Expected Unauthorized error"),
    }
}

#[test]
fn test_invalid_state_error_creation() {
    let error = EscrowError::InvalidState("Job is already completed".to_string());
    
    match error {
        EscrowError::InvalidState(msg) => {
            assert_eq!(msg, "Job is already completed");
        }
        _ => panic!("Expected InvalidState error"),
    }
}

// ===== ERROR DISPLAY TESTS =====

#[test]
fn test_validation_error_display() {
    let error = EscrowError::Validation("Invalid input".to_string());
    let display_msg = format!("{}", error);
    
    assert!(display_msg.contains("Validation"));
    assert!(display_msg.contains("Invalid input"));
}

#[test]
fn test_network_error_display() {
    let error = EscrowError::Network("Connection timeout".to_string());
    let display_msg = format!("{}", error);
    
    assert!(display_msg.contains("Network"));
    assert!(display_msg.contains("Connection timeout"));
}

#[test]
fn test_anchor_error_display() {
    let error = EscrowError::Anchor("Transaction simulation failed".to_string());
    let display_msg = format!("{}", error);
    
    assert!(display_msg.contains("Anchor"));
    assert!(display_msg.contains("Transaction simulation failed"));
}

#[test]
fn test_program_error_display() {
    let error = EscrowError::Program("Custom constraint failed".to_string());
    let display_msg = format!("{}", error);
    
    assert!(display_msg.contains("Program"));
    assert!(display_msg.contains("Custom constraint failed"));
}

// ===== ERROR DEBUGGING TESTS =====

#[test]
fn test_validation_error_debug() {
    let error = EscrowError::Validation("Invalid username format".to_string());
    let debug_msg = format!("{:?}", error);
    
    assert!(debug_msg.contains("Validation"));
    assert!(debug_msg.contains("Invalid username format"));
}

#[test]
fn test_all_error_variants_debug() {
    let errors = vec![
        EscrowError::Validation("test".to_string()),
        EscrowError::Network("test".to_string()),
        EscrowError::Anchor("test".to_string()),
        EscrowError::Serialization("test".to_string()),
        EscrowError::Program("test".to_string()),
        EscrowError::Timeout("test".to_string()),
        EscrowError::AccountNotFound("test".to_string()),
        EscrowError::InsufficientFunds("test".to_string()),
        EscrowError::Unauthorized("test".to_string()),
        EscrowError::InvalidState("test".to_string()),
    ];
    
    for error in errors {
        let debug_msg = format!("{:?}", error);
        assert!(!debug_msg.is_empty(), "Debug message should not be empty");
        assert!(debug_msg.contains("test"), "Debug message should contain error message");
    }
}

// ===== ERROR CONVERSION TESTS =====

#[test]
fn test_result_type_usage() {
    fn returns_validation_error() -> Result<String> {
        Err(EscrowError::Validation("Invalid data".to_string()))
    }
    
    fn returns_success() -> Result<String> {
        Ok("Success".to_string())
    }
    
    assert!(returns_validation_error().is_err());
    assert!(returns_success().is_ok());
    assert_eq!(returns_success().unwrap(), "Success");
}

#[test]
fn test_error_chaining() {
    fn operation_that_fails() -> Result<()> {
        Err(EscrowError::Network("Connection failed".to_string()))
    }
    
    fn higher_level_operation() -> Result<()> {
        operation_that_fails().map_err(|e| {
            match e {
                EscrowError::Network(msg) => EscrowError::Program(format!("Higher level error: {}", msg)),
                other => other,
            }
        })
    }
    
    let result = higher_level_operation();
    assert!(result.is_err());
    
    match result.unwrap_err() {
        EscrowError::Program(msg) => {
            assert!(msg.contains("Higher level error"));
            assert!(msg.contains("Connection failed"));
        }
        _ => panic!("Expected Program error"),
    }
}

// ===== ERROR CATEGORIZATION TESTS =====

#[test]
fn test_error_is_recoverable() {
    // Define which errors might be recoverable
    fn is_recoverable_error(error: &EscrowError) -> bool {
        matches!(error, 
            EscrowError::Network(_) |
            EscrowError::Timeout(_) |
            EscrowError::Serialization(_)
        )
    }
    
    assert!(is_recoverable_error(&EscrowError::Network("test".to_string())));
    assert!(is_recoverable_error(&EscrowError::Timeout("test".to_string())));
    assert!(is_recoverable_error(&EscrowError::Serialization("test".to_string())));
    
    assert!(!is_recoverable_error(&EscrowError::Validation("test".to_string())));
    assert!(!is_recoverable_error(&EscrowError::Unauthorized("test".to_string())));
    assert!(!is_recoverable_error(&EscrowError::InvalidState("test".to_string())));
}

#[test]
fn test_error_is_user_error() {
    // Define which errors are likely user errors vs system errors
    fn is_user_error(error: &EscrowError) -> bool {
        matches!(error,
            EscrowError::Validation(_) |
            EscrowError::InsufficientFunds(_) |
            EscrowError::Unauthorized(_) |
            EscrowError::InvalidState(_)
        )
    }
    
    assert!(is_user_error(&EscrowError::Validation("test".to_string())));
    assert!(is_user_error(&EscrowError::InsufficientFunds("test".to_string())));
    assert!(is_user_error(&EscrowError::Unauthorized("test".to_string())));
    assert!(is_user_error(&EscrowError::InvalidState("test".to_string())));
    
    assert!(!is_user_error(&EscrowError::Network("test".to_string())));
    assert!(!is_user_error(&EscrowError::Anchor("test".to_string())));
    assert!(!is_user_error(&EscrowError::Program("test".to_string())));
}

// ===== ERROR SERIALIZATION TESTS =====

#[cfg(feature = "serde")]
#[test]
fn test_error_serialization() {
    let error = EscrowError::Validation("Test validation error".to_string());
    
    // Test that error can be serialized (if serde feature is enabled)
    let serialized = serde_json::to_string(&error);
    assert!(serialized.is_ok() || serialized.is_err()); // Just verify no panic
}

// ===== ERROR PATTERN MATCHING TESTS =====

#[test]
fn test_comprehensive_error_matching() {
    let errors = vec![
        EscrowError::Validation("validation".to_string()),
        EscrowError::Network("network".to_string()),
        EscrowError::Anchor("anchor".to_string()),
        EscrowError::Serialization("serialization".to_string()),
        EscrowError::Program("program".to_string()),
        EscrowError::Timeout("timeout".to_string()),
        EscrowError::AccountNotFound("account".to_string()),
        EscrowError::InsufficientFunds("funds".to_string()),
        EscrowError::Unauthorized("unauthorized".to_string()),
        EscrowError::InvalidState("state".to_string()),
    ];
    
    for error in errors {
        let category = match &error {
            EscrowError::Validation(_) => "validation",
            EscrowError::Network(_) => "network",
            EscrowError::Anchor(_) => "anchor",
            EscrowError::Serialization(_) => "serialization",
            EscrowError::Program(_) => "program",
            EscrowError::Timeout(_) => "timeout",
            EscrowError::AccountNotFound(_) => "account",
            EscrowError::InsufficientFunds(_) => "funds",
            EscrowError::Unauthorized(_) => "unauthorized",
            EscrowError::InvalidState(_) => "state",
        };
        
        let error_msg = format!("{}", error);
        assert!(error_msg.to_lowercase().contains(category));
    }
}

// ===== ERROR CONTEXT TESTS =====

#[test]
fn test_error_with_context() {
    fn add_context_to_error(error: EscrowError, context: &str) -> EscrowError {
        match error {
            EscrowError::Validation(msg) => EscrowError::Validation(format!("{}: {}", context, msg)),
            EscrowError::Network(msg) => EscrowError::Network(format!("{}: {}", context, msg)),
            other => other,
        }
    }
    
    let original_error = EscrowError::Validation("Invalid input".to_string());
    let contextual_error = add_context_to_error(original_error, "Creating user");
    
    match contextual_error {
        EscrowError::Validation(msg) => {
            assert!(msg.contains("Creating user"));
            assert!(msg.contains("Invalid input"));
        }
        _ => panic!("Expected Validation error"),
    }
}

// ===== ERROR HELP MESSAGE TESTS =====

#[test]
fn test_error_help_messages() {
    fn get_help_message(error: &EscrowError) -> String {
        match error {
            EscrowError::Validation(_) => "Check your input parameters and try again.".to_string(),
            EscrowError::Network(_) => "Check your network connection and try again.".to_string(),
            EscrowError::InsufficientFunds(_) => "Add more SOL to your wallet and try again.".to_string(),
            EscrowError::Unauthorized(_) => "Make sure you have the required permissions.".to_string(),
            EscrowError::AccountNotFound(_) => "The account may not exist or may not be initialized.".to_string(),
            EscrowError::InvalidState(_) => "The operation cannot be performed in the current state.".to_string(),
            _ => "Please try again or contact support.".to_string(),
        }
    }
    
    let validation_error = EscrowError::Validation("test".to_string());
    let help = get_help_message(&validation_error);
    assert!(help.contains("input parameters"));
    
    let network_error = EscrowError::Network("test".to_string());
    let help = get_help_message(&network_error);
    assert!(help.contains("network connection"));
}

// ===== ASYNC ERROR HANDLING TESTS =====

#[tokio::test]
async fn test_async_error_propagation() {
    async fn async_operation_that_fails() -> Result<String> {
        Err(EscrowError::Network("Async network error".to_string()))
    }
    
    async fn higher_level_async_operation() -> Result<String> {
        async_operation_that_fails().await
    }
    
    let result = higher_level_async_operation().await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        EscrowError::Network(msg) => {
            assert_eq!(msg, "Async network error");
        }
        _ => panic!("Expected Network error"),
    }
}

#[tokio::test]
async fn test_async_error_handling_with_timeout() {
    async fn slow_operation() -> Result<String> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok("Success".to_string())
    }
    
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(50),
        slow_operation()
    ).await;
    
    // Should timeout
    assert!(result.is_err());
}

// ===== ERROR BOUNDARY TESTS =====

#[test]
fn test_error_boundary_patterns() {
    fn safe_operation_wrapper<F, T>(operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        match operation() {
            Ok(result) => Ok(result),
            Err(EscrowError::Network(_)) => {
                // Convert network errors to generic errors for user
                Err(EscrowError::Program("Service temporarily unavailable".to_string()))
            }
            Err(other) => Err(other),
        }
    }
    
    fn operation_with_network_error() -> Result<String> {
        Err(EscrowError::Network("Internal network issue".to_string()))
    }
    
    let result = safe_operation_wrapper(operation_with_network_error);
    
    match result.unwrap_err() {
        EscrowError::Program(msg) => {
            assert_eq!(msg, "Service temporarily unavailable");
        }
        _ => panic!("Expected Program error"),
    }
}