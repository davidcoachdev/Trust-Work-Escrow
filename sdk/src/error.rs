//! Error types and handling for the Trust Escrow SDK
//!
//! This module provides comprehensive error handling that maps Anchor/Solana errors
//! to semantic error types with proper context for consuming applications.

use thiserror::Error;

/// Result type alias for SDK operations
pub type Result<T> = std::result::Result<T, EscrowError>;

/// Comprehensive error enum for Trust Escrow SDK operations
#[derive(Error, Debug)]
pub enum EscrowError {
    /// Anchor client errors (RPC, instruction building, etc.)
    #[error("Anchor client error: {0}")]
    Anchor(#[from] anchor_client::ClientError),

    /// Solana client errors (network, transaction, etc.)
    #[error("Solana client error: {0}")]
    Solana(#[from] solana_client::client_error::ClientError),

    /// Solana SDK errors (pubkey parsing, etc.)
    #[error("Solana SDK error: {0}")]
    SolanaSDK(#[from] solana_sdk::pubkey::ParsePubkeyError),

    /// Invalid parameter validation errors
    #[error("Invalid parameter: {msg}")]
    InvalidParameter { msg: String },

    /// Trust Escrow contract-specific errors mapped from ErrorCode
    #[error("Contract error {code}: {msg}")]
    Contract { code: u32, msg: String },

    /// PDA derivation errors
    #[error("PDA derivation failed: {msg}")]
    PdaDerivation { msg: String },

    /// Account not found or invalid state
    #[error("Account error: {msg}")]
    Account { msg: String },

    /// Insufficient funds for operation
    #[error("Insufficient funds: required {required} lamports, available {available}")]
    InsufficientFunds { required: u64, available: u64 },

    /// Operation not permitted due to state or permissions
    #[error("Operation not permitted: {reason}")]
    NotPermitted { reason: String },

    /// Network or RPC related errors
    #[error("Network error: {msg}")]
    Network { msg: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Generic errors that don't fit other categories
    #[error("SDK error: {msg}")]
    Sdk { msg: String },
}

impl EscrowError {
    /// Create a new InvalidParameter error
    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        EscrowError::InvalidParameter { msg: msg.into() }
    }

    /// Create a new Contract error with code and message
    pub fn contract_error(code: u32, msg: impl Into<String>) -> Self {
        EscrowError::Contract {
            code,
            msg: msg.into(),
        }
    }

    /// Create a new PdaDerivation error
    pub fn pda_derivation(msg: impl Into<String>) -> Self {
        EscrowError::PdaDerivation { msg: msg.into() }
    }

    /// Create a new Account error
    pub fn account_error(msg: impl Into<String>) -> Self {
        EscrowError::Account { msg: msg.into() }
    }

    /// Create a new InsufficientFunds error
    pub fn insufficient_funds(required: u64, available: u64) -> Self {
        EscrowError::InsufficientFunds {
            required,
            available,
        }
    }

    /// Create a new NotPermitted error
    pub fn not_permitted(reason: impl Into<String>) -> Self {
        EscrowError::NotPermitted {
            reason: reason.into(),
        }
    }

    /// Create a new Network error
    pub fn network_error(msg: impl Into<String>) -> Self {
        EscrowError::Network { msg: msg.into() }
    }

    /// Create a new generic SDK error
    pub fn sdk_error(msg: impl Into<String>) -> Self {
        EscrowError::Sdk { msg: msg.into() }
    }
}

/// Trust Escrow v2 contract error codes that map to EscrowError::Contract
///
/// These codes should match the ErrorCode enum in the smart contract
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractErrorCode {
    // User errors
    UserAlreadyExists = 6000,
    UserNotFound = 6001,
    InvalidUsername = 6002,
    WalletAlreadyAssociated = 6003,
    WalletNotAssociated = 6004,
    MaxWalletsReached = 6005,

    // Team errors
    TeamAlreadyExists = 6006,
    TeamNotFound = 6007,
    MemberAlreadyExists = 6008,
    MemberNotFound = 6009,
    NotTeamOwner = 6010,

    // Job errors
    JobNotFound = 6011,
    InvalidJobAmount = 6012,
    JobNotOpen = 6013,
    JobAlreadyStarted = 6014,
    NotJobClient = 6015,
    NotJobFreelancer = 6016,
    ApplicationNotFound = 6017,
    ApplicationAlreadyExists = 6018,

    // Dispute errors
    DisputeAlreadyExists = 6019,
    DisputeNotFound = 6020,
    DisputeNotActive = 6021,
    NotDisputed = 6022,
    InvalidArbiter = 6023,

    // Milestone errors
    MilestoneNotFound = 6024,
    InvalidMilestoneAmount = 6025,
    MilestoneNotSubmitted = 6026,
    MaxMilestonesReached = 6027,

    // System errors
    ProgramPaused = 6028,
    Unauthorized = 6029,
    InsufficientFunds = 6030,
    InvalidCalculation = 6031,

    // Config errors
    ConfigAlreadyInitialized = 6032,
    ConfigNotInitialized = 6033,
    InvalidFeePercentage = 6034,
    InvalidTreasuryAddress = 6035,
}

impl ContractErrorCode {
    /// Convert error code to human-readable message
    pub fn message(&self) -> &'static str {
        match self {
            // User errors
            Self::UserAlreadyExists => "User account already exists",
            Self::UserNotFound => "User account not found",
            Self::InvalidUsername => "Username is invalid or too long",
            Self::WalletAlreadyAssociated => "Wallet is already associated with this user",
            Self::WalletNotAssociated => "Wallet is not associated with this user",
            Self::MaxWalletsReached => "Maximum number of wallets (5) already associated",

            // Team errors
            Self::TeamAlreadyExists => "Team already exists for this owner",
            Self::TeamNotFound => "Team not found",
            Self::MemberAlreadyExists => "Member already exists in team",
            Self::MemberNotFound => "Member not found in team",
            Self::NotTeamOwner => "Only team owner can perform this action",

            // Job errors
            Self::JobNotFound => "Job not found",
            Self::InvalidJobAmount => "Job amount is below minimum required",
            Self::JobNotOpen => "Job is not accepting applications",
            Self::JobAlreadyStarted => "Job has already been started",
            Self::NotJobClient => "Only job client can perform this action",
            Self::NotJobFreelancer => "Only assigned freelancer can perform this action",
            Self::ApplicationNotFound => "Application not found for this job",
            Self::ApplicationAlreadyExists => "Application already exists for this job",

            // Dispute errors
            Self::DisputeAlreadyExists => "Dispute already exists for this job",
            Self::DisputeNotFound => "Dispute not found",
            Self::DisputeNotActive => "Dispute is not in active state",
            Self::NotDisputed => "Job is not in disputed state",
            Self::InvalidArbiter => "Invalid or unauthorized arbiter",

            // Milestone errors
            Self::MilestoneNotFound => "Milestone not found",
            Self::InvalidMilestoneAmount => "Milestone amount exceeds job total",
            Self::MilestoneNotSubmitted => "Milestone has not been submitted",
            Self::MaxMilestonesReached => "Maximum number of milestones (20) reached",

            // System errors
            Self::ProgramPaused => "Program is currently paused",
            Self::Unauthorized => "Unauthorized access",
            Self::InsufficientFunds => "Insufficient funds for operation",
            Self::InvalidCalculation => "Invalid calculation result",

            // Config errors
            Self::ConfigAlreadyInitialized => "Config has already been initialized",
            Self::ConfigNotInitialized => "Config has not been initialized",
            Self::InvalidFeePercentage => "Fee percentage must be between 0-100",
            Self::InvalidTreasuryAddress => "Invalid treasury address",
        }
    }

    /// Convert from u32 error code to enum variant
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            6000 => Some(Self::UserAlreadyExists),
            6001 => Some(Self::UserNotFound),
            6002 => Some(Self::InvalidUsername),
            6003 => Some(Self::WalletAlreadyAssociated),
            6004 => Some(Self::WalletNotAssociated),
            6005 => Some(Self::MaxWalletsReached),
            6006 => Some(Self::TeamAlreadyExists),
            6007 => Some(Self::TeamNotFound),
            6008 => Some(Self::MemberAlreadyExists),
            6009 => Some(Self::MemberNotFound),
            6010 => Some(Self::NotTeamOwner),
            6011 => Some(Self::JobNotFound),
            6012 => Some(Self::InvalidJobAmount),
            6013 => Some(Self::JobNotOpen),
            6014 => Some(Self::JobAlreadyStarted),
            6015 => Some(Self::NotJobClient),
            6016 => Some(Self::NotJobFreelancer),
            6017 => Some(Self::ApplicationNotFound),
            6018 => Some(Self::ApplicationAlreadyExists),
            6019 => Some(Self::DisputeAlreadyExists),
            6020 => Some(Self::DisputeNotFound),
            6021 => Some(Self::DisputeNotActive),
            6022 => Some(Self::NotDisputed),
            6023 => Some(Self::InvalidArbiter),
            6024 => Some(Self::MilestoneNotFound),
            6025 => Some(Self::InvalidMilestoneAmount),
            6026 => Some(Self::MilestoneNotSubmitted),
            6027 => Some(Self::MaxMilestonesReached),
            6028 => Some(Self::ProgramPaused),
            6029 => Some(Self::Unauthorized),
            6030 => Some(Self::InsufficientFunds),
            6031 => Some(Self::InvalidCalculation),
            6032 => Some(Self::ConfigAlreadyInitialized),
            6033 => Some(Self::ConfigNotInitialized),
            6034 => Some(Self::InvalidFeePercentage),
            6035 => Some(Self::InvalidTreasuryAddress),
            _ => None,
        }
    }
}

impl From<ContractErrorCode> for EscrowError {
    fn from(code: ContractErrorCode) -> Self {
        EscrowError::contract_error(code as u32, code.message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = EscrowError::invalid_parameter("Test message");
        assert!(matches!(error, EscrowError::InvalidParameter { .. }));
    }

    #[test]
    fn test_contract_error_codes() {
        let code = ContractErrorCode::UserAlreadyExists;
        assert_eq!(code as u32, 6000);
        assert_eq!(code.message(), "User account already exists");

        let from_code = ContractErrorCode::from_code(6000);
        assert_eq!(from_code, Some(ContractErrorCode::UserAlreadyExists));
    }

    #[test]
    fn test_error_conversion() {
        let contract_error: EscrowError = ContractErrorCode::JobNotFound.into();
        assert!(matches!(
            contract_error,
            EscrowError::Contract { code: 6011, .. }
        ));
    }
}
