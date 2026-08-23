use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Program is paused")]
    ProgramPaused,
    #[msg("Amount too small")]
    AmountTooSmall,
    #[msg("Invalid fee basis points (must be 0-10000)")]
    InvalidFeeBps,
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("Not authorized - not the job client")]
    NotJobClient,
    #[msg("Not authorized - not the job freelancer")]
    NotJobFreelancer,
    #[msg("Cannot work on your own job")]
    CannotWorkOnOwnJob,
    #[msg("Invalid job status for this operation")]
    InvalidJobStatus,
    #[msg("Job pausado")]
    JobPaused,
    #[msg("Job pausado demasiado tiempo; cancela el job")]
    JobPausedExpired,
    #[msg("No se puede pausar un job con freelancer asignado")]
    CannotPauseWithFreelancer,
    #[msg("No freelancer assigned")]
    NoFreelancerAssigned,
    #[msg("Treasury invalido (no coincide con config.treasury)")]
    InvalidTreasury,
    #[msg("Invalid job id / PDA mismatch")]
    InvalidJob,
    #[msg("Deadline must be in the future")]
    DeadlineMustBeFuture,
    #[msg("Insufficient funds in source account")]
    InsufficientFunds,
    #[msg("Cannot raise dispute at this stage")]
    CannotDisputeAtStage,
    #[msg("Dispute reason cannot be empty")]
    EmptyDisputeReason,
    #[msg("Dispute already has the maximum number of evidence items")]
    EvidenceLimitReached,
    #[msg("Evidence index must equal the next dispute evidence index")]
    InvalidEvidenceIndex,
    #[msg("Evidence account does not match the deterministic dispute PDA")]
    InvalidEvidenceAccount,
    #[msg("Evidence cleanup accounts do not match the expected contiguous range")]
    InvalidEvidenceCleanupAccounts,
    #[msg("Dispute already resolved")]
    DisputeAlreadyResolved,
    #[msg("Dispute deadline passed; only platform advisor can resolve")]
    DisputeDeadlinePassed,
    #[msg("Not a valid arbiter")]
    NotValidArbiter,
    #[msg("Not the assigned arbiter")]
    NotArbiter,
    #[msg("El arbitro no puede ser el cliente ni el freelancer")]
    ArbiterCannotBeParty,
    #[msg("Payout percent exceeds 100")]
    InvalidPercent,
    #[msg("Milestone not found")]
    MilestoneNotFound,
    #[msg("Milestone already completed")]
    MilestoneAlreadyCompleted,
    #[msg("Invalid milestone index (must be sequential: == milestones_total)")]
    InvalidMilestoneIndex,
    #[msg("Milestone amount exceeds remaining job funds")]
    MilestoneAmountExceedsFunds,
    #[msg("All milestones must be completed before release")]
    AllMilestonesRequired,
    #[msg("Already applied to this job")]
    AlreadyApplied,
    #[msg("Invalid application index")]
    InvalidApplicationIndex,
    #[msg("A dispute or support ticket is already open for this job")]
    CaseAlreadyOpen,
    #[msg("Auto-approval deadline has not been reached")]
    AutoApprovalNotReady,
    #[msg("Auto-approval is blocked by an open dispute")]
    AutoApprovalBlocked,
    #[msg("Invalid bootstrap authority")]
    InvalidBootstrapAuthority,
    #[msg("Application index must equal the next job application index")]
    ApplicationIndexMismatch,
    #[msg("Application does not belong to this job")]
    InvalidApplicationAccount,
    #[msg("Application is not pending")]
    ApplicationNotPending,
    #[msg("Application proposal cannot be empty")]
    EmptyProposal,
    #[msg("Application cleanup accounts do not match the deterministic job range")]
    InvalidApplicationCleanupAccounts,
    #[msg("No pending authority proposal")]
    NoPendingAuthority,
    #[msg("Authority timelock has not expired (2 days required)")]
    AuthorityTimelockNotExpired,
    #[msg("Invalid new authority")]
    InvalidNewAuthority,
    #[msg("Authority proposal already pending — cancel first")]
    AlreadyPending,
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Program is globally paused")]
    Paused,
}
