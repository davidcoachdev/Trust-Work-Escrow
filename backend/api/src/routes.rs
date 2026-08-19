//! HTTP routes for the Trust Work Escrow API.
//!
//! All handlers are stubs: they validate the route shape and return
//! `501 Not Implemented` with the intended operation description. Once the
//! Postgres/Mongo repositories are wired, these handlers will delegate to the
//! service layer and build Solana transactions via `trust-escrow-sdk`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

use crate::models::*;
use crate::state::AppState;

/// Helper: consistent `501 Not Implemented` response.
fn not_impl(operation: &str) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ApiStatus {
            status: "not_implemented".to_string(),
            message: format!(
                "{} — pending DB integration (Docker services not running)",
                operation
            ),
        }),
    )
}

/// Assemble all API routers.
pub fn api_router() -> Router<AppState> {
    Router::new()
        // Health is kept in main.rs, but config is a resource like any other.
        .route("/config", get(get_config))
        // Jobs
        .route("/jobs", get(list_jobs).post(create_job))
        .route("/jobs/{job_id}", get(get_job))
        .route("/jobs/{job_id}/deposit", post(deposit_funds))
        // Applications
        .route("/jobs/{job_id}/apply", post(apply_to_job))
        .route(
            "/jobs/{job_id}/applications/{application_index}/accept",
            post(accept_application),
        )
        // Work lifecycle
        .route("/jobs/{job_id}/submit-work", post(submit_work))
        .route("/jobs/{job_id}/approve-work", post(approve_work))
        .route("/jobs/{job_id}/reject-work", post(reject_work))
        .route("/jobs/{job_id}/cancel", post(cancel_job))
        .route("/jobs/{job_id}/pause", post(pause_job))
        .route("/jobs/{job_id}/unpause", post(unpause_job))
        // Milestones
        .route("/jobs/{job_id}/milestones", post(create_milestone))
        .route(
            "/jobs/{job_id}/milestones/{milestone_index}/submit",
            post(submit_milestone),
        )
        .route(
            "/jobs/{job_id}/milestones/{milestone_index}/approve",
            post(approve_milestone),
        )
        .route(
            "/jobs/{job_id}/milestones/{milestone_index}/reject",
            post(reject_milestone),
        )
        // Disputes
        .route("/jobs/{job_id}/disputes", post(raise_dispute))
        .route("/jobs/{job_id}/disputes/accept", post(accept_dispute))
        .route("/jobs/{job_id}/disputes/evidence", post(submit_evidence))
        .route(
            "/jobs/{job_id}/disputes/assign-arbiter",
            post(assign_arbiter),
        )
        .route("/jobs/{job_id}/disputes/resolve", post(resolve_dispute))
        .route(
            "/jobs/{job_id}/disputes/platform-resolve",
            post(resolve_platform_case),
        )
        .route(
            "/jobs/{job_id}/disputes/request-intervention",
            post(request_platform_intervention),
        )
        .route(
            "/jobs/{job_id}/disputes/finalize",
            post(finalize_dispute_payouts),
        )
        // Support tickets
        .route("/jobs/{job_id}/support", post(open_support_ticket))
        .route(
            "/jobs/{job_id}/support/resolve",
            post(resolve_support_ticket),
        )
        // Arbiter pool
        .route(
            "/arbiter-pool",
            get(get_arbiter_pool).post(create_arbiter_pool),
        )
        .route("/arbiter-pool/arbiters", post(add_arbiter))
        .route("/arbiter-pool/arbiters/{arbiter}", delete(remove_arbiter))
}

// ---- Config ----

#[utoipa::path(
    get,
    path = "/config",
    tag = "Config",
    responses((status = 200, description = "Protocol config", body = ConfigResponse))
)]
async fn get_config(State(_state): State<AppState>) -> impl IntoResponse {
    not_impl("get protocol config")
}

// ---- Jobs ----

#[utoipa::path(
    get,
    path = "/jobs",
    tag = "Jobs",
    responses((status = 200, description = "List of jobs", body = [JobResponse]))
)]
async fn list_jobs(State(_state): State<AppState>) -> impl IntoResponse {
    not_impl("list jobs")
}

#[utoipa::path(
    post,
    path = "/jobs",
    tag = "Jobs",
    request_body = CreateJobRequest,
    responses((status = 201, description = "Job created", body = JobResponse))
)]
async fn create_job(
    State(_state): State<AppState>,
    Json(_req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    not_impl("create job")
}

#[utoipa::path(
    get,
    path = "/jobs/{job_id}",
    tag = "Jobs",
    responses((status = 200, description = "Job details", body = JobResponse))
)]
async fn get_job(State(_state): State<AppState>, Path(_job_id): Path<u64>) -> impl IntoResponse {
    not_impl("get job")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/deposit",
    tag = "Jobs",
    responses((status = 200, description = "Funds deposited", body = ApiStatus))
)]
async fn deposit_funds(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("deposit funds")
}

// ---- Applications ----

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/apply",
    tag = "Applications",
    request_body = ApplyRequest,
    responses((status = 201, description = "Application submitted", body = ApplicationResponse))
)]
async fn apply_to_job(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
    Json(_req): Json<ApplyRequest>,
) -> impl IntoResponse {
    not_impl("apply to job")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/applications/{application_index}/accept",
    tag = "Applications",
    responses((status = 200, description = "Application accepted", body = ApiStatus))
)]
async fn accept_application(
    State(_state): State<AppState>,
    Path((_job_id, _application_index)): Path<(u64, u8)>,
) -> impl IntoResponse {
    not_impl("accept application")
}

// ---- Work lifecycle ----

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/submit-work",
    tag = "Work",
    responses((status = 200, description = "Work submitted", body = ApiStatus))
)]
async fn submit_work(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("submit work")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/approve-work",
    tag = "Work",
    responses((status = 200, description = "Work approved", body = ApiStatus))
)]
async fn approve_work(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("approve work")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/reject-work",
    tag = "Work",
    responses((status = 200, description = "Work rejected", body = ApiStatus))
)]
async fn reject_work(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("reject work")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/cancel",
    tag = "Jobs",
    responses((status = 200, description = "Job cancelled", body = ApiStatus))
)]
async fn cancel_job(State(_state): State<AppState>, Path(_job_id): Path<u64>) -> impl IntoResponse {
    not_impl("cancel job")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/pause",
    tag = "Jobs",
    responses((status = 200, description = "Job paused", body = ApiStatus))
)]
async fn pause_job(State(_state): State<AppState>, Path(_job_id): Path<u64>) -> impl IntoResponse {
    not_impl("pause job")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/unpause",
    tag = "Jobs",
    responses((status = 200, description = "Job unpaused", body = ApiStatus))
)]
async fn unpause_job(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("unpause job")
}

// ---- Milestones ----

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/milestones",
    tag = "Milestones",
    request_body = CreateMilestoneRequest,
    responses((status = 201, description = "Milestone created", body = MilestoneResponse))
)]
async fn create_milestone(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
    Json(_req): Json<CreateMilestoneRequest>,
) -> impl IntoResponse {
    not_impl("create milestone")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/milestones/{milestone_index}/submit",
    tag = "Milestones",
    responses((status = 200, description = "Milestone submitted", body = MilestoneResponse))
)]
async fn submit_milestone(
    State(_state): State<AppState>,
    Path((_job_id, _milestone_index)): Path<(u64, u8)>,
) -> impl IntoResponse {
    not_impl("submit milestone")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/milestones/{milestone_index}/approve",
    tag = "Milestones",
    responses((status = 200, description = "Milestone approved", body = MilestoneResponse))
)]
async fn approve_milestone(
    State(_state): State<AppState>,
    Path((_job_id, _milestone_index)): Path<(u64, u8)>,
) -> impl IntoResponse {
    not_impl("approve milestone")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/milestones/{milestone_index}/reject",
    tag = "Milestones",
    responses((status = 200, description = "Milestone rejected", body = MilestoneResponse))
)]
async fn reject_milestone(
    State(_state): State<AppState>,
    Path((_job_id, _milestone_index)): Path<(u64, u8)>,
) -> impl IntoResponse {
    not_impl("reject milestone")
}

// ---- Disputes ----

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/disputes",
    tag = "Disputes",
    responses((status = 201, description = "Dispute raised", body = DisputeResponse))
)]
async fn raise_dispute(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("raise dispute")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/disputes/accept",
    tag = "Disputes",
    responses((status = 200, description = "Dispute accepted", body = DisputeResponse))
)]
async fn accept_dispute(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("accept dispute")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/disputes/evidence",
    tag = "Disputes",
    request_body = EvidenceRequest,
    responses((status = 201, description = "Evidence submitted", body = EvidenceResponse))
)]
async fn submit_evidence(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
    Json(_req): Json<EvidenceRequest>,
) -> impl IntoResponse {
    not_impl("submit evidence")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/disputes/assign-arbiter",
    tag = "Disputes",
    responses((status = 200, description = "Arbiter assigned", body = DisputeResponse))
)]
async fn assign_arbiter(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("assign arbiter")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/disputes/resolve",
    tag = "Disputes",
    request_body = ResolveDisputeRequest,
    responses((status = 200, description = "Dispute resolved", body = DisputeResponse))
)]
async fn resolve_dispute(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
    Json(_req): Json<ResolveDisputeRequest>,
) -> impl IntoResponse {
    not_impl("resolve dispute")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/disputes/platform-resolve",
    tag = "Disputes",
    request_body = ResolveDisputeRequest,
    responses((status = 200, description = "Platform resolved dispute", body = DisputeResponse))
)]
async fn resolve_platform_case(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
    Json(_req): Json<ResolveDisputeRequest>,
) -> impl IntoResponse {
    not_impl("resolve platform case")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/disputes/request-intervention",
    tag = "Disputes",
    responses((status = 200, description = "Platform intervention requested", body = DisputeResponse))
)]
async fn request_platform_intervention(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("request platform intervention")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/disputes/finalize",
    tag = "Disputes",
    responses((status = 200, description = "Dispute payouts finalized", body = ApiStatus))
)]
async fn finalize_dispute_payouts(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("finalize dispute payouts")
}

// ---- Support tickets ----

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/support",
    tag = "Support",
    responses((status = 201, description = "Support ticket opened", body = SupportTicketResponse))
)]
async fn open_support_ticket(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("open support ticket")
}

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/support/resolve",
    tag = "Support",
    responses((status = 200, description = "Support ticket resolved", body = SupportTicketResponse))
)]
async fn resolve_support_ticket(
    State(_state): State<AppState>,
    Path(_job_id): Path<u64>,
) -> impl IntoResponse {
    not_impl("resolve support ticket")
}

// ---- Arbiter pool ----

#[utoipa::path(
    get,
    path = "/arbiter-pool",
    tag = "Arbiter Pool",
    responses((status = 200, description = "Arbiter pool", body = ArbiterPoolResponse))
)]
async fn get_arbiter_pool(State(_state): State<AppState>) -> impl IntoResponse {
    not_impl("get arbiter pool")
}

#[utoipa::path(
    post,
    path = "/arbiter-pool",
    tag = "Arbiter Pool",
    responses((status = 201, description = "Arbiter pool created", body = ArbiterPoolResponse))
)]
async fn create_arbiter_pool(State(_state): State<AppState>) -> impl IntoResponse {
    not_impl("create arbiter pool")
}

#[utoipa::path(
    post,
    path = "/arbiter-pool/arbiters",
    tag = "Arbiter Pool",
    request_body = AddArbiterRequest,
    responses((status = 200, description = "Arbiter added", body = ArbiterPoolResponse))
)]
async fn add_arbiter(
    State(_state): State<AppState>,
    Json(_req): Json<AddArbiterRequest>,
) -> impl IntoResponse {
    not_impl("add arbiter")
}

#[utoipa::path(
    delete,
    path = "/arbiter-pool/arbiters/{arbiter}",
    tag = "Arbiter Pool",
    responses((status = 200, description = "Arbiter removed", body = ArbiterPoolResponse))
)]
async fn remove_arbiter(
    State(_state): State<AppState>,
    Path(_arbiter): Path<String>,
) -> impl IntoResponse {
    not_impl("remove arbiter")
}
