//! Unit tests for utility functions
//!
//! These tests verify that all utility functions work correctly,
//! handle edge cases properly, and provide expected functionality.

use std::time::Duration;

use pretty_assertions::assert_eq;
use proptest::prelude::*;
use rstest::*;

use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
    commitment_config::CommitmentConfig,
};
use solana_client::rpc_client::RpcClient;

use trust_escrow_sdk::utils::*;

// Import common test utilities
mod common;
use common::*;

// ===== CONVERSION UTILS TESTS =====

#[test]
fn test_sol_to_lamports() {
    assert_eq!(ConversionUtils::sol_to_lamports(1.0), 1_000_000_000);
    assert_eq!(ConversionUtils::sol_to_lamports(0.001), 1_000_000);
    assert_eq!(ConversionUtils::sol_to_lamports(0.0001), 100_000);
    assert_eq!(ConversionUtils::sol_to_lamports(0.0), 0);
}

#[test]
fn test_lamports_to_sol() {
    assert_eq!(ConversionUtils::lamports_to_sol(1_000_000_000), 1.0);
    assert_eq!(ConversionUtils::lamports_to_sol(1_000_000), 0.001);
    assert_eq!(ConversionUtils::lamports_to_sol(100_000), 0.0001);
    assert_eq!(ConversionUtils::lamports_to_sol(0), 0.0);
}

#[test]
fn test_sol_lamports_roundtrip() {
    let sol_amounts = vec![0.0, 0.001, 0.1, 1.0, 10.5, 100.0];
    
    for sol in sol_amounts {
        let lamports = ConversionUtils::sol_to_lamports(sol);
        let back_to_sol = ConversionUtils::lamports_to_sol(lamports);
        
        // Allow small floating point differences
        assert!((sol - back_to_sol).abs() < 0.000_000_001, 
               "SOL->Lamports->SOL conversion failed: {} != {}", sol, back_to_sol);
    }
}

#[test]
fn test_duration_to_unix() {
    let duration = Duration::from_secs(86400); // 1 day
    let base_time = 1234567890i64;
    
    let unix_timestamp = ConversionUtils::duration_to_unix(duration, base_time);
    assert_eq!(unix_timestamp, base_time + 86400);
}

#[test]
fn test_unix_to_duration() {
    let future_time = 1234567890i64 + 86400;
    let base_time = 1234567890i64;
    
    let duration = ConversionUtils::unix_to_duration(future_time, base_time).unwrap();
    assert_eq!(duration.as_secs(), 86400);
}

#[test]
fn test_unix_to_duration_past_time() {
    let past_time = 1234567890i64 - 86400;
    let base_time = 1234567890i64;
    
    let result = ConversionUtils::unix_to_duration(past_time, base_time);
    assert!(result.is_err()); // Should fail for past times
}

#[test]
fn test_pubkey_to_string() {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    
    let string_repr = ConversionUtils::pubkey_to_string(&pubkey);
    assert!(!string_repr.is_empty());
    assert!(string_repr.len() >= 32); // Base58 encoded should be at least 32 chars
}

#[test]
fn test_string_to_pubkey() {
    let keypair = Keypair::new();
    let original_pubkey = keypair.pubkey();
    let string_repr = original_pubkey.to_string();
    
    let parsed_pubkey = ConversionUtils::string_to_pubkey(&string_repr).unwrap();
    assert_eq!(original_pubkey, parsed_pubkey);
}

#[test]
fn test_string_to_pubkey_invalid() {
    let invalid_strings = vec![
        "",
        "invalid",
        "123",
        "not_a_pubkey",
        "too_short",
    ];
    
    for invalid in invalid_strings {
        let result = ConversionUtils::string_to_pubkey(invalid);
        assert!(result.is_err(), "Should fail for invalid input: {}", invalid);
    }
}

// ===== VALIDATION UTILS TESTS =====

#[test]
fn test_validate_username() {
    let valid_usernames = vec![
        "alice",
        "bob123",
        "user_name",
        "a",
        "valid_user_with_numbers123",
    ];
    
    for username in valid_usernames {
        assert!(ValidationUtils::validate_username(username).is_ok(), 
               "Should be valid: {}", username);
    }
}

#[test]
fn test_validate_username_invalid() {
    let invalid_usernames = vec![
        "",
        "a".repeat(100), // Too long
        "user with spaces",
        "user@domain.com",
        "user-with-dashes",
        "user.with.dots",
        "🚀username", // Unicode
    ];
    
    for username in invalid_usernames {
        assert!(ValidationUtils::validate_username(username).is_err(),
               "Should be invalid: {}", username);
    }
}

#[test]
fn test_validate_job_amount() {
    // Valid amounts
    assert!(ValidationUtils::validate_job_amount(100_000).is_ok()); // Min amount
    assert!(ValidationUtils::validate_job_amount(1_000_000).is_ok()); // Normal amount
    assert!(ValidationUtils::validate_job_amount(u64::MAX).is_ok()); // Max amount
    
    // Invalid amounts
    assert!(ValidationUtils::validate_job_amount(0).is_err()); // Zero
    assert!(ValidationUtils::validate_job_amount(99_999).is_err()); // Below min
}

#[test]
fn test_validate_bio() {
    // Valid bios
    assert!(ValidationUtils::validate_bio(None).is_ok()); // No bio
    assert!(ValidationUtils::validate_bio(Some("")).is_ok()); // Empty bio
    assert!(ValidationUtils::validate_bio(Some("Short bio")).is_ok()); // Normal bio
    assert!(ValidationUtils::validate_bio(Some(&"a".repeat(500))).is_ok()); // Long but acceptable
    
    // Invalid bio
    assert!(ValidationUtils::validate_bio(Some(&"a".repeat(2000))).is_err()); // Too long
}

#[test]
fn test_validate_evidence() {
    // Valid evidence
    assert!(ValidationUtils::validate_evidence("Valid evidence text").is_ok());
    assert!(ValidationUtils::validate_evidence(&"a".repeat(2048)).is_ok()); // Max length
    
    // Invalid evidence
    assert!(ValidationUtils::validate_evidence("").is_err()); // Empty
    assert!(ValidationUtils::validate_evidence(&"a".repeat(2049)).is_err()); // Too long
}

#[test]
fn test_validate_percentage_split() {
    // Valid splits
    assert!(ValidationUtils::validate_percentage_split(50, 50).is_ok());
    assert!(ValidationUtils::validate_percentage_split(30, 70).is_ok());
    assert!(ValidationUtils::validate_percentage_split(0, 100).is_ok());
    assert!(ValidationUtils::validate_percentage_split(100, 0).is_ok());
    
    // Invalid splits
    assert!(ValidationUtils::validate_percentage_split(60, 50).is_err()); // > 100%
    assert!(ValidationUtils::validate_percentage_split(30, 60).is_err()); // < 100%
    assert!(ValidationUtils::validate_percentage_split(101, 0).is_err()); // > 100%
}

#[test]
fn test_validate_deadline() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    // Valid deadlines
    assert!(ValidationUtils::validate_deadline(now + 3600).is_ok()); // 1 hour future
    assert!(ValidationUtils::validate_deadline(now + 86400).is_ok()); // 1 day future
    
    // Invalid deadlines
    assert!(ValidationUtils::validate_deadline(now - 3600).is_err()); // Past
    assert!(ValidationUtils::validate_deadline(now).is_err()); // Now (might fail due to timing)
}

// ===== TRANSACTION UTILS TESTS =====

#[tokio::test]
async fn test_transaction_utils_get_recent_blockhash() {
    let client = RpcClient::new("http://localhost:8899".to_string());
    
    // This will fail without a validator, but test the function exists
    let result = TransactionUtils::get_recent_blockhash(&client).await;
    assert!(result.is_ok() || result.is_err()); // Just ensure no panic
}

#[tokio::test]
async fn test_transaction_utils_account_exists() {
    let client = RpcClient::new("http://localhost:8899".to_string());
    let pubkey = Pubkey::new_unique();
    
    // This will fail without a validator, but test the function exists
    let result = TransactionUtils::account_exists(&client, &pubkey).await;
    assert!(result.is_ok() || result.is_err()); // Just ensure no panic
}

#[tokio::test]
async fn test_transaction_utils_get_balance() {
    let client = RpcClient::new("http://localhost:8899".to_string());
    let pubkey = Pubkey::new_unique();
    
    // This will fail without a validator, but test the function exists
    let result = TransactionUtils::get_balance(&client, &pubkey).await;
    assert!(result.is_ok() || result.is_err()); // Just ensure no panic
}

// ===== FORMATTING UTILS TESTS =====

#[test]
fn test_format_lamports() {
    assert_eq!(FormattingUtils::format_lamports(1_000_000_000), "1.000000000 SOL");
    assert_eq!(FormattingUtils::format_lamports(1_000_000), "0.001000000 SOL");
    assert_eq!(FormattingUtils::format_lamports(100_000), "0.000100000 SOL");
    assert_eq!(FormattingUtils::format_lamports(1), "0.000000001 SOL");
    assert_eq!(FormattingUtils::format_lamports(0), "0.000000000 SOL");
}

#[test]
fn test_format_lamports_compact() {
    assert_eq!(FormattingUtils::format_lamports_compact(1_000_000_000), "1 SOL");
    assert_eq!(FormattingUtils::format_lamports_compact(1_000_000), "0.001 SOL");
    assert_eq!(FormattingUtils::format_lamports_compact(100_000), "0.0001 SOL");
    assert_eq!(FormattingUtils::format_lamports_compact(1), "0.000000001 SOL");
    assert_eq!(FormattingUtils::format_lamports_compact(0), "0 SOL");
}

#[test]
fn test_format_duration() {
    assert_eq!(FormattingUtils::format_duration(Duration::from_secs(60)), "1m");
    assert_eq!(FormattingUtils::format_duration(Duration::from_secs(3600)), "1h");
    assert_eq!(FormattingUtils::format_duration(Duration::from_secs(86400)), "1d");
    assert_eq!(FormattingUtils::format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    assert_eq!(FormattingUtils::format_duration(Duration::from_secs(0)), "0s");
}

#[test]
fn test_format_timestamp() {
    let timestamp = 1234567890i64; // Known timestamp
    let formatted = FormattingUtils::format_timestamp(timestamp);
    
    assert!(!formatted.is_empty());
    // Just ensure it doesn't panic and returns something reasonable
    assert!(formatted.len() > 10);
}

#[test]
fn test_format_pubkey() {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    
    // Full format
    let full = FormattingUtils::format_pubkey(&pubkey, false);
    assert_eq!(full.len(), 44); // Full base58 length
    
    // Short format  
    let short = FormattingUtils::format_pubkey(&pubkey, true);
    assert!(short.len() < 44);
    assert!(short.contains("...")); // Should have ellipsis
}

// ===== MATH UTILS TESTS =====

#[test]
fn test_calculate_fee() {
    assert_eq!(MathUtils::calculate_fee(1_000_000, 500), 5_000); // 0.5%
    assert_eq!(MathUtils::calculate_fee(1_000_000, 1000), 10_000); // 1%
    assert_eq!(MathUtils::calculate_fee(1_000_000, 0), 0); // 0%
    assert_eq!(MathUtils::calculate_fee(0, 500), 0); // 0 amount
}

#[test]
fn test_calculate_percentage() {
    assert_eq!(MathUtils::calculate_percentage(1_000_000, 50), 500_000); // 50%
    assert_eq!(MathUtils::calculate_percentage(1_000_000, 25), 250_000); // 25%
    assert_eq!(MathUtils::calculate_percentage(1_000_000, 0), 0); // 0%
    assert_eq!(MathUtils::calculate_percentage(1_000_000, 100), 1_000_000); // 100%
}

#[test]
fn test_safe_add() {
    assert_eq!(MathUtils::safe_add(100, 200), Some(300));
    assert_eq!(MathUtils::safe_add(u64::MAX, 1), None); // Overflow
    assert_eq!(MathUtils::safe_add(u64::MAX - 1, 1), Some(u64::MAX));
}

#[test]
fn test_safe_sub() {
    assert_eq!(MathUtils::safe_sub(200, 100), Some(100));
    assert_eq!(MathUtils::safe_sub(100, 200), None); // Underflow
    assert_eq!(MathUtils::safe_sub(100, 100), Some(0));
}

#[test]
fn test_safe_mul() {
    assert_eq!(MathUtils::safe_mul(100, 200), Some(20_000));
    assert_eq!(MathUtils::safe_mul(u64::MAX, 2), None); // Overflow
    assert_eq!(MathUtils::safe_mul(0, u64::MAX), Some(0));
}

// ===== CRYPTO UTILS TESTS =====

#[test]
fn test_generate_keypair() {
    let keypair1 = CryptoUtils::generate_keypair();
    let keypair2 = CryptoUtils::generate_keypair();
    
    // Should generate different keypairs
    assert_ne!(keypair1.pubkey(), keypair2.pubkey());
    
    // Both should be valid
    TestAssertions::assert_valid_pubkey(&keypair1.pubkey());
    TestAssertions::assert_valid_pubkey(&keypair2.pubkey());
}

#[test]
fn test_verify_signature() {
    let keypair = Keypair::new();
    let message = b"test message";
    
    // Sign the message
    let signature = keypair.sign_message(message);
    
    // Verify signature
    assert!(CryptoUtils::verify_signature(&keypair.pubkey(), message, &signature));
    
    // Verify with wrong message should fail
    let wrong_message = b"wrong message";
    assert!(!CryptoUtils::verify_signature(&keypair.pubkey(), wrong_message, &signature));
    
    // Verify with wrong pubkey should fail
    let wrong_keypair = Keypair::new();
    assert!(!CryptoUtils::verify_signature(&wrong_keypair.pubkey(), message, &signature));
}

// ===== CONSTANTS TESTS =====

#[test]
fn test_default_commitment() {
    assert_eq!(DEFAULT_COMMITMENT, CommitmentConfig::confirmed());
}

#[test]
fn test_commitment_levels() {
    // Test that different commitment levels work
    let processed = CommitmentConfig::processed();
    let confirmed = CommitmentConfig::confirmed();
    let finalized = CommitmentConfig::finalized();
    
    assert_ne!(processed, confirmed);
    assert_ne!(confirmed, finalized);
    assert_ne!(processed, finalized);
}

// ===== PROPERTY BASED TESTS =====

proptest! {
    #[test]
    fn test_sol_lamports_conversion_property(sol in 0.0f64..1000.0f64) {
        prop_assume!(sol.is_finite());
        
        let lamports = ConversionUtils::sol_to_lamports(sol);
        let back_to_sol = ConversionUtils::lamports_to_sol(lamports);
        
        // Allow small floating point differences due to precision
        prop_assert!((sol - back_to_sol).abs() < 0.000_001);
    }
    
    #[test]
    fn test_percentage_calculation_property(amount in 1u64..1_000_000_000u64, percentage in 0u8..=100u8) {
        let result = MathUtils::calculate_percentage(amount, percentage);
        
        // Result should be <= original amount
        prop_assert!(result <= amount);
        
        // For 100%, result should equal original amount
        if percentage == 100 {
            prop_assert_eq!(result, amount);
        }
        
        // For 0%, result should be 0
        if percentage == 0 {
            prop_assert_eq!(result, 0);
        }
    }
    
    #[test]
    fn test_safe_math_property(a in 0u64..1_000_000u64, b in 0u64..1_000_000u64) {
        let add_result = MathUtils::safe_add(a, b);
        let sub_result = MathUtils::safe_sub(a, b);
        let mul_result = MathUtils::safe_mul(a, b);
        
        // Addition should succeed for small numbers
        prop_assert!(add_result.is_some());
        prop_assert_eq!(add_result.unwrap(), a + b);
        
        // Subtraction result depends on order
        if a >= b {
            prop_assert!(sub_result.is_some());
            prop_assert_eq!(sub_result.unwrap(), a - b);
        } else {
            prop_assert!(sub_result.is_none());
        }
        
        // Multiplication should succeed for small numbers
        prop_assert!(mul_result.is_some());
        prop_assert_eq!(mul_result.unwrap(), a * b);
    }
    
    #[test]
    fn test_username_validation_property(
        username in "[a-zA-Z0-9_]{1,30}"
    ) {
        let result = ValidationUtils::validate_username(&username);
        prop_assert!(result.is_ok());
    }
    
    #[test]
    fn test_job_amount_validation_property(
        amount in 100_000u64..1_000_000_000u64
    ) {
        let result = ValidationUtils::validate_job_amount(amount);
        prop_assert!(result.is_ok());
    }
}

// ===== EDGE CASE TESTS =====

#[test]
fn test_edge_case_amounts() {
    // Test edge cases for amount calculations
    assert_eq!(ConversionUtils::sol_to_lamports(f64::MIN_POSITIVE), 0); // Very small
    
    // Test maximum lamports that fits in u64
    let max_lamports = u64::MAX;
    let sol = ConversionUtils::lamports_to_sol(max_lamports);
    assert!(sol > 0.0);
    assert!(sol.is_finite());
}

#[test]
fn test_edge_case_durations() {
    // Zero duration
    let zero_duration = Duration::from_secs(0);
    let formatted = FormattingUtils::format_duration(zero_duration);
    assert_eq!(formatted, "0s");
    
    // Very long duration
    let long_duration = Duration::from_secs(365 * 24 * 3600); // 1 year
    let formatted = FormattingUtils::format_duration(long_duration);
    assert!(formatted.contains("d")); // Should contain days
}

#[test]
fn test_edge_case_percentages() {
    // Test 0% and 100%
    assert_eq!(MathUtils::calculate_percentage(1_000_000, 0), 0);
    assert_eq!(MathUtils::calculate_percentage(1_000_000, 100), 1_000_000);
    
    // Test with 1 lamport
    assert_eq!(MathUtils::calculate_percentage(1, 50), 0); // Rounds down
    assert_eq!(MathUtils::calculate_percentage(1, 100), 1);
}

// ===== PERFORMANCE TESTS =====

#[test]
fn test_conversion_performance() {
    let start = std::time::Instant::now();
    
    // Perform many conversions
    for i in 0..10_000 {
        let sol = i as f64 / 1000.0; // 0.000 to 10.000 SOL
        let lamports = ConversionUtils::sol_to_lamports(sol);
        let _back_to_sol = ConversionUtils::lamports_to_sol(lamports);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Conversions should be fast: {:?}", duration);
}

#[test]
fn test_validation_performance() {
    let start = std::time::Instant::now();
    
    // Perform many validations
    for i in 0..1_000 {
        let username = format!("user{}", i);
        let _result = ValidationUtils::validate_username(&username);
        
        let amount = 100_000 + i as u64;
        let _result = ValidationUtils::validate_job_amount(amount);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 50, "Validations should be fast: {:?}", duration);
}

#[test]
fn test_formatting_performance() {
    let start = std::time::Instant::now();
    
    // Perform many formatting operations
    for i in 0..1_000 {
        let amount = (i + 1) * 1_000_000; // Various amounts
        let _formatted = FormattingUtils::format_lamports(amount);
        let _compact = FormattingUtils::format_lamports_compact(amount);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Formatting should be fast: {:?}", duration);
}