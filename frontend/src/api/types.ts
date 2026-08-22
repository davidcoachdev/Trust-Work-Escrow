/**
 * api/types — tipos compartidos on-chain + off-chain.
 * Jobs on-chain: Vec<Job> (index = jobId) + off-chain metadata title/description.
 * Estos tipos espejan backend/api/src/models.rs y backend/sdk/src/types.rs.
 */

// ---------- Jobs ----------
export type JobStatus = "Open" | "InProgress" | "Completed" | "Cancelled" | "Disputed";

export const BACKEND_TO_FRONT_STATUS: Record<string, JobStatus> = {
  Created: "Open",
  Funded: "Open",
  InProgress: "InProgress",
  Submitted: "InProgress",
  Released: "Completed",
  Disputed: "Disputed",
  Resolved: "Completed",
  Cancelled: "Cancelled",
};

export interface Job {
  jobId: string;
  client: string;
  title: string;
  description: string;
  amount: string;
  deadline: number;
  status: JobStatus;
  freelancer?: string | null;
  createdAt: number;
  applicantsCount?: number;
}

export interface CreateJobParams {
  title: string;
  description: string;
  amount: number;
  deadline: number;
}

export interface PaginatedJobs {
  jobs: Job[];
  nextCursor: string | null;
  hasMore: boolean;
}

// Backend wire type (snake_case)
export interface JobResponse {
  job_id: number;
  client: string;
  freelancer: string | null;
  title: string;
  description: string;
  amount: number;
  fee_amount: number;
  status: string;
  deadline: number;
  applicants_count: number;
}

export function mapJobResponse(r: JobResponse): Job {
  return {
    jobId: String(r.job_id),
    client: r.client,
    title: r.title,
    description: r.description,
    amount: String(r.amount),
    deadline: r.deadline,
    status: (BACKEND_TO_FRONT_STATUS[r.status] as JobStatus) || "Open",
    freelancer: r.freelancer,
    createdAt: r.deadline - 86400,
    applicantsCount: r.applicants_count,
  };
}

// ---------- Applications ----------
export interface ApplicationResponse {
  index: number;
  applicant: string;
  proposal_hash: string;
  status: string;
}

export interface ApplyParams {
  jobId: number;
  proposal: string;
  proposalHash: string;
}

// ---------- Milestones ----------
export interface MilestoneResponse {
  index: number;
  title: string;
  description: string;
  amount: number;
  status: string;
}

export interface CreateMilestoneParams {
  title: string;
  description: string;
  amount: number;
}

// ---------- Disputes ----------
export interface DisputeResponse {
  job_id: number;
  raised_by: string;
  arbiter: string | null;
  status: string;
  evidence_count: number;
  client_payout_percent: number;
  freelancer_payout_percent: number;
}

export interface EvidenceResponse {
  index: number;
  author: string;
  content_hash: string;
}

export interface ResolveDisputeParams {
  clientPayoutPercent: number;
}

// ---------- Support ----------
export interface SupportTicketResponse {
  job_id: number;
  opened_by: string;
  status: string;
}

// ---------- Config / Protocol ----------
export interface ConfigResponse {
  authority: string;
  advisor: string;
  treasury: string;
  arbitration_treasury: string;
  fee_bps: number;
  paused: boolean;
}

// ---------- Arbiter Pool ----------
export interface ArbiterPoolResponse {
  authority: string;
  arbiters: string[];
}

// ---------- Generic ----------
export interface ApiStatus {
  status: string;
  message: string;
}

// Validation mirrors backend validation.rs
export const MAX_TITLE_LEN = 100;
export const MAX_DESC_LEN = 500;
export const MAX_PROPOSAL_LEN = 512;
export const MIN_AMOUNT = 1;
export const MAX_AMOUNT = 10_000 * 1_000_000_000;

export function validateCreateJobParams(p: Partial<CreateJobParams>): string | null {
  if (!p.title || !p.title.trim()) return "title requerido";
  if (p.title.trim().length > MAX_TITLE_LEN) return `title máximo ${MAX_TITLE_LEN} chars`;
  if (!p.description || !p.description.trim()) return "description requerida";
  if (p.description.trim().length > MAX_DESC_LEN) return `description máximo ${MAX_DESC_LEN} chars`;
  if (p.amount == null || !Number.isFinite(p.amount)) return "amount requerido";
  if (p.amount < MIN_AMOUNT) return `amount mínimo ${MIN_AMOUNT}`;
  if (p.amount > MAX_AMOUNT) return `amount máximo ${MAX_AMOUNT}`;
  if (p.deadline == null || !Number.isFinite(p.deadline)) return "deadline requerido";
  if (p.deadline <= Math.floor(Date.now() / 1000)) return "deadline debe ser futuro";
  return null;
}

export function validateProposalHash(hash: string): string | null {
  const t = hash.trim();
  if (!t) return "proposal_hash requerido (sha256)";
  if (!/^[0-9a-fA-F]{64}$/.test(t)) return "proposal_hash debe ser hex 64";
  if (t === "0".repeat(64)) return "proposal_hash vacío";
  return null;
}
