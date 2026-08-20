import { apiFetch, parseApiError } from "../client";
import type { ApiStatus } from "../types";

export async function depositFunds(jobId: string | number): Promise<ApiStatus> {
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(jobId))}/deposit`, { method: "POST" });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ApiStatus;
}
