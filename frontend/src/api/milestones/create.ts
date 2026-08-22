import { apiFetch, parseApiError, ApiError } from "../client";
import type { MilestoneResponse, CreateMilestoneParams } from "../types";

export async function createMilestone(jobId: string | number, params: CreateMilestoneParams): Promise<MilestoneResponse> {
  if (!params.title?.trim()) throw new ApiError("milestone title requerido", { status: 400 });
  if (!params.description?.trim()) throw new ApiError("milestone description requerida", { status: 400 });
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(jobId))}/milestones`, {
    method: "POST",
    body: JSON.stringify({ title: params.title.trim(), description: params.description.trim(), amount: params.amount }),
  });
  if (!res.ok) throw await parseApiError(res);
  // backend returns 201 with body
  return (await res.json()) as MilestoneResponse;
}
