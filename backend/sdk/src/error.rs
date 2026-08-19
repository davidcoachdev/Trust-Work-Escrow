//! Error types for the Trust Escrow v3 SDK.
//!
//! Maps Anchor/Solana errors to typed [`BackendError`] variants and mirrors the
//! on-chain `ErrorCode` enum from `trust-escrow-v3` so contract failures are
//! identifiable without panicking.

use thiserror::Error;

/// Result alias for SDK operations.
pub type Result<T> = std::result::Result<T, BackendError>;

/// Typed error for the Trust Escrow v3 SDK.
#[derive(Error, Debug)]
pub enum BackendError {
    /// A contract (program) error identified by its on-chain [`ErrorCode`].
    #[error("contract error {0:?}")]
    Contract(ErrorCode),

    /// Anchor client error (RPC, instruction building, etc.).
    #[cfg(feature = "solana")]
    #[error("anchor client error: {0}")]
    Anchor(#[from] Box<anchor_client::ClientError>),

    /// Solana RPC client error.
    #[cfg(feature = "solana")]
    #[error("solana client error: {0}")]
    Solana(#[from] Box<solana_client::client_error::ClientError>),

    /// Solana SDK error (e.g. pubkey parsing).
    #[cfg(feature = "solana")]
    #[error("solana sdk error: {0}")]
    SolanaSdk(#[from] solana_sdk::pubkey::ParsePubkeyError),

    /// Keypair could not be loaded from the given path or environment.
    #[error("keypair error: {msg}")]
    Keypair { msg: String },

    /// Configuration (env/path) error.
    #[error("config error: {msg}")]
    Config { msg: String },

    /// Invalid parameter supplied by the caller.
    #[error("invalid parameter: {msg}")]
    InvalidParameter { msg: String },

    /// PDA derivation failed.
    #[error("pda derivation failed: {msg}")]
    PdaDerivation { msg: String },

    /// Account not found or in an invalid state.
    #[error("account error: {msg}")]
    Account { msg: String },

    /// Serialization / deserialization failure.
    #[error("serialization error: {msg}")]
    Serialization { msg: String },

    /// Generic SDK error.
    #[error("sdk error: {msg}")]
    Sdk { msg: String },
}

impl BackendError {
    /// Create a [`BackendError::Contract`] from an on-chain error code.
    pub fn contract(code: ErrorCode) -> Self {
        BackendError::Contract(code)
    }

    /// Create a [`BackendError::Keypair`].
    pub fn keypair_error(msg: impl Into<String>) -> Self {
        BackendError::Keypair { msg: msg.into() }
    }

    /// Create a [`BackendError::Config`].
    pub fn config_error(msg: impl Into<String>) -> Self {
        BackendError::Config { msg: msg.into() }
    }

    /// Create a [`BackendError::InvalidParameter`].
    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        BackendError::InvalidParameter { msg: msg.into() }
    }

    /// Create a [`BackendError::PdaDerivation`].
    pub fn pda_derivation(msg: impl Into<String>) -> Self {
        BackendError::PdaDerivation { msg: msg.into() }
    }

    /// Create a [`BackendError::Account`].
    pub fn account_error(msg: impl Into<String>) -> Self {
        BackendError::Account { msg: msg.into() }
    }

    /// Create a [`BackendError::Serialization`].
    pub fn serialization_error(msg: impl Into<String>) -> Self {
        BackendError::Serialization { msg: msg.into() }
    }

    /// Create a [`BackendError::Sdk`].
    pub fn sdk_error(msg: impl Into<String>) -> Self {
        BackendError::Sdk { msg: msg.into() }
    }
}

/// On-chain error codes for `trust-escrow-v3`.
///
/// Anchor assigns discriminants starting at `6000`, in declaration order, so
/// each variant equals `6000 + index`. Mirrors `pub enum ErrorCode` in the
/// contract's `lib.rs` (including the `#[msg(...)]` text).
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    MathOverflow = 6000,
    ProgramPaused = 6001,
    AmountTooSmall = 6002,
    EmptyTitle = 6003,
    TitleTooLong = 6004,
    DescriptionTooLong = 6005,
    ProposalTooLong = 6006,
    InvalidFeeBps = 6007,
    NotAuthorized = 6008,
    NotJobClient = 6009,
    NotJobFreelancer = 6010,
    CannotWorkOnOwnJob = 6011,
    InvalidJobStatus = 6012,
    JobPaused = 6013,
    JobPausedExpired = 6014,
    CannotPauseWithFreelancer = 6015,
    NoFreelancerAssigned = 6016,
    InvalidTreasury = 6017,
    InvalidJob = 6018,
    DeadlineMustBeFuture = 6019,
    InsufficientFunds = 6020,
    CannotDisputeAtStage = 6021,
    EmptyDisputeReason = 6022,
    EvidenceTooLong = 6023,
    EmptyEvidence = 6024,
    EvidenceLimitReached = 6025,
    InvalidEvidenceIndex = 6026,
    InvalidEvidenceAccount = 6027,
    InvalidEvidenceCleanupAccounts = 6028,
    DisputeAlreadyResolved = 6029,
    DisputeDeadlinePassed = 6030,
    NotValidArbiter = 6031,
    NotArbiter = 6032,
    ArbiterCannotBeParty = 6033,
    InvalidPercent = 6034,
    MilestoneNotFound = 6035,
    MilestoneAlreadyCompleted = 6036,
    InvalidMilestoneIndex = 6037,
    MilestoneAmountExceedsFunds = 6038,
    AllMilestonesRequired = 6039,
    AlreadyApplied = 6040,
    InvalidApplicationIndex = 6041,
    CaseAlreadyOpen = 6042,
    AutoApprovalNotReady = 6043,
    AutoApprovalBlocked = 6044,
    InvalidBootstrapAuthority = 6045,
    ApplicationIndexMismatch = 6046,
    InvalidApplicationAccount = 6047,
    ApplicationNotPending = 6048,
    EmptyProposal = 6049,
    InvalidApplicationCleanupAccounts = 6050,
}

impl ErrorCode {
    /// Human-readable message matching the contract's `#[msg(...)]`.
    pub fn message(&self) -> &'static str {
        match self {
            ErrorCode::MathOverflow => "Math overflow",
            ErrorCode::ProgramPaused => "Program is paused",
            ErrorCode::AmountTooSmall => "Amount too small",
            ErrorCode::EmptyTitle => "Title cannot be empty",
            ErrorCode::TitleTooLong => "Title exceeds maximum length",
            ErrorCode::DescriptionTooLong => "Description exceeds maximum length",
            ErrorCode::ProposalTooLong => "Proposal exceeds maximum length",
            ErrorCode::InvalidFeeBps => "Invalid fee basis points (must be 0-10000)",
            ErrorCode::NotAuthorized => "Not authorized",
            ErrorCode::NotJobClient => "Not authorized - not the job client",
            ErrorCode::NotJobFreelancer => "Not authorized - not the job freelancer",
            ErrorCode::CannotWorkOnOwnJob => "Cannot work on your own job",
            ErrorCode::InvalidJobStatus => "Invalid job status for this operation",
            ErrorCode::JobPaused => "Job pausado",
            ErrorCode::JobPausedExpired => "Job pausado demasiado tiempo; cancela el job",
            ErrorCode::CannotPauseWithFreelancer => {
                "No se puede pausar un job con freelancer asignado"
            }
            ErrorCode::NoFreelancerAssigned => "No freelancer assigned",
            ErrorCode::InvalidTreasury => "Treasury invalido (no coincide con config.treasury)",
            ErrorCode::InvalidJob => "Invalid job id / PDA mismatch",
            ErrorCode::DeadlineMustBeFuture => "Deadline must be in the future",
            ErrorCode::InsufficientFunds => "Insufficient funds in source account",
            ErrorCode::CannotDisputeAtStage => "Cannot raise dispute at this stage",
            ErrorCode::EmptyDisputeReason => "Dispute reason cannot be empty",
            ErrorCode::EvidenceTooLong => "Evidence exceeds maximum length",
            ErrorCode::EmptyEvidence => "Evidence cannot be empty",
            ErrorCode::EvidenceLimitReached => {
                "Dispute already has the maximum number of evidence items"
            }
            ErrorCode::InvalidEvidenceIndex => {
                "Evidence index must equal the next dispute evidence index"
            }
            ErrorCode::InvalidEvidenceAccount => {
                "Evidence account does not match the deterministic dispute PDA"
            }
            ErrorCode::InvalidEvidenceCleanupAccounts => {
                "Evidence cleanup accounts do not match the expected contiguous range"
            }
            ErrorCode::DisputeAlreadyResolved => "Dispute already resolved",
            ErrorCode::DisputeDeadlinePassed => {
                "Dispute deadline passed; only platform advisor can resolve"
            }
            ErrorCode::NotValidArbiter => "Not a valid arbiter",
            ErrorCode::NotArbiter => "Not the assigned arbiter",
            ErrorCode::ArbiterCannotBeParty => {
                "El arbitro no puede ser el cliente ni el freelancer"
            }
            ErrorCode::InvalidPercent => "Payout percent exceeds 100",
            ErrorCode::MilestoneNotFound => "Milestone not found",
            ErrorCode::MilestoneAlreadyCompleted => "Milestone already completed",
            ErrorCode::InvalidMilestoneIndex => {
                "Invalid milestone index (must be sequential: == milestones_total)"
            }
            ErrorCode::MilestoneAmountExceedsFunds => {
                "Milestone amount exceeds remaining job funds"
            }
            ErrorCode::AllMilestonesRequired => "All milestones must be completed before release",
            ErrorCode::AlreadyApplied => "Already applied to this job",
            ErrorCode::InvalidApplicationIndex => "Invalid application index",
            ErrorCode::CaseAlreadyOpen => {
                "A dispute or support ticket is already open for this job"
            }
            ErrorCode::AutoApprovalNotReady => "Auto-approval deadline has not been reached",
            ErrorCode::AutoApprovalBlocked => "Auto-approval is blocked by an open dispute",
            ErrorCode::InvalidBootstrapAuthority => "Invalid bootstrap authority",
            ErrorCode::ApplicationIndexMismatch => {
                "Application index must equal the next job application index"
            }
            ErrorCode::InvalidApplicationAccount => "Application does not belong to this job",
            ErrorCode::ApplicationNotPending => "Application is not pending",
            ErrorCode::EmptyProposal => "Application proposal cannot be empty",
            ErrorCode::InvalidApplicationCleanupAccounts => {
                "Application cleanup accounts do not match the deterministic job range"
            }
        }
    }

    /// Map a raw on-chain error code (u32) to the variant, if known.
    pub fn from_code(code: u32) -> Option<ErrorCode> {
        match code {
            6000 => Some(ErrorCode::MathOverflow),
            6001 => Some(ErrorCode::ProgramPaused),
            6002 => Some(ErrorCode::AmountTooSmall),
            6003 => Some(ErrorCode::EmptyTitle),
            6004 => Some(ErrorCode::TitleTooLong),
            6005 => Some(ErrorCode::DescriptionTooLong),
            6006 => Some(ErrorCode::ProposalTooLong),
            6007 => Some(ErrorCode::InvalidFeeBps),
            6008 => Some(ErrorCode::NotAuthorized),
            6009 => Some(ErrorCode::NotJobClient),
            6010 => Some(ErrorCode::NotJobFreelancer),
            6011 => Some(ErrorCode::CannotWorkOnOwnJob),
            6012 => Some(ErrorCode::InvalidJobStatus),
            6013 => Some(ErrorCode::JobPaused),
            6014 => Some(ErrorCode::JobPausedExpired),
            6015 => Some(ErrorCode::CannotPauseWithFreelancer),
            6016 => Some(ErrorCode::NoFreelancerAssigned),
            6017 => Some(ErrorCode::InvalidTreasury),
            6018 => Some(ErrorCode::InvalidJob),
            6019 => Some(ErrorCode::DeadlineMustBeFuture),
            6020 => Some(ErrorCode::InsufficientFunds),
            6021 => Some(ErrorCode::CannotDisputeAtStage),
            6022 => Some(ErrorCode::EmptyDisputeReason),
            6023 => Some(ErrorCode::EvidenceTooLong),
            6024 => Some(ErrorCode::EmptyEvidence),
            6025 => Some(ErrorCode::EvidenceLimitReached),
            6026 => Some(ErrorCode::InvalidEvidenceIndex),
            6027 => Some(ErrorCode::InvalidEvidenceAccount),
            6028 => Some(ErrorCode::InvalidEvidenceCleanupAccounts),
            6029 => Some(ErrorCode::DisputeAlreadyResolved),
            6030 => Some(ErrorCode::DisputeDeadlinePassed),
            6031 => Some(ErrorCode::NotValidArbiter),
            6032 => Some(ErrorCode::NotArbiter),
            6033 => Some(ErrorCode::ArbiterCannotBeParty),
            6034 => Some(ErrorCode::InvalidPercent),
            6035 => Some(ErrorCode::MilestoneNotFound),
            6036 => Some(ErrorCode::MilestoneAlreadyCompleted),
            6037 => Some(ErrorCode::InvalidMilestoneIndex),
            6038 => Some(ErrorCode::MilestoneAmountExceedsFunds),
            6039 => Some(ErrorCode::AllMilestonesRequired),
            6040 => Some(ErrorCode::AlreadyApplied),
            6041 => Some(ErrorCode::InvalidApplicationIndex),
            6042 => Some(ErrorCode::CaseAlreadyOpen),
            6043 => Some(ErrorCode::AutoApprovalNotReady),
            6044 => Some(ErrorCode::AutoApprovalBlocked),
            6045 => Some(ErrorCode::InvalidBootstrapAuthority),
            6046 => Some(ErrorCode::ApplicationIndexMismatch),
            6047 => Some(ErrorCode::InvalidApplicationAccount),
            6048 => Some(ErrorCode::ApplicationNotPending),
            6049 => Some(ErrorCode::EmptyProposal),
            6050 => Some(ErrorCode::InvalidApplicationCleanupAccounts),
            _ => None,
        }
    }
}

impl From<ErrorCode> for BackendError {
    fn from(code: ErrorCode) -> Self {
        BackendError::Contract(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_discriminants() {
        assert_eq!(ErrorCode::MathOverflow as u32, 6000);
        assert_eq!(ErrorCode::InvalidApplicationCleanupAccounts as u32, 6050);
    }

    #[test]
    fn test_error_code_from_code() {
        assert_eq!(ErrorCode::from_code(6000), Some(ErrorCode::MathOverflow));
        assert_eq!(ErrorCode::from_code(6001), Some(ErrorCode::ProgramPaused));
        assert_eq!(
            ErrorCode::from_code(6050),
            Some(ErrorCode::InvalidApplicationCleanupAccounts)
        );
        assert_eq!(ErrorCode::from_code(9999), None);
    }

    #[test]
    fn test_error_code_message() {
        assert_eq!(ErrorCode::ProgramPaused.message(), "Program is paused");
        assert_eq!(ErrorCode::NotAuthorized.message(), "Not authorized");
    }

    #[test]
    fn test_contract_error_conversion() {
        let err: BackendError = ErrorCode::InvalidJob.into();
        assert!(matches!(err, BackendError::Contract(ErrorCode::InvalidJob)));
    }
}
