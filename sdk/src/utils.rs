//! Utility functions and helpers for the Trust Escrow SDK
//!
//! This module provides common utility functions used throughout the SDK,
//! including transaction building helpers, validation functions, and
//! convenience methods for working with Solana primitives.

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature,
    transaction::Transaction,
};

use crate::error::{EscrowError, Result};

/// Default commitment level for transactions
pub const DEFAULT_COMMITMENT: CommitmentConfig = CommitmentConfig::confirmed();

/// Default transaction timeout in seconds
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 60;

/// Utility functions for transaction handling
pub struct TransactionUtils;

impl TransactionUtils {
    /// Send and confirm transaction with retry logic
    pub async fn send_and_confirm_transaction(
        client: &RpcClient,
        transaction: &Transaction,
        commitment: CommitmentConfig,
    ) -> Result<Signature> {
        let signature = client
            .send_and_confirm_transaction_with_spinner_and_commitment(transaction, commitment)
            .map_err(|e| EscrowError::network_error(format!("Transaction failed: {}", e)))?;

        Ok(signature)
    }

    /// Get recent blockhash for transaction
    pub async fn get_recent_blockhash(client: &RpcClient) -> Result<solana_sdk::hash::Hash> {
        client
            .get_latest_blockhash()
            .map_err(|e| EscrowError::network_error(format!("Failed to get blockhash: {}", e)))
    }

    /// Estimate transaction fee
    pub async fn estimate_fee(client: &RpcClient, transaction: &Transaction) -> Result<u64> {
        client
            .get_fee_for_message(&transaction.message)
            .map_err(|e| EscrowError::network_error(format!("Failed to estimate fee: {}", e)))
    }

    /// Check if account exists
    pub async fn account_exists(client: &RpcClient, pubkey: &Pubkey) -> Result<bool> {
        match client.get_account_with_commitment(pubkey, DEFAULT_COMMITMENT) {
            Ok(response) => Ok(response.value.is_some()),
            Err(_) => Ok(false), // Account doesn't exist
        }
    }

    /// Get account balance in lamports
    pub async fn get_balance(client: &RpcClient, pubkey: &Pubkey) -> Result<u64> {
        client
            .get_balance_with_commitment(pubkey, DEFAULT_COMMITMENT)
            .map(|response| response.value)
            .map_err(|e| EscrowError::network_error(format!("Failed to get balance: {}", e)))
    }

    /// Wait for transaction confirmation
    pub async fn wait_for_confirmation(
        client: &RpcClient,
        signature: &Signature,
        commitment: CommitmentConfig,
    ) -> Result<bool> {
        for _ in 0..30 {
            // Wait up to 30 seconds
            match client.confirm_transaction_with_commitment(signature, commitment) {
                Ok(response) => return Ok(response.value),
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
            }
        }

        Err(EscrowError::network_error(
            "Transaction confirmation timeout",
        ))
    }
}

/// Validation utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate username according to escrow rules
    pub fn validate_username(username: &str) -> Result<()> {
        if username.is_empty() {
            return Err(EscrowError::invalid_parameter("Username cannot be empty"));
        }

        if username.len() > 32 {
            return Err(EscrowError::invalid_parameter(
                "Username cannot exceed 32 characters",
            ));
        }

        // Check for valid characters (alphanumeric, underscore, hyphen)
        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(EscrowError::invalid_parameter(
                "Username can only contain letters, numbers, underscores, and hyphens",
            ));
        }

        Ok(())
    }

    /// Validate bio text
    pub fn validate_bio(bio: &str) -> Result<()> {
        if bio.len() > 500 {
            return Err(EscrowError::invalid_parameter(
                "Bio cannot exceed 500 characters",
            ));
        }
        Ok(())
    }

    /// Validate job title
    pub fn validate_job_title(title: &str) -> Result<()> {
        if title.trim().is_empty() {
            return Err(EscrowError::invalid_parameter("Job title cannot be empty"));
        }

        if title.len() > 100 {
            return Err(EscrowError::invalid_parameter(
                "Job title cannot exceed 100 characters",
            ));
        }

        Ok(())
    }

    /// Validate job description
    pub fn validate_job_description(description: &str) -> Result<()> {
        if description.len() > 2000 {
            return Err(EscrowError::invalid_parameter(
                "Job description cannot exceed 2000 characters",
            ));
        }
        Ok(())
    }

    /// Validate team name
    pub fn validate_team_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(EscrowError::invalid_parameter("Team name cannot be empty"));
        }

        if name.len() > 50 {
            return Err(EscrowError::invalid_parameter(
                "Team name cannot exceed 50 characters",
            ));
        }

        Ok(())
    }

    /// Validate SOL amount (in lamports)
    pub fn validate_amount(amount: u64, min_amount: u64) -> Result<()> {
        if amount < min_amount {
            return Err(EscrowError::invalid_parameter(format!(
                "Amount must be at least {} lamports",
                min_amount
            )));
        }
        Ok(())
    }

    /// Validate percentage (0-100)
    pub fn validate_percentage(percentage: u8) -> Result<()> {
        if percentage > 100 {
            return Err(EscrowError::invalid_parameter(
                "Percentage must be between 0 and 100",
            ));
        }
        Ok(())
    }
}

/// Conversion utilities
pub struct ConversionUtils;

impl ConversionUtils {
    /// Convert SOL to lamports
    pub fn sol_to_lamports(sol: f64) -> u64 {
        (sol * 1_000_000_000.0) as u64
    }

    /// Convert lamports to SOL
    pub fn lamports_to_sol(lamports: u64) -> f64 {
        lamports as f64 / 1_000_000_000.0
    }

    /// Format lamports as SOL string
    pub fn format_sol(lamports: u64) -> String {
        format!("{:.9} SOL", Self::lamports_to_sol(lamports))
    }

    /// Parse SOL string to lamports
    pub fn parse_sol(sol_str: &str) -> Result<u64> {
        let cleaned = sol_str.replace("SOL", "").trim().to_string();
        let sol: f64 = cleaned
            .parse()
            .map_err(|_| EscrowError::invalid_parameter("Invalid SOL amount format"))?;

        if sol < 0.0 {
            return Err(EscrowError::invalid_parameter(
                "SOL amount cannot be negative",
            ));
        }

        Ok(Self::sol_to_lamports(sol))
    }

    /// Convert timestamp to human readable date
    pub fn timestamp_to_date(timestamp: i64) -> String {
        if timestamp == 0 {
            return "Never".to_string();
        }

        // This is a simple implementation - in production you'd want to use chrono
        format!("Timestamp: {}", timestamp)
    }

    /// Get current Unix timestamp
    pub fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }
}

/// Keypair and wallet utilities
pub struct WalletUtils;

impl WalletUtils {
    /// Generate a new keypair
    pub fn generate_keypair() -> solana_sdk::signature::Keypair {
        solana_sdk::signature::Keypair::new()
    }

    /// Load keypair from bytes
    pub fn load_keypair_from_bytes(bytes: &[u8]) -> Result<solana_sdk::signature::Keypair> {
        solana_sdk::signature::Keypair::from_bytes(bytes)
            .map_err(|e| EscrowError::invalid_parameter(format!("Invalid keypair bytes: {}", e)))
    }

    /// Load keypair from base58 string
    pub fn load_keypair_from_base58(base58: &str) -> Result<solana_sdk::signature::Keypair> {
        let bytes = bs58::decode(base58)
            .into_vec()
            .map_err(|e| EscrowError::invalid_parameter(format!("Invalid base58: {}", e)))?;

        Self::load_keypair_from_bytes(&bytes)
    }

    /// Convert keypair to base58 string
    pub fn keypair_to_base58(keypair: &solana_sdk::signature::Keypair) -> String {
        bs58::encode(keypair.to_bytes()).into_string()
    }

    /// Validate pubkey string
    pub fn validate_pubkey_string(pubkey_str: &str) -> Result<Pubkey> {
        pubkey_str
            .parse::<Pubkey>()
            .map_err(|e| EscrowError::invalid_parameter(format!("Invalid pubkey: {}", e)))
    }
}

/// Error handling utilities
pub struct ErrorUtils;

impl ErrorUtils {
    /// Check if error is a program error with specific code
    pub fn is_program_error(error: &EscrowError, expected_code: u32) -> bool {
        match error {
            EscrowError::Contract { code, .. } => *code == expected_code,
            _ => false,
        }
    }

    /// Extract error code from EscrowError
    pub fn extract_error_code(error: &EscrowError) -> Option<u32> {
        match error {
            EscrowError::Contract { code, .. } => Some(*code),
            _ => None,
        }
    }

    /// Get user-friendly error message
    pub fn user_friendly_message(error: &EscrowError) -> String {
        match error {
            EscrowError::InvalidParameter { msg } => format!("Invalid input: {}", msg),
            EscrowError::Contract { msg, .. } => format!("Contract error: {}", msg),
            EscrowError::Network { msg } => format!("Network error: {}", msg),
            EscrowError::InsufficientFunds {
                required,
                available,
            } => {
                format!(
                    "Insufficient funds: need {} SOL, have {} SOL",
                    ConversionUtils::lamports_to_sol(*required),
                    ConversionUtils::lamports_to_sol(*available)
                )
            }
            EscrowError::NotPermitted { reason } => format!("Not allowed: {}", reason),
            _ => format!("Error: {}", error),
        }
    }
}

/// Development and testing utilities
#[cfg(any(test, feature = "dev"))]
pub struct DevUtils;

#[cfg(any(test, feature = "dev"))]
impl DevUtils {
    /// Create a test RPC client (localhost)
    pub fn test_rpc_client() -> RpcClient {
        RpcClient::new("http://localhost:8899".to_string())
    }

    /// Create a test keypair with some deterministic seed
    pub fn test_keypair(seed: u8) -> solana_sdk::signature::Keypair {
        let mut seed_array = [0u8; 32];
        seed_array[0] = seed;
        solana_sdk::signature::Keypair::from_bytes(&seed_array).unwrap()
    }

    /// Airdrop SOL to account (devnet/localnet only)
    pub async fn airdrop_sol(
        client: &RpcClient,
        pubkey: &Pubkey,
        lamports: u64,
    ) -> Result<Signature> {
        client
            .request_airdrop(pubkey, lamports)
            .map_err(|e| EscrowError::network_error(format!("Airdrop failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_utils() {
        // Username validation
        assert!(ValidationUtils::validate_username("valid_user").is_ok());
        assert!(ValidationUtils::validate_username("").is_err());
        assert!(ValidationUtils::validate_username("a".repeat(33).as_str()).is_err());
        assert!(ValidationUtils::validate_username("user@domain").is_err());

        // Amount validation
        assert!(ValidationUtils::validate_amount(1000, 500).is_ok());
        assert!(ValidationUtils::validate_amount(100, 500).is_err());

        // Percentage validation
        assert!(ValidationUtils::validate_percentage(50).is_ok());
        assert!(ValidationUtils::validate_percentage(101).is_err());
    }

    #[test]
    fn test_conversion_utils() {
        // SOL/lamports conversion
        let sol = 1.5;
        let lamports = ConversionUtils::sol_to_lamports(sol);
        let converted_back = ConversionUtils::lamports_to_sol(lamports);

        assert_eq!(lamports, 1_500_000_000);
        assert!((converted_back - sol).abs() < 0.000000001);

        // Format SOL
        let formatted = ConversionUtils::format_sol(1_500_000_000);
        assert!(formatted.contains("1.5"));
        assert!(formatted.contains("SOL"));

        // Parse SOL
        let parsed = ConversionUtils::parse_sol("2.5 SOL").unwrap();
        assert_eq!(parsed, 2_500_000_000);
    }

    #[test]
    fn test_wallet_utils() {
        let keypair = WalletUtils::generate_keypair();
        let base58 = WalletUtils::keypair_to_base58(&keypair);
        let loaded = WalletUtils::load_keypair_from_base58(&base58).unwrap();

        assert_eq!(keypair.pubkey(), loaded.pubkey());
    }

    #[test]
    fn test_error_utils() {
        let error = EscrowError::contract_error(6001, "User not found");

        assert!(ErrorUtils::is_program_error(&error, 6001));
        assert!(!ErrorUtils::is_program_error(&error, 6000));

        let code = ErrorUtils::extract_error_code(&error);
        assert_eq!(code, Some(6001));

        let message = ErrorUtils::user_friendly_message(&error);
        assert!(message.contains("Contract error"));
    }
}
