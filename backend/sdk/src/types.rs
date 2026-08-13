//! On-chain account types for `trust-escrow-v3`.
//!
//! These structs mirror the `#[account]` definitions in the contract's `lib.rs`
//! and implement `AccountDeserialize` so the SDK can read account data returned
//! by RPC. The 8-byte Anchor account discriminator (`sha256("account:<Name>")[..8]`)
//! is verified on deserialization.

#[cfg(feature = "solana")]
mod inner {
    use anchor_lang::{AccountDeserialize, AnchorDeserialize, AnchorSerialize};
    use anchor_lang::solana_program::pubkey::Pubkey;

    // ---- Enums (declaration order must match the on-chain contract) ----

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub enum JobStatus {
        Created,
        Funded,
        InProgress,
        Submitted,
        Released,
        Disputed,
        Resolved,
        Cancelled,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub enum ApplicationStatus {
        Pending,
        Accepted,
        Rejected,
        Withdrawn,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub enum DisputeStatus {
        Open,
        Active,
        EvidenceSubmitted,
        ArbiterAssigned,
        Resolved,
        Expired,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub enum MilestoneStatus {
        Pending,
        Submitted,
        Approved,
        Rejected,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub enum SupportTicketStatus {
        Open,
        Resolved,
    }

    // ---- Accounts ----

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct Config {
        pub authority: Pubkey,
        pub advisor: Pubkey,
        pub treasury: Pubkey,
        pub arbitration_treasury: Pubkey,
        pub fee_bps: u16,
        pub paused: bool,
        pub bump: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct Job {
        pub client: Pubkey,
        pub freelancer: Option<Pubkey>,
        pub amount: u64,
        pub fee_amount: u64,
        pub status: JobStatus,
        pub paused: bool,
        pub paused_at: i64,
        pub title: String,
        pub description: String,
        pub deadline: i64,
        pub created_at: i64,
        pub updated_at: i64,
        pub submitted_at: Option<i64>,
        pub milestones_total: u8,
        pub milestones_approved: u8,
        pub milestones_amount_total: u64,
        pub applicants: Vec<Pubkey>,
        pub bump: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct Application {
        pub job: Pubkey,
        pub index: u8,
        pub applicant: Pubkey,
        pub proposal: String,
        pub applied_at: i64,
        pub status: ApplicationStatus,
        pub bump: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct ArbiterPool {
        pub authority: Pubkey,
        pub arbiters: Vec<Pubkey>,
        pub bump: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct Dispute {
        pub job: Pubkey,
        pub raised_by: Pubkey,
        pub arbiter: Option<Pubkey>,
        pub status: DisputeStatus,
        pub evidence_count: u8,
        pub evidence_cleanup_cursor: u8,
        pub reason: String,
        pub created_at: i64,
        pub deadline: i64,
        pub resolved_at: Option<i64>,
        pub resolution: Option<String>,
        pub client_payout_percent: u8,
        pub freelancer_payout_percent: u8,
        pub bump: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct Evidence {
        pub dispute: Pubkey,
        pub index: u8,
        pub author: Pubkey,
        pub content: Vec<u8>,
        pub submitted_at: i64,
        pub bump: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct Milestone {
        pub job: Pubkey,
        pub title: String,
        pub description: String,
        pub amount: u64,
        pub deadline: i64,
        pub status: MilestoneStatus,
        pub index: u8,
        pub submitted_at: Option<i64>,
        pub approved_at: Option<i64>,
        pub bump: u8,
        pub created_at: i64,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct SupportTicket {
        pub job: Pubkey,
        pub opened_by: Pubkey,
        pub reason: String,
        pub status: SupportTicketStatus,
        pub created_at: i64,
        pub resolved_at: Option<i64>,
        pub resolution: Option<String>,
        pub bump: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
    pub struct ArbitrationEscrow {
        pub job: Pubkey,
        pub client_bond: u64,
        pub freelancer_bond: u64,
        pub bump: u8,
    }

    /// Compute the Anchor account discriminator for a given account name.
    ///
    /// Anchor uses `sha256("account:<Name>")[..8]`.
    pub fn account_discriminator(name: &str) -> [u8; 8] {
        use solana_sdk::hash::hash;
        let mut preimage = b"account:".to_vec();
        preimage.extend_from_slice(name.as_bytes());
        let h = hash(&preimage);
        let mut disc = [0u8; 8];
        disc.copy_from_slice(&h.to_bytes()[..8]);
        disc
    }

    /// Build an `anchor_lang::error::Error` for a discriminator mismatch.
    ///
    /// `AccountDiscriminatorMismatch` is an `ErrorCode` variant, not an
    /// `anchor_lang::error::Error` variant, so it must be wrapped in an
    /// `AnchorError`. The numeric code (3002) matches the on-chain definition.
    fn discriminator_error(name: &str) -> anchor_lang::error::Error {
        anchor_lang::error::Error::AnchorError(Box::new(anchor_lang::error::AnchorError {
            error_name: "AccountDiscriminatorMismatch".to_string(),
            error_code_number: 3002,
            error_msg: "Account discriminator did not match what was expected".to_string(),
            error_origin: Some(anchor_lang::error::ErrorOrigin::AccountName(name.to_string())),
            compared_values: None,
        }))
    }

    /// Build an `anchor_lang::error::Error` for a field-decode failure.
    ///
    /// `AccountDidNotDeserialize` is an `ErrorCode` variant (code 3003); it is
    /// wrapped in an `AnchorError` because it is not an `Error` variant.
    fn deserialize_error() -> anchor_lang::error::Error {
        anchor_lang::error::Error::AnchorError(Box::new(anchor_lang::error::AnchorError {
            error_name: "AccountDidNotDeserialize".to_string(),
            error_code_number: 3003,
            error_msg: "Failed to deserialize the account".to_string(),
            error_origin: None,
            compared_values: None,
        }))
    }

    /// Implement `AccountDeserialize` (discriminator check + field decode) for an
    /// account struct that already derives `AnchorDeserialize`.
    ///
    /// `AnchorDeserialize::deserialize` yields a borsh `io::Error`; we map it to an
    /// `anchor_lang::error::Error` explicitly (rather than via `?`) to avoid the
    /// borsh 0.10 / 1.x version ambiguity in the `From` conversion.
    macro_rules! impl_account_deserialize {
        ($t:ty, $name:literal) => {
            impl AccountDeserialize for $t {
                fn try_deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
                    let disc = account_discriminator($name);
                    if buf.len() < 8 || &buf[..8] != disc.as_slice() {
                        return Err(discriminator_error($name));
                    }
                    let mut data = &buf[8..];
                    AnchorDeserialize::deserialize(&mut data).map_err(|_| deserialize_error())
                }

                fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
                    AnchorDeserialize::deserialize(buf).map_err(|_| deserialize_error())
                }
            }
        };
    }

    impl_account_deserialize!(Config, "Config");
    impl_account_deserialize!(Job, "Job");
    impl_account_deserialize!(Application, "Application");
    impl_account_deserialize!(ArbiterPool, "ArbiterPool");
    impl_account_deserialize!(Dispute, "Dispute");
    impl_account_deserialize!(Evidence, "Evidence");
    impl_account_deserialize!(Milestone, "Milestone");
    impl_account_deserialize!(SupportTicket, "SupportTicket");
    impl_account_deserialize!(ArbitrationEscrow, "ArbitrationEscrow");
}

#[cfg(feature = "solana")]
pub use inner::*;
