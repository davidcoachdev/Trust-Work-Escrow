import { apiFetch, parseApiError } from "../client";
import type { SupportTicketResponse } from "../types";

export async function resolveSupportTicket(jobId: string | number): Promise<SupportTicketResponse> {
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(jobId))}/support/resolve`, { method: "POST" });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as SupportTicketResponse;
}
