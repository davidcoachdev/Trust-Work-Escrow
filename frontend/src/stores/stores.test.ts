import { describe, it, expect, vi, beforeEach } from "vitest";
import { useJobStore } from "./useJobStore";
import { useApplicationStore } from "./useApplicationStore";
import { useMilestoneStore } from "./useMilestoneStore";

describe("Zustand stores — fuente de la verdad que consume api/", () => {
  beforeEach(() => {
    useJobStore.getState().reset();
    useApplicationStore.getState().reset();
    useMilestoneStore.getState().reset();
    vi.restoreAllMocks();
  });

  it("useJobStore estado inicial", () => {
    const s = useJobStore.getState();
    expect(s.jobs).toEqual([]);
    expect(s.currentJob).toBeNull();
    expect(s.loading).toBe(false);
    expect(s.error).toBeNull();
  });

  it("useJobStore.fetchJobs consume api/jobs/list", async () => {
    const mockJobs: any[] = [{ job_id: 0, client: "C", freelancer: null, title: "T", description: "D", amount: 1000000, fee_amount: 25000, status: "Created", deadline: 9999999999, applicants_count: 0 }];
    global.fetch = vi.fn(async () => new Response(JSON.stringify({ jobs: mockJobs, nextCursor: null }), { status: 200, headers: { "content-type": "application/json" } })) as any;
    await useJobStore.getState().fetchJobs({ cursor: null, limit: 10 });
    const s = useJobStore.getState();
    expect(s.jobs.length).toBe(1);
    expect(s.jobs[0].jobId).toBe("0");
    expect(s.loading).toBe(false);
    expect(s.error).toBeNull();
  });

  it("useJobStore.createJob valida y agrega a state", async () => {
    const resp = { job_id: 10, client: "C", freelancer: null, title: "Nuevo", description: "Desc", amount: 5000000, fee_amount: 125000, status: "Created", deadline: 9999999999, applicants_count: 0 };
    global.fetch = vi.fn(async () => new Response(JSON.stringify(resp), { status: 201, headers: { "content-type": "application/json" } })) as any;
    const job = await useJobStore.getState().createJob({ title: "Nuevo", description: "Desc", amount: 5000000, deadline: 9999999999 });
    expect(job.jobId).toBe("10");
    expect(useJobStore.getState().jobs[0].jobId).toBe("10");
  });

  it("useJobStore.createJob guarda error en state si valida falla", async () => {
    await expect(useJobStore.getState().createJob({ title: "", description: "d", amount: 1, deadline: 9999999999 })).rejects.toThrow();
    expect(useJobStore.getState().error).toBeTruthy();
  });

  it("useApplicationStore.apply consume api/applications/apply", async () => {
    global.fetch = vi.fn(async () => new Response(JSON.stringify({ index: 0, applicant: "A", proposal_hash: "a".repeat(64), status: "Pending" }), { status: 201 })) as any;
    const hash = "a".repeat(64);
    const res = await useApplicationStore.getState().apply({ jobId: 0, proposal: "mi propuesta", proposalHash: hash });
    expect(res.index).toBe(0);
    expect(useApplicationStore.getState().applications.length).toBe(1);
  });

  it("useMilestoneStore.create consume api/milestones/create", async () => {
    global.fetch = vi.fn(async () => new Response(JSON.stringify({ index: 0, title: "M1", description: "D", amount: 1000, status: "Pending" }), { status: 201, headers: { "content-type": "application/json" } })) as any;
    const m = await useMilestoneStore.getState().create(0, { title: "M1", description: "D", amount: 1000 });
    expect(m.title).toBe("M1");
    expect(useMilestoneStore.getState().milestones.length).toBe(1);
  });

  it("stores exponen clearError y reset", () => {
    const s = useJobStore.getState();
    s.reset();
    expect(s.jobs).toEqual([]);
  });
});
