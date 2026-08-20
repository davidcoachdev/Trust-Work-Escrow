//! Utilities for `trust-escrow-sdk` — timeouts, retries and cursor pagination (T7).
//!
//! - Cursor pagination: opaque base64-encoded offset, never exposes raw indices.
//! - Timeouts: every RPC path is bounded by [`DEFAULT_RPC_TIMEOUT`]; exceeding it
//!   yields a typed [`BackendError::Timeout`].
//! - Retries: bounded, no infinite poll loops.

use std::time::Duration;

use crate::error::{BackendError, Result};

/// Default per-RPC timeout. Short enough to keep the API responsive, long
/// enough for localnet/devnet.
/// Overridable per-call via `*_with_timeout`.
pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);

/// Default page size for listing endpoints.
pub const DEFAULT_PAGE_LIMIT: usize = 20;

/// Maximum page size — enforced server-side to prevent unbounded scans.
pub const MAX_PAGE_LIMIT: usize = 100;

/// Maximum number of RPC retries (bounded).
pub const MAX_RETRIES: u32 = 3;

/// Encode a pagination cursor (an offset) as opaque base64url (no pad).
pub fn encode_cursor(offset: usize) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let bytes = (offset as u64).to_be_bytes();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Decode a cursor back to an offset.
///
/// `None` or empty means offset 0. Invalid encoding yields `InvalidParameter`.
pub fn decode_cursor(cursor: Option<&str>) -> Result<usize> {
    let Some(c) = cursor else {
        return Ok(0);
    };
    let c = c.trim();
    if c.is_empty() {
        return Ok(0);
    }
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let bytes = URL_SAFE_NO_PAD
        .decode(c)
        .map_err(|e| BackendError::invalid_parameter(format!("invalid cursor: {}", e)))?;
    if bytes.len() != 8 {
        return Err(BackendError::invalid_parameter("invalid cursor length"));
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&bytes);
    let v = u64::from_be_bytes(arr);
    // Guard against offsets that would overflow usize on 32-bit (theoretical).
    if v > (usize::MAX as u64) {
        return Err(BackendError::invalid_parameter("cursor offset overflow"));
    }
    Ok(v as usize)
}

/// Clamp and validate a requested page limit.
pub fn validate_limit(limit: Option<usize>) -> Result<usize> {
    let l = limit.unwrap_or(DEFAULT_PAGE_LIMIT);
    if l == 0 {
        return Err(BackendError::invalid_parameter("limit must be > 0"));
    }
    if l > MAX_PAGE_LIMIT {
        return Ok(MAX_PAGE_LIMIT);
    }
    Ok(l)
}

/// Sleep with a bounded timeout, mapping elapsed to `BackendError::Timeout`.
///
/// This helper is RPC-agnostic: it wraps any future with a deadline.
pub async fn with_timeout<F, T>(duration: Duration, fut: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match tokio::time::timeout(duration, fut).await {
        Ok(res) => res,
        Err(_) => Err(BackendError::timeout(format!(
            "operation timed out after {}ms",
            duration.as_millis()
        ))),
    }
}

/// Retry helper with exponential backoff, bounded to `MAX_RETRIES`.
///
/// `op` is re-executed up to `max_retries` times (so total attempts is
/// `max_retries + 1`). Only retryable errors (currently: Timeout) are retried;
/// contract / validation errors are returned immediately.
pub async fn with_retry<F, Fut, T>(mut max_retries: u32, mut op: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    if max_retries > MAX_RETRIES {
        max_retries = MAX_RETRIES;
    }
    let mut attempt: u32 = 0;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if e.is_timeout() && attempt < max_retries => {
                let backoff = Duration::from_millis(100 * (1u64 << attempt));
                tokio::time::sleep(backoff).await;
                attempt += 1;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Pagination parameters decoded from query inputs.
#[derive(Debug, Clone)]
pub struct PageParams {
    pub offset: usize,
    pub limit: usize,
}

impl PageParams {
    /// Build from optional cursor and limit query params, applying validation.
    pub fn from_cursor_limit(cursor: Option<&str>, limit: Option<usize>) -> Result<Self> {
        let offset = decode_cursor(cursor)?;
        let limit = validate_limit(limit)?;
        Ok(Self { offset, limit })
    }
}

/// Result of a paginated listing.
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl<T> Page<T> {
    /// Build a page from a slice that has already been filtered and sorted.
    ///
    /// `offset` is the starting index in the full set, `limit` the requested
    /// page size. Returns items for the window and a cursor pointing past it
    /// when more data remains.
    pub fn from_slice(all: Vec<T>, offset: usize, limit: usize) -> Self {
        if offset >= all.len() {
            return Self {
                items: Vec::new(),
                next_cursor: None,
                has_more: false,
            };
        }
        let end = (offset + limit).min(all.len());
        let has_more = end < all.len();
        let next_cursor = if has_more {
            Some(encode_cursor(end))
        } else {
            None
        };
        let items = all.into_iter().skip(offset).take(limit).collect();
        Self {
            items,
            next_cursor,
            has_more,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_roundtrip() {
        for off in [0usize, 1, 20, 100, 9999] {
            let c = encode_cursor(off);
            let got = decode_cursor(Some(&c)).unwrap();
            assert_eq!(got, off);
        }
        assert_eq!(decode_cursor(None).unwrap(), 0);
        assert_eq!(decode_cursor(Some("")).unwrap(), 0);
        assert!(decode_cursor(Some("!!!not-base64")).is_err());
    }

    #[test]
    fn limit_validation() {
        assert_eq!(validate_limit(None).unwrap(), DEFAULT_PAGE_LIMIT);
        assert_eq!(validate_limit(Some(5)).unwrap(), 5);
        assert_eq!(validate_limit(Some(1000)).unwrap(), MAX_PAGE_LIMIT);
        assert!(validate_limit(Some(0)).is_err());
    }

    #[test]
    fn page_from_slice_cursor() {
        let all: Vec<u32> = (0..30).collect();
        let p1 = Page::from_slice(all.clone(), 0, 20);
        assert_eq!(p1.items.len(), 20);
        assert!(p1.has_more);
        assert!(p1.next_cursor.is_some());
        let off = decode_cursor(p1.next_cursor.as_deref()).unwrap();
        assert_eq!(off, 20);
        let p2 = Page::from_slice(all.clone(), off, 20);
        assert_eq!(p2.items.len(), 10);
        assert!(!p2.has_more);
        assert!(p2.next_cursor.is_none());
        // no overlap
        let set1: std::collections::HashSet<_> = p1.items.iter().collect();
        for v in &p2.items {
            assert!(!set1.contains(v));
        }
    }

    #[tokio::test]
    async fn timeout_maps_to_typed_error() {
        let res: Result<()> = with_timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        })
        .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().is_timeout());
    }

    #[tokio::test]
    async fn with_retry_retries_timeout_then_succeeds() {
        use std::sync::{Arc, Mutex};
        let calls = Arc::new(Mutex::new(0usize));
        let c = calls.clone();
        let res = with_retry(2, move || {
            let c = c.clone();
            async move {
                let mut g = c.lock().unwrap();
                *g += 1;
                if *g < 2 {
                    Err(BackendError::timeout("fake"))
                } else {
                    Ok(42u32)
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(res, 42);
    }

    #[tokio::test]
    async fn with_retry_does_not_retry_non_timeout() {
        use std::sync::{Arc, Mutex};
        let calls = Arc::new(Mutex::new(0usize));
        let c = calls.clone();
        let res: Result<()> = with_retry(3, move || {
            let c = c.clone();
            async move {
                *c.lock().unwrap() += 1;
                Err(BackendError::invalid_parameter("bad"))
            }
        })
        .await;
        assert!(res.is_err());
        assert_eq!(*calls.lock().unwrap(), 1);
    }
}
