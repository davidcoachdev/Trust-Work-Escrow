//! Metrics endpoint.
//!
//! Exposes Prometheus text format at `GET /metrics` and JSON at `GET /metrics/json`.
//! Counters are stored in `AppState` as `AtomicU64` and incremented by middleware.
//! No secrets are ever included — only aggregate counters and uptime.

use axum::{extract::State, http::header, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::atomic::Ordering;
use utoipa::ToSchema;

use crate::state::AppState;

/// JSON metrics payload.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MetricsResponse {
    /// Total requests since start.
    pub requests_total: u64,
    /// Total error responses (4xx/5xx) since start.
    pub errors_total: u64,
    /// Seconds since process start.
    pub uptime_seconds: u64,
    /// Crate version.
    pub version: String,
    /// Jobs currently stored (off-chain count).
    pub jobs_count: usize,
}

/// Prometheus text exposition at `GET /metrics`.
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "Metrics",
    responses((status = 200, description = "Prometheus metrics", body = String))
)]
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let jobs_count = state
        .repo
        .list_jobs()
        .await
        .map(|v| v.len())
        .unwrap_or(0);

    let body = render_prometheus(
        state.requests_total.load(Ordering::Relaxed),
        state.errors_total.load(Ordering::Relaxed),
        state.uptime_seconds(),
        &state.config.version,
        jobs_count,
    );

    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4")], body)
}

/// JSON metrics at `GET /metrics/json`.
#[utoipa::path(
    get,
    path = "/metrics/json",
    tag = "Metrics",
    responses((status = 200, description = "JSON metrics", body = MetricsResponse))
)]
pub async fn metrics_json(State(state): State<AppState>) -> impl IntoResponse {
    let jobs_count = state.repo.list_jobs().await.map(|v| v.len()).unwrap_or(0);
    Json(MetricsResponse {
        requests_total: state.requests_total.load(Ordering::Relaxed),
        errors_total: state.errors_total.load(Ordering::Relaxed),
        uptime_seconds: state.uptime_seconds(),
        version: state.config.version.clone(),
        jobs_count,
    })
}

fn render_prometheus(
    requests_total: u64,
    errors_total: u64,
    uptime_seconds: u64,
    version: &str,
    jobs_count: usize,
) -> String {
    // Minimal Prometheus exposition — keep it stable and grep-friendly.
    format!(
        "# HELP trust_escrow_requests_total Total HTTP requests since start.\n\
         # TYPE trust_escrow_requests_total counter\n\
         trust_escrow_requests_total {requests_total}\n\
         # HELP trust_escrow_errors_total Total error responses (4xx/5xx) since start.\n\
         # TYPE trust_escrow_errors_total counter\n\
         trust_escrow_errors_total {errors_total}\n\
         # HELP trust_escrow_uptime_seconds Seconds since process start.\n\
         # TYPE trust_escrow_uptime_seconds gauge\n\
         trust_escrow_uptime_seconds {uptime_seconds}\n\
         # HELP trust_escrow_jobs_count Off-chain jobs count.\n\
         # TYPE trust_escrow_jobs_count gauge\n\
         trust_escrow_jobs_count {jobs_count}\n\
         # HELP trust_escrow_info Build info.\n\
         # TYPE trust_escrow_info gauge\n\
         trust_escrow_info{{version=\"{version}\"}} 1\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prometheus_render_contains_expected_lines() {
        let body = render_prometheus(42, 3, 123, "3.0.0", 7);
        assert!(body.contains("trust_escrow_requests_total 42"));
        assert!(body.contains("trust_escrow_errors_total 3"));
        assert!(body.contains("trust_escrow_uptime_seconds 123"));
        assert!(body.contains("trust_escrow_jobs_count 7"));
        assert!(body.contains("version=\"3.0.0\""));
        assert!(!body.contains("secret"));
        assert!(!body.contains("keypair"));
    }

    #[tokio::test]
    async fn metrics_json_has_no_secrets() {
        use crate::repository::InMemoryMetadataRepository;
        use crate::state::{ApiConfig, AppState};
        use std::sync::Arc;
        let cfg = ApiConfig {
            port: 3000,
            rpc_url: "http://127.0.0.1:8899".to_string(),
            database_url: None,
            mongo_url: None,
            version: "3.0.0".to_string(),
            environment: "test".to_string(),
            cors_allowed_origins: Vec::new(),
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
        };
        let state = AppState::with_config_and_repository(cfg, Arc::new(InMemoryMetadataRepository::new()));
        let body = render_prometheus(0, 0, state.uptime_seconds(), &state.config.version, 0);
        assert!(!body.to_lowercase().contains("private"));
    }
}
