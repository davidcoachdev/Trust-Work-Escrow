//! Signature authentication — T15.
//!
//! Verifies ed25519 signatures produced by Solana wallets (e.g. Phantom,
//! Solflare). Clients sign an arbitrary `message` with their keypair and send:
//!   - `x-pubkey`: base58-encoded 32-byte pubkey
//!   - `x-signature`: base64 (standard or url-safe) or hex-encoded 64-byte ed25519 signature
//!   - `x-message`: the exact message that was signed (utf-8)
//!
//! The server recomputes `verify(pubkey, message, signature)` using
//! `ed25519-dalek`. No secrets are stored; verification is stateless and
//! constant-time. Failures map to `401 Unauthorized` via `ApiError`.
//!
//! Axum integration: use `AuthenticatedUser` as an extractor in handlers that
//! require authentication. Missing or invalid headers → `401`.
//!
//! ```ignore
//! async fn create_job(
//!     auth: AuthenticatedUser,
//!     State(state): State<AppState>,
//!     Json(req): Json<CreateJobRequest>,
//! ) -> Result<impl IntoResponse, ApiError> { ... }
//! ```

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::error::ApiError;

// ---------------------------------------------------------------------------
// Core verification — pure functions, easy to unit-test
// ---------------------------------------------------------------------------

/// Decode a base58 pubkey to 32 bytes.
pub fn decode_pubkey(pubkey_b58: &str) -> Result<[u8; 32], ApiError> {
    let trimmed = pubkey_b58.trim();
    if trimmed.is_empty() {
        return Err(ApiError::Unauthorized("x-pubkey is required".into()));
    }
    let decoded = bs58::decode(trimmed)
        .into_vec()
        .map_err(|e| ApiError::Unauthorized(format!("invalid x-pubkey base58: {}", e)))?;
    if decoded.len() != 32 {
        return Err(ApiError::Unauthorized(format!(
            "x-pubkey must decode to 32 bytes, got {}",
            decoded.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&decoded);
    Ok(out)
}

/// Decode a signature that may be base64 (standard/url-safe, padded or not)
/// or hex (128 chars). Returns 64 bytes.
pub fn decode_signature(sig_str: &str) -> Result<[u8; 64], ApiError> {
    let trimmed = sig_str.trim();
    if trimmed.is_empty() {
        return Err(ApiError::Unauthorized("x-signature is required".into()));
    }

    // Try hex first if it looks like 128 hex chars
    if trimmed.len() == 128 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        let bytes = hex::decode(trimmed)
            .map_err(|e| ApiError::Unauthorized(format!("invalid hex signature: {}", e)))?;
        let mut out = [0u8; 64];
        out.copy_from_slice(&bytes);
        return Ok(out);
    }

    // Try base64 — accept standard, url-safe, padded and unpadded
    // Try standard padded
    for engine in [
        base64::engine::general_purpose::STANDARD,
        base64::engine::general_purpose::STANDARD_NO_PAD,
        base64::engine::general_purpose::URL_SAFE,
        base64::engine::general_purpose::URL_SAFE_NO_PAD,
    ] {
        use base64::Engine as _;
        if let Ok(bytes) = engine.decode(trimmed) {
            if bytes.len() == 64 {
                let mut out = [0u8; 64];
                out.copy_from_slice(&bytes);
                return Ok(out);
            }
        }
    }

    Err(ApiError::Unauthorized(
        "x-signature must be 64-byte ed25519 signature encoded as base64 or hex".into(),
    ))
}

/// Verify an ed25519 signature over `message` using a 32-byte pubkey.
pub fn verify_ed25519(
    pubkey_bytes: &[u8; 32],
    message: &[u8],
    signature_bytes: &[u8; 64],
) -> Result<(), ApiError> {
    let vk = VerifyingKey::from_bytes(pubkey_bytes)
        .map_err(|e| ApiError::Unauthorized(format!("invalid ed25519 pubkey: {}", e)))?;
    let sig = Signature::from_bytes(signature_bytes);
    vk.verify(message, &sig)
        .map_err(|_| ApiError::Unauthorized("signature verification failed".into()))
}

/// High-level helper: decode inputs and verify.
pub fn verify_signature(pubkey_b58: &str, message: &[u8], signature_str: &str) -> Result<(), ApiError> {
    let pk = decode_pubkey(pubkey_b58)?;
    let sig = decode_signature(signature_str)?;
    verify_ed25519(&pk, message, &sig)
}

// ---------------------------------------------------------------------------
// Axum extractor — `AuthenticatedUser`
// ---------------------------------------------------------------------------

/// Successfully authenticated caller — contains the verified pubkey.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// Base58 pubkey string as sent by the client.
    pub pubkey: String,
    /// Decoded 32-byte pubkey.
    pub pubkey_bytes: [u8; 32],
    /// The message that was signed.
    pub message: String,
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;

        let pubkey = headers
            .get("x-pubkey")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("missing x-pubkey header".into()))?;

        let signature = headers
            .get("x-signature")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("missing x-signature header".into()))?;

        let message = headers
            .get("x-message")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("missing x-message header".into()))?;

        if message.is_empty() {
            return Err(ApiError::Unauthorized("x-message cannot be empty".into()));
        }

        verify_signature(pubkey, message.as_bytes(), signature)?;

        let pubkey_bytes = decode_pubkey(pubkey)?;
        Ok(Self {
            pubkey: pubkey.to_string(),
            pubkey_bytes,
            message: message.to_string(),
        })
    }
}

impl IntoResponse for AuthenticatedUser {
    fn into_response(self) -> Response {
        // Not used as a response; only as extractor. Provide a trivial impl.
        (StatusCode::OK, axum::Json(serde_json::json!({"pubkey": self.pubkey}))).into_response()
    }
}

// ---------------------------------------------------------------------------
// Helper for optional auth — useful for routes that should enforce auth but
// keep backward compat with tests that omit headers (log warning instead).
// ---------------------------------------------------------------------------

/// Try to authenticate; if headers are absent, return `None` (caller decides
/// whether to require auth).
pub fn try_authenticate(headers: &axum::http::HeaderMap) -> Result<Option<AuthenticatedUser>, ApiError> {
    let has_any = headers.contains_key("x-pubkey")
        || headers.contains_key("x-signature")
        || headers.contains_key("x-message");
    if !has_any {
        return Ok(None);
    }
    // If any is present, all must be present and valid
    let pubkey = headers
        .get("x-pubkey")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("missing x-pubkey header".into()))?;
    let signature = headers
        .get("x-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("missing x-signature header".into()))?;
    let message = headers
        .get("x-message")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("missing x-message header".into()))?;
    verify_signature(pubkey, message.as_bytes(), signature)?;
    let pubkey_bytes = decode_pubkey(pubkey)?;
    Ok(Some(AuthenticatedUser {
        pubkey: pubkey.to_string(),
        pubkey_bytes,
        message: message.to_string(),
    }))
}

// ---------------------------------------------------------------------------
// Tests — deterministic keypair, no network
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn test_keypair() -> SigningKey {
        // Deterministic keypair from fixed seed (not random per test, for reproducibility)
        let seed = [7u8; 32];
        SigningKey::from_bytes(&seed)
    }

    fn pubkey_b58(sk: &SigningKey) -> String {
        bs58::encode(sk.verifying_key().to_bytes()).into_string()
    }

    fn sign_b64(sk: &SigningKey, msg: &[u8]) -> String {
        use base64::Engine as _;
        let sig = sk.sign(msg);
        base64::engine::general_purpose::STANDARD.encode(sig.to_bytes())
    }

    #[test]
    fn verify_roundtrip_ok() {
        let sk = test_keypair();
        let pk = pubkey_b58(&sk);
        let msg = b"hello trust-escrow";
        let sig = sign_b64(&sk, msg);
        assert!(verify_signature(&pk, msg, &sig).is_ok());
    }

    #[test]
    fn verify_wrong_message_fails() {
        let sk = test_keypair();
        let pk = pubkey_b58(&sk);
        let sig = sign_b64(&sk, b"original");
        assert!(verify_signature(&pk, b"tampered", &sig).is_err());
    }

    #[test]
    fn verify_wrong_pubkey_fails() {
        let sk = test_keypair();
        let other_seed = [9u8; 32];
        let other_sk = SigningKey::from_bytes(&other_seed);
        let other_pk = pubkey_b58(&other_sk);
        let sig = sign_b64(&sk, b"msg");
        assert!(verify_signature(&other_pk, b"msg", &sig).is_err());
    }

    #[test]
    fn decode_pubkey_valid() {
        let sk = test_keypair();
        let pk = pubkey_b58(&sk);
        assert!(decode_pubkey(&pk).is_ok());
        assert!(decode_pubkey("").is_err());
        assert!(decode_pubkey("short").is_err());
    }

    #[test]
    fn decode_signature_base64_and_hex() {
        let sk = test_keypair();
        let msg = b"test";
        let sig_b64 = sign_b64(&sk, msg);
        assert!(decode_signature(&sig_b64).is_ok());
        // hex variant
        let sig_bytes = {
            let sig = sk.sign(msg);
            sig.to_bytes()
        };
        let hex_sig = hex::encode(sig_bytes);
        assert!(decode_signature(&hex_sig).is_ok());
        assert!(decode_signature("").is_err());
        assert!(decode_signature("not-valid!!!").is_err());
    }

    #[test]
    fn decode_signature_invalid_length() {
        use base64::Engine as _;
        // 32 bytes instead of 64
        let short = base64::engine::general_purpose::STANDARD.encode([1u8; 32]);
        assert!(decode_signature(&short).is_err());
    }

    #[tokio::test]
    async fn extractor_missing_headers_returns_401() {
        use axum::http::{Method, Request};
        use axum::body::Body;
        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let res = AuthenticatedUser::from_request_parts(&mut parts, &()).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn extractor_valid_headers_ok() {
        use axum::http::{Method, Request};
        use axum::body::Body;
        let sk = test_keypair();
        let pk = pubkey_b58(&sk);
        let msg = "authenticated request";
        let sig = sign_b64(&sk, msg.as_bytes());
        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header("x-pubkey", pk.clone())
            .header("x-signature", sig)
            .header("x-message", msg)
            .body(Body::empty())
            .unwrap();
        let (mut parts, _) = req.into_parts();
        let user = AuthenticatedUser::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(user.pubkey, pk);
    }

    #[test]
    fn try_authenticate_none_when_no_headers() {
        let headers = axum::http::HeaderMap::new();
        assert!(try_authenticate(&headers).unwrap().is_none());
    }

    #[test]
    fn try_authenticate_some_when_valid() {
        let sk = test_keypair();
        let pk = pubkey_b58(&sk);
        let msg = "msg";
        let sig = sign_b64(&sk, msg.as_bytes());
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-pubkey", pk.parse().unwrap());
        headers.insert("x-signature", sig.parse().unwrap());
        headers.insert("x-message", msg.parse().unwrap());
        assert!(try_authenticate(&headers).unwrap().is_some());
    }
}
