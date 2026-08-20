import { apiFetch, parseApiError } from "../client";
import type { ApplicationResponse } from "../types";

export async function acceptApplication(jobId: string | number, applicationIndex: number): Promise<ApplicationResponse> {
  const res = await apiFetch(
    `/jobs/${encodeURIComponent(String(jobId))}/applications/${encodeURIComponent(String(applicationIndex))}/accept`,
    { method: "POST" }
  );
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as ApplicationResponse;
}
