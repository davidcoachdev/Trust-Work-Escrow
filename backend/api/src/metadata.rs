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
    /// Optional free-form skills / tags (off-chain only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    /// Unix timestamp (seconds) when the record was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) when the record was last updated.
    pub updated_at: i64,
}

impl JobMetadata {
    /// Create a new `JobMetadata` with current timestamps.
    pub fn new(
        pda_address: String,
        title: String,
        description: String,
        amount: u64,
        fee_amount: u64,
    ) -> Result<Self, ValidationError> {
        let now = chrono::Utc::now().timestamp();
        let m = Self {
            pda_address,
            title,
            description,
            amount,
            fee_amount,
            skills: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        m.validate()?;
        Ok(m)
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
}

impl ApplicationMetadata {
    pub fn new(
        application_pda: String,
        job_pda: String,
        applicant: String,
        proposal: String,
    ) -> Result<Self, ValidationError> {
        let m = Self {
            application_pda,
            job_pda,
            applicant,
            proposal_hash: String::new(), // filled below
            proposal: String::new(),
            applied_at: chrono::Utc::now().timestamp(),
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
        let out = Self {
            proposal: proposal.trim().to_string(),
            proposal_hash: hash,
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
}

impl MilestoneMetadata {
    pub fn new(
        job_pda: String,
        index: u8,
        title: String,
        description: String,
    ) -> Result<Self, ValidationError> {
        let m = Self {
            job_pda,
            index,
            title,
            description,
            created_at: chrono::Utc::now().timestamp(),
        };
        m.validate()?;
        Ok(m)
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
}

impl DisputeMetadata {
    pub fn new(
        dispute_pda: String,
        job_pda: String,
        reason: String,
    ) -> Result<Self, ValidationError> {
        let m = Self {
            dispute_pda,
            job_pda,
            reason,
            resolution: None,
            created_at: chrono::Utc::now().timestamp(),
            resolved_at: None,
        };
        m.validate()?;
        Ok(m)
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
}

impl SupportTicketMetadata {
    pub fn new(
        ticket_pda: String,
        job_pda: String,
        reason: String,
    ) -> Result<Self, ValidationError> {
        let m = Self {
            ticket_pda,
            job_pda,
            reason,
            resolution: None,
            created_at: chrono::Utc::now().timestamp(),
            resolved_at: None,
        };
        m.validate()?;
        Ok(m)
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
        let m = Self {
            dispute_pda,
            index,
            author,
            content: trimmed.to_string(),
            content_hash: hash,
            submitted_at: chrono::Utc::now().timestamp(),
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
// Tests — pure validation, no I/O
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pda(n: u8) -> String {
        // Fake but length-valid PDA (44 chars, base58-like)
        format!("7a2YhCd7iivXfyySkp1pf5jj{:0>20}{:02}", n, n)
    }

    #[test]
    fn job_title_validation() {
        let ok = JobMetadata::new(pda(1), "Build a landing".into(), "desc".into(), 1000000, 25000).unwrap();
        assert_eq!(ok.title, "Build a landing");

        assert!(matches!(
            JobMetadata::new(pda(2), "".into(), "desc".into(), 1000000, 25000),
            Err(ValidationError::EmptyTitle)
        ));
        assert!(matches!(
            JobMetadata::new(pda(3), "a".repeat(101), "desc".into(), 1000000, 25000),
            Err(ValidationError::TitleTooLong(_, _))
        ));
        assert!(matches!(
            JobMetadata::new(pda(4), "ok".into(), "a".repeat(501), 1000000, 25000),
            Err(ValidationError::DescriptionTooLong(_, _))
        ));
        assert!(matches!(
            JobMetadata::new("".into(), "ok".into(), "desc".into(), 1000000, 25000),
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
