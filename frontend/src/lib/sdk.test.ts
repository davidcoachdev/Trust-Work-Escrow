import { describe, it, expect } from "vitest";
import { PROGRAM_ID_STR, list_jobs, create_job, apply, proposalHashFromText } from "./sdk";

describe("trust-escrow-sdk stub (7a2Y)", () => {
  it("PROGRAM_ID_STR es 7a2Y", () => {
    expect(PROGRAM_ID_STR).toBe("7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh");
  });

  it("list_jobs retorna paginado", async () => {
    const res = await list_jobs(null, 10);
    expect(res.jobs.length).toBeGreaterThan(0);
    expect(Array.isArray(res.jobs)).toBe(true);
  });

  it("create_job y list_jobs", async () => {
    const now = Math.floor(Date.now() / 1000) + 86400;
    const { job } = await create_job({ jobId: 99991, amount: 1_000_000, deadline: now, title: "Test job", description: "desc" });
    expect(job.jobId).toBe("99991");
    const listed = await list_jobs(null, 50);
    expect(listed.jobs.some((j) => j.jobId === "99991")).toBe(true);
  });

  it("apply rechaza hash vacío", async () => {
    await expect(apply({ client: "mock", jobId: 1, applicationIndex: 0, proposalHash: "0".repeat(64) })).rejects.toThrow();
  });

  it("proposalHashFromText no es vacío", () => {
    const h = proposalHashFromText("hola mundo");
    expect(h.length).toBe(64);
    expect(h).not.toBe("0".repeat(64));
  });

  it("apply con texto válido", async () => {
    const res = await apply({ client: "mock", jobId: 1, applicationIndex: 0, proposalText: "mi propuesta", proposalHash: proposalHashFromText("mi propuesta") });
    expect(res.signature).toContain("mock_sig_apply");
  });
});
