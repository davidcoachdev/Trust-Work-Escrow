import { apiFetch, parseApiError } from "../client";
import type { ApiStatus } from "../types";

export async function cancelJob(jobId: string | number): Promise<ApiStatus> {
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(jobId))}/cancel`, { method: "POST" });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ApiStatus;
}
