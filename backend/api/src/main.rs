//! Trust Work Escrow v3 — REST API entrypoint.
//!
//! Exposes a health check, full OpenAPI/Swagger documentation, and business
//! endpoint skeletons. Descriptive metadata lives off-chain (Postgres/Mongo) and
//! will be wired once the Docker-backed DB services are available.

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, Router},
    Json,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod evidence;
pub mod metadata;
pub mod models;
pub mod repository;
mod routes;
mod state;

use crate::models::*;
use crate::state::AppState;

/// API health status.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// API error body.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

/// OpenAPI document for the Trust Work Escrow backend.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Trust Work Escrow API",
        version = "3.0.0",
        description = "REST API for Trust Work Escrow v3. Interactive docs at /swagger-ui."
    ),
    paths(
        health,
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
        ErrorResponse,
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

/// Health check endpoint.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "API is up", body = HealthResponse)
    )
)]
async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "3.0.0".to_string(),
    })
}

/// Fallback handler for unmatched routes.
async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "route not found".to_string(),
        }),
    )
}

/// Build the axum router.
pub fn app() -> Router {
    let state = AppState::default();

    Router::new()
        .route("/health", get(health))
        .merge(routes::api_router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback(not_found)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
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

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("failed to bind port");

    tracing::info!("Trust Escrow API listening on http://0.0.0.0:{}", port);
    axum::serve(listener, app()).await.unwrap();
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
        assert_eq!(response.status(), StatusCode::OK);
    }
}
