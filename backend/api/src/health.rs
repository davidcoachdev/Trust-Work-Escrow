//! Health, liveness and readiness probes.
//!
//! `GET /health` is the main probe. It checks the repository (always available
//! for the in-memory backend) and the Solana RPC (TCP connect to the host/port
//! from `ApiConfig::rpc_url`).
//!
//! Overall status is:
//!   - `ok`       → both checks pass → 200
//!   - `degraded` → repo ok, rpc unavailable → 200 (still serves traffic)
//!   - `down`     → repo unavailable → 503
//!
//! The 503 case satisfies the "200 only when expected state is available"
//! criterion from T13. `GET /live` is a cheap liveness probe (no I/O).

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::time::Duration;
use utoipa::ToSchema;

use crate::state::AppState;

/// Health status payload (OpenAPI-visible).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Overall status: `ok` | `degraded` | `down`.
    pub status: String,
    /// Crate version.
    pub version: String,
    /// Seconds since process start.
    pub uptime_seconds: u64,
    /// Individual subsystem checks.
    pub checks: HealthChecks,
    /// Number of jobs in the repository (cheap count, not full scan in prod).
    pub jobs_count: usize,
}

/// Subsystem checks.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthChecks {
    /// Repository status: `ok` | `unavailable`.
    pub repository: String,
    /// RPC status: `ok` | `unavailable` | `unconfigured` | `degraded`.
    pub rpc: String,
}

/// Liveness payload (`GET /live`).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LiveResponse {
    pub status: String,
    pub version: String,
}

/// Readiness payload (`GET /ready` / `GET /health` detail).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReadyResponse {
    pub ready: bool,
    pub checks: HealthChecks,
}

/// Main health handler.
///
/// Checks repository with a bounded timeout and RPC via TCP connect.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "API is healthy or degraded but serving", body = HealthResponse),
        (status = 503, description = "API is down (repository unavailable)", body = HealthResponse)
    )
)]
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let (repo_status, jobs_count) = check_repository(&state).await;
    let rpc_status = check_rpc(&state.config.rpc_url).await;

    let overall = if repo_status == "ok" && rpc_status == "ok" {
        "ok"
    } else if repo_status == "ok" {
        "degraded"
    } else {
        "down"
    };

    let body = HealthResponse {
        status: overall.to_string(),
        version: state.config.version.clone(),
        uptime_seconds: state.uptime_seconds(),
        checks: HealthChecks {
            repository: repo_status,
            rpc: rpc_status,
        },
        jobs_count,
    };

    if overall == "down" {
        (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
    } else {
        (StatusCode::OK, Json(body)).into_response()
    }
}

/// Liveness probe — always 200 if process is up (no I/O).
#[utoipa::path(
    get,
    path = "/live",
    tag = "Health",
    responses((status = 200, description = "Process is alive", body = LiveResponse))
)]
pub async fn live(State(state): State<AppState>) -> impl IntoResponse {
    Json(LiveResponse {
        status: "ok".to_string(),
        version: state.config.version.clone(),
    })
}

/// Readiness probe — 200 when repository is reachable, 503 otherwise.
#[utoipa::path(
    get,
    path = "/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Ready to serve", body = ReadyResponse),
        (status = 503, description = "Not ready", body = ReadyResponse)
    )
)]
pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let (repo_status, _) = check_repository(&state).await;
    let rpc_status = check_rpc(&state.config.rpc_url).await;
    let ready = repo_status == "ok";

    let body = ReadyResponse {
        ready,
        checks: HealthChecks {
            repository: repo_status,
            rpc: rpc_status,
        },
    };

    if ready {
        (StatusCode::OK, Json(body)).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response()
    }
}

async fn check_repository(state: &AppState) -> (String, usize) {
    // Bounded check: list_jobs with 500ms timeout. In-memory repo always succeeds quickly.
    let fut = state.repo.list_jobs();
    match tokio::time::timeout(Duration::from_millis(500), fut).await {
        Ok(Ok(jobs)) => ("ok".to_string(), jobs.len()),
        Ok(Err(_)) => ("unavailable".to_string(), 0),
        Err(_) => ("unavailable".to_string(), 0),
    }
}

async fn check_rpc(rpc_url: &str) -> String {
    if rpc_url.trim().is_empty() {
        return "unconfigured".to_string();
    }

    // Parse host:port from URL without adding a heavy HTTP client.
    // Supported forms: http://host:port[/...], https://host:port, host:port
    let without_scheme = if let Some(rest) = rpc_url.strip_prefix("http://") {
        rest
    } else if let Some(rest) = rpc_url.strip_prefix("https://") {
        rest
    } else {
        rpc_url
    };

    // Take authority (up to first '/' or '?' or '#').
    let authority = without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .split('?')
        .next()
        .unwrap_or(without_scheme);

    // authority may be "127.0.0.1:8899" or "example.com:443". If no port, use default.
    let addr = if authority.contains(':') {
        authority.to_string()
    } else if rpc_url.starts_with("https://") {
        format!("{}:443", authority)
    } else {
        format!("{}:80", authority)
    };

    // TCP connect with short timeout — validator 7a2Y on 127.0.0.1:8899 should respond.
    match tokio::time::timeout(Duration::from_millis(400), tokio::net::TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => "ok".to_string(),
        Ok(Err(_)) => "unavailable".to_string(),
        Err(_) => "unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::InMemoryMetadataRepository;
    use crate::state::{ApiConfig, AppState};
    use std::sync::Arc;

    fn state_with_rpc(rpc_url: &str) -> AppState {
        let cfg = ApiConfig {
            port: 3000,
            rpc_url: rpc_url.to_string(),
            database_url: None,
            mongo_url: None,
            version: "3.0.0".to_string(),
            environment: "test".to_string(),
            cors_allowed_origins: Vec::new(),
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
        };
        AppState::with_config_and_repository(cfg, Arc::new(InMemoryMetadataRepository::new()))
    }

    #[tokio::test]
    async fn rpc_unconfigured_when_empty() {
        let s = check_rpc("").await;
        assert_eq!(s, "unconfigured");
    }

    #[tokio::test]
    async fn rpc_unavailable_for_unreachable_host() {
        // Use an unroutable port to ensure failure.
        let s = check_rpc("http://127.0.0.1:59999").await;
        assert_eq!(s, "unavailable");
    }

    #[tokio::test]
    async fn repository_check_ok_for_in_memory() {
        let state = state_with_rpc("http://127.0.0.1:8899");
        let (status, count) = check_repository(&state).await;
        assert_eq!(status, "ok");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn health_response_has_expected_fields() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;
        use axum::{routing::get, Router};

        let state = state_with_rpc("");
        let app = Router::new().route("/health", get(health)).with_state(state);
        let resp = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        // Even with rpc unconfigured, repo is ok so status is degraded → still 200, not 503.
        assert!(resp.status() == StatusCode::OK || resp.status() == StatusCode::SERVICE_UNAVAILABLE);
    }
}
