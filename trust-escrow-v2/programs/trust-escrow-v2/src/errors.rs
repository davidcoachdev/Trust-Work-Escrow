use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Unauthorized: Only admin can perform this action")]
    UnauthorizedAdmin,

    #[msg("Unauthorized: Only the account owner can perform this action")]
    UnauthorizedOwner,

    #[msg("Program is paused")]
    ProgramPaused,

    #[msg("Invalid fee percentage")]
    InvalidFeePercentage,

    #[msg("Wallet already added")]
    WalletAlreadyAdded,

    #[msg("Maximum wallets reached (5)")]
    MaxWalletsReached,

    #[msg("Cannot remove primary wallet")]
    CannotRemovePrimaryWallet,

    #[msg("Invalid wallet index")]
    InvalidWalletIndex,

    #[msg("User already exists")]
    UserAlreadyExists,

    #[msg("User does not exist")]
    UserNotFound,

    #[msg("Invalid treasury wallet")]
    InvalidTreasuryWallet,

    #[msg("Cannot set same wallet as active")]
    SameActiveWallet,

    #[msg("Invalid nonce")]
    InvalidNonce,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
