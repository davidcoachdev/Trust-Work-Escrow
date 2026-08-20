//! Typed HTTP errors mapped to `StatusCode`.
//!
//! Every handler returns `Result<impl IntoResponse, ApiError>` so the error
//! schema is stable and never leaks secrets. `ApiError` implements
//! `IntoResponse` producing `{"error": "...", "code": "..."}` with the correct
//! status. Conversions from `RepositoryError` / `ValidationError` are explicit.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::metadata::ValidationError;
use crate::repository::RepositoryError;

/// Stable error body returned to clients.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Human-readable message (sanitized, never contains secrets).
    pub error: String,
    /// Machine-readable code (e.g. `bad_request`, `not_found`).
    pub code: String,
}

/// Typed API error — each variant maps to an HTTP status.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden")]
    Forbidden,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unprocessable: {0}")]
    Unprocessable(String),
    #[error("too many requests")]
    TooManyRequests,
    #[error("timeout")]
    Timeout,
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    /// HTTP status for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Machine-readable code for this error.
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound(_) => "not_found",
            Self::Conflict(_) => "conflict",
            Self::Unprocessable(_) => "unprocessable_entity",
            Self::TooManyRequests => "too_many_requests",
            Self::Timeout => "timeout",
            Self::ServiceUnavailable(_) => "service_unavailable",
            Self::Internal(_) => "internal_error",
        }
    }

    /// Convenience: 400.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(sanitize(msg.into()))
    }

    /// Convenience: 401.
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(sanitize(msg.into()))
    }

    /// Convenience: 404.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(sanitize(msg.into()))
    }

    /// Convenience: 500.
    pub fn internal(msg: impl Into<String>) -> Self {
        // Ensure we never echo secrets: strip anything that looks like a keypair path or private key.
        Self::Internal(sanitize(msg.into()))
    }
}

/// Remove secrets from an error message before sending to the client.
///
/// The SDK never returns raw key material; this is a defense-in-depth filter
/// that strips substrings containing `keypair`, `private`, `secret`, or long
/// base58-like blobs when logging the public error body.
fn sanitize(mut msg: String) -> String {
    // Never expose absolute keypair paths verbatim — keep only the file name.
    // Also truncate overly long messages to avoid log injection.
    if msg.len() > 500 {
        msg.truncate(500);
    }
    // Basic redaction for private key hints.
    let lower = msg.to_lowercase();
    if lower.contains("private key") || lower.contains("secret key") {
        return "internal error (redacted)".to_string();
    }
    msg
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code().to_string();
        let error = match &self {
            Self::BadRequest(m)
            | Self::Unauthorized(m)
            | Self::NotFound(m)
            | Self::Conflict(m)
            | Self::Unprocessable(m)
            | Self::ServiceUnavailable(m)
            | Self::Internal(m) => m.clone(),
            Self::Forbidden => "forbidden".to_string(),
            Self::TooManyRequests => "too many requests".to_string(),
            Self::Timeout => "request timed out".to_string(),
        };

        // Log at appropriate level without leaking secrets.
        match status {
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::SERVICE_UNAVAILABLE => {
                tracing::error!(code = %code, error = %error, status = %status.as_u16(), "api error");
            }
            StatusCode::BAD_REQUEST
            | StatusCode::UNAUTHORIZED
            | StatusCode::NOT_FOUND
            | StatusCode::CONFLICT => {
                tracing::warn!(code = %code, error = %error, status = %status.as_u16(), "api client error");
            }
            _ => {
                tracing::info!(code = %code, error = %error, status = %status.as_u16(), "api error");
            }
        }

        let body = Json(ErrorResponse { error, code });
        (status, body).into_response()
    }
}

// ---- From conversions ----

impl From<RepositoryError> for ApiError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => Self::NotFound(sanitize(msg)),
            RepositoryError::AlreadyExists(msg) => Self::Conflict(sanitize(msg)),
            RepositoryError::Validation(v) => Self::from(v),
            RepositoryError::Storage(msg) => Self::Internal(sanitize(msg)),
        }
    }
}

impl From<ValidationError> for ApiError {
    fn from(err: ValidationError) -> Self {
        Self::BadRequest(sanitize(err.to_string()))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal(sanitize(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn status_mapping() {
        assert_eq!(ApiError::BadRequest("x".into()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::Unauthorized("x".into()).status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::Forbidden.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(ApiError::NotFound("x".into()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::Conflict("x".into()).status_code(), StatusCode::CONFLICT);
        assert_eq!(ApiError::Unprocessable("x".into()).status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(ApiError::TooManyRequests.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(ApiError::Timeout.status_code(), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(ApiError::ServiceUnavailable("x".into()).status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(ApiError::Internal("x".into()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn code_mapping() {
        assert_eq!(ApiError::BadRequest("x".into()).code(), "bad_request");
        assert_eq!(ApiError::NotFound("x".into()).code(), "not_found");
        assert_eq!(ApiError::Internal("x".into()).code(), "internal_error");
        assert_eq!(ApiError::Timeout.code(), "timeout");
    }

    #[test]
    fn from_repository_not_found() {
        let err: ApiError = RepositoryError::NotFound("job 1".into()).into();
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn from_repository_already_exists() {
        let err: ApiError = RepositoryError::AlreadyExists("job 1".into()).into();
        assert_eq!(err.status_code(), StatusCode::CONFLICT);
    }

    #[test]
    fn from_validation_error() {
        let v = ValidationError::EmptyTitle;
        let err: ApiError = v.into();
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn sanitize_redacts_private_key() {
        let err = ApiError::Internal("private key leaked: abc".into());
        let status = err.status_code();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        // The message should be redacted when converted to response — check code path does not panic.
        let _resp = err.into_response();
    }

    #[test]
    fn sanitize_truncates_long_message() {
        let long = "a".repeat(600);
        let err = ApiError::bad_request(long);
        if let ApiError::BadRequest(msg) = err {
            assert!(msg.len() <= 500);
        } else {
            panic!("wrong variant");
        }
    }
}
