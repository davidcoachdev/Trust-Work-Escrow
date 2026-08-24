//! Shared request/response schemas for the Trust Work Escrow API.
//!
//! These types are intentionally decoupled from the on-chain SDK types: the
//! backend stores descriptive metadata (titles, descriptions, proposals,
//! evidence content) in Postgres/Mongo and only keeps functional data on-chain.
//! For now every endpoint returns a `501 Not Implemented` stub because the DB
//! layer is pending (blocked until Docker is available).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Generic API status wrapper.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ApiStatus {
    pub status: String,
    pub message: String,
}

/// Job status values that mirror the on-chain `JobStatus` enum.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum JobStatusDto {
    Created,
    Funded,
    InProgress,
    Submitted,
    Released,
    Disputed,
    Resolved,
    Cancelled,
    Applied,
    Assigned,
    Approved,
    Rejected,
}

/// Application status values.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum ApplicationStatusDto {
    Pending,
    Accepted,
    Rejected,
    Withdrawn,
}

/// Milestone status values.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum MilestoneStatusDto {
    Pending,
    Submitted,
    Approved,
    Rejected,
}

/// Dispute status values.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum DisputeStatusDto {
    Open,
    Active,
    EvidenceSubmitted,
    ArbiterAssigned,
    Resolved,
    Expired,
}

/// Support ticket status values.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum SupportTicketStatusDto {
    Open,
    Resolved,
}

/// Request: create a new job.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct CreateJobRequest {
    /// Off-chain title (stored in Postgres).
    pub title: String,
    /// Off-chain description (stored in Postgres).
    pub description: String,
    /// Job amount in lamports.
    pub amount: u64,
    /// Unix timestamp after which the job is considered overdue.
    pub deadline: i64,
}

/// Request for a wallet-signed transaction template.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct UnsignedTransactionRequest {
    pub signer: String,
    pub amount: u64,
    pub deadline: i64,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DepositTransactionRequest {
    pub signer: String,
}

/// Request containing bytes already signed by Phantom.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct SignedTransactionRequest {
    pub signer: String,
    /// Base64-encoded bincode `solana_sdk::transaction::Transaction`.
    pub transaction: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct UnsignedTransactionResponse {
    pub job_id: u64,
    pub signer: String,
    pub transaction: String,
    pub job_pda: String,
    pub cluster: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct RelayedTransactionResponse {
    pub signature: String,
    pub cluster: String,
}

/// Response: job summary.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct JobResponse {
    pub job_id: u64,
    pub client: String,
    pub freelancer: Option<String>,
    pub title: String,
    pub description: String,
    pub amount: u64,
    pub fee_amount: u64,
    pub status: JobStatusDto,
    pub deadline: i64,
    pub applicants_count: u32,
    /// Confirmed transaction returned by the relay after wallet signing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_signature: Option<String>,
    /// Real PDA derived from the SDK for this backend-owned client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_pda: Option<String>,
    /// On-chain confirmation state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_chain_status: Option<String>,
}

/// Request: apply to a funded job.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ApplyRequest {
    /// SHA-256 hash of the full proposal text stored off-chain.
    pub proposal_hash: String,
    /// Human-readable proposal stored off-chain (ignored on-chain).
    pub proposal: String,
}

/// Response: job application.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ApplicationResponse {
    pub index: u8,
    pub applicant: String,
    pub proposal_hash: String,
    pub status: ApplicationStatusDto,
}

/// Request: create a milestone.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct CreateMilestoneRequest {
    pub title: String,
    pub description: String,
    pub amount: u64,
}

/// Response: milestone.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct MilestoneResponse {
    pub index: u8,
    pub title: String,
    pub description: String,
    pub amount: u64,
    pub status: MilestoneStatusDto,
}

/// Request: submit evidence in a dispute.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct EvidenceRequest {
    pub content_hash: String,
    pub content: String,
}

/// Response: evidence entry.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct EvidenceResponse {
    pub index: u8,
    pub author: String,
    pub content_hash: String,
}

/// Request: resolve a dispute.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ResolveDisputeRequest {
    /// Percentage of the principal that goes back to the client (0-100).
    pub client_payout_percent: u8,
}

/// Response: dispute.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DisputeResponse {
    pub job_id: u64,
    pub raised_by: String,
    pub arbiter: Option<String>,
    pub status: DisputeStatusDto,
    pub evidence_count: u8,
    pub client_payout_percent: u8,
    pub freelancer_payout_percent: u8,
}

/// Response: support ticket.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct SupportTicketResponse {
    pub job_id: u64,
    pub opened_by: String,
    pub status: SupportTicketStatusDto,
}

/// Response: protocol config.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ConfigResponse {
    pub authority: String,
    pub advisor: String,
    pub treasury: String,
    pub arbitration_treasury: String,
    pub fee_bps: u16,
    pub paused: bool,
}

/// Response: arbiter pool.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ArbiterPoolResponse {
    pub authority: String,
    pub arbiters: Vec<String>,
}

/// Request: add an arbiter to the pool.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct AddArbiterRequest {
    pub arbiter: String,
}
