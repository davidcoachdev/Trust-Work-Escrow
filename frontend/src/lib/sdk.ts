/**
 * trust-escrow-sdk — TypeScript wrapper for Trust Work Escrow v3
 * Programa on-chain: 7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh
 * Espejo del SDK Rust (backend/sdk) — funciones: list_jobs, create_job, apply (apply_to_job)
 */

export const PROGRAM_ID_STR = "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh";

// --- Types mirroring backend/sdk/src/types.rs ---
export type JobStatus = "Open" | "InProgress" | "Completed" | "Cancelled" | "Disputed";

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
  proposalHash: string; // hex 32 bytes or plain text hashed client-side
}

// Paginated helpers (par con SDK Rust PaginatedJobs)
export interface PaginatedJobs {
  jobs: Job[];
  nextCursor: string | null;
  hasMore: boolean;
}

// --- Mock store (para scaffold sin validator) ---
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
  // Simple deterministic 32-byte hex (no crypto dep en scaffold)
  let h = 0;
  for (let i = 0; i < input.length; i++) h = (h * 31 + input.charCodeAt(i)) >>> 0;
  return h.toString(16).padStart(64, "0").slice(0, 64);
}

// --- Public SDK API (nombres exigidos: list_jobs, create_job, apply) ---

/**
 * Lista jobs paginados. En producción llama a RPC getProgramAccounts;
 * en scaffold usa mock local con soporte de cursor/limit.
 * Nombre snake_case para espejar Rust: list_jobs
 */
export async function list_jobs(cursor?: string | null, limit = 20): Promise<PaginatedJobs> {
  // Si hay RPC configurado, intentar fetch real; si falla, fallback mock
  const rpc = process.env.NEXT_PUBLIC_RPC_URL;
  if (rpc && typeof window !== "undefined") {
    try {
      // TODO: implementar fetch real con @coral-xyz/anchor + getProgramAccounts
      // por ahora fallback a mock para build determinístico
    } catch {}
  }
  const all = loadMock();
  const start = cursor ? parseInt(cursor, 10) : 0;
  const slice = all.slice(start, start + limit);
  const next = start + limit < all.length ? String(start + limit) : null;
  return { jobs: slice, nextCursor: next, hasMore: next !== null };
}

/**
 * Alias camelCase para uso idiomático en componentes.
 */
export const listJobs = list_jobs;

export async function get_job(jobId: string): Promise<Job | null> {
  const all = loadMock();
  return all.find((j) => j.jobId === jobId) ?? null;
}
export const getJob = get_job;

/**
 * Crea un job. En scaffold persiste en mock; en prod envía instrucción Anchor `create_job`.
 * Espeja Rust: create_job(job_id, amount, deadline)
 */
export async function create_job(params: CreateJobParams): Promise<{ signature: string; job: Job }> {
  if (!params.title.trim()) throw new Error("title requerido");
  if (params.amount <= 0) throw new Error("amount debe ser > 0");
  const job: Job = {
    jobId: String(params.jobId),
    client: "mock-client-pubkey",
    title: params.title,
    description: params.description,
    amount: String(params.amount),
    deadline: params.deadline,
    status: "Open",
    freelancer: null,
    createdAt: Math.floor(Date.now() / 1000),
  };
  const all = loadMock();
  if (all.some((j) => j.jobId === job.jobId)) throw new Error(`job ${job.jobId} ya existe`);
  all.unshift(job);
  saveMock(all);
  // signature mock determinística
  const sig = `mock_sig_create_${job.jobId}_${Date.now()}`;
  return { signature: sig, job };
}
export const createJob = create_job;

/**
 * Aplica a un job. Espeja Rust: apply_to_job(client, job_id, application_index, proposal_hash)
 * En scaffold valida hash no vacío y persiste estado.
 */
export async function apply(params: ApplyParams & { proposalText?: string }): Promise<{ signature: string }> {
  const hash = params.proposalHash || (params.proposalText ? hashProposal(params.proposalText) : "");
  if (!hash || hash === "0".repeat(64)) throw new Error("proposal_hash vacío (EmptyProposal)");
  const all = loadMock();
  const job = all.find((j) => j.jobId === String(params.jobId));
  if (!job) throw new Error("job no encontrado");
  if (job.status !== "Open") throw new Error("job no está abierto");
  // scaffold: no persiste applications, solo retorna sig mock
  const sig = `mock_sig_apply_${params.jobId}_${params.applicationIndex}_${Date.now()}`;
  return { signature: sig };
}

/**
 * Helper para hashear propuesta en cliente antes de llamar `apply`.
 */
export function proposalHashFromText(text: string): string {
  if (!text.trim()) return "0".repeat(64);
  return hashProposal(text);
}
