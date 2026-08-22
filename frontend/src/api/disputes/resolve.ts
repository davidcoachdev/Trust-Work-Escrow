import { apiFetch, parseApiError } from "../client";
import type { DisputeResponse } from "../types";

export async function resolveDispute(jobId: string | number, clientPayoutPercent: number): Promise<DisputeResponse> {
  const res = await apiFetch(`/jobs/${encodeURIComponent(String(jobId))}/disputes/resolve`, {
    method: "POST",
    body: JSON.stringify({ client_payout_percent: clientPayoutPercent }),
  });
  if (!res.ok) throw await parseApiError(res);
  return (await res.json()) as DisputeResponse;
}
