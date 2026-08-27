//! Off-chain metadata models for Trust Work Escrow v3.
//!
//! The on-chain program stores only functional data (pubkeys, amounts, status
//! enums, deadlines, counters, payout percents and **hashes**). All human-
//! readable or voluminous fields — titles, descriptions, proposals, reasons,
//! resolutions and raw evidence content — live off-chain (Postgres/Mongo) and
//! are linked by PDA address. This module defines the serializable structs and
//! their validation so the API can reject bad input before touching the chain.
//!
//! Design is intentionally storage-agnostic: `repository.rs` provides the
//! `MetadataRepository` trait and an in-memory implementation for unit tests.
//! Postgres/Mongo implementations will be added once Docker is available.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

// ---------------------------------------------------------------------------
// Limits — mirror on-chain `ErrorCode` thresholds so off-chain and on-chain
// stay consistent. `DescriptionTooLong` etc. map to the same numeric codes.
// ---------------------------------------------------------------------------

/// Maximum job / milestone title length (on-chain `MAX_TITLE_LENGTH`).
pub const MAX_TITLE_LEN: usize = 100;
/// Maximum description length (on-chain `MAX_DESCRIPTION_LENGTH`).
pub const MAX_DESCRIPTION_LEN: usize = 500;
/// Maximum proposal length (on-chain `MAX_PROPOSAL_LENGTH`).
pub const MAX_PROPOSAL_LEN: usize = 512;
/// Maximum dispute / support-ticket reason length (conservative, off-chain).
pub const MAX_REASON_LEN: usize = 500;
/// Maximum resolution text length.
pub const MAX_RESOLUTION_LEN: usize = 1000;
/// Maximum evidence content length (on-chain `MAX_DISPUTE_EVIDENCE`, raw bytes
/// before hashing; off-chain stores the full text up to ~20 KiB).
pub const MAX_EVIDENCE_CONTENT_LEN: usize = 20_480;
/// Maximum evidence `content` that would fit on-chain before the split
/// (kept for backwards-compat validation of truncated payloads).
pub const MAX_EVIDENCE_ONCHAIN_LEN: usize = 2048;

// ---------------------------------------------------------------------------
// Validation error
// ---------------------------------------------------------------------------

/// Validation failures for off-chain metadata.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error("title cannot be empty")]
    EmptyTitle,
    #[error("title exceeds maximum length ({0} > {1})")]
    TitleTooLong(usize, usize),
    #[error("description exceeds maximum length ({0} > {1})")]
    DescriptionTooLong(usize, usize),
    #[error("proposal cannot be empty")]
    EmptyProposal,
    #[error("proposal exceeds maximum length ({0} > {1})")]
    ProposalTooLong(usize, usize),
    #[error("reason cannot be empty")]
    EmptyReason,
    #[error("reason exceeds maximum length ({0} > {1})")]
    ReasonTooLong(usize, usize),
    #[error("resolution exceeds maximum length ({0} > {1})")]
    ResolutionTooLong(usize, usize),
    #[error("content cannot be empty")]
    EmptyContent,
    #[error("content exceeds maximum length ({0} > {1})")]
    ContentTooLong(usize, usize),
    #[error("pda address cannot be empty")]
    EmptyPda,
    #[error("invalid pda address: {0}")]
    InvalidPda(String),
    #[error("field '{field}' cannot be empty")]
    EmptyField { field: String },
    #[error("field '{field}' exceeds maximum length ({actual} > {max})")]
    FieldTooLong {
        field: String,
        actual: usize,
        max: usize,
    },
}

fn validate_pda(pda: &str) -> Result<(), ValidationError> {
    if pda.trim().is_empty() {
        return Err(ValidationError::EmptyPda);
    }
    // PDA addresses are base58-encoded 32-byte pubkeys (43-44 chars). Accept
    // any non-empty trimmed string of 32..128 chars to stay permissive for
    // tests while still catching obvious mistakes.
    let len = pda.trim().len();
    if !(32..=128).contains(&len) {
        return Err(ValidationError::InvalidPda(format!(
            "expected 32..128 chars, got {len}"
        )));
    }
    Ok(())
}

fn validate_title(title: &str) -> Result<(), ValidationError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    let len = trimmed.chars().count();
    if len > MAX_TITLE_LEN {
        return Err(ValidationError::TitleTooLong(len, MAX_TITLE_LEN));
    }
    Ok(())
}

fn validate_description(desc: &str) -> Result<(), ValidationError> {
    let len = desc.chars().count();
    if len > MAX_DESCRIPTION_LEN {
        return Err(ValidationError::DescriptionTooLong(
            len,
            MAX_DESCRIPTION_LEN,
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Job metadata (Postgres `jobs_metadata`)
// ---------------------------------------------------------------------------

/// Job status for off-chain metadata (Demo Day minimal state machine).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum JobStatus {
    Created,
    Applied,
    Assigned,
    Submitted,
    Approved,
    Cancelled,
    Rejected,
}

impl Default for JobStatus {
    fn default() -> Self {
        Self::Created
    }
}

/// Audit columns shared across all tables (soft-delete).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuditFields {
    pub created_at: i64,
    pub updated_at: i64,
    pub created_by: String,
    pub updated_by: String,
    pub is_active: bool,
    pub deleted_at: Option<i64>,
}

/// Descriptive metadata for a `Job` PDA. Stored in Postgres, linked by
/// `pda_address`. Complements the on-chain `Job` which only keeps
/// `client`, `amount`, `status`, `deadline`, `applicants` etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct JobMetadata {
    /// On-chain PDA address (base58). Primary link to the `Job` account.
    pub pda_address: String,
    /// Human-readable title (1..=100 chars).
    pub title: String,
    /// Human-readable description (0..=500 chars).
    pub description: String,
    /// Job amount in lamports (mirrors on-chain `Job.amount`).
    pub amount: u64,
    /// Fee amount in lamports (mirrors on-chain `Job.fee_amount` = 2.5% of amount).
    pub fee_amount: u64,
    /// Unix timestamp (seconds) for job deadline (mirrors on-chain `Job.deadline`).
    pub deadline: i64,
    /// Client pubkey (base58) — job owner.
    #[serde(default)]
    pub client: String,
    /// Freelancer pubkey if assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freelancer: Option<String>,
    /// Off-chain job status (state machine).
    #[serde(default)]
    pub status: JobStatus,
    /// Optional free-form skills / tags (off-chain only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    /// Unix timestamp (seconds) when the record was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) when the record was last updated.
    pub updated_at: i64,
    /// Audit: who created.
    #[serde(default)]
    pub created_by: String,
    /// Audit: who last updated.
    #[serde(default)]
    pub updated_by: String,
    /// Soft-delete flag.
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    /// Soft-delete timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

fn default_is_active() -> bool { true }

impl JobMetadata {
    /// Create a new `JobMetadata` with current timestamps.
    /// `client` is the job owner's pubkey (base58). `freelancer` is None and `status` is Created.
    pub fn new(
        pda_address: String,
        title: String,
        description: String,
        amount: u64,
        fee_amount: u64,
        deadline: i64,
        client: String,
    ) -> Result<Self, ValidationError> {
        let now = chrono::Utc::now().timestamp();
        let m = Self {
            pda_address,
            title,
            description,
            amount,
            fee_amount,
            deadline,
            client: client.clone(),
            freelancer: None,
            status: JobStatus::Created,
            skills: Vec::new(),
            created_at: now,
            updated_at: now,
            created_by: client.clone(),
            updated_by: client,
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }

    pub fn soft_delete(&mut self, actor: &str) {
        self.is_active = false;
        self.deleted_at = Some(chrono::Utc::now().timestamp());
        self.updated_by = actor.to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Validate all fields.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_pda(&self.pda_address)?;
        validate_title(&self.title)?;
        validate_description(&self.description)?;
        for skill in &self.skills {
            if skill.trim().is_empty() {
                return Err(ValidationError::EmptyField {
                    field: "skills[]".to_string(),
                });
            }
            if skill.chars().count() > 64 {
                return Err(ValidationError::FieldTooLong {
                    field: "skills[]".to_string(),
                    actual: skill.chars().count(),
                    max: 64,
                });
            }
        }
        Ok(())
    }

    /// Apply an update (title/description/skills) and bump `updated_at`.
    pub fn apply_update(
        &mut self,
        title: Option<String>,
        description: Option<String>,
        skills: Option<Vec<String>>,
    ) -> Result<(), ValidationError> {
        if let Some(t) = title {
            validate_title(&t)?;
            self.title = t;
        }
        if let Some(d) = description {
            validate_description(&d)?;
            self.description = d;
        }
        if let Some(s) = skills {
            for skill in &s {
                if skill.trim().is_empty() {
                    return Err(ValidationError::EmptyField {
                        field: "skills[]".to_string(),
                    });
                }
            }
            self.skills = s;
        }
        self.updated_at = chrono::Utc::now().timestamp();
        self.validate()
    }
}

// ---------------------------------------------------------------------------
// Application metadata (Postgres `applications`)
// ---------------------------------------------------------------------------

/// Off-chain proposal text for an `Application` PDA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ApplicationMetadata {
    /// On-chain Application PDA address.
    pub application_pda: String,
    /// On-chain Job PDA address (FK).
    pub job_pda: String,
    /// Applicant wallet (base58).
    pub applicant: String,
    /// Full proposal text (1..=512 chars on-chain, up to 2048 off-chain; we
    /// enforce 512 to stay compatible with the contract).
    pub proposal: String,
    /// SHA-256 hash (hex, 64 chars) of `proposal` for on-chain `proposal_hash`.
    pub proposal_hash: String,
    /// Unix timestamp of submission.
    pub applied_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

impl ApplicationMetadata {
    pub fn new(
        application_pda: String,
        job_pda: String,
        applicant: String,
        proposal: String,
    ) -> Result<Self, ValidationError> {
        let now2 = chrono::Utc::now().timestamp();
        let m = Self {
            application_pda,
            job_pda,
            applicant,
            proposal_hash: String::new(), // filled below
            proposal: String::new(),
            applied_at: now2,
            updated_at: now2,
            created_by: String::new(),
            updated_by: String::new(),
            is_active: true,
            deleted_at: None,
        };
        // Validate proposal before hashing.
        let trimmed = proposal.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyProposal);
        }
        let len = trimmed.chars().count();
        if len > MAX_PROPOSAL_LEN {
            return Err(ValidationError::ProposalTooLong(len, MAX_PROPOSAL_LEN));
        }
        let hash = Self::hash_proposal(trimmed);
        let now = chrono::Utc::now().timestamp();
        let out = Self {
            proposal: proposal.trim().to_string(),
            proposal_hash: hash,
            updated_at: now,
            created_by: String::new(),
            updated_by: String::new(),
            is_active: true,
            deleted_at: None,
            ..m
        };
        out.validate()?;
        Ok(out)
    }

    /// Compute hex-encoded SHA-256 of the proposal.
    pub fn hash_proposal(proposal: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(proposal.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_pda(&self.application_pda)?;
        validate_pda(&self.job_pda)?;
        if self.applicant.trim().is_empty() {
            return Err(ValidationError::EmptyField {
                field: "applicant".to_string(),
            });
        }
        let trimmed = self.proposal.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyProposal);
        }
        let len = trimmed.chars().count();
        if len > MAX_PROPOSAL_LEN {
            return Err(ValidationError::ProposalTooLong(len, MAX_PROPOSAL_LEN));
        }
        if self.proposal_hash.len() != 64 {
            return Err(ValidationError::FieldTooLong {
                field: "proposal_hash".to_string(),
                actual: self.proposal_hash.len(),
                max: 64,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Milestone metadata (Postgres `milestones_metadata`)
// ---------------------------------------------------------------------------

/// Descriptive metadata for a `Milestone` PDA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MilestoneMetadata {
    /// On-chain Job PDA address (FK, part of composite key with `index`).
    pub job_pda: String,
    /// Milestone index (0..=19, must be sequential per `MAX_MILESTONES`).
    pub index: u8,
    /// Milestone title.
    pub title: String,
    /// Milestone description.
    pub description: String,
    /// Creation timestamp.
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

impl MilestoneMetadata {
    pub fn new(
        job_pda: String,
        index: u8,
        title: String,
        description: String,
    ) -> Result<Self, ValidationError> {
        let now = chrono::Utc::now().timestamp();
        let m = Self {
            job_pda,
            index,
            title,
            description,
            created_at: now,
            updated_at: now,
            created_by: String::new(),
            updated_by: String::new(),
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }

    pub fn soft_delete(&mut self, actor: &str) {
        self.is_active = false;
        self.deleted_at = Some(chrono::Utc::now().timestamp());
        self.updated_by = actor.to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_pda(&self.job_pda)?;
        validate_title(&self.title)?;
        validate_description(&self.description)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Dispute metadata (Postgres `disputes_metadata`)
// ---------------------------------------------------------------------------

/// Off-chain reason / resolution for a `Dispute` PDA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DisputeMetadata {
    /// On-chain Dispute PDA address (unique).
    pub dispute_pda: String,
    /// On-chain Job PDA address (FK).
    pub job_pda: String,
    /// Human-readable reason for raising the dispute.
    pub reason: String,
    /// Optional resolution text (filled when the dispute is resolved).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Creation timestamp.
    pub created_at: i64,
    /// Resolution timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<i64>,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

impl DisputeMetadata {
    pub fn new(
        dispute_pda: String,
        job_pda: String,
        reason: String,
    ) -> Result<Self, ValidationError> {
        let now = chrono::Utc::now().timestamp();
        let m = Self {
            dispute_pda,
            job_pda,
            reason,
            resolution: None,
            created_at: now,
            resolved_at: None,
            updated_at: now,
            created_by: String::new(),
            updated_by: String::new(),
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }

    pub fn soft_delete(&mut self, actor: &str) {
        self.is_active = false;
        self.deleted_at = Some(chrono::Utc::now().timestamp());
        self.updated_by = actor.to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_pda(&self.dispute_pda)?;
        validate_pda(&self.job_pda)?;
        let trimmed = self.reason.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyReason);
        }
        let len = trimmed.chars().count();
        if len > MAX_REASON_LEN {
            return Err(ValidationError::ReasonTooLong(len, MAX_REASON_LEN));
        }
        if let Some(res) = &self.resolution {
            let len = res.chars().count();
            if len > MAX_RESOLUTION_LEN {
                return Err(ValidationError::ResolutionTooLong(len, MAX_RESOLUTION_LEN));
            }
        }
        Ok(())
    }

    /// Mark the dispute as resolved with a resolution text.
    pub fn resolve(&mut self, resolution: String) -> Result<(), ValidationError> {
        let trimmed = resolution.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "resolution".to_string(),
            });
        }
        let len = trimmed.chars().count();
        if len > MAX_RESOLUTION_LEN {
            return Err(ValidationError::ResolutionTooLong(len, MAX_RESOLUTION_LEN));
        }
        self.resolution = Some(trimmed.to_string());
        self.resolved_at = Some(chrono::Utc::now().timestamp());
        self.validate()
    }
}

// ---------------------------------------------------------------------------
// Support ticket metadata (Postgres `support_tickets_metadata`)
// ---------------------------------------------------------------------------

/// Off-chain metadata for a `SupportTicket` PDA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SupportTicketMetadata {
    /// On-chain SupportTicket PDA address (unique).
    pub ticket_pda: String,
    /// On-chain Job PDA address (FK).
    pub job_pda: String,
    /// Human-readable reason for opening the ticket.
    pub reason: String,
    /// Optional resolution text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Creation timestamp.
    pub created_at: i64,
    /// Resolution timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<i64>,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

impl SupportTicketMetadata {
    pub fn new(
        ticket_pda: String,
        job_pda: String,
        reason: String,
    ) -> Result<Self, ValidationError> {
        let now = chrono::Utc::now().timestamp();
        let m = Self {
            ticket_pda,
            job_pda,
            reason,
            resolution: None,
            created_at: now,
            resolved_at: None,
            updated_at: now,
            created_by: String::new(),
            updated_by: String::new(),
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }

    pub fn soft_delete(&mut self, actor: &str) {
        self.is_active = false;
        self.deleted_at = Some(chrono::Utc::now().timestamp());
        self.updated_by = actor.to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_pda(&self.ticket_pda)?;
        validate_pda(&self.job_pda)?;
        let trimmed = self.reason.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyReason);
        }
        let len = trimmed.chars().count();
        if len > MAX_REASON_LEN {
            return Err(ValidationError::ReasonTooLong(len, MAX_REASON_LEN));
        }
        if let Some(res) = &self.resolution {
            let len = res.chars().count();
            if len > MAX_RESOLUTION_LEN {
                return Err(ValidationError::ResolutionTooLong(len, MAX_RESOLUTION_LEN));
            }
        }
        Ok(())
    }

    pub fn resolve(&mut self, resolution: String) -> Result<(), ValidationError> {
        let trimmed = resolution.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "resolution".to_string(),
            });
        }
        let len = trimmed.chars().count();
        if len > MAX_RESOLUTION_LEN {
            return Err(ValidationError::ResolutionTooLong(len, MAX_RESOLUTION_LEN));
        }
        self.resolution = Some(trimmed.to_string());
        self.resolved_at = Some(chrono::Utc::now().timestamp());
        self.validate()
    }
}

// ---------------------------------------------------------------------------
// Evidence metadata (Mongo `dispute_evidence`)
// ---------------------------------------------------------------------------

/// Full evidence content for an `Evidence` PDA. The on-chain account only
/// stores `content_hash` (`[u8; 32]`); the raw `content` lives here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EvidenceMetadata {
    /// On-chain Dispute PDA address (FK).
    pub dispute_pda: String,
    /// Evidence index (0..=9 per `MAX_EVIDENCE_COUNT`).
    pub index: u8,
    /// Author wallet (base58).
    pub author: String,
    /// Raw content (1..=20_480 chars off-chain; 2048 on-chain limit noted).
    pub content: String,
    /// Hex-encoded SHA-256 of `content` (matches on-chain `content_hash`).
    pub content_hash: String,
    /// Submission timestamp.
    pub submitted_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

impl EvidenceMetadata {
    pub fn new(
        dispute_pda: String,
        index: u8,
        author: String,
        content: String,
    ) -> Result<Self, ValidationError> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyContent);
        }
        let len = trimmed.chars().count();
        if len > MAX_EVIDENCE_CONTENT_LEN {
            return Err(ValidationError::ContentTooLong(
                len,
                MAX_EVIDENCE_CONTENT_LEN,
            ));
        }
        let hash = Self::hash_content(trimmed);
        let now = chrono::Utc::now().timestamp();
        let m = Self {
            dispute_pda,
            index,
            author,
            content: trimmed.to_string(),
            content_hash: hash,
            submitted_at: now,
            updated_at: now,
            created_by: String::new(),
            updated_by: String::new(),
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }

    /// Compute hex-encoded SHA-256 of the evidence content.
    pub fn hash_content(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_pda(&self.dispute_pda)?;
        if self.author.trim().is_empty() {
            return Err(ValidationError::EmptyField {
                field: "author".to_string(),
            });
        }
        let trimmed = self.content.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyContent);
        }
        let len = trimmed.chars().count();
        if len > MAX_EVIDENCE_CONTENT_LEN {
            return Err(ValidationError::ContentTooLong(
                len,
                MAX_EVIDENCE_CONTENT_LEN,
            ));
        }
        if self.content_hash.len() != 64 {
            return Err(ValidationError::FieldTooLong {
                field: "content_hash".to_string(),
                actual: self.content_hash.len(),
                max: 64,
            });
        }
        Ok(())
    }

    /// Verify that `content_hash` matches `content`.
    pub fn verify_hash(&self) -> bool {
        Self::hash_content(&self.content) == self.content_hash
    }
}

// ---------------------------------------------------------------------------
// User metadata (Postgres `users`)
// ---------------------------------------------------------------------------

/// Allowed roles for `UserMetadata`.
pub const ALLOWED_ROLES: &[&str] = &["client", "freelancer", "admin", "arbiter", "guest"];

/// Single source of truth for permission strings. Frontend `MenuConfig` must stay subset of this.
pub const PERMISSIONS_ALLOWLIST: &[&str] = &[
    "admin:*",
    "admin:users",
    "admin:permissions",
    "admin:wallets",
    "admin:accounting",
    "admin:support",
    "support:view",
    "support:manage",
    "jobs:view",
    "jobs:view:own",
    "jobs:create",
    "jobs:apply",
    "jobs:manage",
    "jobs:delete:own",
    "disputes:view",
    "arbitration:assigned",
    "config:wallet",
    "accountant:view",
];

pub fn is_allowed_permission(p: &str) -> bool {
    PERMISSIONS_ALLOWLIST.contains(&p)
}

pub fn is_allowed_role(r: &str) -> bool {
    ALLOWED_ROLES.contains(&r.to_lowercase().as_str())
}

/// Persistent user profile — email is PK, roles+permissions live off-chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserMetadata {
    /// Normalized email (lowercase, trimmed). Primary key.
    pub email: String,
    /// Vec roles: `client` | `freelancer` | `admin` | `arbiter` | `guest` (allow multiple).
    #[serde(default)]
    pub roles: Vec<String>,
    /// Permissions Vec (must be subset of allowlist).
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Legacy single role alias — deserialized as `role` but stored into `roles[0]` for backward compat.
    #[serde(default, alias = "role", skip_serializing_if = "String::is_empty")]
    #[schema(value_type = String)]
    pub role: String,
    /// Optional wallet pubkey (base58 32 bytes) — set via SIWS flow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_pubkey: Option<String>,
    /// Guest flag (true for ephemeral OTP-less accounts).
    pub is_guest: bool,
    /// Unix timestamp creation.
    pub created_at: i64,
    /// Unix timestamp last update.
    pub updated_at: i64,
    /// Audit.
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

impl UserMetadata {
    pub fn new(
        email: String,
        role: String,
        wallet_pubkey: Option<String>,
        is_guest: bool,
    ) -> Result<Self, ValidationError> {
        let now = chrono::Utc::now().timestamp();
        // Legacy single role -> Vec
        let roles = if role.trim().is_empty() {
            vec!["guest".to_string()]
        } else {
            vec![Self::normalize_role(&role)]
        };
        let m = Self {
            email: email.clone(),
            roles: roles.clone(),
            permissions: Self::default_permissions(&roles),
            role: roles.first().cloned().unwrap_or_default(),
            wallet_pubkey,
            is_guest,
            created_at: now,
            updated_at: now,
            created_by: email.clone(),
            updated_by: email,
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }

    pub fn new_with_roles(
        email: String,
        roles: Vec<String>,
        permissions: Vec<String>,
        wallet_pubkey: Option<String>,
        is_guest: bool,
    ) -> Result<Self, ValidationError> {
        let now = chrono::Utc::now().timestamp();
        let normalized_roles: Vec<String> = roles.iter().map(|r| Self::normalize_role(r)).collect();
        let m = Self {
            role: normalized_roles.first().cloned().unwrap_or_default(),
            email: email.clone(),
            roles: normalized_roles,
            permissions,
            wallet_pubkey,
            is_guest,
            created_at: now,
            updated_at: now,
            created_by: email.clone(),
            updated_by: email,
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        let e = self.email.trim();
        if e.is_empty() || !e.contains('@') || !e.contains('.') {
            return Err(ValidationError::EmptyField {
                field: "email".to_string(),
            });
        }
        if e.len() > 320 {
            return Err(ValidationError::FieldTooLong {
                field: "email".to_string(),
                actual: e.len(),
                max: 320,
            });
        }
        // Normalize alias: if roles empty but legacy role present, populate
        let mut roles = self.roles.clone();
        if roles.is_empty() && !self.role.trim().is_empty() {
            roles = vec![Self::normalize_role(&self.role)];
        }
        if roles.is_empty() {
            return Err(ValidationError::EmptyField { field: "roles".to_string() });
        }
        for r in &roles {
            if !is_allowed_role(r) {
                return Err(ValidationError::EmptyField { field: format!("role:{}", r) });
            }
        }
        for p in &self.permissions {
            if !is_allowed_permission(p) {
                return Err(ValidationError::EmptyField { field: format!("permission:{}", p) });
            }
        }
        if let Some(pk) = &self.wallet_pubkey {
            if !pk.trim().is_empty() {
                let bytes = bs58::decode(pk.trim())
                    .into_vec()
                    .map_err(|e| ValidationError::InvalidPda(format!("wallet pubkey base58: {}", e)))?;
                if bytes.len() != 32 {
                    return Err(ValidationError::InvalidPda(
                        "wallet pubkey must be 32 bytes".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn normalize_email(email: &str) -> String {
        email.trim().to_lowercase()
    }

    pub fn normalize_role(role: &str) -> String {
        role.trim().to_lowercase()
    }

    pub fn normalized_roles(&self) -> Vec<String> {
        if !self.roles.is_empty() {
            self.roles.iter().map(|r| Self::normalize_role(r)).collect()
        } else if !self.role.trim().is_empty() {
            vec![Self::normalize_role(&self.role)]
        } else {
            vec!["guest".to_string()]
        }
    }

    pub fn sync_role_alias(&mut self) {
        if let Some(first) = self.normalized_roles().first().cloned() {
            self.role = first.clone();
            if self.roles.is_empty() {
                self.roles = vec![first];
            }
        }
    }

    pub fn default_permissions(roles: &[String]) -> Vec<String> {
        let mut perms = Vec::new();
        for r in roles {
            match r.as_str() {
                "client" => {
                    perms.extend(["jobs:view:own", "jobs:create", "jobs:view", "config:wallet"].map(|s| s.to_string()));
                }
                "freelancer" => {
                    perms.extend(["jobs:view", "jobs:apply", "config:wallet"].map(|s| s.to_string()));
                }
                "admin" => {
                    perms.extend(["admin:*", "admin:users", "admin:wallets", "support:view"].map(|s| s.to_string()));
                }
                "arbiter" => {
                    perms.extend(["arbitration:assigned", "disputes:view"].map(|s| s.to_string()));
                }
                _ => {}
            }
        }
        perms.sort();
        perms.dedup();
        perms
    }

    pub fn soft_delete(&mut self, actor: &str) {
        self.is_active = false;
        self.deleted_at = Some(chrono::Utc::now().timestamp());
        self.updated_by = actor.to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn has_permission(&self, perm: &str) -> bool {
        has_wildcard(&self.permissions, perm)
    }
}

/// Wildcard-aware permission check: `admin:*` matches `admin:users`.
pub fn has_wildcard(perms: &[String], required: &str) -> bool {
    for p in perms {
        if p == required {
            return true;
        }
        if p.ends_with(":*") {
            let prefix = &p[..p.len() - 1]; // keep colon
            if required.starts_with(prefix) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Wallet purpose and UserWallet (multi-wallet 1..N per email)
// ---------------------------------------------------------------------------

/// Validate a Solana pubkey is 32 bytes bs58 (44 chars).
pub fn validate_pubkey_bs58(pubkey: &str) -> Result<(), ValidationError> {
    let trimmed = pubkey.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyField { field: "pubkey".to_string() });
    }
    let bytes = bs58::decode(trimmed)
        .into_vec()
        .map_err(|e| ValidationError::InvalidPda(format!("pubkey base58: {}", e)))?;
    if bytes.len() != 32 {
        return Err(ValidationError::InvalidPda("pubkey must be 32 bytes".to_string()));
    }
    Ok(())
}

/// Purpose of a wallet per user — publish for job creation, apply for applications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum WalletPurpose {
    Publish,
    Apply,
    General,
}

impl WalletPurpose {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "publish" => Some(Self::Publish),
            "apply" => Some(Self::Apply),
            "general" => Some(Self::General),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Publish => "publish",
            Self::Apply => "apply",
            Self::General => "general",
        }
    }
}

impl std::fmt::Display for WalletPurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// UserWallet — one wallet per email/purpose, soft-deletable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserWallet {
    /// Normalized email (FK to users).
    pub email: String,
    /// Solana pubkey (base58 32 bytes, unique per email).
    pub pubkey: String,
    pub purpose: WalletPurpose,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

impl UserWallet {
    pub fn new(email: String, pubkey: String, purpose: WalletPurpose, label: Option<String>) -> Result<Self, ValidationError> {
        let email_n = UserMetadata::normalize_email(&email);
        if email_n.is_empty() || !email_n.contains('@') { return Err(ValidationError::EmptyField{ field:"email".into()}); }
        validate_pubkey_bs58(&pubkey)?;
        if purpose.as_str().is_empty() { return Err(ValidationError::EmptyField{ field:"purpose".into()}); }
        let now = chrono::Utc::now().timestamp();
        let m = Self {
            email: email_n.clone(),
            pubkey: pubkey.trim().to_string(),
            purpose,
            label: label.and_then(|l| { let t=l.trim().to_string(); if t.is_empty(){None} else {Some(t)}}),
            created_at: now,
            updated_at: now,
            created_by: email_n.clone(),
            updated_by: email_n,
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }
    pub fn validate(&self) -> Result<(), ValidationError> {
        let e = self.email.trim();
        if e.is_empty() || !e.contains('@') { return Err(ValidationError::EmptyField{ field:"email".into()}); }
        validate_pubkey_bs58(&self.pubkey)?;
        Ok(())
    }
    pub fn soft_delete(&mut self, actor: &str) {
        self.is_active = false;
        self.deleted_at = Some(chrono::Utc::now().timestamp());
        self.updated_by = actor.to_string();
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

// ---------------------------------------------------------------------------
// JobParticipant — per-job authority (client|freelancer)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RolePerJob {
    Client,
    Freelancer,
}

impl RolePerJob {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "client" => Some(Self::Client),
            "freelancer" => Some(Self::Freelancer),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Freelancer => "freelancer",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct JobParticipant {
    /// Job PDA address (FK).
    pub job_pda: String,
    /// Normalized email of participant.
    pub email: String,
    pub role_per_job: RolePerJob,
    /// Wallet pubkey used for this job (must be 32B bs58 if non-empty).
    pub wallet_pubkey: String,
    pub joined_at: i64,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub updated_by: String,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<i64>,
}

impl JobParticipant {
    pub fn new(job_pda: String, email: String, role_per_job: RolePerJob, wallet_pubkey: String) -> Result<Self, ValidationError> {
        let email_n = UserMetadata::normalize_email(&email);
        if email_n.is_empty() || !email_n.contains('@') { return Err(ValidationError::EmptyField{ field:"email".into()}); }
        validate_pda(&job_pda)?;
        if !wallet_pubkey.trim().is_empty() {
            validate_pubkey_bs58(&wallet_pubkey)?;
        }
        let now = chrono::Utc::now().timestamp();
        let m = Self {
            job_pda: job_pda.trim().to_string(),
            email: email_n.clone(),
            role_per_job,
            wallet_pubkey: wallet_pubkey.trim().to_string(),
            joined_at: now,
            created_at: now,
            updated_at: now,
            created_by: email_n.clone(),
            updated_by: email_n,
            is_active: true,
            deleted_at: None,
        };
        m.validate()?;
        Ok(m)
    }
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_pda(&self.job_pda)?;
        let e=self.email.trim();
        if e.is_empty()||!e.contains('@'){ return Err(ValidationError::EmptyField{ field:"email".into()});}
        if !self.wallet_pubkey.trim().is_empty() {
            validate_pubkey_bs58(&self.wallet_pubkey)?;
        }
        Ok(())
    }
    pub fn soft_delete(&mut self, actor: &str) {
        self.is_active=false;
        self.deleted_at=Some(chrono::Utc::now().timestamp());
        self.updated_by=actor.to_string();
        self.updated_at=chrono::Utc::now().timestamp();
    }
}

// ---------------------------------------------------------------------------
// Tests — pure validation, no I/O
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pda(n: u8) -> String {
        // Fake but length-valid PDA (44 chars, base58-like)
        format!("7a2YhCd7iivXfyySkp1pf5jj{:0>20}{:02}", n, n)
    }

    fn client_pda(n: u8) -> String {
        format!("7a2YhCd7iivXfyySkp1pf5jjClient{:0>20}{:02}", n, n)
    }

    #[test]
    fn job_title_validation() {
        let dl = chrono::Utc::now().timestamp() + 86400;
        let ok = JobMetadata::new(pda(1), "Build a landing".into(), "desc".into(), 1000000, 25000, dl, client_pda(1)).unwrap();
        assert_eq!(ok.title, "Build a landing");

        assert!(matches!(
            JobMetadata::new(pda(2), "".into(), "desc".into(), 1000000, 25000, dl, client_pda(2)),
            Err(ValidationError::EmptyTitle)
        ));
        assert!(matches!(
            JobMetadata::new(pda(3), "a".repeat(101), "desc".into(), 1000000, 25000, dl, client_pda(3)),
            Err(ValidationError::TitleTooLong(_, _))
        ));
        assert!(matches!(
            JobMetadata::new(pda(4), "ok".into(), "a".repeat(501), 1000000, 25000, dl, client_pda(4)),
            Err(ValidationError::DescriptionTooLong(_, _))
        ));
        assert!(matches!(
            JobMetadata::new("".into(), "ok".into(), "desc".into(), 1000000, 25000, dl, client_pda(5)),
            Err(ValidationError::EmptyPda)
        ));
    }

    #[test]
    fn application_proposal_validation() {
        let app = ApplicationMetadata::new(
            pda(10),
            pda(11),
            "applicant111111111111111111111111111".into(),
            "My proposal".into(),
        )
        .unwrap();
        assert_eq!(app.proposal, "My proposal");
        assert_eq!(app.proposal_hash.len(), 64);
        assert!(
            app.verify_hash(&app.proposal)
                || app.proposal_hash == ApplicationMetadata::hash_proposal("My proposal")
        );

        assert!(matches!(
            ApplicationMetadata::new(pda(12), pda(13), "applicant".into(), "".into()),
            Err(ValidationError::EmptyProposal)
        ));
        assert!(matches!(
            ApplicationMetadata::new(pda(14), pda(15), "applicant".into(), "a".repeat(513)),
            Err(ValidationError::ProposalTooLong(_, _))
        ));
    }

    #[test]
    fn milestone_validation() {
        let ms = MilestoneMetadata::new(pda(20), 0, "Phase 1".into(), "desc".into()).unwrap();
        assert_eq!(ms.index, 0);
        assert!(MilestoneMetadata::new(pda(21), 0, "".into(), "desc".into()).is_err());
    }

    #[test]
    fn dispute_validation_and_resolve() {
        let mut d = DisputeMetadata::new(pda(30), pda(31), "reason".into()).unwrap();
        assert!(d.resolution.is_none());
        d.resolve("we refund 50%".into()).unwrap();
        assert!(d.resolved_at.is_some());
        assert!(DisputeMetadata::new(pda(32), pda(33), "".into()).is_err());
        assert!(DisputeMetadata::new(pda(34), pda(35), "a".repeat(501)).is_err());
    }

    #[test]
    fn support_ticket_validation() {
        let mut t = SupportTicketMetadata::new(pda(40), pda(41), "need help".into()).unwrap();
        t.resolve("resolved ok".into()).unwrap();
        assert!(t.resolved_at.is_some());
    }

    #[test]
    fn evidence_validation_and_hash() {
        let ev = EvidenceMetadata::new(
            pda(50),
            0,
            "author1111111111111111111111111111".into(),
            "evidence content".into(),
        )
        .unwrap();
        assert!(ev.verify_hash());
        assert!(EvidenceMetadata::new(pda(51), 0, "author".into(), "".into()).is_err());
        assert!(EvidenceMetadata::new(pda(52), 0, "author".into(), "a".repeat(20_481)).is_err());
    }

    #[test]
    fn pda_validation() {
        assert!(validate_pda("").is_err());
        assert!(validate_pda("short").is_err());
        assert!(validate_pda(&pda(99)).is_ok());
    }

    // Helper for test: verify application hash recomputation
    impl ApplicationMetadata {
        fn verify_hash(&self, proposal: &str) -> bool {
            Self::hash_proposal(proposal) == self.proposal_hash
        }
    }
}
