//! HTTP routes for the Trust Work Escrow API.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};

use std::sync::atomic::Ordering;

use crate::error::ApiError;
use crate::metadata::{
    ApplicationMetadata, DisputeMetadata, EvidenceMetadata, JobMetadata, JobStatus,
    MilestoneMetadata, SupportTicketMetadata,
};
use crate::models::*;
use crate::state::{AppState, ArbiterPoolState};
use crate::validation;

fn job_pda(job_id: u64) -> String {
    format!("7a2YhCd7iivXfyySkp1pf5jjJob{:0>12}", job_id)
}
fn application_pda(job_id: u64, idx: u8, applicant_suffix: &str) -> String {
    let s = applicant_suffix.chars().take(8).collect::<String>();
    format!("7a2YhCd7iivXfyySkp1pf5jjApp{:0>8}{:02}{}", job_id, idx, s)
}
#[allow(dead_code)]
fn milestone_pda(job_id: u64, _idx: u8) -> String {
    job_pda(job_id)
}
fn dispute_pda(job_id: u64) -> String {
    format!("7a2YhCd7iivXfyySkp1pf5jjDispute{:0>10}", job_id)
}
fn ticket_pda(job_id: u64) -> String {
    format!("7a2YhCd7iivXfyySkp1pf5jjTicket{:0>10}", job_id)
}
fn placeholder_pubkey(label: &str) -> String {
    let base = format!("7a2YhCd7iivXfyySkp1pf5jj{}", label);
    if base.len() >= 44 {
        base[..44].to_string()
    } else {
        format!("{}{:0>width$}", base, 0, width = 44 - base.len())
    }
}
fn fee_amount(amount: u64) -> u64 {
    amount * 250 / 10_000
}

fn job_id_from_pda(pda: &str) -> Option<u64> {
    if pda.len() < 12 {
        return None;
    }
    pda[pda.len() - 12..].parse().ok()
}

fn job_status_to_dto(s: &JobStatus) -> JobStatusDto {
    match s {
        JobStatus::Created => JobStatusDto::Created,
        JobStatus::Applied => JobStatusDto::Applied,
        JobStatus::Assigned => JobStatusDto::Assigned,
        JobStatus::Submitted => JobStatusDto::Submitted,
        JobStatus::Approved => JobStatusDto::Approved,
        JobStatus::Cancelled => JobStatusDto::Cancelled,
        JobStatus::Rejected => JobStatusDto::Rejected,
    }
}

fn can_transition(from: &JobStatus, to: &JobStatus) -> bool {
    use JobStatus::*;
    match (from, to) {
        (Created, Applied) => true,
        (Applied, Assigned) => true,
        (Assigned, Submitted) => true,
        (Submitted, Approved) => true,
        (Submitted, Rejected) => true,
        (_, Cancelled) => !matches!(from, Approved | Cancelled | Rejected),
        _ => false,
    }
}

#[utoipa::path(post, path = "/auth/verify", tag = "Auth", responses((status = 200, description = "Signature verified", body = ApiStatus), (status = 401, description = "Invalid signature", body = crate::error::ErrorResponse)))]
async fn verify_auth(auth: crate::auth::AuthenticatedUser) -> Result<impl IntoResponse, ApiError> {
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("authenticated as {}", auth.pubkey),
        }),
    ))
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_config))
        .route("/auth/verify", post(verify_auth))
        .route("/jobs", get(list_jobs).post(create_job))
        .route("/jobs/:job_id", get(get_job))
        .route("/jobs/:job_id/deposit", post(deposit_funds))
        .route("/jobs/:job_id/apply", post(apply_to_job))
        .route(
            "/jobs/:job_id/applications/:application_index/accept",
            post(accept_application),
        )
        .route("/jobs/:job_id/submit-work", post(submit_work))
        .route("/jobs/:job_id/approve-work", post(approve_work))
        .route("/jobs/:job_id/reject-work", post(reject_work))
        .route("/jobs/:job_id/cancel", post(cancel_job))
        .route("/jobs/:job_id/pause", post(pause_job))
        .route("/jobs/:job_id/unpause", post(unpause_job))
        .route("/jobs/:job_id/milestones", post(create_milestone))
        .route(
            "/jobs/:job_id/milestones/:milestone_index/submit",
            post(submit_milestone),
        )
        .route(
            "/jobs/:job_id/milestones/:milestone_index/approve",
            post(approve_milestone),
        )
        .route(
            "/jobs/:job_id/milestones/:milestone_index/reject",
            post(reject_milestone),
        )
        .route("/jobs/:job_id/disputes", post(raise_dispute))
        .route("/jobs/:job_id/disputes/accept", post(accept_dispute))
        .route("/jobs/:job_id/disputes/evidence", post(submit_evidence))
        .route(
            "/jobs/:job_id/disputes/assign-arbiter",
            post(assign_arbiter),
        )
        .route("/jobs/:job_id/disputes/resolve", post(resolve_dispute))
        .route(
            "/jobs/:job_id/disputes/platform-resolve",
            post(resolve_platform_case),
        )
        .route(
            "/jobs/:job_id/disputes/request-intervention",
            post(request_platform_intervention),
        )
        .route(
            "/jobs/:job_id/disputes/finalize",
            post(finalize_dispute_payouts),
        )
        .route("/jobs/:job_id/support", post(open_support_ticket))
        .route(
            "/jobs/:job_id/support/resolve",
            post(resolve_support_ticket),
        )
        .route(
            "/arbiter-pool",
            get(get_arbiter_pool).post(create_arbiter_pool),
        )
        .route("/arbiter-pool/arbiters", post(add_arbiter))
        .route("/arbiter-pool/arbiters/:arbiter", delete(remove_arbiter))
}

#[utoipa::path(get, path = "/config", tag = "Config", responses((status = 200, description = "Protocol config", body = ConfigResponse)))]
async fn get_config(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let resp = ConfigResponse {
        authority: placeholder_pubkey("Authority"),
        advisor: placeholder_pubkey("Advisor"),
        treasury: placeholder_pubkey("Treasury"),
        arbitration_treasury: placeholder_pubkey("ArbTreasury"),
        fee_bps: 250,
        paused: false,
    };
    let _ = &state.config.rpc_url;
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(get, path = "/jobs", tag = "Jobs", responses((status = 200, description = "List of jobs", body = [JobResponse])))]
async fn list_jobs(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let mut jobs = state.repo.list_jobs().await?;
    jobs.sort_by(|a, b| a.pda_address.cmp(&b.pda_address));
    let resp: Vec<JobResponse> = jobs
        .into_iter()
        .enumerate()
        .map(|(idx, j)| {
            let job_id = job_id_from_pda(&j.pda_address).unwrap_or(idx as u64);
            let client = if j.client.is_empty() {
                placeholder_pubkey("Client")
            } else {
                j.client.clone()
            };
            JobResponse {
                job_id,
                client,
                freelancer: j.freelancer.clone(),
                title: j.title,
                description: j.description,
                amount: j.amount,
                fee_amount: j.fee_amount,
                status: job_status_to_dto(&j.status),
                deadline: j.deadline,
                applicants_count: 0,
            }
        })
        .collect();
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs", tag = "Jobs", request_body = CreateJobRequest, responses((status = 201, description = "Job created", body = JobResponse)))]
async fn create_job(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateJobRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validation::validate_create_job(&req)?;
    if state.next_job_id.load(Ordering::SeqCst) == 0 {
        let count = state.repo.list_jobs().await?.len() as u64;
        if count > 0 {
            let _ = state
                .next_job_id
                .compare_exchange(0, count, Ordering::SeqCst, Ordering::SeqCst);
        }
    }
    let job_id = state.next_job_id.fetch_add(1, Ordering::SeqCst);
    let pda = job_pda(job_id);
    let fee = fee_amount(req.amount);
    let meta = JobMetadata::new(
        pda.clone(),
        req.title.clone(),
        req.description.clone(),
        req.amount,
        fee,
        req.deadline,
        auth.pubkey.clone(),
    )?;
    state.repo.create_job(meta).await.map_err(ApiError::from)?;
    let resp = JobResponse {
        job_id,
        client: auth.pubkey,
        freelancer: None,
        title: req.title,
        description: req.description,
        amount: req.amount,
        fee_amount: fee,
        status: JobStatusDto::Created,
        deadline: req.deadline,
        applicants_count: 0,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(get, path = "/jobs/:job_id", tag = "Jobs", responses((status = 200, description = "Job details", body = JobResponse)))]
async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let job = state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let app_count = state
        .repo
        .list_applications_by_job(&pda)
        .await
        .map(|v| v.len() as u32)
        .unwrap_or(0);
    let client = if job.client.is_empty() {
        placeholder_pubkey("Client")
    } else {
        job.client.clone()
    };
    let resp = JobResponse {
        job_id,
        client,
        freelancer: job.freelancer.clone(),
        title: job.title,
        description: job.description,
        amount: job.amount,
        fee_amount: job.fee_amount,
        status: job_status_to_dto(&job.status),
        deadline: job.deadline,
        applicants_count: app_count,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/deposit", tag = "Jobs", responses((status = 200, description = "Funds deposited", body = ApiStatus)))]
async fn deposit_funds(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("funds deposited for job {}", job_id),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/apply", tag = "Applications", request_body = ApplyRequest, responses((status = 201, description = "Application submitted", body = ApplicationResponse)))]
async fn apply_to_job(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
    Json(req): Json<ApplyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let mut job = state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    validation::validate_apply(&req)?;
    let existing = state.repo.list_applications_by_job(&pda).await?;
    if existing.len() >= 50 {
        return Err(ApiError::BadRequest(
            "maximum 50 applications reached".into(),
        ));
    }
    let applicant = auth.pubkey.clone();
    if existing.iter().any(|a| a.applicant == applicant) {
        return Err(ApiError::Conflict("already applied".into()));
    }
    // State machine: Created -> Applied, or stay Applied for additional applicants
    if job.status == JobStatus::Created {
        if !can_transition(&job.status, &JobStatus::Applied) {
            return Err(ApiError::Conflict(format!(
                "invalid status transition from {:?} to Applied",
                job.status
            )));
        }
        job.status = JobStatus::Applied;
        job.updated_at = chrono::Utc::now().timestamp();
        state.repo.update_job(job.clone()).await?;
    } else if job.status != JobStatus::Applied {
        return Err(ApiError::Conflict(format!(
            "job not in a state to accept applications: {:?}",
            job.status
        )));
    }
    let index = existing.len() as u8;
    let app_pda = application_pda(job_id, index, &applicant);
    let meta = ApplicationMetadata::new(
        app_pda.clone(),
        pda.clone(),
        applicant.clone(),
        req.proposal,
    )?;
    state.repo.create_application(meta.clone()).await?;
    let resp = ApplicationResponse {
        index,
        applicant,
        proposal_hash: meta.proposal_hash,
        status: ApplicationStatusDto::Pending,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/applications/:application_index/accept", tag = "Applications", responses((status = 200, description = "Application accepted", body = ApiStatus)))]
async fn accept_application(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Path((job_id, application_index)): Path<(u64, u8)>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let mut job = state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    if job.client != auth.pubkey {
        return Err(ApiError::Forbidden);
    }
    let apps = state.repo.list_applications_by_job(&pda).await?;
    if (application_index as usize) >= apps.len() {
        return Err(ApiError::NotFound(format!(
            "application {} for job {} not found",
            application_index, job_id
        )));
    }
    if !can_transition(&job.status, &JobStatus::Assigned) {
        return Err(ApiError::Conflict(format!(
            "invalid status transition from {:?} to Assigned",
            job.status
        )));
    }
    let chosen = &apps[application_index as usize];
    job.freelancer = Some(chosen.applicant.clone());
    job.status = JobStatus::Assigned;
    job.updated_at = chrono::Utc::now().timestamp();
    state.repo.update_job(job).await?;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!(
                "application {} accepted for job {}",
                application_index, job_id
            ),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/submit-work", tag = "Work", responses((status = 200, description = "Work submitted", body = ApiStatus)))]
async fn submit_work(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let mut job = state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let _ = &auth.pubkey;
    if !can_transition(&job.status, &JobStatus::Submitted) {
        return Err(ApiError::Conflict(format!(
            "invalid status transition from {:?} to Submitted",
            job.status
        )));
    }
    job.status = JobStatus::Submitted;
    job.updated_at = chrono::Utc::now().timestamp();
    state.repo.update_job(job).await?;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("work submitted for job {}", job_id),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/approve-work", tag = "Work", responses((status = 200, description = "Work approved", body = ApiStatus)))]
async fn approve_work(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let mut job = state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let _ = &auth.pubkey;
    if !can_transition(&job.status, &JobStatus::Approved) {
        return Err(ApiError::Conflict(format!(
            "invalid status transition from {:?} to Approved",
            job.status
        )));
    }
    job.status = JobStatus::Approved;
    job.updated_at = chrono::Utc::now().timestamp();
    state.repo.update_job(job).await?;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("work approved for job {}", job_id),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/reject-work", tag = "Work", responses((status = 200, description = "Work rejected", body = ApiStatus)))]
async fn reject_work(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let mut job = state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let _ = &auth.pubkey;
    if !can_transition(&job.status, &JobStatus::Rejected) {
        return Err(ApiError::Conflict(format!(
            "invalid status transition from {:?} to Rejected",
            job.status
        )));
    }
    job.status = JobStatus::Rejected;
    job.updated_at = chrono::Utc::now().timestamp();
    state.repo.update_job(job).await?;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("work rejected for job {}", job_id),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/cancel", tag = "Jobs", responses((status = 200, description = "Job cancelled", body = ApiStatus)))]
async fn cancel_job(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let mut job = state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let _ = &auth.pubkey;
    if !can_transition(&job.status, &JobStatus::Cancelled) {
        return Err(ApiError::Conflict(format!(
            "invalid status transition from {:?} to Cancelled",
            job.status
        )));
    }
    job.status = JobStatus::Cancelled;
    job.updated_at = chrono::Utc::now().timestamp();
    state.repo.update_job(job).await?;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("job {} cancelled", job_id),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/pause", tag = "Jobs", responses((status = 200, description = "Job paused", body = ApiStatus)))]
async fn pause_job(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let _ = &auth.pubkey;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("job {} paused", job_id),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/unpause", tag = "Jobs", responses((status = 200, description = "Job unpaused", body = ApiStatus)))]
async fn unpause_job(
    auth: crate::auth::AuthenticatedUser,
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let _ = &auth.pubkey;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("job {} unpaused", job_id),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/milestones", tag = "Milestones", request_body = CreateMilestoneRequest, responses((status = 201, description = "Milestone created", body = MilestoneResponse)))]
async fn create_milestone(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
    Json(req): Json<CreateMilestoneRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validation::validate_create_milestone(&req)?;
    let pda = job_pda(job_id);
    state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let existing = state.repo.list_milestones_by_job(&pda).await?;
    let idx = existing.len() as u8;
    let meta =
        MilestoneMetadata::new(pda.clone(), idx, req.title.clone(), req.description.clone())?;
    state.repo.create_milestone(meta).await?;
    let resp = MilestoneResponse {
        index: idx,
        title: req.title,
        description: req.description,
        amount: req.amount,
        status: MilestoneStatusDto::Pending,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/milestones/:milestone_index/submit", tag = "Milestones", responses((status = 200, description = "Milestone submitted", body = MilestoneResponse)))]
async fn submit_milestone(
    State(state): State<AppState>,
    Path((job_id, milestone_index)): Path<(u64, u8)>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let ms = state
        .repo
        .get_milestone(&pda, milestone_index)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "milestone {} for job {} not found",
                milestone_index, job_id
            ))
        })?;
    let resp = MilestoneResponse {
        index: ms.index,
        title: ms.title,
        description: ms.description,
        amount: 0,
        status: MilestoneStatusDto::Submitted,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/milestones/:milestone_index/approve", tag = "Milestones", responses((status = 200, description = "Milestone approved", body = MilestoneResponse)))]
async fn approve_milestone(
    State(state): State<AppState>,
    Path((job_id, milestone_index)): Path<(u64, u8)>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let ms = state
        .repo
        .get_milestone(&pda, milestone_index)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "milestone {} for job {} not found",
                milestone_index, job_id
            ))
        })?;
    let resp = MilestoneResponse {
        index: ms.index,
        title: ms.title,
        description: ms.description,
        amount: 0,
        status: MilestoneStatusDto::Approved,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/milestones/:milestone_index/reject", tag = "Milestones", responses((status = 200, description = "Milestone rejected", body = MilestoneResponse)))]
async fn reject_milestone(
    State(state): State<AppState>,
    Path((job_id, milestone_index)): Path<(u64, u8)>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    let ms = state
        .repo
        .get_milestone(&pda, milestone_index)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "milestone {} for job {} not found",
                milestone_index, job_id
            ))
        })?;
    let resp = MilestoneResponse {
        index: ms.index,
        title: ms.title,
        description: ms.description,
        amount: 0,
        status: MilestoneStatusDto::Rejected,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/disputes", tag = "Disputes", responses((status = 201, description = "Dispute raised", body = DisputeResponse)))]
async fn raise_dispute(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let dpda = dispute_pda(job_id);
    let meta = DisputeMetadata::new(dpda.clone(), pda.clone(), "dispute raised via API".into())?;
    state
        .repo
        .create_dispute(meta)
        .await
        .map_err(ApiError::from)?;
    let resp = DisputeResponse {
        job_id,
        raised_by: placeholder_pubkey("RaisedBy"),
        arbiter: None,
        status: DisputeStatusDto::Open,
        evidence_count: 0,
        client_payout_percent: 0,
        freelancer_payout_percent: 0,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/disputes/accept", tag = "Disputes", responses((status = 200, description = "Dispute accepted", body = DisputeResponse)))]
async fn accept_dispute(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let dpda = dispute_pda(job_id);
    state
        .repo
        .get_dispute(&dpda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dispute for job {} not found", job_id)))?;
    let resp = DisputeResponse {
        job_id,
        raised_by: placeholder_pubkey("RaisedBy"),
        arbiter: None,
        status: DisputeStatusDto::Active,
        evidence_count: 0,
        client_payout_percent: 0,
        freelancer_payout_percent: 0,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/disputes/evidence", tag = "Disputes", request_body = EvidenceRequest, responses((status = 201, description = "Evidence submitted", body = EvidenceResponse)))]
async fn submit_evidence(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
    Json(req): Json<EvidenceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validation::validate_evidence(&req)?;
    let dpda = dispute_pda(job_id);
    state
        .repo
        .get_dispute(&dpda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dispute for job {} not found", job_id)))?;
    let existing = state.repo.list_evidence_by_dispute(&dpda).await?;
    let idx = existing.len() as u8;
    let author = placeholder_pubkey("EvidenceAuthor");
    let meta = EvidenceMetadata::new(dpda.clone(), idx, author.clone(), req.content.clone())?;
    state.repo.create_evidence(meta.clone()).await?;
    let resp = EvidenceResponse {
        index: idx,
        author,
        content_hash: meta.content_hash,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/disputes/assign-arbiter", tag = "Disputes", responses((status = 200, description = "Arbiter assigned", body = DisputeResponse)))]
async fn assign_arbiter(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let dpda = dispute_pda(job_id);
    state
        .repo
        .get_dispute(&dpda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dispute for job {} not found", job_id)))?;
    let pool = state.arbiter_pool.read().await;
    if pool.is_none() {
        return Err(ApiError::NotFound("arbiter pool not initialized".into()));
    }
    drop(pool);
    let resp = DisputeResponse {
        job_id,
        raised_by: placeholder_pubkey("RaisedBy"),
        arbiter: Some(placeholder_pubkey("Arbiter")),
        status: DisputeStatusDto::ArbiterAssigned,
        evidence_count: 0,
        client_payout_percent: 0,
        freelancer_payout_percent: 0,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/disputes/resolve", tag = "Disputes", request_body = ResolveDisputeRequest, responses((status = 200, description = "Dispute resolved", body = DisputeResponse)))]
async fn resolve_dispute(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
    Json(req): Json<ResolveDisputeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validation::validate_payout_percent(req.client_payout_percent)?;
    let dpda = dispute_pda(job_id);
    let mut d = state
        .repo
        .get_dispute(&dpda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dispute for job {} not found", job_id)))?;
    d.resolve(format!("resolved {}% to client", req.client_payout_percent))?;
    state.repo.update_dispute(d).await?;
    let resp = DisputeResponse {
        job_id,
        raised_by: placeholder_pubkey("RaisedBy"),
        arbiter: Some(placeholder_pubkey("Arbiter")),
        status: DisputeStatusDto::Resolved,
        evidence_count: 0,
        client_payout_percent: req.client_payout_percent,
        freelancer_payout_percent: 100 - req.client_payout_percent,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/disputes/platform-resolve", tag = "Disputes", request_body = ResolveDisputeRequest, responses((status = 200, description = "Platform resolved dispute", body = DisputeResponse)))]
async fn resolve_platform_case(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
    Json(req): Json<ResolveDisputeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validation::validate_payout_percent(req.client_payout_percent)?;
    let dpda = dispute_pda(job_id);
    let mut d = state
        .repo
        .get_dispute(&dpda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dispute for job {} not found", job_id)))?;
    d.resolve(format!(
        "platform resolved {}% to client",
        req.client_payout_percent
    ))?;
    state.repo.update_dispute(d).await?;
    let resp = DisputeResponse {
        job_id,
        raised_by: placeholder_pubkey("RaisedBy"),
        arbiter: None,
        status: DisputeStatusDto::Resolved,
        evidence_count: 0,
        client_payout_percent: req.client_payout_percent,
        freelancer_payout_percent: 100 - req.client_payout_percent,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/disputes/request-intervention", tag = "Disputes", responses((status = 200, description = "Platform intervention requested", body = DisputeResponse)))]
async fn request_platform_intervention(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let dpda = dispute_pda(job_id);
    state
        .repo
        .get_dispute(&dpda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dispute for job {} not found", job_id)))?;
    let resp = DisputeResponse {
        job_id,
        raised_by: placeholder_pubkey("RaisedBy"),
        arbiter: None,
        status: DisputeStatusDto::Open,
        evidence_count: 0,
        client_payout_percent: 0,
        freelancer_payout_percent: 0,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/disputes/finalize", tag = "Disputes", responses((status = 200, description = "Dispute payouts finalized", body = ApiStatus)))]
async fn finalize_dispute_payouts(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let dpda = dispute_pda(job_id);
    state
        .repo
        .get_dispute(&dpda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dispute for job {} not found", job_id)))?;
    Ok((
        StatusCode::OK,
        Json(ApiStatus {
            status: "ok".into(),
            message: format!("dispute payouts finalized for job {}", job_id),
        }),
    ))
}

#[utoipa::path(post, path = "/jobs/:job_id/support", tag = "Support", responses((status = 201, description = "Support ticket opened", body = SupportTicketResponse)))]
async fn open_support_ticket(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let pda = job_pda(job_id);
    state
        .repo
        .get_job(&pda)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("job {} not found", job_id)))?;
    let tpda = ticket_pda(job_id);
    let meta = SupportTicketMetadata::new(
        tpda.clone(),
        pda.clone(),
        "support requested via API".into(),
    )?;
    state
        .repo
        .create_support_ticket(meta)
        .await
        .map_err(ApiError::from)?;
    let resp = SupportTicketResponse {
        job_id,
        opened_by: placeholder_pubkey("Opener"),
        status: SupportTicketStatusDto::Open,
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(post, path = "/jobs/:job_id/support/resolve", tag = "Support", responses((status = 200, description = "Support ticket resolved", body = SupportTicketResponse)))]
async fn resolve_support_ticket(
    State(state): State<AppState>,
    Path(job_id): Path<u64>,
) -> Result<impl IntoResponse, ApiError> {
    let tpda = ticket_pda(job_id);
    let mut t = state.repo.get_support_ticket(&tpda).await?.ok_or_else(|| {
        ApiError::NotFound(format!("support ticket for job {} not found", job_id))
    })?;
    t.resolve("resolved via API".into())?;
    state.repo.update_support_ticket(t).await?;
    let resp = SupportTicketResponse {
        job_id,
        opened_by: placeholder_pubkey("Opener"),
        status: SupportTicketStatusDto::Resolved,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(get, path = "/arbiter-pool", tag = "Arbiter Pool", responses((status = 200, description = "Arbiter pool", body = ArbiterPoolResponse)))]
async fn get_arbiter_pool(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let pool = state.arbiter_pool.read().await;
    let p = pool
        .clone()
        .ok_or_else(|| ApiError::NotFound("arbiter pool not initialized".into()))?;
    let resp = ArbiterPoolResponse {
        authority: p.authority,
        arbiters: p.arbiters,
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(post, path = "/arbiter-pool", tag = "Arbiter Pool", responses((status = 201, description = "Arbiter pool created", body = ArbiterPoolResponse)))]
async fn create_arbiter_pool(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let mut pool = state.arbiter_pool.write().await;
    if pool.is_some() {
        return Err(ApiError::Conflict("arbiter pool already exists".into()));
    }
    let authority = placeholder_pubkey("Authority");
    *pool = Some(ArbiterPoolState {
        authority: authority.clone(),
        arbiters: Vec::new(),
    });
    let resp = ArbiterPoolResponse {
        authority,
        arbiters: Vec::new(),
    };
    Ok((StatusCode::CREATED, Json(resp)))
}

#[utoipa::path(post, path = "/arbiter-pool/arbiters", tag = "Arbiter Pool", request_body = AddArbiterRequest, responses((status = 200, description = "Arbiter added", body = ArbiterPoolResponse)))]
async fn add_arbiter(
    State(state): State<AppState>,
    Json(req): Json<AddArbiterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    validation::validate_pubkey(&req.arbiter)?;
    let mut pool = state.arbiter_pool.write().await;
    let p = pool
        .as_mut()
        .ok_or_else(|| ApiError::NotFound("arbiter pool not initialized".into()))?;
    if p.arbiters.contains(&req.arbiter) {
        return Err(ApiError::Conflict("arbiter already in pool".into()));
    }
    p.arbiters.push(req.arbiter.clone());
    let resp = ArbiterPoolResponse {
        authority: p.authority.clone(),
        arbiters: p.arbiters.clone(),
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[utoipa::path(delete, path = "/arbiter-pool/arbiters/:arbiter", tag = "Arbiter Pool", responses((status = 200, description = "Arbiter removed", body = ArbiterPoolResponse)))]
async fn remove_arbiter(
    State(state): State<AppState>,
    Path(arbiter): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let mut pool = state.arbiter_pool.write().await;
    let p = pool
        .as_mut()
        .ok_or_else(|| ApiError::NotFound("arbiter pool not initialized".into()))?;
    let pos = p
        .arbiters
        .iter()
        .position(|a| a == &arbiter)
        .ok_or_else(|| ApiError::NotFound(format!("arbiter {} not in pool", arbiter)))?;
    p.arbiters.remove(pos);
    let resp = ArbiterPoolResponse {
        authority: p.authority.clone(),
        arbiters: p.arbiters.clone(),
    };
    Ok((StatusCode::OK, Json(resp)))
}

#[cfg(test)]
mod routes_tests {
    use crate::state::AppState;
    use axum::body::{to_bytes, Body};
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    fn test_app() -> axum::Router {
        crate::app_with_state(AppState::default())
    }
    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let b = to_bytes(resp.into_body(), 16384).await.unwrap();
        serde_json::from_slice(&b).unwrap()
    }

    fn auth_headers(seed: u8, msg: &str) -> (String, String, String) {
        use base64::Engine as _;
        use ed25519_dalek::{Signer, SigningKey};
        let sk = SigningKey::from_bytes(&[seed; 32]);
        let pk = bs58::encode(sk.verifying_key().to_bytes()).into_string();
        let sig = base64::engine::general_purpose::STANDARD.encode(sk.sign(msg.as_bytes()).to_bytes());
        (pk, sig, msg.to_string())
    }
    fn add_auth(req: Request<Body>, seed: u8) -> Request<Body> {
        // helper not used directly; we construct headers inline
        req
    }

    #[tokio::test]
    async fn config_ok() {
        let r = test_app()
            .oneshot(
                Request::builder()
                    .uri("/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        assert!(body_json(r).await.get("fee_bps").is_some());
    }
    #[tokio::test]
    async fn jobs_crud_and_validation() {
        let app = test_app();
        let (pk, sig, msg) = auth_headers(7, "create-job");
        let payload = serde_json::json!({"title":"Build landing","description":"desc","amount":1000000,"deadline": 9999999999i64});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", pk.clone())
                    .header("x-signature", sig.clone())
                    .header("x-message", msg.clone())
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let v = body_json(r).await;
        assert_eq!(v["title"], "Build landing");
        assert_eq!(v["client"], pk);
        // without auth -> 401
        let payload_no_auth = serde_json::json!({"title":"No auth","description":"desc","amount":1000,"deadline": 9999999999i64});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(payload_no_auth.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
        let (pk2, sig2, msg2) = auth_headers(7, "bad-title");
        let bad = serde_json::json!({"title":"","description":"desc","amount":1000,"deadline": 9999999999i64});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", pk2)
                    .header("x-signature", sig2)
                    .header("x-message", msg2)
                    .body(Body::from(bad.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        let (pk3, sig3, msg3) = auth_headers(7, "bad-amount");
        let bad2 = serde_json::json!({"title":"ok","description":"desc","amount":0,"deadline": 9999999999i64});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", pk3)
                    .header("x-signature", sig3)
                    .header("x-message", msg3)
                    .body(Body::from(bad2.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        let r = app
            .clone()
            .oneshot(Request::builder().uri("/jobs").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let b = to_bytes(r.into_body(), 16384).await.unwrap();
        let list: Vec<serde_json::Value> = serde_json::from_slice(&b).unwrap();
        assert_eq!(list.len(), 1);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let v = body_json(r).await;
        assert_eq!(v["client"], pk);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }
    #[tokio::test]
    async fn deposit_and_work_lifecycle() {
        let app = test_app();
        let (client_pk, client_sig, client_msg) = auth_headers(7, "lifecycle");
        let payload = serde_json::json!({"title":"T","description":"D","amount":5000,"deadline": 9999999999i64});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", client_pk.clone())
                    .header("x-signature", client_sig.clone())
                    .header("x-message", client_msg.clone())
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        // deposit (no auth required) -> OK
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/deposit")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        // submit before accept -> should be 409 (needs Assigned)
        let (any_pk, any_sig, any_msg) = auth_headers(7, "submit-illegal");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/submit-work")
                    .method(Method::POST)
                    .header("x-pubkey", any_pk)
                    .header("x-signature", any_sig)
                    .header("x-message", any_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CONFLICT);
        // drive valid lifecycle: apply as freelancer (seed 9), accept as client, submit, approve
        let (freelancer_pk, freelancer_sig, freelancer_msg) = auth_headers(9, "apply-lifecycle");
        let proposal = "lifecycle proposal";
        let hash = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(proposal.as_bytes());
            hex::encode(h.finalize())
        };
        let apply = serde_json::json!({"proposal": proposal, "proposal_hash": hash});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/apply")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", freelancer_pk.clone())
                    .header("x-signature", freelancer_sig)
                    .header("x-message", freelancer_msg)
                    .body(Body::from(apply.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let (c2_pk, c2_sig, c2_msg) = auth_headers(7, "accept-lifecycle");
        // need to use same client pubkey as job creator (seed 7)
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/applications/0/accept")
                    .method(Method::POST)
                    .header("x-pubkey", c2_pk)
                    .header("x-signature", c2_sig)
                    .header("x-message", c2_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let (s_pk, s_sig, s_msg) = auth_headers(7, "submit-ok");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/submit-work")
                    .method(Method::POST)
                    .header("x-pubkey", s_pk)
                    .header("x-signature", s_sig)
                    .header("x-message", s_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let (a_pk, a_sig, a_msg) = auth_headers(7, "approve-ok");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/approve-work")
                    .method(Method::POST)
                    .header("x-pubkey", a_pk)
                    .header("x-signature", a_sig)
                    .header("x-message", a_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        // after Approved, reject should be 409
        let (rj_pk, rj_sig, rj_msg) = auth_headers(7, "reject-after-approve");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/reject-work")
                    .method(Method::POST)
                    .header("x-pubkey", rj_pk)
                    .header("x-signature", rj_sig)
                    .header("x-message", rj_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CONFLICT);
        // cancel after terminal should be 409
        let (can_pk, can_sig, can_msg) = auth_headers(7, "cancel-after-terminal");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/cancel")
                    .method(Method::POST)
                    .header("x-pubkey", can_pk)
                    .header("x-signature", can_sig)
                    .header("x-message", can_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CONFLICT);
        // pause/unpause still OK (any auth)
        let (p_pk, p_sig, p_msg) = auth_headers(7, "pause");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/pause")
                    .method(Method::POST)
                    .header("x-pubkey", p_pk)
                    .header("x-signature", p_sig)
                    .header("x-message", p_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let (up_pk, up_sig, up_msg) = auth_headers(7, "unpause");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/unpause")
                    .method(Method::POST)
                    .header("x-pubkey", up_pk)
                    .header("x-signature", up_sig)
                    .header("x-message", up_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/99/deposit")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }
    #[tokio::test]
    async fn applications_flow() {
        let app = test_app();
        let (client_pk, client_sig, client_msg) = auth_headers(7, "app-create");
        let payload = serde_json::json!({"title":"T","description":"D","amount":5000,"deadline": 9999999999i64});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", client_pk.clone())
                    .header("x-signature", client_sig)
                    .header("x-message", client_msg)
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let proposal = "My proposal text";
        let hash = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(proposal.as_bytes());
            hex::encode(h.finalize())
        };
        let apply = serde_json::json!({"proposal": proposal, "proposal_hash": hash});
        let (app_pk, app_sig, app_msg) = auth_headers(9, "apply-1");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/apply")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", app_pk.clone())
                    .header("x-signature", app_sig)
                    .header("x-message", app_msg)
                    .body(Body::from(apply.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        // duplicate apply same applicant -> 409
        let (dup_pk, dup_sig, dup_msg) = auth_headers(9, "apply-dup");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/apply")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", dup_pk)
                    .header("x-signature", dup_sig)
                    .header("x-message", dup_msg)
                    .body(Body::from(apply.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CONFLICT);
        // accept by non-client should be 403
        let (other_pk, other_sig, other_msg) = auth_headers(11, "accept-wrong");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/applications/0/accept")
                    .method(Method::POST)
                    .header("x-pubkey", other_pk)
                    .header("x-signature", other_sig)
                    .header("x-message", other_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::FORBIDDEN);
        // accept by correct client -> OK
        let (c_pk, c_sig, c_msg) = auth_headers(7, "accept-ok");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/applications/0/accept")
                    .method(Method::POST)
                    .header("x-pubkey", c_pk)
                    .header("x-signature", c_sig)
                    .header("x-message", c_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let bad = serde_json::json!({"proposal":"hi","proposal_hash":"nothex"});
        let (bad_pk, bad_sig, bad_msg) = auth_headers(9, "bad-proposal");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/apply")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", bad_pk)
                    .header("x-signature", bad_sig)
                    .header("x-message", bad_msg)
                    .body(Body::from(bad.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    }
    #[tokio::test]
    async fn milestones_flow() {
        let app = test_app();
        let (pk, sig, msg) = auth_headers(7, "milestone-create");
        let payload = serde_json::json!({"title":"T","description":"D","amount":5000,"deadline": 9999999999i64});
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", pk)
                    .header("x-signature", sig)
                    .header("x-message", msg)
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let ms = serde_json::json!({"title":"Phase 1","description":"desc","amount":1000});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/milestones")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(ms.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        for uri in [
            "/jobs/0/milestones/0/submit",
            "/jobs/0/milestones/0/approve",
            "/jobs/0/milestones/0/reject",
        ] {
            let r = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(uri)
                        .method(Method::POST)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(r.status(), StatusCode::OK, "{}", uri);
        }
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/milestones/9/submit")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }
    #[tokio::test]
    async fn disputes_and_support_flow() {
        let app = test_app();
        let (pk, sig, msg) = auth_headers(7, "dispute-create");
        let payload = serde_json::json!({"title":"T","description":"D","amount":5000,"deadline": 9999999999i64});
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", pk)
                    .header("x-signature", sig)
                    .header("x-message", msg)
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/accept")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let content = "evidence content here";
        let hash = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(content.as_bytes());
            hex::encode(h.finalize())
        };
        let ev = serde_json::json!({"content": content, "content_hash": hash});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/evidence")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(ev.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let bad = serde_json::json!({"content":"hi","content_hash":"bad"});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/evidence")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(bad.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/assign-arbiter")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/arbiter-pool")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/assign-arbiter")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let resolve = serde_json::json!({"client_payout_percent": 60});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/resolve")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(resolve.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/platform-resolve")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(resolve.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let badp = serde_json::json!({"client_payout_percent": 200});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/resolve")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(badp.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/request-intervention")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/disputes/finalize")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/support")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/support/resolve")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }
    #[tokio::test]
    async fn arbiter_pool_crud() {
        let app = test_app();
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/arbiter-pool")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/arbiter-pool")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/arbiter-pool")
                    .method(Method::POST)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CONFLICT);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/arbiter-pool")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let arb = "11111111111111111111111111111111";
        let payload = serde_json::json!({"arbiter": arb});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/arbiter-pool/arbiters")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/arbiter-pool/arbiters")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CONFLICT);
        let bad = serde_json::json!({"arbiter": "short"});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/arbiter-pool/arbiters")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .body(Body::from(bad.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/arbiter-pool/arbiters/{}", arb))
                    .method(Method::DELETE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/arbiter-pool/arbiters/{}", arb))
                    .method(Method::DELETE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn cancel_transitions() {
        let app = test_app();
        let (pk, sig, msg) = auth_headers(7, "cancel-create");
        let payload = serde_json::json!({"title":"CancelMe","description":"desc","amount":1000,"deadline": 9999999999i64});
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs")
                    .method(Method::POST)
                    .header("content-type", "application/json")
                    .header("x-pubkey", pk.clone())
                    .header("x-signature", sig)
                    .header("x-message", msg)
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
        let (c_pk, c_sig, c_msg) = auth_headers(7, "cancel");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/cancel")
                    .method(Method::POST)
                    .header("x-pubkey", c_pk)
                    .header("x-signature", c_sig)
                    .header("x-message", c_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        // second cancel should be 409
        let (c2_pk, c2_sig, c2_msg) = auth_headers(7, "cancel2");
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/jobs/0/cancel")
                    .method(Method::POST)
                    .header("x-pubkey", c2_pk)
                    .header("x-signature", c2_sig)
                    .header("x-message", c2_msg)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::CONFLICT);
    }
}
