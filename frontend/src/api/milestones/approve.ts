import { apiFetch, parseApiError } from "../client";
import type { MilestoneResponse } from "../types";

export async function approveMilestone(jobId: string | number, milestoneIndex: number): Promise<MilestoneResponse> {
  const res = await apiFetch(
    `/jobs/${encodeURIComponent(String(jobId))}/milestones/${encodeURIComponent(String(milestoneIndex))}/approve`,
    { method: "POST" }
  );
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as MilestoneResponse;
}
