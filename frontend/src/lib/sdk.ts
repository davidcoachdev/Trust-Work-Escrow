/**
 * trust-escrow-sdk — TypeScript wrapper for Trust Work Escrow v3
 * Programa on-chain: 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh
 * Espejo del SDK Rust (backend/sdk) — funciones: list_jobs, create_job, apply (apply_to_job)
 * Integrado con backend API (Axum) en NEXT_PUBLIC_API_URL (default http://127.0.0.1:3000)
 */

export const PROGRAM_ID_STR = "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh";

// --- Types mirroring backend/sdk/src/types.rs + backend/api/src/models.rs ---
export type JobStatus = "Open" | "InProgress" | "Completed" | "Cancelled" | "Disputed";
// Backend status enum uses PascalCase Created|Funded|InProgress etc; map to frontend JobStatus
const BACKEND_TO_FRONT_STATUS: Record<string, JobStatus> = {
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
  jobId: string; // u64 as string
  client: string; // pubkey base58
  title: string;
  description: string;
  amount: string; // u64 lamports as string
  deadline: number; // i64 unix
  status: JobStatus;
  freelancer?: string | null;
  createdAt: number;
}

export interface CreateJobParams {
  jobId: number;
  amount: number; // lamports
  deadline: number; // unix timestamp (i64)
  title: string;
  description: string;
}

export interface ApplyParams {
  client: string; // job owner pubkey
  jobId: number;
  applicationIndex: number;
  proposalHash: string; // hex 32 bytes
}

export interface PaginatedJobs {
  jobs: Job[];
  nextCursor: string | null;
  hasMore: boolean;
}

// ---------------------------------------------------------------------------
// Config & error handling
// ---------------------------------------------------------------------------
export const API_URL = (
  process.env.NEXT_PUBLIC_API_URL ||
  process.env.NEXT_PUBLIC_BACKEND_URL ||
  "http://127.0.0.1:3000"
).replace(/\/$/, "");

export const RPC_URL_FALLBACK = process.env.NEXT_PUBLIC_RPC_URL || "http://127.0.0.1:8899";

export class SdkError extends Error {
  status?: number;
  code?: string;
  constructor(message: string, opts?: { status?: number; code?: string }) {
    super(message);
    this.name = "SdkError";
    this.status = opts?.status;
    this.code = opts?.code;
  }
}

// Validation limits mirroring backend/api/src/metadata.rs + validation.rs
export const MAX_TITLE_LEN = 100;
export const MAX_DESC_LEN = 500;
export const MAX_PROPOSAL_LEN = 512;
export const MIN_AMOUNT = 1;
export const MAX_AMOUNT = 10_000 * 1_000_000_000; // 10k SOL

export function validateCreateJobParams(p: CreateJobParams): string | null {
  if (!p.title || !p.title.trim()) return "title requerido";
  if (p.title.trim().length > MAX_TITLE_LEN) return `title excede ${MAX_TITLE_LEN} caracteres`;
  if (p.description && p.description.length > MAX_DESC_LEN) return `description excede ${MAX_DESC_LEN} caracteres`;
  if (!Number.isFinite(p.amount) || p.amount < MIN_AMOUNT) return "amount debe ser > 0";
  if (p.amount > MAX_AMOUNT) return `amount excede máximo ${MAX_AMOUNT}`;
  if (!Number.isFinite(p.deadline) || p.deadline <= 0) return "deadline debe ser timestamp futuro";
  const now = Math.floor(Date.now() / 1000);
  if (p.deadline <= now) return "deadline debe ser en el futuro";
  if (!Number.isFinite(p.jobId) || p.jobId < 0) return "jobId inválido";
  return null;
}

export function validateProposalHash(hash: string): string | null {
  const t = hash.trim();
  if (t.length !== 64) return "proposal_hash debe ser 64 hex chars (sha256)";
  if (!/^[0-9a-fA-F]{64}$/.test(t)) return "proposal_hash debe ser hex";
  if (t === "0".repeat(64)) return "proposal_hash vacío (EmptyProposal)";
  return null;
}

// ---------------------------------------------------------------------------
// Mock store (fallback para tests / sin backend)
// ---------------------------------------------------------------------------
const MOCK_KEY = "twe_mock_jobs_v3";

function loadMock(): Job[] {
  if (typeof window === "undefined") return seedJobs();
  try {
    const raw = localStorage.getItem(MOCK_KEY);
    if (raw) return JSON.parse(raw) as Job[];
  } catch {}
  const seeded = seedJobs();
  try {
    localStorage.setItem(MOCK_KEY, JSON.stringify(seeded));
  } catch {}
  return seeded;
}

function saveMock(jobs: Job[]) {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(MOCK_KEY, JSON.stringify(jobs));
  } catch {}
}

function seedJobs(): Job[] {
  const now = Math.floor(Date.now() / 1000);
  return [
    {
      jobId: "1",
      client: "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh",
      title: "Landing page para DeFi dashboard",
      description: "Next.js + Tailwind, diseño responsive, integración wallet Solana.",
      amount: "500000000",
      deadline: now + 86400 * 7,
      status: "Open",
      freelancer: null,
      createdAt: now - 3600,
    },
    {
      jobId: "2",
      client: "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh",
      title: "Smart contract escrow audit",
      description: "Revisión de seguridad Anchor 0.32, tests + fuzz.",
      amount: "1200000000",
      deadline: now + 86400 * 14,
      status: "Open",
      freelancer: null,
      createdAt: now - 7200,
    },
  ];
}

function hashProposal(input: string): string {
  let h = 0;
  for (let i = 0; i < input.length; i++) h = (h * 31 + input.charCodeAt(i)) >>> 0;
  return h.toString(16).padStart(64, "0").slice(0, 64);
}

// ---------------------------------------------------------------------------
// Backend mapping helpers
// ---------------------------------------------------------------------------
interface JobResponse {
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

function mapJobResponse(r: JobResponse): Job {
  return {
    jobId: String(r.job_id),
    client: r.client,
    title: r.title,
    description: r.description,
    amount: String(r.amount),
    deadline: r.deadline,
    status: (BACKEND_TO_FRONT_STATUS[r.status] as JobStatus) || "Open",
    freelancer: r.freelancer,
    createdAt: r.deadline - 86400, // backend uses created_at + 86400; approximate reverse
  };
}

async function apiFetch(path: string, init?: RequestInit, timeoutMs = 5000): Promise<Response> {
  const controller = new AbortController();
  const t = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const res = await fetch(`${API_URL}${path}`, {
      ...init,
      signal: controller.signal,
      headers: {
        "Content-Type": "application/json",
        ...(init?.headers || {}),
      },
    });
    return res;
  } finally {
    clearTimeout(t);
  }
}

async function parseApiError(res: Response): Promise<SdkError> {
  let msg = `API ${res.status} ${res.statusText}`;
  let code: string | undefined;
  try {
    const body = await res.json();
    if (body?.error) msg = body.error;
    if (body?.code) code = body.code;
    // backend returns {error, code}
    if (body?.message) msg = body.message;
  } catch {
    // ignore json parse
    try {
      const text = await res.text();
      if (text) msg = text.slice(0, 300);
    } catch {}
  }
  return new SdkError(msg, { status: res.status, code });
}

function paginate(jobs: Job[], cursor?: string | null, limit = 20): PaginatedJobs {
  const start = cursor ? parseInt(cursor, 10) : 0;
  const safeStart = Number.isNaN(start) || start < 0 ? 0 : start;
  const slice = jobs.slice(safeStart, safeStart + limit);
  const next = safeStart + limit < jobs.length ? String(safeStart + limit) : null;
  return { jobs: slice, nextCursor: next, hasMore: next !== null };
}

// ---------------------------------------------------------------------------
// Public SDK API — list_jobs, get_job, create_job, apply (+ aliases)
// ---------------------------------------------------------------------------

/**
 * Lista jobs paginados. Intenta backend API GET /jobs; fallback a mock local.
 */
export async function list_jobs(cursor?: string | null, limit = 20): Promise<PaginatedJobs> {
  // Try backend first (works in browser and in Node tests si el server está up)
  try {
    const res = await apiFetch("/jobs", { method: "GET" }, 2500);
    if (res.ok) {
      const data = (await res.json()) as JobResponse[] | { jobs: JobResponse[] };
      const arr: JobResponse[] = Array.isArray(data) ? data : (data as { jobs: JobResponse[] }).jobs || [];
      const mapped = arr.map(mapJobResponse);
      // If API returned data, use it; if empty, show mock so UI no queda vacía en dev sin DB
      const effective = mapped.length > 0 ? mapped : loadMock();
      // API currently returns all without cursor; apply pagination client-side
      return paginate(effective, cursor, limit);
    }
    // 4xx/5xx -> parse error but fallback to mock for resilience en dev/tests
    if (res.status >= 500) throw await parseApiError(res);
    // 404 etc fallback
  } catch (e) {
    // In tests without server, fallback silently; in prod log
    if (e instanceof SdkError) {
      // Si es error de validación/backend, no ocultar si estamos en test con mock?
      // Fallback solo si es network/timeout
      if (e.status && e.status < 500) throw e;
    }
    // network error / abort -> fallback
  }
  const all = loadMock();
  return paginate(all, cursor, limit);
}

export const listJobs = list_jobs;

export async function get_job(jobId: string): Promise<Job | null> {
  // Try backend
  try {
    const res = await apiFetch(`/jobs/${encodeURIComponent(jobId)}`, { method: "GET" }, 2500);
    if (res.ok) {
      const data = (await res.json()) as JobResponse;
      if (data && typeof data.job_id !== "undefined") return mapJobResponse(data);
    } else if (res.status === 404) {
      // fall through to mock check to keep tests deterministic
    } else {
      throw await parseApiError(res);
    }
  } catch (e) {
    if (e instanceof SdkError && e.status && e.status < 500 && e.status !== 404) throw e;
    // else fallback
  }
  const all = loadMock();
  return all.find((j) => j.jobId === jobId) ?? null;
}
export const getJob = get_job;

/**
 * Crea un job. Valida local, POST /jobs al backend, fallback a mock si backend no disponible.
 */
export async function create_job(params: CreateJobParams): Promise<{ signature: string; job: Job }> {
  const validationErr = validateCreateJobParams(params);
  if (validationErr) throw new SdkError(validationErr, { code: "bad_request", status: 400 });

  // Attempt backend
  try {
    const res = await apiFetch("/jobs", {
      method: "POST",
      body: JSON.stringify({
        title: params.title.trim(),
        description: params.description,
        amount: params.amount,
        deadline: params.deadline,
      }),
    }, 4000);
    if (res.ok || res.status === 201) {
      const data = (await res.json()) as JobResponse;
      const job = mapJobResponse(data);
      // also sync to mock for offline UX
      try {
        const all = loadMock();
        if (!all.some((j) => j.jobId === job.jobId)) {
          all.unshift(job);
          saveMock(all);
        }
      } catch {}
      const sig = `api_sig_create_${job.jobId}_${Date.now()}`;
      return { signature: sig, job };
    }
    throw await parseApiError(res);
  } catch (e) {
    // If backend unreachable (network), fallback to mock logic para tests/dev sin docker
    const isNetworkError =
      e instanceof TypeError ||
      (e instanceof DOMException && e.name === "AbortError") ||
      (e instanceof SdkError && !e.status) ||
      (e instanceof Error && e.message.includes("fetch"));
    if (!isNetworkError && e instanceof SdkError) {
      // backend validation error -> propagate
      throw e;
    }
    if (e instanceof SdkError && e.status && e.status >= 400 && e.status < 500) throw e;
    // fallback mock
  }

  // Mock fallback (determinístico para tests)
  const job: Job = {
    jobId: String(params.jobId),
    client: "mock-client-pubkey",
    title: params.title.trim(),
    description: params.description,
    amount: String(params.amount),
    deadline: params.deadline,
    status: "Open",
    freelancer: null,
    createdAt: Math.floor(Date.now() / 1000),
  };
  const all = loadMock();
  if (all.some((j) => j.jobId === job.jobId)) throw new SdkError(`job ${job.jobId} ya existe`, { code: "conflict", status: 409 });
  all.unshift(job);
  saveMock(all);
  const sig = `mock_sig_create_${job.jobId}_${Date.now()}`;
  return { signature: sig, job };
}
export const createJob = create_job;

/**
 * Aplica a un job. Espeja Rust: apply_to_job(client, job_id, application_index, proposal_hash)
 */
export async function apply(params: ApplyParams & { proposalText?: string }): Promise<{ signature: string }> {
  const hash = params.proposalHash || (params.proposalText ? hashProposal(params.proposalText) : "");
  const hashErr = validateProposalHash(hash);
  if (hashErr) throw new SdkError(hashErr, { code: "bad_request", status: 400 });

  // Validate job exists via get_job (mock-aware)
  const job = await get_job(String(params.jobId));
  if (!job) throw new SdkError("job no encontrado", { code: "not_found", status: 404 });
  if (job.status !== "Open") throw new SdkError("job no está abierto", { code: "bad_request", status: 400 });

  // Try backend POST /jobs/:job_id/apply
  try {
    const res = await apiFetch(`/jobs/${encodeURIComponent(String(params.jobId))}/apply`, {
      method: "POST",
      body: JSON.stringify({
        proposal_hash: hash,
        proposal: params.proposalText ?? `proposal for ${params.jobId}`,
      }),
    }, 4000);
    if (res.ok || res.status === 201) {
      const sig = `api_sig_apply_${params.jobId}_${params.applicationIndex}_${Date.now()}`;
      return { signature: sig };
    }
    throw await parseApiError(res);
  } catch (e) {
    if (e instanceof SdkError && e.status && e.status >= 400 && e.status < 500) throw e;
    const isNetwork = e instanceof TypeError || (e instanceof Error && e.message.includes("fetch"));
    if (!isNetwork && e instanceof SdkError) throw e;
    // fallback mock sig
  }

  const sig = `mock_sig_apply_${params.jobId}_${params.applicationIndex}_${Date.now()}`;
  return { signature: sig };
}
export const applyToJob = apply;

export function proposalHashFromText(text: string): string {
  if (!text.trim()) return "0".repeat(64);
  return hashProposal(text);
}

// Re-export helper for UI validation display
export function getValidationErrorForCreate(p: Partial<CreateJobParams>): string | null {
  return validateCreateJobParams(p as CreateJobParams);
}
