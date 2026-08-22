import { describe, it, expect, vi, beforeEach } from "vitest";
import { useDashboardStore } from "./useDashboardStore";

describe("useDashboardStore", () => {
  beforeEach(() => {
    useDashboardStore.getState().reset();
    vi.restoreAllMocks();
  });
  it("estado inicial", () => {
    const s = useDashboardStore.getState();
    expect(s.jobs).toEqual([]);
    expect(s.role).toBeNull();
    expect(s.loading).toBe(false);
  });
  it("fetchJobs por status con cursor opaco", async () => {
    const mockJobs:any[]=[{job_id:1, client:"C", freelancer:null, title:"T1", description:"D", amount:1000000, fee_amount:25000, status:"Created", deadline:9999999999, applicants_count:5}];
    global.fetch = vi.fn(async () => new Response(JSON.stringify({jobs:mockJobs, nextCursor:"opaque123"}),{status:200, headers:{"content-type":"application/json"}})) as any;
    await useDashboardStore.getState().fetchJobs({cursor:null, limit:10, status:"Open"});
    const s = useDashboardStore.getState();
    expect(s.jobs.length).toBe(1);
    expect(s.nextCursor).toBe("opaque123");
    expect(s.hasMore).toBe(true);
  });
  it("fetchByClient filtra por cliente", async () => {
    const mockJobs:any[]=[
      {job_id:0, client:"ClientA", freelancer:null, title:"T1", description:"D", amount:1000000, fee_amount:25000, status:"Created", deadline:9999999999, applicants_count:0},
      {job_id:1, client:"ClientB", freelancer:null, title:"T2", description:"D", amount:1000000, fee_amount:25000, status:"Created", deadline:9999999999, applicants_count:0},
    ];
    global.fetch = vi.fn(async () => new Response(JSON.stringify({jobs:mockJobs}),{status:200, headers:{"content-type":"application/json"}})) as any;
    await useDashboardStore.getState().fetchByClient("ClientA");
    expect(useDashboardStore.getState().jobs.length).toBe(1);
    expect(useDashboardStore.getState().jobs[0].client).toBe("ClientA");
  });
  it("polling y notifications", () => {
    const { pushNotification, markAllRead } = useDashboardStore.getState();
    pushNotification({title:"Test", body:"body"});
    expect(useDashboardStore.getState().notifications.length).toBe(1);
    markAllRead();
    expect(useDashboardStore.getState().notifications[0].read).toBe(true);
  });
  it("setRole persiste", () => {
    useDashboardStore.getState().setRole("freelancer");
    expect(useDashboardStore.getState().role).toBe("freelancer");
    useDashboardStore.getState().setRole("client");
    expect(useDashboardStore.getState().role).toBe("client");
  });
});
