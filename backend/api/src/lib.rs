//! Library root for `trust-escrow-api` — re-exports the same modules as
//! `main.rs` so integration tests (`tests/*.rs`) can `use trust_escrow_api::*`.

pub mod auth;
pub mod config;
pub mod error;
pub mod evidence;
pub mod health;
pub mod integration;
pub mod logging;
pub mod metadata;
pub mod metrics;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod sync;
pub mod validation;
pub mod routes;
pub mod state;

use axum::{
    http::{Request, StatusCode},
    middleware::{self as axum_middleware, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum::extract::State;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::error::ErrorResponse;
use crate::health::{HealthResponse, LiveResponse, ReadyResponse};
use crate::metrics::MetricsResponse;
use crate::models::*;
use crate::state::AppState;

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

pub async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        axum::Json(ErrorResponse {
            error: "route not found".to_string(),
            code: "not_found".to_string(),
        }),
    )
}

pub async fn track_metrics(State(state): State<AppState>, req: Request<axum::body::Body>, next: Next) -> Response {
    state.inc_requests();
    let res = next.run(req).await;
    if res.status().is_client_error() || res.status().is_server_error() {
        state.inc_errors();
    }
    res
}

pub fn app() -> Router {
    app_with_state(AppState::default())
}

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
        .layer(axum_middleware::from_fn_with_state(state.clone(), middleware::rate_limit_middleware))
        .layer(axum_middleware::from_fn_with_state(state.clone(), middleware::https_enforcement_middleware))
        .layer(axum_middleware::from_fn_with_state(state.clone(), track_metrics))
        .with_state(state)
}
