//! Trust Work Escrow v3 — REST API entrypoint.
//!
//! Exposes health/liveness/readiness, Prometheus metrics, full OpenAPI/Swagger
//! documentation and business endpoint skeletons. Descriptive metadata lives
//! off-chain (Postgres/Mongo via `MetadataRepository`) and is wired through
//! `AppState` so handlers already receive `State<AppState>`.
//!
//! Runtime: axum + tokio, tracing, CORS + Trace middleware, structured errors.

use axum::{
    http::StatusCode,
    middleware::{self as axum_middleware, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum::http::Request;
use axum::extract::State;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod error;
pub mod evidence;
pub mod health;
pub mod metadata;
pub mod metrics;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod sync;
pub mod validation;
mod routes;
mod state;

use crate::error::ErrorResponse;
use crate::health::{HealthResponse, LiveResponse, ReadyResponse};
use crate::metrics::MetricsResponse;
use crate::models::*;
use crate::state::{ApiConfig, AppState};

/// OpenAPI document for the Trust Work Escrow backend.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Trust Work Escrow API",
        version = "3.0.0",
        description = "REST API for Trust Work Escrow v3. Interactive docs at /swagger-ui."
    ),
    paths(
        health::health,
        health::live,
        health::ready,
        metrics::metrics,
        metrics::metrics_json,
        routes::verify_auth,
        routes::get_config,
        routes::list_jobs,
        routes::create_job,
        routes::get_job,
        routes::deposit_funds,
        routes::apply_to_job,
        routes::accept_application,
        routes::submit_work,
        routes::approve_work,
        routes::reject_work,
        routes::cancel_job,
        routes::pause_job,
        routes::unpause_job,
        routes::create_milestone,
        routes::submit_milestone,
        routes::approve_milestone,
        routes::reject_milestone,
        routes::raise_dispute,
        routes::accept_dispute,
        routes::submit_evidence,
        routes::assign_arbiter,
        routes::resolve_dispute,
        routes::resolve_platform_case,
        routes::request_platform_intervention,
        routes::finalize_dispute_payouts,
        routes::open_support_ticket,
        routes::resolve_support_ticket,
        routes::get_arbiter_pool,
        routes::create_arbiter_pool,
        routes::add_arbiter,
        routes::remove_arbiter,
    ),
    components(schemas(
        HealthResponse,
        LiveResponse,
        ReadyResponse,
        MetricsResponse,
        ErrorResponse,
        crate::health::HealthChecks,
        ApiStatus,
        JobStatusDto,
        ApplicationStatusDto,
        MilestoneStatusDto,
        DisputeStatusDto,
        SupportTicketStatusDto,
        CreateJobRequest,
        JobResponse,
        ApplyRequest,
        ApplicationResponse,
        CreateMilestoneRequest,
        MilestoneResponse,
        EvidenceRequest,
        EvidenceResponse,
        ResolveDisputeRequest,
        DisputeResponse,
        SupportTicketResponse,
        ConfigResponse,
        ArbiterPoolResponse,
        AddArbiterRequest,
    ))
)]
pub struct ApiDoc;

/// Fallback handler for unmatched routes.
async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        axum::Json(ErrorResponse {
            error: "route not found".to_string(),
            code: "not_found".to_string(),
        }),
    )
}

/// Middleware: count requests/errors for `/metrics`.
async fn track_metrics(State(state): State<AppState>, req: Request<axum::body::Body>, next: Next) -> Response {
    state.inc_requests();
    let res = next.run(req).await;
    if res.status().is_client_error() || res.status().is_server_error() {
        state.inc_errors();
    }
    res
}

/// Build the axum router with default state (convenient for tests).
pub fn app() -> Router {
    app_with_state(AppState::default())
}

/// Build the axum router with explicit state.
pub fn app_with_state(state: AppState) -> Router {
    let cors = middleware::cors_layer(&state);
    Router::new()
        .route("/health", get(health::health))
        .route("/live", get(health::live))
        .route("/ready", get(health::ready))
        .route("/metrics", get(metrics::metrics))
        .route("/metrics/json", get(metrics::metrics_json))
        .merge(routes::api_router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback(not_found)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum_middleware::from_fn(middleware::security_headers_middleware))
        .layer(axum_middleware::from_fn(middleware::request_size_guard))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::https_enforcement_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(state.clone(), track_metrics))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trust_escrow_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = ApiConfig::from_env();
    let port = config.port;

    // Currently the repository is in-memory; wiring Postgres/Mongo is
    // deferred until Docker services are healthy. The state already carries
    // `Arc<dyn MetadataRepository>` so no handler signature changes later.
    let state = AppState::with_config(config.clone());

    let app = app_with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("failed to bind port");

    tracing::info!(
        version = %config.version,
        rpc_url = %config.rpc_url,
        environment = %config.environment,
        port = port,
        "Trust Escrow API listening"
    );
    tracing::info!("Swagger UI at http://0.0.0.0:{}/swagger-ui", port);

    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_ok() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // degraded is still 200 (repo ok, rpc may be unavailable in CI)
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn live_ok() {
        let resp = app()
            .oneshot(Request::builder().uri("/live").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ready_ok_or_unavailable() {
        let resp = app()
            .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(
            resp.status() == StatusCode::OK
                || resp.status() == StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn metrics_ok() {
        let resp = app()
            .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(ct.contains("text/plain"));
    }

    #[tokio::test]
    async fn metrics_json_ok() {
        let resp = app()
            .oneshot(Request::builder().uri("/metrics/json").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn not_found_returns_404() {
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/does-not-exist-xyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn health_response_has_version() {
        use axum::body::to_bytes;
        let resp = app()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = to_bytes(resp.into_body(), 8192).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(v.get("version").is_some());
        assert!(v.get("status").is_some());
        assert!(v.get("checks").is_some());
    }

    #[tokio::test]
    async fn metrics_prometheus_has_expected_lines() {
        use axum::body::to_bytes;
        let resp = app()
            .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = to_bytes(resp.into_body(), 8192).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("trust_escrow_requests_total"));
        assert!(text.contains("trust_escrow_uptime_seconds"));
        // Must not leak secrets
        assert!(!text.to_lowercase().contains("private"));
    }

    #[tokio::test]
    async fn security_headers_present_on_health() {
        let resp = app()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let h = resp.headers();
        assert_eq!(h.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(h.get("x-frame-options").unwrap(), "DENY");
        assert!(h.contains_key("strict-transport-security"));
        assert!(h.contains_key("content-security-policy"));
    }

    #[tokio::test]
    async fn auth_verify_requires_signature() {
        use axum::http::Method;
        // without headers -> 401
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/auth/verify")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // with valid signature -> 200
        use ed25519_dalek::{Signer, SigningKey};
        use base64::Engine as _;
        let seed = [7u8; 32];
        let sk = SigningKey::from_bytes(&seed);
        let pk = bs58::encode(sk.verifying_key().to_bytes()).into_string();
        let msg = "test-auth-message";
        let sig = base64::engine::general_purpose::STANDARD.encode(sk.sign(msg.as_bytes()).to_bytes());
        let resp = app()
            .oneshot(
                Request::builder()
                    .uri("/auth/verify")
                    .method(Method::POST)
                    .header("x-pubkey", pk)
                    .header("x-signature", sig)
                    .header("x-message", msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn validation_blocks_invalid_job_and_does_not_create() {
        use axum::http::Method;
        let state = AppState::default();
        let app = app_with_state(state);
        let payload = serde_json::json!({"title":"","description":"desc","amount":0,"deadline":0});
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        // Ensure no job was created (list should be empty)
        let resp = app
            .clone()
            .oneshot(Request::builder().uri("/jobs").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert!(list.is_empty());
    }
}
