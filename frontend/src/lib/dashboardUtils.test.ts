import { describe, it, expect } from "vitest";
import { countdown, autoApproveCountdown, toCsv, filterByRange, computeMetrics } from "./dashboardUtils";

describe("dashboardUtils", ()=>{
  it("countdown futuro", ()=>{
    const dl = Math.floor(Date.now()/1000)+86400;
    const c = countdown(dl);
    expect(c.overdue).toBe(false);
    expect(c.text).toContain("restantes");
  });
  it("autoApprove 7d", ()=>{
    const submitted = Math.floor(Date.now()/1000)-86400;
    const a = autoApproveCountdown(submitted,7);
    expect(a.pct).toBeGreaterThan(10);
    expect(a.text).toContain("Auto-approve");
  });
  it("toCsv", ()=>{
    const csv = toCsv([{a:1,b:"hi"}, {a:2,b:"bye"}]);
    expect(csv).toContain("a");
    expect(csv.split("\n").length).toBe(3);
  });
  it("filterByRange 30d", ()=>{
    const now = Math.floor(Date.now()/1000);
    const items:any[]=[{createdAt: now}, {createdAt: now-40*86400}];
    const f = filterByRange(items,"30d");
    expect(f.length).toBe(1);
  });
  it("computeMetrics", ()=>{
    const jobs:any[]=[{amount:"1000000000", status:"Completed", createdAt:100, deadline:100+86400}, {amount:"2000000000", status:"Disputed", createdAt:100, deadline:100+86400}];
    const m = computeMetrics(jobs);
    expect(m.totalGastado).toBe(3000000000);
    expect(m.disputasPct).toBe(50);
  });
});
