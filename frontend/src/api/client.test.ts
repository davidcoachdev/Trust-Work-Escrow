import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ApiError, API_URL, apiFetch, parseApiError } from "./client";
import { listJobs } from "./jobs/list";
import { getJob } from "./jobs/get";
import { createJob } from "./jobs/create";
import { applyToJob } from "./applications/apply";
import { createMilestone } from "./milestones/create";
import { raiseDispute } from "./disputes/raise";
import { getArbiterPool } from "./arbiterPool/get";

describe("api/client root", () => {
  it("API_URL default sin trailing slash", () => {
    expect(API_URL).toMatch(/^http/);
    expect(API_URL.endsWith("/")).toBe(false);
  });

  it("parseApiError lee JSON error", async () => {
    const res = new Response(JSON.stringify({ error: "bad request", code: "bad_request" }), { status: 400 });
    const err = await parseApiError(res as any);
    expect(err.message).toBe("bad request");
    expect(err.status).toBe(400);
    expect(err.code).toBe("bad_request");
  });
});

describe("api/jobs", () => {
  const originalFetch = global.fetch;

  beforeEach(() => { vi.resetAllMocks(); });
  afterEach(() => { global.fetch = originalFetch; });

  it("listJobs mapea JobResponse → Job (on-chain Vec + off-chain metadata)", async () => {
    const mock: any[] = [
      { job_id: 0, client: "Client111111111111111111111111111111111", freelancer: null, title: "T1", description: "D1", amount: 1_000_000, fee_amount: 25000, status: "Created", deadline: 9999999999, applicants_count: 2 },
      { job_id: 1, client: "Client222222222222222222222222222222222", freelancer: null, title: "T2", description: "D2", amount: 2_000_000, fee_amount: 50000, status: "Created", deadline: 9999999999, applicants_count: 0 },
    ];
    global.fetch = vi.fn(async () => new Response(JSON.stringify({ jobs: mock, nextCursor: null }), { status: 200, headers: { "content-type": "application/json" } })) as any;
    const res = await listJobs({ limit: 10 });
    expect(res.jobs.length).toBe(2);
    expect(res.jobs[0].jobId).toBe("0");
    expect(res.jobs[0].title).toBe("T1");
    expect(res.jobs[0].status).toBe("Open");
    expect(res.jobs[0].applicantsCount).toBe(2);
  });

  it("getJob retorna null en 404", async () => {
    global.fetch = vi.fn(async () => new Response("not found", { status: 404 })) as any;
    const job = await getJob(9999);
    expect(job).toBeNull();
  });

  it("createJob valida título requerido", async () => {
    await expect(createJob({ title: "", description: "d", amount: 1_000_000, deadline: 9999999999 })).rejects.toThrow(ApiError);
  });

  it("createJob POST /jobs y mapea respuesta", async () => {
    const resp = { job_id: 5, client: "ClientX", freelancer: null, title: "Nuevo", description: "Desc", amount: 5_000_000, fee_amount: 125000, status: "Created", deadline: 9999999999, applicants_count: 0 };
    global.fetch = vi.fn(async () => new Response(JSON.stringify(resp), { status: 201, headers: { "content-type": "application/json" } })) as any;
    const { job } = await createJob({ title: "Nuevo", description: "Desc", amount: 5_000_000, deadline: 9999999999 });
    expect(job.jobId).toBe("5");
    expect(job.title).toBe("Nuevo");
  });

  it("applyToJob valida hash", async () => {
    await expect(applyToJob({ jobId: 0, proposal: "hi", proposalHash: "0".repeat(64) })).rejects.toThrow();
  });
});

describe("api/applications + milestones + disputes (cada endpoint un archivo)", () => {
  const originalFetch = global.fetch;
  beforeEach(() => { vi.resetAllMocks(); });
  afterEach(() => { global.fetch = originalFetch; });

  it("createMilestone valida title", async () => {
    await expect(createMilestone(0, { title: "", description: "d", amount: 1000 })).rejects.toThrow();
  });

  it("milestone y dispute fetch shape", async () => {
    global.fetch = vi.fn(async () => new Response(JSON.stringify({ index: 0, title: "M1", description: "D", amount: 1000, status: "Pending" }), { status: 201, headers: { "content-type": "application/json" } })) as any;
    const m = await createMilestone(0, { title: "M1", description: "D", amount: 1000 });
    expect(m.index).toBe(0);
    expect(m.title).toBe("M1");
  });

  it("raiseDispute resuelve correct", async () => {
    global.fetch = vi.fn(async () => new Response(JSON.stringify({ job_id: 0, raised_by: "Client", arbiter: null, status: "Open", evidence_count: 0, client_payout_percent: 0, freelancer_payout_percent: 0 }), { status: 201 })) as any;
    const d = await raiseDispute(0);
    expect(d.job_id).toBe(0);
    expect(d.status).toBe("Open");
  });

  it("getArbiterPool fetch", async () => {
    global.fetch = vi.fn(async () => new Response(JSON.stringify({ authority: "Auth", arbiters: [] }), { status: 200 })) as any;
    const p = await getArbiterPool();
    expect(p.authority).toBe("Auth");
  });
});
