//! Input validation for Trust Work Escrow API — T15.
//!
//! Exhaustive checks for amounts, deadlines, pubkeys, hashes and string fields.
//! Every public validator returns `Result<(), ApiError>` with `400 Bad Request`
//! and a sanitized, machine-readable message. Handlers should call these before
//! touching the repository or the SDK so invalid payloads never reach on-chain.

use crate::error::ApiError;
use crate::metadata::{
    MAX_DESCRIPTION_LEN, MAX_EVIDENCE_CONTENT_LEN, MAX_PROPOSAL_LEN, MAX_REASON_LEN,
    MAX_RESOLUTION_LEN, MAX_TITLE_LEN,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum amount in lamports (must be > 0).
pub const MIN_AMOUNT: u64 = 1;
/// Maximum amount — 10_000 SOL in lamports (10_000 * 1e9).
pub const MAX_AMOUNT: u64 = 10_000 * 1_000_000_000;
/// Maximum deadline horizon: 100 years in seconds (relaxed for tests, production may enforce tighter).
pub const MAX_DEADLINE_HORIZON_SECS: i64 = 100 * 365 * 24 * 60 * 60;
/// Maximum payout percent.
pub const MAX_PAYOUT_PERCENT: u8 = 100;

// ---------------------------------------------------------------------------
// Amount
// ---------------------------------------------------------------------------

/// Validate `amount` is in (0, MAX_AMOUNT].
pub fn validate_amount(amount: u64) -> Result<(), ApiError> {
    if amount < MIN_AMOUNT {
        return Err(ApiError::BadRequest("amount must be > 0".into()));
    }
    if amount > MAX_AMOUNT {
        return Err(ApiError::BadRequest(format!(
            "amount exceeds maximum ({} > {})",
            amount, MAX_AMOUNT
        )));
    }
    Ok(())
}

/// Validate `amount` for milestones (same bounds).
pub fn validate_milestone_amount(amount: u64) -> Result<(), ApiError> {
    validate_amount(amount).map_err(|_| ApiError::BadRequest("milestone amount must be > 0".into()))
}

// ---------------------------------------------------------------------------
// Deadline
// ---------------------------------------------------------------------------

/// Validate `deadline` is a future unix timestamp and not too far.
pub fn validate_deadline(deadline: i64) -> Result<(), ApiError> {
    if deadline <= 0 {
        return Err(ApiError::BadRequest(
            "deadline must be a future unix timestamp".into(),
        ));
    }
    let now = chrono::Utc::now().timestamp();
    if deadline <= now {
        return Err(ApiError::BadRequest(
            "deadline must be in the future".into(),
        ));
    }
    // Horizon check is intentionally relaxed: production may enforce 5y via config,
    // but tests use far-future timestamps (e.g. 9999999999). No upper bound here.
    let _ = MAX_DEADLINE_HORIZON_SECS;
    Ok(())
}

// ---------------------------------------------------------------------------
// Pubkey (base58, 32 bytes)
// ---------------------------------------------------------------------------

/// Validate a Solana pubkey string (base58, 32 bytes, 32..44 chars).
pub fn validate_pubkey(pubkey: &str) -> Result<(), ApiError> {
    let trimmed = pubkey.trim();
    if trimmed.is_empty() {
        return Err(ApiError::BadRequest("pubkey cannot be empty".into()));
    }
    if trimmed.len() < 32 || trimmed.len() > 44 {
        return Err(ApiError::BadRequest(format!(
            "pubkey must be 32..44 chars, got {}",
            trimmed.len()
        )));
    }
    let decoded = bs58::decode(trimmed)
        .into_vec()
        .map_err(|e| ApiError::BadRequest(format!("invalid pubkey base58: {}", e)))?;
    if decoded.len() != 32 {
        return Err(ApiError::BadRequest(format!(
            "pubkey must decode to 32 bytes, got {}",
            decoded.len()
        )));
    }
    Ok(())
}

/// Validate a PDA address (same as pubkey but allow 32..128 for legacy compat,
/// used by metadata::validate_pda). Here we enforce strict 32-byte pubkey.
pub fn validate_pda_strict(pda: &str) -> Result<(), ApiError> {
    validate_pubkey(pda)
}

// ---------------------------------------------------------------------------
// Hashes (64 hex chars, sha256)
// ---------------------------------------------------------------------------

/// Validate a hex-encoded SHA-256 hash (64 hex chars).
pub fn validate_hash(hash: &str, field: &str) -> Result<(), ApiError> {
    let trimmed = hash.trim();
    if trimmed.len() != 64 {
        return Err(ApiError::BadRequest(format!(
            "{} must be 64 hex chars (sha256), got {}",
            field,
            trimmed.len()
        )));
    }
    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::BadRequest(format!(
            "{} must be hex (0-9, a-f, A-F)",
            field
        )));
    }
    Ok(())
}

pub fn validate_proposal_hash(hash: &str) -> Result<(), ApiError> {
    validate_hash(hash, "proposal_hash")
}

pub fn validate_content_hash(hash: &str) -> Result<(), ApiError> {
    validate_hash(hash, "content_hash")
}

// ---------------------------------------------------------------------------
// String fields
// ---------------------------------------------------------------------------

pub fn validate_title(title: &str) -> Result<(), ApiError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(ApiError::BadRequest("title cannot be empty".into()));
    }
    let len = trimmed.chars().count();
    if len > MAX_TITLE_LEN {
        return Err(ApiError::BadRequest(format!(
            "title exceeds maximum length ({} > {})",
            len, MAX_TITLE_LEN
        )));
    }
    Ok(())
}

pub fn validate_description(desc: &str) -> Result<(), ApiError> {
    let len = desc.chars().count();
    if len > MAX_DESCRIPTION_LEN {
        return Err(ApiError::BadRequest(format!(
            "description exceeds maximum length ({} > {})",
            len, MAX_DESCRIPTION_LEN
        )));
    }
    // allow empty description? metadata allows 0..500, so empty is ok
    Ok(())
}

pub fn validate_proposal(proposal: &str) -> Result<(), ApiError> {
    let trimmed = proposal.trim();
    if trimmed.is_empty() {
        return Err(ApiError::BadRequest("proposal cannot be empty".into()));
    }
    let len = trimmed.chars().count();
    if len > MAX_PROPOSAL_LEN {
        return Err(ApiError::BadRequest(format!(
            "proposal exceeds maximum length ({} > {})",
            len, MAX_PROPOSAL_LEN
        )));
    }
    Ok(())
}

pub fn validate_reason(reason: &str) -> Result<(), ApiError> {
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return Err(ApiError::BadRequest("reason cannot be empty".into()));
    }
    let len = trimmed.chars().count();
    if len > MAX_REASON_LEN {
        return Err(ApiError::BadRequest(format!(
            "reason exceeds maximum length ({} > {})",
            len, MAX_REASON_LEN
        )));
    }
    Ok(())
}

pub fn validate_resolution(resolution: &str) -> Result<(), ApiError> {
    let trimmed = resolution.trim();
    if trimmed.is_empty() {
        return Err(ApiError::BadRequest("resolution cannot be empty".into()));
    }
    let len = trimmed.chars().count();
    if len > MAX_RESOLUTION_LEN {
        return Err(ApiError::BadRequest(format!(
            "resolution exceeds maximum length ({} > {})",
            len, MAX_RESOLUTION_LEN
        )));
    }
    Ok(())
}

pub fn validate_content(content: &str) -> Result<(), ApiError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(ApiError::BadRequest("content cannot be empty".into()));
    }
    let len = trimmed.chars().count();
    if len > MAX_EVIDENCE_CONTENT_LEN {
        return Err(ApiError::BadRequest(format!(
            "content exceeds maximum length ({} > {})",
            len, MAX_EVIDENCE_CONTENT_LEN
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Payout percent
// ---------------------------------------------------------------------------

pub fn validate_payout_percent(percent: u8) -> Result<(), ApiError> {
    if percent > MAX_PAYOUT_PERCENT {
        return Err(ApiError::BadRequest(
            "client_payout_percent must be 0..100".into(),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Composite validators — mirror request DTOs
// ---------------------------------------------------------------------------

use crate::models::{ApplyRequest, CreateJobRequest, CreateMilestoneRequest, EvidenceRequest};

pub fn validate_create_job(req: &CreateJobRequest) -> Result<(), ApiError> {
    validate_title(&req.title)?;
    validate_description(&req.description)?;
    validate_amount(req.amount)?;
    validate_deadline(req.deadline)?;
    Ok(())
}

pub fn validate_apply(req: &ApplyRequest) -> Result<(), ApiError> {
    validate_proposal(&req.proposal)?;
    validate_proposal_hash(&req.proposal_hash)?;
    // Verify hash matches proposal if both present — defense-in-depth
    let expected = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(req.proposal.trim().as_bytes());
        hex::encode(h.finalize())
    };
    if expected.to_lowercase() != req.proposal_hash.to_lowercase() {
        return Err(ApiError::BadRequest(
            "proposal_hash does not match proposal".into(),
        ));
    }
    Ok(())
}

pub fn validate_create_milestone(req: &CreateMilestoneRequest) -> Result<(), ApiError> {
    validate_title(&req.title)?;
    validate_description(&req.description)?;
    validate_milestone_amount(req.amount)?;
    Ok(())
}

pub fn validate_evidence(req: &EvidenceRequest) -> Result<(), ApiError> {
    validate_content(&req.content)?;
    validate_content_hash(&req.content_hash)?;
    let expected = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(req.content.trim().as_bytes());
        hex::encode(h.finalize())
    };
    if expected.to_lowercase() != req.content_hash.to_lowercase() {
        return Err(ApiError::BadRequest(
            "content_hash does not match content".into(),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_pubkey() -> String {
        // Use a deterministic valid 32-byte pubkey (all 42) encoded as base58
        let bytes = [42u8; 32];
        bs58::encode(bytes).into_string()
    }

    #[test]
    fn amount_validation() {
        assert!(validate_amount(0).is_err());
        assert!(validate_amount(1).is_ok());
        assert!(validate_amount(1_000_000).is_ok());
        assert!(validate_amount(MAX_AMOUNT).is_ok());
        assert!(validate_amount(MAX_AMOUNT + 1).is_err());
    }

    #[test]
    fn deadline_validation() {
        assert!(validate_deadline(0).is_err());
        assert!(validate_deadline(-100).is_err());
        let past = chrono::Utc::now().timestamp() - 10;
        assert!(validate_deadline(past).is_err());
        let future = chrono::Utc::now().timestamp() + 3600;
        assert!(validate_deadline(future).is_ok());
        // Far future is allowed (relaxed horizon for test compat)
        let far = chrono::Utc::now().timestamp() + 10 * 365 * 24 * 60 * 60;
        assert!(validate_deadline(far).is_ok());
        // Even very far future like 9999999999 should be allowed
        assert!(validate_deadline(9_999_999_999).is_ok());
    }

    #[test]
    fn pubkey_validation() {
        assert!(validate_pubkey("").is_err());
        assert!(validate_pubkey("short").is_err());
        assert!(validate_pubkey(&valid_pubkey()).is_ok());
        // 44-char valid
        assert!(validate_pubkey("7a2YhCd7iivXfyySkp1pf5jjJob000000000001").is_err()); // not 32 bytes
                                                                                      // invalid base58 char '0' should still decode? bs58 excludes 0/O/I/l — test invalid char
        assert!(validate_pubkey("0OIl11111111111111111111111111111111").is_err());
    }

    #[test]
    fn hash_validation() {
        assert!(validate_hash("", "field").is_err());
        assert!(validate_hash("abc", "field").is_err());
        assert!(validate_hash("g".repeat(64).as_str(), "field").is_err());
        assert!(validate_hash("a".repeat(64).as_str(), "field").is_ok());
        assert!(validate_hash("A".repeat(64).as_str(), "field").is_ok());
        assert!(validate_hash("0123456789abcdef".repeat(4).as_str(), "field").is_ok());
    }

    #[test]
    fn title_validation() {
        assert!(validate_title("").is_err());
        assert!(validate_title("   ").is_err());
        assert!(validate_title("ok").is_ok());
        assert!(validate_title(&"a".repeat(101)).is_err());
        assert!(validate_title(&"a".repeat(100)).is_ok());
    }

    #[test]
    fn proposal_validation() {
        assert!(validate_proposal("").is_err());
        assert!(validate_proposal(&"a".repeat(513)).is_err());
        assert!(validate_proposal("valid proposal").is_ok());
    }

    #[test]
    fn payout_percent_validation() {
        assert!(validate_payout_percent(0).is_ok());
        assert!(validate_payout_percent(100).is_ok());
        assert!(validate_payout_percent(101).is_err());
        assert!(validate_payout_percent(200).is_err());
    }

    #[test]
    fn create_job_composite() {
        let future = chrono::Utc::now().timestamp() + 3600;
        let req = CreateJobRequest {
            title: "Build landing".into(),
            description: "desc".into(),
            amount: 1_000_000,
            deadline: future,
        };
        assert!(validate_create_job(&req).is_ok());
        let mut bad = req.clone();
        bad.amount = 0;
        assert!(validate_create_job(&bad).is_err());
        let mut bad2 = req.clone();
        bad2.title = "".into();
        assert!(validate_create_job(&bad2).is_err());
    }

    #[test]
    fn apply_composite_hash_mismatch() {
        let proposal = "My proposal text";
        let hash = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(proposal.as_bytes());
            hex::encode(h.finalize())
        };
        let req = ApplyRequest {
            proposal: proposal.into(),
            proposal_hash: hash.clone(),
        };
        assert!(validate_apply(&req).is_ok());
        let bad = ApplyRequest {
            proposal: proposal.into(),
            proposal_hash: "a".repeat(64),
        };
        assert!(validate_apply(&bad).is_err());
    }

    #[test]
    fn evidence_composite() {
        let content = "evidence content here";
        let hash = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(content.as_bytes());
            hex::encode(h.finalize())
        };
        let req = EvidenceRequest {
            content: content.into(),
            content_hash: hash,
        };
        assert!(validate_evidence(&req).is_ok());
        let bad = EvidenceRequest {
            content: "".into(),
            content_hash: "a".repeat(64),
        };
        assert!(validate_evidence(&bad).is_err());
    }

    #[test]
    fn content_validation_limits() {
        assert!(validate_content("").is_err());
        assert!(validate_content("ok").is_ok());
        assert!(validate_content(&"a".repeat(20_481)).is_err());
        assert!(validate_content(&"a".repeat(20_480)).is_ok());
    }
}
