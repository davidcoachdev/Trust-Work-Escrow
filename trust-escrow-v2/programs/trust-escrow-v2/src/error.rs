//! Custom error codes for Trust Work Escrow v2

use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("User account already exists")]
    UserAlreadyExists,
    
    #[msg("Wallet already associated to this user")]
    WalletAlreadyAssociated,
    
    #[msg("Wallet not associated to this user")]
    WalletNotAssociated,
    
    #[msg("No active wallet set")]
    NoActiveWallet,
    
    #[msg("Invalid number of wallets")]
    InvalidWalletCount,
    
    #[msg("Maximum number of wallets reached (10)")]
    MaxWalletsReached,
    
    #[msg("Maximum number of arbiters reached (50)")]
    MaxArbitersReached,
    
    #[msg("Maximum number of multisig owners reached (5)")]
    MaxMultisigOwnersReached,
    
    #[msg("Multisig threshold must be at least 1")]
    InvalidMultisigThreshold,
    
    #[msg("Multisig threshold exceeds number of owners")]
    ThresholdExceedsOwners,
    
    #[msg("Not authorized - not the user owner")]
    NotAuthorized,
    
    #[msg("Not authorized - not the admin")]
    NotAdmin,
    
    #[msg("Not authorized - not a valid arbiter")]
    NotArbiter,
    
    #[msg("Not authorized - not the job client")]
    NotJobClient,
    
    #[msg("Not authorized - not the job freelancer")]
    NotJobFreelancer,
    
    #[msg("Cannot accept your own job")]
    CannotAcceptOwnJob,
    
    #[msg("Program is paused")]
    ProgramPaused,
    
    #[msg("Invalid job status for this operation")]
    InvalidJobStatus,
    
    #[msg("Amount too small")]
    AmountTooSmall,
    
    #[msg("Title cannot be empty")]
    EmptyTitle,
    
    #[msg("Title exceeds maximum length (100)")]
    TitleTooLong,
    
    #[msg("Description exceeds maximum length (500)")]
    DescriptionTooLong,
    
    #[msg("Dispute reason cannot be empty")]
    EmptyDisputeReason,
    
    #[msg("Dispute reason exceeds maximum length (200)")]
    DisputeReasonTooLong,
    
    #[msg("Username cannot be empty")]
    EmptyUsername,
    
    #[msg("Username exceeds maximum length (32)")]
    UsernameTooLong,
    
    #[msg("Bio exceeds maximum length (500)")]
    BioTooLong,
    
    #[msg("Invalid fee percentage (must be 0-100)")]
    InvalidFeePercentage,
    
    #[msg("Invalid freelancer percentage")]
    InvalidFreelancerPercent,
    
    #[msg("Insufficient funds for this operation")]
    InsufficientFunds,
    
    #[msg("Treasury address mismatch")]
    InvalidTreasury,
    
    #[msg("Job not funded")]
    JobNotFunded,
    
    #[msg("No freelancer assigned")]
    NoFreelancerAssigned,
    
    #[msg("No arbiter assigned")]
    NoArbiterAssigned,
}