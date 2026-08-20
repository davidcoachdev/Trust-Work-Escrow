//! Evidence off-chain hash + index linkage for Trust Work Escrow v3.
//!
//! Links large off-chain evidence content (up to [`MAX_EVIDENCE_CONTENT_LEN`] = 20_480 chars)
//! with on-chain [`content_hash`] (`[u8; 32]` SHA-256) and sequential `index`.
//!
//! On-chain program (`trust-escrow-v3`) stores only `content_hash: [u8; 32]` in the
//! `Evidence` PDA and enforces `index == dispute.evidence_count` (see `lib.rs`
//! `submit_evidence`). This module is the off-chain counterpart:
//! - computes `SHA-256(content) → [u8;32] ↔ hex(64)` (compatible with `EvidenceMetadata`)
//! - validates `content` length and `content_hash` round-trip
//! - assigns / validates sequential `index` (0..`MAX_EVIDENCE_COUNT-1`)
//! - verifies off-chain content against an on-chain hash (`verify_against_onchain`)
//! - paginates evidence per dispute with opaque cursor, mirroring
//!   `trust_escrow_sdk::client::TrustEscrowClient::list_applications` and
//!   `trust_escrow_sdk::utils::{encode_cursor, Page}` (T7-T8)
//!
//! Integration notes:
//! - `trust_escrow_sdk::events::try_decode_anchor_log` decodes `EvidenceSubmitted`
//!   (future `#[event]`); until then callers paginate via `list_evidence_paginated`.
//! - `trust_escrow_sdk::client::TrustEscrowClient::list_applications` is the
//!   pagination reference for this evidence pagination (opaque base64url cursor,
//!   sorted by `index`, `has_more`, gap handling).

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

use crate::metadata::{EvidenceMetadata, ValidationError, MAX_EVIDENCE_CONTENT_LEN};
use crate::repository::{MetadataRepository, RepositoryError};

// ---------------------------------------------------------------------------
// Constants — mirrors on-chain `MAX_EVIDENCE_COUNT` in `trust-escrow-v3`
// ---------------------------------------------------------------------------

/// Maximum number of evidence entries per dispute (on-chain `MAX_EVIDENCE_COUNT`).
pub const MAX_EVIDENCE_COUNT: u8 = 10;

/// Maximum valid evidence index (`MAX_EVIDENCE_COUNT - 1`).
pub const MAX_EVIDENCE_INDEX: u8 = MAX_EVIDENCE_COUNT - 1;

/// Default page limit for evidence listings (mirrors SDK `DEFAULT_PAGE_LIMIT`).
pub const DEFAULT_PAGE_LIMIT: usize = 20;
/// Maximum page limit (mirrors SDK `MAX_PAGE_LIMIT`).
pub const MAX_PAGE_LIMIT: usize = 100;

// ---------------------------------------------------------------------------
// Hash helpers — off-chain ↔ on-chain `content_hash: [u8; 32]`
// ---------------------------------------------------------------------------

/// Compute raw SHA-256 bytes of `content` (UTF-8).
pub fn hash_content_bytes(content: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hasher.finalize().into()
}

/// Compute hex-encoded SHA-256 of `content` (64-char lowercase hex).
pub fn hash_content_hex(content: &str) -> String {
    hex::encode(hash_content_bytes(content))
}

/// Convert a 64-char hex string to `[u8; 32]`.
///
/// Validates length and hex alphabet; maps failures to `ValidationError`.
pub fn hex_to_bytes32(hex_str: &str) -> Result<[u8; 32], ValidationError> {
    let trimmed = hex_str.trim();
    if trimmed.len() != 64 {
        return Err(ValidationError::FieldTooLong {
            field: "content_hash".to_string(),
            actual: trimmed.len(),
            max: 64,
        });
    }
    let bytes = hex::decode(trimmed).map_err(|e| ValidationError::InvalidPda(e.to_string()))?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Convert `[u8; 32]` to 64-char hex string.
pub fn bytes32_to_hex(bytes: &[u8; 32]) -> String {
    hex::encode(bytes)
}

/// Verify `content` matches hex `expected_hex` (case-insensitive via lowercase).
pub fn verify_hash_hex(content: &str, expected_hex: &str) -> bool {
    hash_content_hex(content) == expected_hex.to_lowercase()
}

/// Verify `content` matches on-chain `[u8; 32]` hash.
pub fn verify_hash_bytes32(content: &str, expected: &[u8; 32]) -> bool {
    &hash_content_bytes(content) == expected
}

/// Verify an `EvidenceMetadata` record against an on-chain hash and index.
///
/// Returns `true` iff `evidence.content_hash` round-trips to `onchain_hash`
/// and `evidence.index == onchain_index` and `verify_hash_bytes32` passes.
pub fn verify_evidence_link(
    evidence: &EvidenceMetadata,
    onchain_hash: &[u8; 32],
    onchain_index: u8,
) -> bool {
    if evidence.index != onchain_index {
        return false;
    }
    if !verify_hash_bytes32(&evidence.content, onchain_hash) {
        return false;
    }
    // Also ensure stored hex matches bytes hex (covers tampered stored hash).
    match hex_to_bytes32(&evidence.content_hash) {
        Ok(bytes) => bytes == *onchain_hash,
        Err(_) => false,
    }
}

/// Verify `content` length against [`MAX_EVIDENCE_CONTENT_LEN`].
pub fn validate_evidence_content_len(content: &str) -> Result<(), ValidationError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyContent);
    }
    let len = trimmed.chars().count();
    if len > MAX_EVIDENCE_CONTENT_LEN {
        return Err(ValidationError::ContentTooLong(len, MAX_EVIDENCE_CONTENT_LEN));
    }
    Ok(())
}

/// Validate evidence index is within `0..MAX_EVIDENCE_COUNT`.
pub fn validate_evidence_index(index: u8) -> Result<(), ValidationError> {
    if index > MAX_EVIDENCE_INDEX {
        return Err(ValidationError::FieldTooLong {
            field: "index".to_string(),
            actual: index as usize,
            max: MAX_EVIDENCE_INDEX as usize,
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Pagination — opaque cursor, mirrors SDK `utils::Page`
// ---------------------------------------------------------------------------

/// Encode pagination offset as opaque base64url (no pad).
pub fn encode_cursor(offset: usize) -> String {
    let bytes = (offset as u64).to_be_bytes();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Decode opaque cursor to offset. `None`/empty → 0.
pub fn decode_cursor(cursor: Option<&str>) -> Result<usize, RepositoryError> {
    let Some(c) = cursor else {
        return Ok(0);
    };
    let c = c.trim();
    if c.is_empty() {
        return Ok(0);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(c)
        .map_err(|e| RepositoryError::Storage(format!("invalid cursor: {e}")))?;
    if bytes.len() != 8 {
        return Err(RepositoryError::Storage("invalid cursor length".to_string()));
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes);
    let v = u64::from_be_bytes(arr);
    if v > (usize::MAX as u64) {
        return Err(RepositoryError::Storage("cursor offset overflow".to_string()));
    }
    Ok(v as usize)
}

/// Validate and clamp page limit.
pub fn validate_limit(limit: Option<usize>) -> Result<usize, RepositoryError> {
    let l = limit.unwrap_or(DEFAULT_PAGE_LIMIT);
    if l == 0 {
        return Err(RepositoryError::Storage("limit must be > 0".to_string()));
    }
    if l > MAX_PAGE_LIMIT {
        return Ok(MAX_PAGE_LIMIT);
    }
    Ok(l)
}

/// Paginated evidence result (mirrors `PaginatedApplications`).
#[derive(Debug, Clone)]
pub struct PaginatedEvidence {
    pub evidence: Vec<EvidenceMetadata>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// List evidence by dispute with cursor pagination (sorted by `index`, stable).
///
/// Mirrors `TrustEscrowClient::list_applications(job, cursor, limit)`:
/// - filters by `dispute_pda ==` dispute PDA
/// - sorts by `index` ascending, then `content_hash` as tie-breaker
/// - slices with opaque cursor
pub async fn list_evidence_paginated(
    repo: &dyn MetadataRepository,
    dispute_pda: &str,
    cursor: Option<&str>,
    limit: Option<usize>,
) -> Result<PaginatedEvidence, RepositoryError> {
    let offset = decode_cursor(cursor)?;
    let lim = validate_limit(limit)?;
    let mut all = repo.list_evidence_by_dispute(dispute_pda).await?;
    // Ensure sorted (repo already sorts, but be explicit for cursor stability).
    all.sort_by(|a, b| {
        a.index
            .cmp(&b.index)
            .then_with(|| a.content_hash.cmp(&b.content_hash))
    });

    if offset >= all.len() {
        return Ok(PaginatedEvidence {
            evidence: Vec::new(),
            next_cursor: None,
            has_more: false,
        });
    }
    let end = (offset + lim).min(all.len());
    let has_more = end < all.len();
    let next_cursor = if has_more {
        Some(encode_cursor(end))
    } else {
        None
    };
    let evidence = all.into_iter().skip(offset).take(lim).collect();
    Ok(PaginatedEvidence {
        evidence,
        next_cursor,
        has_more,
    })
}

// ---------------------------------------------------------------------------
// Linked creation — assigns next sequential index, hashes, validates
// ---------------------------------------------------------------------------

/// Create evidence linked to on-chain hash+index.
///
/// - Validates content length (`MAX_EVIDENCE_CONTENT_LEN`) and non-empty
/// - Computes SHA-256 hex and verifies it matches `content` (internal `verify_hash`)
/// - Assigns `index`: if `Some(idx)` validates sequential `idx == current_count`,
///   else auto-assigns `current_count` (next sequential index)
/// - Enforces `MAX_EVIDENCE_COUNT` and stores via `repo.create_evidence`
pub async fn create_evidence_linked(
    repo: &dyn MetadataRepository,
    dispute_pda: String,
    author: String,
    content: String,
    index: Option<u8>,
) -> Result<EvidenceMetadata, RepositoryError> {
    validate_evidence_content_len(&content).map_err(RepositoryError::Validation)?;
    let existing = repo.list_evidence_by_dispute(&dispute_pda).await?;
    if existing.len() >= MAX_EVIDENCE_COUNT as usize {
        return Err(RepositoryError::Storage(format!(
            "evidence limit reached ({}/{})",
            existing.len(),
            MAX_EVIDENCE_COUNT
        )));
    }
    let next_index = existing.len() as u8;
    let idx = match index {
        Some(i) => {
            validate_evidence_index(i).map_err(RepositoryError::Validation)?;
            if i != next_index {
                return Err(RepositoryError::Storage(format!(
                    "invalid evidence index: expected {}, got {}",
                    next_index, i
                )));
            }
            i
        }
        None => next_index,
    };
    let evidence = EvidenceMetadata::new(dispute_pda, idx, author, content)?;
    // Double-check hash (EvidenceMetadata::new already hashes, but be explicit).
    if !evidence.verify_hash() {
        return Err(RepositoryError::Storage(
            "evidence hash verification failed after creation".to_string(),
        ));
    }
    repo.create_evidence(evidence).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::EvidenceMetadata;
    use crate::repository::InMemoryMetadataRepository;

    fn pda(n: u8) -> String {
        format!("7a2YhCd7iivXfyySkp1pf5jj{:0>20}{:02}", n, n)
    }

    #[test]
    fn hash_hex_roundtrip_bytes32() {
        let content = "hello evidence content";
        let hex = hash_content_hex(content);
        assert_eq!(hex.len(), 64);
        let bytes = hex_to_bytes32(&hex).unwrap();
        assert_eq!(bytes32_to_hex(&bytes), hex);
        assert!(verify_hash_hex(content, &hex));
        assert!(verify_hash_bytes32(content, &bytes));
        // tampered content fails
        assert!(!verify_hash_hex("tampered", &hex));
        assert!(!verify_hash_bytes32("tampered", &bytes));
        // case-insensitive hex
        assert!(verify_hash_hex(content, &hex.to_uppercase()));
    }

    #[test]
    fn hash_bytes_vs_evidence_metadata() {
        let content = "evidence large content " .repeat(10);
        let ev = EvidenceMetadata::new(pda(50), 0, "author1111111111111111111111111111".into(), content.clone()).unwrap();
        let bytes = hash_content_bytes(content.trim());
        // EvidenceMetadata stores hex of trimmed content
        let expected_hex = hash_content_hex(content.trim());
        assert_eq!(ev.content_hash, expected_hex);
        assert_eq!(hex_to_bytes32(&ev.content_hash).unwrap(), bytes);
        assert!(ev.verify_hash());
        assert!(verify_evidence_link(&ev, &bytes, 0));
        // wrong index fails link
        assert!(!verify_evidence_link(&ev, &bytes, 1));
        // wrong hash fails link
        let mut bad = [0u8; 32];
        bad[0] = 1;
        assert!(!verify_evidence_link(&ev, &bad, 0));
    }

    #[test]
    fn hex_to_bytes32_errors() {
        assert!(hex_to_bytes32("short").is_err());
        assert!(hex_to_bytes32(&"zz".repeat(32)).is_err());
        assert!(hex_to_bytes32(&"00".repeat(33)).is_err());
    }

    #[test]
    fn validate_content_len_boundaries() {
        assert!(validate_evidence_content_len("a").is_ok());
        assert!(validate_evidence_content_len(&"a".repeat(MAX_EVIDENCE_CONTENT_LEN)).is_ok());
        assert!(validate_evidence_content_len(&"a".repeat(MAX_EVIDENCE_CONTENT_LEN + 1)).is_err());
        assert!(validate_evidence_content_len("   ").is_err());
        assert!(validate_evidence_content_len("").is_err());
    }

    #[test]
    fn validate_index_boundaries() {
        assert!(validate_evidence_index(0).is_ok());
        assert!(validate_evidence_index(MAX_EVIDENCE_INDEX).is_ok());
        assert!(validate_evidence_index(MAX_EVIDENCE_COUNT).is_err());
        assert!(validate_evidence_index(255).is_err());
    }

    #[test]
    fn cursor_opaque_and_roundtrip() {
        for off in [0usize, 1, 19, 20, 21, 99, 10_000] {
            let c = encode_cursor(off);
            assert_ne!(c, off.to_string(), "cursor must be opaque");
            assert_eq!(decode_cursor(Some(&c)).unwrap(), off);
        }
        assert_eq!(decode_cursor(None).unwrap(), 0);
        assert_eq!(decode_cursor(Some("")).unwrap(), 0);
        assert!(decode_cursor(Some("not-base64!!!")).is_err());
    }

    #[test]
    fn validate_limit_clamp() {
        assert_eq!(validate_limit(None).unwrap(), DEFAULT_PAGE_LIMIT);
        assert_eq!(validate_limit(Some(10)).unwrap(), 10);
        assert_eq!(validate_limit(Some(10_000)).unwrap(), MAX_PAGE_LIMIT);
        assert!(validate_limit(Some(0)).is_err());
    }

    #[tokio::test]
    async fn pagination_sorts_by_index_and_cursor_advances() {
        let repo = InMemoryMetadataRepository::new();
        let dispute = pda(100);
        // Insert out of order indices, verify pagination sorts
        for idx in [2u8, 0, 1] {
            let ev = EvidenceMetadata::new(
                dispute.clone(),
                idx,
                format!("author{:02}", idx),
                format!("content {}", idx),
            )
            .unwrap();
            repo.create_evidence(ev).await.unwrap();
        }
        // page 1 limit 2
        let p1 = list_evidence_paginated(&repo, &dispute, None, Some(2))
            .await
            .unwrap();
        assert_eq!(p1.evidence.len(), 2);
        assert_eq!(p1.evidence[0].index, 0);
        assert_eq!(p1.evidence[1].index, 1);
        assert!(p1.has_more);
        assert!(p1.next_cursor.is_some());
        assert_ne!(p1.next_cursor.as_ref().unwrap(), "2");
        // page 2
        let p2 = list_evidence_paginated(&repo, &dispute, p1.next_cursor.as_deref(), Some(2))
            .await
            .unwrap();
        assert_eq!(p2.evidence.len(), 1);
        assert_eq!(p2.evidence[0].index, 2);
        assert!(!p2.has_more);
        assert!(p2.next_cursor.is_none());
        // no duplicates, sorted preserved
        let combined: Vec<u8> = p1
            .evidence
            .iter()
            .chain(p2.evidence.iter())
            .map(|e| e.index)
            .collect();
        assert_eq!(combined, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn pagination_empty_and_offset_beyond() {
        let repo = InMemoryMetadataRepository::new();
        let dispute = pda(101);
        let p = list_evidence_paginated(&repo, &dispute, None, Some(10))
            .await
            .unwrap();
        assert_eq!(p.evidence.len(), 0);
        assert!(!p.has_more);
        // offset beyond length
        let c = encode_cursor(99);
        let p2 = list_evidence_paginated(&repo, &dispute, Some(&c), Some(10))
            .await
            .unwrap();
        assert_eq!(p2.evidence.len(), 0);
        assert!(!p2.has_more);
    }

    #[tokio::test]
    async fn pagination_invalid_cursor_and_limit() {
        let repo = InMemoryMetadataRepository::new();
        let dispute = pda(102);
        let bad = list_evidence_paginated(&repo, &dispute, Some("!!!bad"), Some(2)).await;
        assert!(bad.is_err());
        let zero = list_evidence_paginated(&repo, &dispute, None, Some(0)).await;
        assert!(zero.is_err());
    }

    #[tokio::test]
    async fn create_linked_auto_index_and_sequential_validation() {
        let repo = InMemoryMetadataRepository::new();
        let dispute = pda(110);
        // auto-index 0
        let e0 = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author1111111111111111111111111111".into(),
            "content 0".into(),
            None,
        )
        .await
        .unwrap();
        assert_eq!(e0.index, 0);
        assert!(verify_hash_hex("content 0", &e0.content_hash));
        // explicit correct index 1
        let e1 = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author1111111111111111111111111111".into(),
            "content 1".into(),
            Some(1),
        )
        .await
        .unwrap();
        assert_eq!(e1.index, 1);
        // wrong sequential index should fail
        let err = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author1111111111111111111111111111".into(),
            "content bad".into(),
            Some(5),
        )
        .await;
        assert!(err.is_err());
        // gap also fails if skipping
        let err2 = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author1111111111111111111111111111".into(),
            "content bad2".into(),
            Some(0),
        )
        .await;
        assert!(err2.is_err());
        // verify on-chain link
        let onchain_bytes = hex_to_bytes32(&e0.content_hash).unwrap();
        assert!(verify_evidence_link(&e0, &onchain_bytes, 0));
        // list paginated after creates
        let page = list_evidence_paginated(&repo, &dispute, None, Some(10))
            .await
            .unwrap();
        assert_eq!(page.evidence.len(), 2);
        assert_eq!(page.evidence[0].index, 0);
    }

    #[tokio::test]
    async fn create_linked_large_content_hash_verification() {
        let repo = InMemoryMetadataRepository::new();
        let dispute = pda(120);
        // 20_480 chars is allowed, 20_481 is not
        let large_ok = "a".repeat(MAX_EVIDENCE_CONTENT_LEN);
        let e = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author1111111111111111111111111111".into(),
            large_ok.clone(),
            None,
        )
        .await
        .unwrap();
        assert_eq!(e.content.chars().count(), MAX_EVIDENCE_CONTENT_LEN);
        assert!(e.verify_hash());
        assert!(verify_hash_bytes32(&large_ok, &hex_to_bytes32(&e.content_hash).unwrap()));
        // too large should fail via validation
        let too_large = "a".repeat(MAX_EVIDENCE_CONTENT_LEN + 1);
        let err = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author1111111111111111111111111111".into(),
            too_large,
            None,
        )
        .await;
        assert!(err.is_err());
        // empty also fails
        let err2 = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author1111111111111111111111111111".into(),
            "   ".into(),
            None,
        )
        .await;
        assert!(err2.is_err());
    }

    #[tokio::test]
    async fn create_linked_limit_enforced() {
        let repo = InMemoryMetadataRepository::new();
        let dispute = pda(130);
        for i in 0..MAX_EVIDENCE_COUNT {
            create_evidence_linked(
                &repo,
                dispute.clone(),
                format!("author{:02}", i),
                format!("content {}", i),
                None,
            )
            .await
            .unwrap();
        }
        // 11th should fail (limit reached)
        let err = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author-extra".into(),
            "extra content".into(),
            None,
        )
        .await;
        assert!(err.is_err());
        // list should have 10 sorted
        let page = list_evidence_paginated(&repo, &dispute, None, Some(100))
            .await
            .unwrap();
        assert_eq!(page.evidence.len(), 10);
        assert_eq!(page.evidence[9].index, 9);
        // cursor pagination with limit 3 -> 4 pages 3+3+3+1
        let p1 = list_evidence_paginated(&repo, &dispute, None, Some(3))
            .await
            .unwrap();
        assert_eq!(p1.evidence.len(), 3);
        assert!(p1.has_more);
        let p2 = list_evidence_paginated(&repo, &dispute, p1.next_cursor.as_deref(), Some(3))
            .await
            .unwrap();
        assert_eq!(p2.evidence.len(), 3);
        let p3 = list_evidence_paginated(&repo, &dispute, p2.next_cursor.as_deref(), Some(3))
            .await
            .unwrap();
        assert_eq!(p3.evidence.len(), 3);
        let p4 = list_evidence_paginated(&repo, &dispute, p3.next_cursor.as_deref(), Some(3))
            .await
            .unwrap();
        assert_eq!(p4.evidence.len(), 1);
        assert!(!p4.has_more);
        let mut seen = std::collections::HashSet::new();
        for ev in p1
            .evidence
            .iter()
            .chain(p2.evidence.iter())
            .chain(p3.evidence.iter())
            .chain(p4.evidence.iter())
        {
            assert!(seen.insert(ev.index));
        }
        assert_eq!(seen.len(), 10);
    }

    #[tokio::test]
    async fn create_linked_verifies_stored_hash() {
        let repo = InMemoryMetadataRepository::new();
        let dispute = pda(140);
        let e = create_evidence_linked(
            &repo,
            dispute.clone(),
            "author1111111111111111111111111111".into(),
            "verifiable content".into(),
            None,
        )
        .await
        .unwrap();
        let stored = repo.get_evidence(&dispute, 0).await.unwrap().unwrap();
        assert_eq!(stored.content_hash, e.content_hash);
        // simulate on-chain verification: content_hash bytes match
        let onchain = hex_to_bytes32(&stored.content_hash).unwrap();
        assert!(verify_evidence_link(&stored, &onchain, 0));
        // tamper stored content_hash hex (simulate mismatch)
        let mut tampered = stored.clone();
        tampered.content_hash = "00".repeat(32);
        assert!(!verify_evidence_link(&tampered, &onchain, 0));
    }
}
