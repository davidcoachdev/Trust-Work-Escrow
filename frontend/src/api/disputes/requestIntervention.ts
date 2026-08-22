import { apiFetch, parseApiError } from "../client";
import type { DisputeResponse } from "../types";

export async function requestPlatformIntervention(jobId: string | number): Promise<DisputeResponse> {
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(jobId))}/disputes/request-intervention`, { method: "POST" });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as DisputeResponse;
}
